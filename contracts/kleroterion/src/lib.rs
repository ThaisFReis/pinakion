#![no_std]
//! Kleroterion: o rol de jurados como um ativo clássico da Stellar.
//!
//! O *pinakion* era um bilhete de bronze. Aqui ele é uma unidade de um ativo
//! clássico, com `AUTH_REQUIRED` no emissor: ter o bilhete e estar autorizado
//! é o que dá assento. O rol é a lista de trustlines autorizadas, legível por
//! qualquer Horizon em `/accounts?asset=`; a ordem do rol é a ordem de
//! inscrição guardada aqui; o compromisso é `lock()`, depois do qual ninguém
//! entra nem sai — e o emissor, com master weight 0, não emite nem autoriza
//! mais nada. O `lock()` também fixa a rodada do beacon e o número de
//! assentos; `seal()` usa essa rodada uma única vez (INV-13, INV-14).
//!
//! Cada jurado se inscreve com um compromisso `c = Poseidon(s)` (T-035). O
//! contrato mantém a árvore de Merkle de Poseidon, profundidade 6, cuja folha
//! `i` é o `c` do `i`-ésimo inscrito, ou zero se ele foi impedido; o caminho
//! é refeito a cada `enroll` e `impede`, e o `lock()` congela a raiz. Ninguém
//! posta raiz: ela é sempre a que o contrato calculou.
//!
//! Uma prova de assento (T-037) é conferida por um contrato verificador à
//! parte, cujo endereço o construtor fixa e cuja chave está embutida nele.
//! As entradas públicas `[raiz, seed, y, claimer]` são montadas aqui: a raiz
//! do fechamento, a semente, o `y` declarado e `SHA-256(strkey) >> 8` da
//! conta de assento — ninguém entrega ao verificador um vetor pronto.
//!
//! A entropia vem da drand quicknet, verificada pelas funções de host da
//! BLS12-381 (CAP-59). `verify` e `randomness` são cópias literais do
//! `beacon-verifier`. O sorteio é selado (T-038, DEC-015): `seal` fixa a
//! semente e o limiar `T`; cada jurado com `y = Poseidon(seed, s) < T` prova
//! assento de uma conta de assento sem dizer quem é; `finalize` senta os `k`
//! menores `y`; depois de `revela_em`, cada um revela `s` e o painel fica
//! recomputável entre os que revelaram. O Fisher-Yates aberto fica só no
//! `beacon-verifier` (DEC-016).

use pinakion_poseidon::Sponge;
use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype,
    crypto::bls12_381::{Bls12381G1Affine, Bls12381G2Affine},
    panic_with_error, token, vec, Address, Bytes, BytesN, Env, Vec, U256,
};

/// Domain separation tag do drand quicknet.
const DST: &[u8] = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";

/// Gerador de G2 negado, codificação não comprimida.
pub const NEG_G2_GENERATOR: [u8; 192] = *include_bytes!("neg_g2.bin");

/// Chave pública do grupo de limiar da quicknet, pinada (INV-10).
pub const BEACON_PUBLIC_KEY: [u8; 192] = *include_bytes!("quicknet_pk.bin");

/// Hash da chain quicknet, publicado por `beacon()`.
pub const BEACON_CHAIN_HASH: [u8; 32] = *include_bytes!("quicknet_chain.bin");

/// Genesis e período da quicknet, em segundos (§5). A rodada `r` é publicada
/// em `genesis + (r-1)·period`.
pub const GENESIS_QUICKNET: u64 = 1_692_803_367;
pub const PERIODO_QUICKNET: u64 = 3;

/// Rodadas entre a primeira publicada depois do fechamento e a que o sorteio
/// usa. Dez rodadas são 30 s: a transação do `lock()` tem tempo de entrar num
/// ledger e ser vista antes que a rodada alvo exista (§9 T-033: ≥ 10).
pub const MARGEM_DE_RODADAS: u64 = 10;

/// Um bilhete: 1.0000000 PINAKION, em stroops. Ter menos que isto é não ter
/// bilhete (INV-12).
pub const UM_BILHETE: i128 = 10_000_000;

/// Profundidade da árvore de compromissos: 2⁶ = 64 jurados no máximo. É a do
/// circuito (`circuits/pinakion_claim.circom`).
pub const PROFUNDIDADE: u32 = 6;

