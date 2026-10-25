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
//! A entropia vem da drand quicknet, verificada pelas funções de host da
//! BLS12-381 (CAP-59); o embaralhamento é o PRNG do host. `verify`,
//! `randomness` e `draw` são cópias literais do `beacon-verifier`, e os
//! vetores de ouro mantêm as duas cópias honestas (DEC-014).

use pinakion_poseidon::Sponge;
use soroban_sdk::{
    contract, contracterror, contractevent, contractimpl, contracttype,
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

#[contracttype]
pub enum DataKey {
    Escrivao,
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
    /// este contrato passa a ser; `minimo` é o menor rol que pode fechar.
    pub fn __constructor(env: Env, escrivao: Address, bilhete: Address, minimo: u32) {
        let s = env.storage().instance();
        s.set(&DataKey::Escrivao, &escrivao);
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
        let r = U256::from_be_bytes(&env, &Bytes::from_array(&env, &R_BN254));
        if c == U256::from_u32(&env, 0) || c >= r {
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

    /// O painel, se já houve sorteio.
    pub fn panel(env: Env) -> Option<Vec<Address>> {
        env.storage().persistent().get(&DataKey::Painel)
    }

    /// Sorteia os `k` assentos com a rodada que o fechamento fixou, uma vez
    /// só (INV-13): verifica a assinatura dessa rodada, deriva a semente,
    /// embaralha a ordem do rol e senta os `k` primeiros que têm bilhete e
    /// autorização. Qualquer um pode chamar: não há o que escolher.
    pub fn seal(env: Env, sig: BytesN<96>) -> Vec<Address> {
        let f = match Self::closing(env.clone()) {
            Some(f) => f,
            None => panic_with_error!(&env, Erro::RolAberto),
        };
        if Self::panel(env.clone()).is_some() {
            panic_with_error!(&env, Erro::JaSelado);
        }
        if !Self::verify(env.clone(), f.rodada, sig.clone()) {
            panic_with_error!(&env, Erro::AssinaturaInvalida);
        }
        let seed = Self::randomness(env.clone(), sig);
        let rol = Self::roster(env.clone());
        let ordem = Self::draw(env.clone(), seed, rol.len(), rol.len());

        let bilhete = Self::bilhete(&env);
        let sac = token::StellarAssetClient::new(&env, &bilhete);
        let saldo = token::TokenClient::new(&env, &bilhete);
        let mut painel: Vec<Address> = Vec::new(&env);
        for i in ordem.iter() {
            if painel.len() == f.k {
                break;
            }
            let j = rol.get(i).unwrap();
            if sac.authorized(&j) && saldo.balance(&j) >= UM_BILHETE {
                painel.push_back(j);
            }
        }
        if painel.len() < f.k {
            panic_with_error!(&env, Erro::ElegiveisInsuficientes);
        }
        env.storage().persistent().set(&DataKey::Painel, &painel);
        painel
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

    /// Sorteia `k` posições de um rol de tamanho `n`, usando a aleatoriedade
    /// do beacon como semente. Fisher-Yates, determinístico e recomputável.
    pub fn draw(env: Env, seed: BytesN<32>, n: u32, k: u32) -> soroban_sdk::Vec<u32> {
        let prng = env.prng();
        prng.seed(seed.into());
        let mut roll: soroban_sdk::Vec<u32> = soroban_sdk::Vec::new(&env);
        for i in 0..n {
            roll.push_back(i);
        }
        prng.shuffle(&mut roll);
        let mut out: soroban_sdk::Vec<u32> = soroban_sdk::Vec::new(&env);
        for i in 0..k {
            out.push_back(roll.get(i).unwrap());
        }
        out
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