/// Raiz da subárvore vazia de cada nível: `Z₀ = 0`, `Zₙ₊₁ = Poseidon(Zₙ, Zₙ)`.
/// Embutidas para não pagar seis hashes por chamada; `os_zeros_sao_os_da_circomlib`
/// confere contra tests/vetores-arvore.json.
const ZEROS: [[u8; 32]; 7] = [
    [0; 32],
    hex32(b"2098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864"),
    hex32(b"1069673dcdb12263df301a6ff584a7ec261a44cb9dc68df067a4774460b1f1e1"),
    hex32(b"18f43331537ee2af2e3d758d50f72106467c6eea50371dd528d57eb2b856d238"),
    hex32(b"07f9d837cb17b0d36320ffe93ba52345f1b728571a568265caac97559dbc952a"),
    hex32(b"2b94cf5e8746b3f5c9631f4c5df32907a699c58c94b2ad4d7b5cec1639183f55"),
    hex32(b"2dee93c5a666459646ea7d22cca9e1bcfed71e6951b953611d11dda32ea09d78"),
];

/// Módulo r do campo escalar do BN254: um compromisso é um elemento do campo.
const R_BN254: [u8; 32] =
    hex32(b"30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");

const fn hex32(h: &[u8; 64]) -> [u8; 32] {
    const fn nib(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            _ => panic!("hex"),
        }
    }
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = nib(h[2 * i]) << 4 | nib(h[2 * i + 1]);
        i += 1;
    }
    out
}

/// (p-1)/2 para o campo base da BLS12-381, big-endian.
const METADE_DE_P: [u8; 48] = [
    0x0d, 0x00, 0x88, 0xf5, 0x1c, 0xbf, 0xf3, 0x4d, 0x25, 0x8d, 0xd3, 0xdb, 0x21, 0xa5, 0xd6, 0x6b,
    0xb2, 0x3b, 0xa5, 0xc2, 0x79, 0xc2, 0x89, 0x5f, 0xb3, 0x98, 0x69, 0x50, 0x7b, 0x58, 0x7b, 0x12,
    0x0f, 0x55, 0xff, 0xff, 0x58, 0xa9, 0xff, 0xff, 0xdc, 0xff, 0x7f, 0xff, 0xff, 0xff, 0xd5, 0x55,
];

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Erro {
    /// A rodada pedida é anterior à que o fechamento fixou (INV-06). Desde
    /// T-033 ninguém pede rodada, então nada mais devolve este erro; o número
    /// fica reservado para não mudar o dos outros.
    RodadaAnteriorAoRol = 1,
    /// A assinatura não é da quicknet, ou não é da rodada que o fechamento
    /// fixou.
    AssinaturaInvalida = 2,
    /// O rol já fechou: ninguém mais entra (INV-11).
    RolFechado = 3,
    /// O rol ainda está aberto: não há o que sortear.
    RolAberto = 4,
    /// Fechar abaixo do mínimo do rol (INV-09).
    RolAbaixoDoMinimo = 5,
    /// O jurado já está inscrito.
    JuradoRepetido = 6,
    /// Não há jurados elegíveis suficientes para `k` assentos.
    ElegiveisInsuficientes = 7,
    /// `k` é zero ou maior que o rol.
    AssentosInvalidos = 8,
    /// O sorteio já foi feito: não há segundo (INV-13).
    JaSelado = 9,
    /// O compromisso é zero (a folha vazia) ou não é elemento do campo.
    CompromissoInvalido = 10,
    /// Este compromisso já foi usado, por quem quer que seja (INV-17).
    CompromissoRepetido = 11,
    /// A árvore tem 2⁶ folhas e estão todas ocupadas.
    RolCheio = 12,
    /// Impedir quem não está no rol.
    JuradoDesconhecido = 13,
    /// O jurado já está impedido.
    JaImpedido = 14,
    /// Ainda não há selo: nem semente nem limiar.
    NaoSelado = 15,
    /// `y` não está abaixo do limiar.
    YAcimaDoLimiar = 16,
    /// Este `y` já foi reivindicado, ou é de quem se recusou (INV-15).
    YRepetido = 17,
    /// A prova não vale contra a raiz, a semente, o `y` e a conta.
    ProvaInvalida = 18,
    /// A janela de reivindicação ainda está aberta.
    JanelaAberta = 19,
    /// O painel já saiu (INV-16).
    JaFinalizado = 20,
    /// O painel ainda não saiu.
    NaoFinalizado = 21,
    /// O ledger de revelação (`revela_em`) ainda não chegou.
    CedoParaRevelar = 22,
    /// `Poseidon(s)` não é o compromisso deste jurado.
    SegredoNaoConfere = 23,
    /// Este jurado já revelou.
    JaRevelado = 24,
    /// Este jurado já se recusou.
    JaRecusado = 25,
    /// A janela de reivindicação fechou.
    JanelaFechada = 26,
}

/// O que `lock()` fixa, de uma vez por todas: quando, qual rodada do beacon
/// decide, quantos assentos, o ledger a partir do qual os jurados revelam
/// (`revela_em`, usado a partir de T-038; Q-24), a raiz da árvore de
/// compromissos e `n`, quantos estavam autorizados.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fechamento {
    pub ledger: u32,
    pub tempo: u64,
    pub rodada: u64,
    pub k: u32,
    pub revela_em: u32,
    pub raiz: U256,
    pub n: u32,
}

/// Um jurado entrou no rol, na posição `indice`, com o compromisso `c`. Com
/// estes eventos e os de `Impedido`, qualquer um refaz a raiz
/// (scripts/arvore.py, web/arvore.mjs).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Inscrito {
    #[topic]
    pub jurado: Address,
    pub indice: u32,
    pub c: U256,
}

/// Um jurado foi impedido antes do fechamento: a folha `indice` vira zero.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Impedido {
    #[topic]
    pub jurado: Address,
    pub indice: u32,
}

/// O que `seal` fixa: a semente, o limiar `T` e o último ledger em que se
/// reivindica. `ampliado` diz se `finalize` já levou `T` ao campo inteiro.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selo {
    pub seed: U256,
    pub limiar: U256,
    pub prazo: u32,
    pub ampliado: bool,
}

/// O painel: os `y` que sentam, em ordem, e se chegou a `k`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Final {
    pub painel: Vec<U256>,
    pub completo: bool,
    pub ledger: u32,
}

/// O que um `reveal` provou de um jurado.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revelacao {
    pub y: U256,
    pub sentado: bool,
    pub sem_bilhete: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Selado {
    pub seed: U256,
    pub limiar: U256,
    pub prazo: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Ampliado {
    pub limiar: U256,
    pub prazo: u32,
}

/// Um assento reivindicado. Só o `y`: nenhum endereço ao lado dele (INV-17).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reivindicado {
    pub y: U256,
}

/// Um jurado se recusou, às claras, antes do painel. Recusar é revelar: o
/// evento leva `s` e o `y` que saiu, para que quem lê o ledger confira
/// `Poseidon(s) = c` e `y = Poseidon(seed, s)` sem confiar no contrato (T-039).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recusado {
    #[topic]
    pub jurado: Address,
    pub s: U256,
    pub y: U256,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finalizado {
    pub painel: Vec<U256>,
    pub completo: bool,
}

/// Depois do veredito, o elo entre um jurado e seu `y` — de propósito. Leva
/// `s`, para que a recomputação não dependa do contrato (T-039).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revelado {
    #[topic]
    pub jurado: Address,
    pub s: U256,
    pub y: U256,
    pub sentado: bool,
    pub sem_bilhete: bool,
}

/// O rol fechou com esta raiz, `n` autorizados, esta rodada e `k` assentos.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fechado {
    pub raiz: U256,
    pub n: u32,
    pub rodada: u64,
    pub k: u32,
    pub revela_em: u32,
}

/// Uma prova Groth16 no formato do verificador (`contracts/groth16-verifier`):
/// A e C em G1 (64 bytes), B em G2 (128 bytes). Mesmos campos, mesmo XDR.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Prova {
    pub a: BytesN<64>,
    pub b: BytesN<128>,
    pub c: BytesN<64>,
}

/// O que o kleroterion pede ao verificador. Declarado aqui para não embutir
/// o código do verificador neste contrato.
#[contractclient(name = "VerificadorClient")]
pub trait Verificador {
    fn verify(env: Env, proof: Prova, public: Vec<U256>) -> bool;
}

#[contracttype]
pub enum DataKey {
    Escrivao,
    Verificador,
    Janela,
    Selo,
    /// Os `y` reivindicados, em ordem crescente.
    Reivindicados,
    /// Um `y` já usado: reivindicado, ou de quem se recusou.
    YUsado(U256),
    Recusou(Address),
    Revelou(Address),
    Bilhete,
    Minimo,
    Rol,
    Fechamento,
    Painel,
    /// Quantos inscritos continuam autorizados.
    Autorizados,
    /// O compromisso de um jurado.
    Compromisso(Address),
    /// A posição de um jurado na ordem de inscrição — e na árvore.
    Indice(Address),
    /// Um compromisso já usado, para sempre.
    Usado(U256),
    /// Nó da árvore: (nível, posição). Ausente é a subárvore vazia.
    No(u32, u32),
}

#[contract]
pub struct Kleroterion;

#[contractimpl]
impl Kleroterion {
    /// `escrivao` inscreve e fecha; `bilhete` é o SAC do ativo cujo admin
    /// este contrato passa a ser; `minimo` é o menor rol que pode fechar;
    /// `verificador` é o contrato Groth16 que confere as provas de assento,
    /// fixado aqui para sempre (T-037); `janela` é quantos ledgers, depois
    /// do selo, os jurados têm para reivindicar (T-038).
    pub fn __constructor(
        env: Env,
        escrivao: Address,
        bilhete: Address,
        minimo: u32,
        verificador: Address,
        janela: u32,
    ) {
        let s = env.storage().instance();
        s.set(&DataKey::Escrivao, &escrivao);
        s.set(&DataKey::Janela, &janela);
        s.set(&DataKey::Verificador, &verificador);
        s.set(&DataKey::Bilhete, &bilhete);
        s.set(&DataKey::Minimo, &minimo);
        s.set(&DataKey::Autorizados, &0u32);
        let vazio: Vec<Address> = Vec::new(&env);
        env.storage().persistent().set(&DataKey::Rol, &vazio);
    }

    /// Inscreve um jurado: cria a trustline (SAC `trust`, CAP-73), autoriza e
    /// emite um bilhete. O escrivão e o jurado assinam — o jurado consente em
    /// servir; é a parte afetada pelo caso que nunca consentiu com o foro.
    ///
    /// `c` é o compromisso do jurado, `Poseidon(s)` para um segredo `s` que só
    /// ele guarda. Fica preso ao endereço, vira a folha da posição de
    /// inscrição, e nunca mais é aceito de ninguém (INV-17).
    pub fn enroll(env: Env, jurado: Address, c: U256) {
        Self::escrivao(&env).require_auth();
        jurado.require_auth();
        if Self::closing(env.clone()).is_some() {
            panic_with_error!(&env, Erro::RolFechado);
        }
        let mut rol = Self::roster(env.clone());
        if rol.contains(&jurado) {
            panic_with_error!(&env, Erro::JuradoRepetido);
        }
        if c == U256::from_u32(&env, 0) || c >= Self::r(&env) {
            panic_with_error!(&env, Erro::CompromissoInvalido);
        }
        let p = env.storage().persistent();
        if p.has(&DataKey::Usado(c.clone())) {
            panic_with_error!(&env, Erro::CompromissoRepetido);
        }
        let indice = rol.len();
        if indice >= 1 << PROFUNDIDADE {
            panic_with_error!(&env, Erro::RolCheio);
        }
        let sac = token::StellarAssetClient::new(&env, &Self::bilhete(&env));
        sac.trust(&jurado);
        sac.set_authorized(&jurado, &true);
        sac.mint(&jurado, &UM_BILHETE);
        rol.push_back(jurado.clone());
        p.set(&DataKey::Rol, &rol);
        p.set(&DataKey::Usado(c.clone()), &true);
        p.set(&DataKey::Compromisso(jurado.clone()), &c);
        p.set(&DataKey::Indice(jurado.clone()), &indice);
        Self::por_folha(&env, indice, c.clone());
        let i = env.storage().instance();
        let autorizados: u32 = i.get(&DataKey::Autorizados).unwrap();
        i.set(&DataKey::Autorizados, &(autorizados + 1));
        Inscrito { jurado, indice, c }.publish(&env);
    }

    /// Fecha o rol: a partir daqui ninguém entra nem sai (INV-11, INV-14).
    /// A rodada do beacon fica fixada pelo relógio do ledger, não por quem
    /// chama (INV-06, INV-13), e `k` fica fixado aqui, antes de a rodada
    /// existir, e a raiz da árvore com ele. Recusa abaixo do mínimo (INV-09)
    /// e `k` fora de `1..=N`, sendo `N` os inscritos ainda autorizados.
    pub fn lock(env: Env, k: u32, revela_em: u32) -> Fechamento {
        Self::escrivao(&env).require_auth();
        if Self::closing(env.clone()).is_some() {
            panic_with_error!(&env, Erro::RolFechado);
        }
        let minimo: u32 = env.storage().instance().get(&DataKey::Minimo).unwrap();
        if Self::roster(env.clone()).len() < minimo {
            panic_with_error!(&env, Erro::RolAbaixoDoMinimo);
        }
        let n: u32 = env.storage().instance().get(&DataKey::Autorizados).unwrap();
        if k == 0 || k > n {
            panic_with_error!(&env, Erro::AssentosInvalidos);
        }
        let tempo = env.ledger().timestamp();
        let f = Fechamento {
            ledger: env.ledger().sequence(),
            tempo,
            rodada: Self::primeira_rodada_depois_de(tempo) + MARGEM_DE_RODADAS,
            k,
            revela_em,
            raiz: Self::root(env.clone()),
            n,
        };
        env.storage().instance().set(&DataKey::Fechamento, &f);
        Fechado {
            raiz: f.raiz.clone(),
            n: f.n,
            rodada: f.rodada,
            k: f.k,
            revela_em: f.revela_em,
        }
        .publish(&env);
        f
    }

    /// Impedimento ou recusa (art. 463): tira a autorização do bilhete. O
    /// jurado continua no rol e na ordem; só não senta (INV-12), e o
    /// substituto é o próximo da ordem sorteada (INV-08). Só antes do
    /// fechamento: depois dele o escrivão não escolhe quem sai (INV-14, Q-21).
    /// A folha do impedido vira zero: ele não tem como provar assento.
    pub fn impede(env: Env, jurado: Address) {
        Self::escrivao(&env).require_auth();
        if Self::closing(env.clone()).is_some() {
            panic_with_error!(&env, Erro::RolFechado);
        }
        let indice: u32 = match env
            .storage()
            .persistent()
            .get(&DataKey::Indice(jurado.clone()))
        {
            Some(i) => i,
            None => panic_with_error!(&env, Erro::JuradoDesconhecido),
        };
        let sac = token::StellarAssetClient::new(&env, &Self::bilhete(&env));
        if !sac.authorized(&jurado) {
            panic_with_error!(&env, Erro::JaImpedido);
        }
        sac.set_authorized(&jurado, &false);
        Self::por_folha(&env, indice, U256::from_u32(&env, 0));
        let i = env.storage().instance();
        let autorizados: u32 = i.get(&DataKey::Autorizados).unwrap();
        i.set(&DataKey::Autorizados, &(autorizados - 1));
        Impedido { jurado, indice }.publish(&env);
    }

    /// O compromisso de um jurado, se inscrito.
    pub fn commitment(env: Env, jurado: Address) -> Option<U256> {
        env.storage()
            .persistent()
            .get(&DataKey::Compromisso(jurado))
    }

    /// A raiz da árvore de compromissos agora. Depois do `lock()` é a mesma
    /// de `closing().raiz`, porque nada mais entra nem sai.
    pub fn root(env: Env) -> U256 {
        Self::no(&env, PROFUNDIDADE, 0)
    }

    /// O rol, na ordem de inscrição. É esta ordem que o sorteio embaralha.
    pub fn roster(env: Env) -> Vec<Address> {
        env.storage().persistent().get(&DataKey::Rol).unwrap()
    }

    /// O fechamento, se já houve.
    pub fn closing(env: Env) -> Option<Fechamento> {
        env.storage().instance().get(&DataKey::Fechamento)
    }

    /// O contrato que confere as provas de assento.
    pub fn verifier(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Verificador).unwrap()
    }

    /// O painel, se já saiu.
    pub fn panel(env: Env) -> Option<Final> {
        env.storage().persistent().get(&DataKey::Painel)
    }

    /// O selo, se já houve.
    pub fn seal_state(env: Env) -> Option<Selo> {
        env.storage().instance().get(&DataKey::Selo)
    }

    /// Os `y` reivindicados até agora, em ordem crescente.
    pub fn claims(env: Env) -> Vec<U256> {
        env.storage()
            .persistent()
            .get(&DataKey::Reivindicados)
            .unwrap_or(Vec::new(&env))
    }

    /// O que o `reveal` de um jurado provou, se ele revelou.
    pub fn revelation(env: Env, jurado: Address) -> Option<Revelacao> {
        env.storage().persistent().get(&DataKey::Revelou(jurado))
    }

    /// Verifica a rodada que o fechamento fixou, uma vez só (INV-13), e fixa
    /// a semente — a aleatoriedade da rodada deslocada 8 bits, para ser
    /// elemento do campo — e o limiar `T = ⌊r·2k/N⌋`, o campo inteiro se
    /// `2k ≥ N`. Abre a janela de reivindicação. Qualquer um pode chamar.
    pub fn seal(env: Env, sig: BytesN<96>) -> Selo {
        let f = match Self::closing(env.clone()) {
            Some(f) => f,
            None => panic_with_error!(&env, Erro::RolAberto),
        };
        if Self::seal_state(env.clone()).is_some() {
            panic_with_error!(&env, Erro::JaSelado);
        }
        if !Self::verify(env.clone(), f.rodada, sig.clone()) {
            panic_with_error!(&env, Erro::AssinaturaInvalida);
        }
        let aleatorio: Bytes = Self::randomness(env.clone(), sig).into();
        let seed = U256::from_be_bytes(&env, &aleatorio).shr(8);
        let r = Self::r(&env);
        let (n, dois_k) = (f.n, 2 * f.k);
        let limiar = if dois_k >= n {
            r
        } else {
            let n_ = U256::from_u32(&env, n);
            let k_ = U256::from_u32(&env, dois_k);
            r.div(&n_)
                .mul(&k_)
                .add(&r.rem_euclid(&n_).mul(&k_).div(&n_))
        };
        let janela: u32 = env.storage().instance().get(&DataKey::Janela).unwrap();
        let selo = Selo {
            seed,
            limiar,
            prazo: env.ledger().sequence() + janela,
            ampliado: false,
        };
        env.storage().instance().set(&DataKey::Selo, &selo);
        Selado {
            seed: selo.seed.clone(),
            limiar: selo.limiar.clone(),
            prazo: selo.prazo,
        }
        .publish(&env);
        selo
    }

    /// Reivindica um assento, da conta de assento que assina: `y` abaixo do
    /// limiar, ainda não usado, e a prova de que `y = Poseidon(seed, s)` para
    /// algum `s` cuja `Poseidon(s)` é folha da raiz (INV-15). O contrato monta
    /// as entradas públicas; o evento leva só o `y` (INV-17).
    pub fn claim(env: Env, assento: Address, prova: Prova, y: U256) {
        assento.require_auth();
        let selo = Self::selo_aberto(&env);
        if env.ledger().sequence() > selo.prazo {
            panic_with_error!(&env, Erro::JanelaFechada);
        }
        if y >= selo.limiar {
            panic_with_error!(&env, Erro::YAcimaDoLimiar);
        }
        let p = env.storage().persistent();
        if p.has(&DataKey::YUsado(y.clone())) {
            panic_with_error!(&env, Erro::YRepetido);
        }
        if !Self::prova_de_assento_vale(&env, &prova, &selo.seed, &y, &assento) {
            panic_with_error!(&env, Erro::ProvaInvalida);
        }
        p.set(&DataKey::YUsado(y.clone()), &true);
        let ys = Self::claims(env.clone());
        let mut pos = ys.len();
        for (i, v) in ys.iter().enumerate() {
            if y < v {
                pos = i as u32;
                break;
            }
        }
        let mut ys = ys;
        ys.insert(pos, y.clone());
        p.set(&DataKey::Reivindicados, &ys);
        Reivindicado { y }.publish(&env);
    }

    /// Recusa às claras, antes do painel, pela conta principal: o jurado
    /// mostra `s`; seu `y` sai da lista e não entra mais (INV-07, INV-08 no
    /// modo selado: o próximo `y` da mesma lista toma o lugar).
    pub fn recuse(env: Env, jurado: Address, s: U256) {
        jurado.require_auth();
        let selo = Self::selo_aberto(&env);
        let p = env.storage().persistent();
        if p.has(&DataKey::Recusou(jurado.clone())) {
            panic_with_error!(&env, Erro::JaRecusado);
        }
        let y = Self::y_de(&env, &jurado, &s, &selo.seed);
        let mut ys = Self::claims(env.clone());
        if let Some(i) = ys.first_index_of(&y) {
            ys.remove(i);
            p.set(&DataKey::Reivindicados, &ys);
        }
        p.set(&DataKey::YUsado(y.clone()), &true);
        p.set(&DataKey::Recusou(jurado.clone()), &true);
        Recusado { jurado, s, y }.publish(&env);
    }

    /// Fecha a janela. Com `k` ou mais reivindicações, os `k` menores `y`
    /// sentam. Com menos, na primeira vez o limiar vai ao campo inteiro e a
    /// janela reabre (devolve `None`); na segunda, senta quem há e o painel
    /// fica `completo = false`. Nunca um segundo sorteio. Depois disto nada
    /// muda (INV-16). Qualquer um pode chamar.
    pub fn finalize(env: Env) -> Option<Final> {
        let mut selo = Self::selo_aberto(&env);
        if env.ledger().sequence() <= selo.prazo {
            panic_with_error!(&env, Erro::JanelaAberta);
        }
        let k = Self::closing(env.clone()).unwrap().k;
        let ys = Self::claims(env.clone());
        if ys.len() < k && !selo.ampliado {
            let janela: u32 = env.storage().instance().get(&DataKey::Janela).unwrap();
            selo.limiar = Self::r(&env);
            selo.prazo = env.ledger().sequence() + janela;
            selo.ampliado = true;
            env.storage().instance().set(&DataKey::Selo, &selo);
            Ampliado {
                limiar: selo.limiar,
                prazo: selo.prazo,
            }
            .publish(&env);
            return None;
        }
        let painel = ys.slice(0..ys.len().min(k));
        let fim = Final {
            completo: painel.len() == k,
            painel,
            ledger: env.ledger().sequence(),
        };
        env.storage().persistent().set(&DataKey::Painel, &fim);
        Finalizado {
            painel: fim.painel.clone(),
            completo: fim.completo,
        }
        .publish(&env);
        Some(fim)
    }

    /// Depois do painel e de `revela_em`, o jurado mostra `s`: o contrato
    /// confere `Poseidon(s) = c` e calcula `y = Poseidon(seed, s)`, e registra
    /// se aquele `y` sentou e se a conta ainda tem o bilhete (INV-12 no modo
    /// selado). É prova, nunca correção: o painel não muda (INV-16).
    pub fn reveal(env: Env, jurado: Address, s: U256) -> Revelacao {
        jurado.require_auth();
        let fim = match Self::panel(env.clone()) {
            Some(f) => f,
            None => panic_with_error!(&env, Erro::NaoFinalizado),
        };
        if env.ledger().sequence() < Self::closing(env.clone()).unwrap().revela_em {
            panic_with_error!(&env, Erro::CedoParaRevelar);
        }
        let p = env.storage().persistent();
        if p.has(&DataKey::Revelou(jurado.clone())) {
            panic_with_error!(&env, Erro::JaRevelado);
        }
        let seed = Self::seal_state(env.clone()).unwrap().seed;
        let y = Self::y_de(&env, &jurado, &s, &seed);
        let sentado = fim.painel.contains(&y);
        let saldo = token::TokenClient::new(&env, &Self::bilhete(&env)).balance(&jurado);
        let r = Revelacao {
            y: y.clone(),
            sentado,
            sem_bilhete: saldo < UM_BILHETE,
        };
        p.set(&DataKey::Revelou(jurado.clone()), &r);
        Revelado {
            jurado,
            s,
            y,
            sentado: r.sentado,
            sem_bilhete: r.sem_bilhete,
        }
        .publish(&env);
        r
    }

    /// O selo, se há e o painel ainda não saiu.
    fn selo_aberto(env: &Env) -> Selo {
        if Self::panel(env.clone()).is_some() {
            panic_with_error!(env, Erro::JaFinalizado);
        }
        match Self::seal_state(env.clone()) {
            Some(s) => s,
            None => panic_with_error!(env, Erro::NaoSelado),
        }
    }

    /// `Poseidon(seed, s)`, depois de conferir que `Poseidon(s)` é o
    /// compromisso do jurado.
    fn y_de(env: &Env, jurado: &Address, s: &U256, seed: &U256) -> U256 {
        let c = Self::commitment(env.clone(), jurado.clone());
        if c.is_none() || c.unwrap() != pinakion_poseidon::hash1(env, s) {
            panic_with_error!(env, Erro::SegredoNaoConfere);
        }
        pinakion_poseidon::hash2(env, seed, s)
    }

    fn r(env: &Env) -> U256 {
        U256::from_be_bytes(env, &Bytes::from_array(env, &R_BN254))
    }

    /// A primeira rodada publicada estritamente depois do instante `t`:
    /// a rodada `r` sai em `genesis + (r-1)·period`, logo
    /// `r = ⌊(t − genesis)/period⌋ + 2`. Uma rodada publicada no mesmo
    /// segundo do fechamento já podia ser conhecida; por isso "estritamente".
    fn primeira_rodada_depois_de(t: u64) -> u64 {
        t.saturating_sub(GENESIS_QUICKNET) / PERIODO_QUICKNET + 2
    }

    /// Põe `valor` na folha `indice` e refaz o caminho até a raiz: uma
    /// Poseidon(2) por nível, com o irmão lido do armazenamento ou, se nunca
    /// escrito, a subárvore vazia daquele nível.
    fn por_folha(env: &Env, indice: u32, valor: U256) {
        let p = env.storage().persistent();
        let h = Sponge::t3(env);
        let mut pos = indice;
        let mut atual = valor;
        p.set(&DataKey::No(0, pos), &atual);
        for nivel in 0..PROFUNDIDADE {
            let irmao = Self::no(env, nivel, pos ^ 1);
            atual = if pos & 1 == 0 {
                h.hash(&vec![env, atual, irmao])
            } else {
                h.hash(&vec![env, irmao, atual])
            };
            pos >>= 1;
            p.set(&DataKey::No(nivel + 1, pos), &atual);
        }
    }

    fn no(env: &Env, nivel: u32, pos: u32) -> U256 {
        env.storage()
            .persistent()
            .get(&DataKey::No(nivel, pos))
            .unwrap_or_else(|| Self::zero(env, nivel))
    }

    pub(crate) fn zero(env: &Env, nivel: u32) -> U256 {
        U256::from_be_bytes(env, &Bytes::from_array(env, &ZEROS[nivel as usize]))
    }

    /// A prova de assento vale? INV-15: contra a raiz que o fechamento
    /// guardou, com esta semente, este `y` e esta conta de assento. As
    /// entradas públicas são montadas aqui, na ordem do circuito. T-038 chama
    /// `claim` chama isto com a semente de `seal` e a conta que assina.
    pub(crate) fn prova_de_assento_vale(
        env: &Env,
        prova: &Prova,
        seed: &U256,
        y: &U256,
        assento: &Address,
    ) -> bool {
        let f = match Self::closing(env.clone()) {
            Some(f) => f,
            None => panic_with_error!(env, Erro::RolAberto),
        };
        let publicas = vec![
            env,
            f.raiz,
            seed.clone(),
            y.clone(),
            Self::claimer(env, assento),
        ];
        VerificadorClient::new(env, &Self::verifier(env.clone())).verify(prova, &publicas)
    }

    /// `SHA-256(strkey da conta) >> 8`: um elemento do campo que amarra a
    /// prova à conta que vai enviá-la (DEC-024; `circuits/pinakion.mjs`).
    pub(crate) fn claimer(env: &Env, assento: &Address) -> U256 {
        let h = env
            .crypto()
            .sha256(&assento.to_string().to_bytes())
            .to_array();
        let mut b = [0u8; 32];
        b[1..].copy_from_slice(&h[..31]);
        U256::from_be_bytes(env, &Bytes::from_array(env, &b))
    }

    fn escrivao(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Escrivao).unwrap()
    }

    fn bilhete(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Bilhete).unwrap()
    }

    // ---- cópias literais do beacon-verifier ----------------------------

    /// Verifica uma rodada do beacon contra a chave pinada da quicknet.
    /// Devolve `true` se a assinatura confere.
    pub fn verify(env: Env, round: u64, sig: BytesN<96>) -> bool {
        let bls = env.crypto().bls12_381();

        let mut preimage = Bytes::new(&env);
        preimage.extend_from_array(&round.to_be_bytes());
        let digest = env.crypto().sha256(&preimage);
        let msg: Bytes = digest.to_bytes().into();

        let dst = Bytes::from_slice(&env, DST);
        let h = bls.hash_to_g1(&msg, &dst);

        let pk = Bls12381G2Affine::from_bytes(BytesN::from_array(&env, &BEACON_PUBLIC_KEY));
        let neg_g2 = Bls12381G2Affine::from_bytes(BytesN::from_array(&env, &NEG_G2_GENERATOR));

        bls.pairing_check(
            vec![&env, h, Bls12381G1Affine::from_bytes(sig)],
            vec![&env, pk, neg_g2],
        )
    }

    /// Em que beacon este contrato confia. Leitura pública, sem argumento:
    /// a resposta é a mesma para todo mundo, para sempre.
    pub fn beacon(env: Env) -> BytesN<32> {
        BytesN::from_array(&env, &BEACON_CHAIN_HASH)
    }

    /// A aleatoriedade da rodada, como a drand a publica: o sha256 da
    /// assinatura **comprimida**.
    ///
    /// A assinatura chega aqui descomprimida, porque é isso que as funções de
    /// host da BLS12-381 recebem. Comprimir de volta é só recolocar os bits de
    /// bandeira em X: 0x80 sempre, e 0x20 quando Y é o maior dos dois valores
    /// possíveis. Sem isso o valor devolvido não seria o da drand, e a semente
    /// daqui não bateria com a do verificador independente.
    pub fn randomness(env: Env, sig: BytesN<96>) -> BytesN<32> {
        let bytes = sig.to_array();
        let mut x = [0u8; 48];
        x.copy_from_slice(&bytes[..48]);

        let mut y_e_o_maior = false;
        for i in 0..48 {
            if bytes[48 + i] != METADE_DE_P[i] {
                y_e_o_maior = bytes[48 + i] > METADE_DE_P[i];
                break;
            }
        }
        x[0] |= 0x80;
        if y_e_o_maior {
            x[0] |= 0x20;
        }

        let comprimida = Bytes::from_array(&env, &x);
        env.crypto().sha256(&comprimida).to_bytes()
    }
}

mod test;
