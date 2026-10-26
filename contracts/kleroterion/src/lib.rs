#![no_std]
//! Kleroterion: o roster de jurados como um ativo clássico da Stellar.
//!
//! O *pinakion* era um ticket de bronze. Aqui ele é uma unidade de um ativo
//! clássico, com `AUTH_REQUIRED` no emissor: ter o ticket e estar autorizado
//! é o que dá seat. O roster é a lista de trustlines autorizadas, legível por
//! qualquer Horizon em `/accounts?asset=`; a ordem do roster é a ordem de
//! inscrição guardada aqui; o compromisso é `lock()`, depois do qual ninguém
//! entra nem sai — e o emissor, com master weight 0, não emite nem autoriza
//! mais nada. O `lock()` também fixa a round do beacon e o número de
//! assentos; `seal()` usa essa round uma única vez (INV-13, INV-14).
//!
//! Cada juror se inscreve com um compromisso `c = Poseidon(s)` (T-035). O
//! contrato mantém a árvore de Merkle de Poseidon, profundidade 6, cuja folha
//! `i` é o `c` do `i`-ésimo inscrito, ou zero se ele foi impedido; o caminho
//! é refeito a cada `enroll` e `impede`, e o `lock()` congela a root. Ninguém
//! posta root: ela é sempre a que o contrato calculou.
//!
//! Uma proof de seat (T-037) é conferida por um contrato verificador à
//! parte, cujo endereço o construtor fixa e cuja chave está embutida nele.
//! As entradas públicas `[root, seed, y, claimer]` são montadas aqui: a root
//! do fechamento, a semente, o `y` declarado e `SHA-256(strkey) >> 8` da
//! conta de seat — ninguém entrega ao verificador um vetor pronto.
//!
//! A entropia vem da drand quicknet, verificada pelas funções de host da
//! BLS12-381 (CAP-59). `verify` e `randomness` são cópias literais do
//! `beacon-verifier`. O sorteio é selado (T-038, DEC-015): `seal` fixa a
//! semente e o threshold `T`; cada juror com `y = Poseidon(seed, s) < T` proof
//! seat de uma conta de seat sem dizer quem é; `finalize` senta os `k`
//! menores `y`; depois de `reveal_at`, cada um revela `s` e o seats fica
//! recomputável entre os que revelaram. O Fisher-Yates aberto fica só no
//! `beacon-verifier` (DEC-016).
//!
//! O caso em volta do sorteio (T-044, T-045): o construtor fixa uma tabela de
//! severity — para cada nível, `k` e o roster mínimo —, e o `lock()` recebe o
//! nível e a pessoa julgada. `k` vem da tabela, não de quem fecha (INV-19);
//! a pessoa julgada, se inscrita, é impedida no próprio fechamento e nunca
//! senta no próprio caso (INV-18); e `suspended()` diz, a quem quiser ler,
//! que ela está afastada até um deadline também fixado no construtor (INV-20).
//!
//! A revelação tardia (T-046): o construtor fixa `minimum_wait`, e o
//! `lock()` recusa um `reveal_at` mais cedo que isso. Como a revelação pode
//! vir meses depois, o `lock()` e o `finalize()` estendem o TTL de tudo o que
//! `reveal` lê até `reveal_at + window`, no máximo o teto da rede (INV-21).

use pinakion_poseidon::Sponge;
use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype,
    crypto::bls12_381::{Bls12381G1Affine, Bls12381G2Affine},
    panic_with_error, token, vec, Address, Bytes, BytesN, Env, Vec, U256,
};

/// Domain separation tag do drand quicknet.
const DST: &[u8] = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";

/// Gerador de G2 negado, codificação não compressed.
pub const NEG_G2_GENERATOR: [u8; 192] = *include_bytes!("neg_g2.bin");

/// Chave pública do grupo de threshold da quicknet, pinada (INV-10).
pub const BEACON_PUBLIC_KEY: [u8; 192] = *include_bytes!("quicknet_pk.bin");

/// Hash da chain quicknet, publicado por `beacon()`.
pub const BEACON_CHAIN_HASH: [u8; 32] = *include_bytes!("quicknet_chain.bin");

/// Genesis e período da quicknet, em segundos (§5). A round `r` é publicada
/// em `genesis + (r-1)·period`.
pub const QUICKNET_GENESIS: u64 = 1_692_803_367;
pub const QUICKNET_PERIOD: u64 = 3;

/// Rodadas entre a primeira publicada depois do fechamento e a que o sorteio
/// usa. Dez rodadas são 30 s: a transação do `lock()` tem time de entrar num
/// ledger e ser vista antes que a round target exista (§9 T-033: ≥ 10).
pub const ROUND_MARGIN: u64 = 10;

/// Um ticket: 1.0000000 PINAKION, em stroops. Ter menos que isto é não ter
/// ticket (INV-12).
pub const ONE_TICKET: i128 = 10_000_000;

/// Profundidade da árvore de compromissos: 2⁶ = 64 jurados no máximo. É a do
/// circuito (`circuits/pinakion_claim.circom`).
pub const DEPTH: u32 = 6;

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
const HALF_OF_P: [u8; 48] = [
    0x0d, 0x00, 0x88, 0xf5, 0x1c, 0xbf, 0xf3, 0x4d, 0x25, 0x8d, 0xd3, 0xdb, 0x21, 0xa5, 0xd6, 0x6b,
    0xb2, 0x3b, 0xa5, 0xc2, 0x79, 0xc2, 0x89, 0x5f, 0xb3, 0x98, 0x69, 0x50, 0x7b, 0x58, 0x7b, 0x12,
    0x0f, 0x55, 0xff, 0xff, 0x58, 0xa9, 0xff, 0xff, 0xdc, 0xff, 0x7f, 0xff, 0xff, 0xff, 0xd5, 0x55,
];

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// A round pedida é anterior à que o fechamento fixou (INV-06). Desde
    /// T-033 ninguém pede round, então nada mais devolve este erro; o número
    /// fica reservado para não mudar o dos outros.
    RoundBeforeRoster = 1,
    /// A assinatura não é da quicknet, ou não é da round que o fechamento
    /// fixou.
    InvalidSignature = 2,
    /// O roster já fechou: ninguém mais entra (INV-11).
    RosterClosed = 3,
    /// O roster ainda está aberto: não há o que sortear.
    RosterOpen = 4,
    /// Fechar abaixo do mínimo do roster (INV-09).
    RosterBelowMinimum = 5,
    /// O juror já está inscrito.
    DuplicateJuror = 6,
    /// Não há jurados elegíveis suficientes para `k` assentos.
    NotEnoughEligible = 7,
    /// `k` é zero ou maior que o roster.
    InvalidSeatCount = 8,
    /// O sorteio já foi feito: não há segundo (INV-13).
    AlreadySealed = 9,
    /// O compromisso é zero (a folha vazia) ou não é elemento do campo.
    InvalidCommitment = 10,
    /// Este compromisso já foi usado, por quem quer que seja (INV-17).
    DuplicateCommitment = 11,
    /// A árvore tem 2⁶ folhas e estão todas ocupadas.
    RosterFull = 12,
    /// Impedir quem não está no roster.
    UnknownJuror = 13,
    /// O juror já está impedido.
    AlreadyImpeded = 14,
    /// Ainda não há seal: nem semente nem threshold.
    NotSealed = 15,
    /// `y` não está abaixo do threshold.
    YAboveThreshold = 16,
    /// Este `y` já foi reivindicado, ou é de quem se recusou (INV-15).
    DuplicateY = 17,
    /// A proof não vale contra a root, a semente, o `y` e a conta.
    InvalidProof = 18,
    /// A window de reivindicação ainda está aberta.
    WindowOpen = 19,
    /// O seats já saiu (INV-16).
    AlreadyFinalized = 20,
    /// O seats ainda não saiu.
    NotFinalized = 21,
    /// O ledger de revelação (`reveal_at`) ainda não chegou.
    TooEarlyToReveal = 22,
    /// `Poseidon(s)` não é o compromisso deste juror.
    SecretDoesNotMatch = 23,
    /// Este juror já revelou.
    AlreadyRevealed = 24,
    /// Este juror já se recusou.
    AlreadyRecused = 25,
    /// A window de reivindicação fechou.
    WindowClosed = 26,
    /// O nível de severity não está na tabela do construtor.
    InvalidSeverity = 27,
    /// Quem fecha o roster não pode ser a pessoa julgada (INV-18).
    ClerkIsParty = 28,
    /// A tabela de severity é vazia ou tem um nível com `k = 0`.
    InvalidTable = 29,
    /// `reveal_at` vem antes de `lock + minimum_wait` (INV-21).
    RevealTooEarly = 30,
    /// Esta conta de seat já reivindicou um seat (T-047).
    DuplicateSeat = 31,
}

/// Um nível da tabela de severity: quantos assentos e o menor roster que pode
/// fechar com ele (INV-09, INV-19). O nível `g` é a posição `g - 1`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Level {
    pub k: u32,
    pub roster_minimum: u32,
}

/// O que `lock()` fixa, de uma vez por todas: quando, qual round do beacon
/// decide, quantos assentos, o ledger a partir do qual os jurados revelam
/// (`reveal_at`, usado a partir de T-038; Q-24), a root da árvore de
/// compromissos e `n`, quantos estavam authorized; e, desde T-045, o nível
/// de severity, a pessoa julgada e o ledger em que o suspension dela acaba.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Closing {
    pub ledger: u32,
    pub time: u64,
    pub round: u64,
    pub k: u32,
    pub reveal_at: u32,
    pub root: U256,
    pub n: u32,
    pub severity: u32,
    pub accused: Address,
    pub suspended_until: u32,
}

/// O construtor publica a regra antes de qualquer caso: a tabela de
/// severity e o suspension, em ledgers. Com este evento, quem lê o ledger
/// confere que o `k` do `Closed` é o do nível (T-045).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Constituted {
    pub levels: Vec<Level>,
    pub suspension: u32,
    pub minimum_wait: u32,
}

/// Um juror entrou no roster, na posição `index`, com o compromisso `c`. Com
/// estes eventos e os de `Impeded`, qualquer um refaz a root
/// (scripts/arvore.py, web/arvore.mjs).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Enrolled {
    #[topic]
    pub juror: Address,
    pub index: u32,
    pub c: U256,
}

/// Um juror foi impedido antes do fechamento: a folha `index` vira zero.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Impeded {
    #[topic]
    pub juror: Address,
    pub index: u32,
}

/// O que `seal` fixa: a semente, o threshold `T` e o último ledger em que se
/// reivindica. `widened` diz se `finalize` já levou `T` ao campo inteiro.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Seal {
    pub seed: U256,
    pub threshold: U256,
    pub deadline: u32,
    pub widened: bool,
}

/// O seats: os `y` que sentam, em ordem, e se chegou a `k`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Panel {
    pub seats: Vec<U256>,
    pub complete: bool,
    pub ledger: u32,
}

/// O que um `reveal` provou de um juror.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revelation {
    pub y: U256,
    pub seated: bool,
    pub without_ticket: bool,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Sealed {
    pub seed: U256,
    pub threshold: U256,
    pub deadline: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Widened {
    pub threshold: U256,
    pub deadline: u32,
}

/// Um seat reivindicado. Só o `y`: nenhum endereço ao lado dele (INV-17).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Claimed {
    pub y: U256,
}

/// Um juror se recusou, às claras, antes do seats. Recusar é revelar: o
/// evento leva `s` e o `y` que saiu, para que quem lê o ledger confira
/// `Poseidon(s) = c` e `y = Poseidon(seed, s)` sem confiar no contrato (T-039).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Recused {
    #[topic]
    pub juror: Address,
    pub s: U256,
    pub y: U256,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Finalized {
    pub seats: Vec<U256>,
    pub complete: bool,
}

/// Depois do veredito, o elo entre um juror e seu `y` — de propósito. Leva
/// `s`, para que a recomputação não dependa do contrato (T-039).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Revealed {
    #[topic]
    pub juror: Address,
    pub s: U256,
    pub y: U256,
    pub seated: bool,
    pub without_ticket: bool,
}

/// O roster fechou com esta root, `n` authorized, esta round e `k` assentos,
/// para julgar `accused` no nível `severity`, afastado até `suspended_until`.
/// O nível é um teto: o seats não decide acima dele (T-044 §1).
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Closed {
    #[topic]
    pub accused: Address,
    pub root: U256,
    pub n: u32,
    pub round: u64,
    pub k: u32,
    pub reveal_at: u32,
    pub severity: u32,
    pub suspended_until: u32,
}

/// Uma proof Groth16 no formato do verificador (`contracts/groth16-verifier`):
/// A e C em G1 (64 bytes), B em G2 (128 bytes). Mesmos campos, mesmo XDR.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Proof {
    pub a: BytesN<64>,
    pub b: BytesN<128>,
    pub c: BytesN<64>,
}

/// O que o kleroterion pede ao verificador. Declarado aqui para não embutir
/// o código do verificador neste contrato.
#[contractclient(name = "VerifierClient")]
pub trait Verifier {
    fn verify(env: Env, proof: Proof, public: Vec<U256>) -> bool;
}

#[contracttype]
pub enum DataKey {
    Clerk,
    Verifier,
    Window,
    Seal,
    /// Os `y` reivindicados, em ordem crescente.
    Claims,
    /// Um `y` já usado: reivindicado, ou de quem se recusou.
    YUsed(U256),
    Recused(Address),
    Revealed(Address),
    Ticket,
    /// A tabela de severity.
    Levels,
    /// Quantos ledgers dura o suspension da pessoa julgada.
    Suspension,
    /// O menor intervalo, em ledgers, entre o fechamento e a revelação.
    MinimumWait,
    Roster,
    Closing,
    Panel,
    /// Quantos inscritos continuam authorized.
    Authorized,
    /// O compromisso de um juror.
    Commitment(Address),
    /// A posição de um juror na ordem de inscrição — e na árvore.
    Index(Address),
    /// Um compromisso já usado, para sempre.
    Used(U256),
    /// Nó da árvore: (nível, posição). Ausente é a subárvore vazia.
    Node(u32, u32),
    /// O `y` que uma conta de seat reivindicou (T-047). A conta já é a
    /// fonte pública da transação do `claim`; guardar o elo não revela nada
    /// novo, e deixa o processo conferir quem sentou sem saber quem é.
    YOf(Address),
}

#[contract]
pub struct Kleroterion;

#[contractimpl]
impl Kleroterion {
    /// `clerk` inscreve e fecha; `ticket` é o SAC do ativo cujo admin
    /// este contrato passa a ser; `verificador` é o contrato Groth16 que
    /// confere as provas de seat, fixado aqui para sempre (T-037);
    /// `window` é quantos ledgers, depois do seal, os jurados têm para
    /// reivindicar (T-038). `levels` é a tabela de severity — o nível `g`
    /// senta `levels[g-1].k` e só fecha com pelo menos `roster_minimum` inscritos
    /// —, e `suspension` é quantos ledgers a pessoa julgada fica afastada a
    /// partir do fechamento (T-045). `minimum_wait` é o menor intervalo, em
    /// ledgers, entre o fechamento e `reveal_at` (T-046). Os três ficam
    /// publicados em `Constituted`.
    #[allow(clippy::too_many_arguments)]
    pub fn __constructor(
        env: Env,
        clerk: Address,
        ticket: Address,
        verificador: Address,
        window: u32,
        levels: Vec<Level>,
        suspension: u32,
        minimum_wait: u32,
    ) {
        if levels.is_empty() || levels.iter().any(|n| n.k == 0) {
            panic_with_error!(&env, Error::InvalidTable);
        }
        let s = env.storage().instance();
        s.set(&DataKey::Clerk, &clerk);
        s.set(&DataKey::Window, &window);
        s.set(&DataKey::Verifier, &verificador);
        s.set(&DataKey::Ticket, &ticket);
        s.set(&DataKey::Levels, &levels);
        s.set(&DataKey::Suspension, &suspension);
        s.set(&DataKey::MinimumWait, &minimum_wait);
        s.set(&DataKey::Authorized, &0u32);
        let empty: Vec<Address> = Vec::new(&env);
        env.storage().persistent().set(&DataKey::Roster, &empty);
        Constituted {
            levels,
            suspension,
            minimum_wait,
        }
        .publish(&env);
    }

    /// Inscreve um juror: cria a trustline (SAC `trust`, CAP-73), autoriza e
    /// emite um ticket. O escrivão e o juror assinam — o juror consente em
    /// servir; é a parte afetada pelo caso que nunca consentiu com o foro.
    ///
    /// `c` é o compromisso do juror, `Poseidon(s)` para um segredo `s` que só
    /// ele guarda. Fica preso ao endereço, vira a folha da posição de
    /// inscrição, e nunca mais é aceito de ninguém (INV-17).
    pub fn enroll(env: Env, juror: Address, c: U256) {
        Self::clerk(&env).require_auth();
        juror.require_auth();
        if Self::closing(env.clone()).is_some() {
            panic_with_error!(&env, Error::RosterClosed);
        }
        let mut roster = Self::roster(env.clone());
        if roster.contains(&juror) {
            panic_with_error!(&env, Error::DuplicateJuror);
        }
        if c == U256::from_u32(&env, 0) || c >= Self::r(&env) {
            panic_with_error!(&env, Error::InvalidCommitment);
        }
        let p = env.storage().persistent();
        if p.has(&DataKey::Used(c.clone())) {
            panic_with_error!(&env, Error::DuplicateCommitment);
        }
        let index = roster.len();
        if index >= 1 << DEPTH {
            panic_with_error!(&env, Error::RosterFull);
        }
        let sac = token::StellarAssetClient::new(&env, &Self::ticket(&env));
        sac.trust(&juror);
        sac.set_authorized(&juror, &true);
        sac.mint(&juror, &ONE_TICKET);
        roster.push_back(juror.clone());
        p.set(&DataKey::Roster, &roster);
        p.set(&DataKey::Used(c.clone()), &true);
        p.set(&DataKey::Commitment(juror.clone()), &c);
        p.set(&DataKey::Index(juror.clone()), &index);
        Self::per_leaf(&env, index, c.clone());
        let i = env.storage().instance();
        let authorized: u32 = i.get(&DataKey::Authorized).unwrap();
        i.set(&DataKey::Authorized, &(authorized + 1));
        Enrolled { juror, index, c }.publish(&env);
    }

    /// Fecha o roster para julgar `accused` no nível `severity`: a partir daqui
    /// ninguém entra nem sai (INV-11, INV-14). A round do beacon fica fixada
    /// pelo relógio do ledger, não por quem chama (INV-06, INV-13); `k` vem
    /// da tabela do construtor, não de quem chama (INV-19), e fica fixado
    /// aqui, antes de a round existir, e a root da árvore com ele.
    ///
    /// Se `accused` está inscrito e autorizado, é impedido nesta mesma
    /// chamada, antes de a root ser fixada: a folha dele vira zero e ele
    /// nunca senta no próprio caso (INV-18). Quem fecha não pode ser ele.
    /// Recusa abaixo do roster mínimo do nível (INV-09), `k` acima de `N`, sendo
    /// `N` os inscritos ainda authorized, e `reveal_at` antes de
    /// `ledger + minimum_wait` (INV-21). Estende o TTL da instância e do
    /// compromisso de cada inscrito até a revelação.
    pub fn lock(env: Env, severity: u32, reveal_at: u32, accused: Address) -> Closing {
        let clerk = Self::clerk(&env);
        clerk.require_auth();
        if Self::closing(env.clone()).is_some() {
            panic_with_error!(&env, Error::RosterClosed);
        }
        if accused == clerk {
            panic_with_error!(&env, Error::ClerkIsParty);
        }
        let level = match Self::levels(env.clone()).get(severity.wrapping_sub(1)) {
            Some(n) if severity > 0 => n,
            _ => panic_with_error!(&env, Error::InvalidSeverity),
        };
        if Self::roster(env.clone()).len() < level.roster_minimum {
            panic_with_error!(&env, Error::RosterBelowMinimum);
        }
        if let Some(index) = Self::index(&env, &accused) {
            let sac = token::StellarAssetClient::new(&env, &Self::ticket(&env));
            if sac.authorized(&accused) {
                Self::impede_leaf(&env, &sac, accused.clone(), index);
            }
        }
        let n: u32 = env.storage().instance().get(&DataKey::Authorized).unwrap();
        let k = level.k;
        if k > n {
            panic_with_error!(&env, Error::InvalidSeatCount);
        }
        let suspension: u32 = env.storage().instance().get(&DataKey::Suspension).unwrap();
        let time = env.ledger().timestamp();
        let ledger = env.ledger().sequence();
        let wait: u32 = env.storage().instance().get(&DataKey::MinimumWait).unwrap();
        if reveal_at < ledger.saturating_add(wait) {
            panic_with_error!(&env, Error::RevealTooEarly);
        }
        let f = Closing {
            ledger,
            time,
            round: Self::first_round_after(time) + ROUND_MARGIN,
            k,
            reveal_at,
            root: Self::root(env.clone()),
            n,
            severity,
            accused,
            suspended_until: ledger.saturating_add(suspension),
        };
        env.storage().instance().set(&DataKey::Closing, &f);
        let target = Self::until_the_reveal(&env, &f);
        env.storage().instance().extend_ttl(target, target);
        let p = env.storage().persistent();
        for juror in Self::roster(env.clone()).iter() {
            p.extend_ttl(&DataKey::Commitment(juror), target, target);
        }
        Closed {
            accused: f.accused.clone(),
            root: f.root.clone(),
            n: f.n,
            round: f.round,
            k: f.k,
            reveal_at: f.reveal_at,
            severity: f.severity,
            suspended_until: f.suspended_until,
        }
        .publish(&env);
        f
    }

    /// A pessoa julgada está afastada? Verdadeiro só para ela, do fechamento
    /// até `suspended_until`, exclusive; falso para todo o resto, e nada o
    /// prorroga (INV-20). É uma regra publicada: tirar os poderes dela é de
    /// quem lê isto — um contrato da governança, um bot do projeto.
    pub fn suspended(env: Env, address: Address) -> bool {
        match Self::closing(env.clone()) {
            Some(f) => f.accused == address && env.ledger().sequence() < f.suspended_until,
            None => false,
        }
    }

    /// Esta conta de seat sentou? Verdadeiro só depois do seats, e só se
    /// o `y` que ela reivindicou está nele (T-047). É o que o processo usa
    /// para o quórum: confere o seat sem saber de quem ele é.
    pub fn seated(env: Env, seat: Address) -> bool {
        let end = match Self::panel(env.clone()) {
            Some(f) => f,
            None => return false,
        };
        match env
            .storage()
            .persistent()
            .get::<_, U256>(&DataKey::YOf(seat))
        {
            Some(y) => end.seats.contains(&y),
            None => false,
        }
    }

    /// A tabela de severity fixada no construtor.
    pub fn levels(env: Env) -> Vec<Level> {
        env.storage().instance().get(&DataKey::Levels).unwrap()
    }

    /// Impedimento ou recusa (art. 463): tira a autorização do ticket. O
    /// juror continua no roster e na ordem; só não senta (INV-12), e o
    /// substituto é o próximo da ordem sorteada (INV-08). Só antes do
    /// fechamento: depois dele o escrivão não escolhe quem sai (INV-14, Q-21).
    /// A folha do impedido vira zero: ele não tem como provar seat.
    pub fn impede(env: Env, juror: Address) {
        Self::clerk(&env).require_auth();
        if Self::closing(env.clone()).is_some() {
            panic_with_error!(&env, Error::RosterClosed);
        }
        let index = match Self::index(&env, &juror) {
            Some(i) => i,
            None => panic_with_error!(&env, Error::UnknownJuror),
        };
        let sac = token::StellarAssetClient::new(&env, &Self::ticket(&env));
        if !sac.authorized(&juror) {
            panic_with_error!(&env, Error::AlreadyImpeded);
        }
        Self::impede_leaf(&env, &sac, juror, index);
    }

    /// Desautoriza o ticket, zera a folha, desconta de `N` e publica
    /// `Impeded`. Used por `impede` e pelo `lock()` com a pessoa julgada.
    fn impede_leaf(env: &Env, sac: &token::StellarAssetClient, juror: Address, index: u32) {
        sac.set_authorized(&juror, &false);
        Self::per_leaf(env, index, U256::from_u32(env, 0));
        let i = env.storage().instance();
        let authorized: u32 = i.get(&DataKey::Authorized).unwrap();
        i.set(&DataKey::Authorized, &(authorized - 1));
        Impeded { juror, index }.publish(env);
    }

    fn index(env: &Env, juror: &Address) -> Option<u32> {
        env.storage()
            .persistent()
            .get(&DataKey::Index(juror.clone()))
    }

    /// O compromisso de um juror, se inscrito.
    pub fn commitment(env: Env, juror: Address) -> Option<U256> {
        env.storage().persistent().get(&DataKey::Commitment(juror))
    }

    /// A root da árvore de compromissos agora. Depois do `lock()` é a mesma
    /// de `closing().root`, porque nada mais entra nem sai.
    pub fn root(env: Env) -> U256 {
        Self::no(&env, DEPTH, 0)
    }

    /// O roster, na ordem de inscrição. É esta ordem que o sorteio embaralha.
    pub fn roster(env: Env) -> Vec<Address> {
        env.storage().persistent().get(&DataKey::Roster).unwrap()
    }

    /// O fechamento, se já houve.
    pub fn closing(env: Env) -> Option<Closing> {
        env.storage().instance().get(&DataKey::Closing)
    }

    /// O contrato que confere as provas de seat.
    pub fn verifier(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Verifier).unwrap()
    }

    /// O seats, se já saiu.
    pub fn panel(env: Env) -> Option<Panel> {
        env.storage().persistent().get(&DataKey::Panel)
    }

    /// O seal, se já houve.
    pub fn seal_state(env: Env) -> Option<Seal> {
        env.storage().instance().get(&DataKey::Seal)
    }

    /// Os `y` reivindicados até agora, em ordem crescente.
    pub fn claims(env: Env) -> Vec<U256> {
        env.storage()
            .persistent()
            .get(&DataKey::Claims)
            .unwrap_or(Vec::new(&env))
    }

    /// O que o `reveal` de um juror provou, se ele revelou.
    pub fn revelation(env: Env, juror: Address) -> Option<Revelation> {
        env.storage().persistent().get(&DataKey::Revealed(juror))
    }

    /// Verifica a round que o fechamento fixou, uma vez só (INV-13), e fixa
    /// a semente — a aleatoriedade da round deslocada 8 bits, para ser
    /// elemento do campo — e o threshold `T = ⌊r·2k/N⌋`, o campo inteiro se
    /// `2k ≥ N`. Abre a window de reivindicação. Qualquer um pode chamar.
    pub fn seal(env: Env, sig: BytesN<96>) -> Seal {
        let f = match Self::closing(env.clone()) {
            Some(f) => f,
            None => panic_with_error!(&env, Error::RosterOpen),
        };
        if Self::seal_state(env.clone()).is_some() {
            panic_with_error!(&env, Error::AlreadySealed);
        }
        if !Self::verify(env.clone(), f.round, sig.clone()) {
            panic_with_error!(&env, Error::InvalidSignature);
        }
        let random: Bytes = Self::randomness(env.clone(), sig).into();
        let seed = U256::from_be_bytes(&env, &random).shr(8);
        let r = Self::r(&env);
        let (n, dois_k) = (f.n, 2 * f.k);
        let threshold = if dois_k >= n {
            r
        } else {
            let n_ = U256::from_u32(&env, n);
            let k_ = U256::from_u32(&env, dois_k);
            r.div(&n_)
                .mul(&k_)
                .add(&r.rem_euclid(&n_).mul(&k_).div(&n_))
        };
        let window: u32 = env.storage().instance().get(&DataKey::Window).unwrap();
        let seal = Seal {
            seed,
            threshold,
            deadline: env.ledger().sequence() + window,
            widened: false,
        };
        env.storage().instance().set(&DataKey::Seal, &seal);
        Sealed {
            seed: seal.seed.clone(),
            threshold: seal.threshold.clone(),
            deadline: seal.deadline,
        }
        .publish(&env);
        seal
    }

    /// Reivindica um seat, da conta de seat que assina: `y` abaixo do
    /// threshold, ainda não usado, e a proof de que `y = Poseidon(seed, s)` para
    /// algum `s` cuja `Poseidon(s)` é folha da root (INV-15). O contrato monta
    /// as entradas públicas; o evento leva só o `y` (INV-17).
    pub fn claim(env: Env, seat: Address, proof: Proof, y: U256) {
        seat.require_auth();
        let seal = Self::open_seal(&env);
        if env.ledger().sequence() > seal.deadline {
            panic_with_error!(&env, Error::WindowClosed);
        }
        if y >= seal.threshold {
            panic_with_error!(&env, Error::YAboveThreshold);
        }
        let p = env.storage().persistent();
        if p.has(&DataKey::YUsed(y.clone())) {
            panic_with_error!(&env, Error::DuplicateY);
        }
        if p.has(&DataKey::YOf(seat.clone())) {
            panic_with_error!(&env, Error::DuplicateSeat);
        }
        if !Self::prova_de_assento_vale(&env, &proof, &seal.seed, &y, &seat) {
            panic_with_error!(&env, Error::InvalidProof);
        }
        p.set(&DataKey::YUsed(y.clone()), &true);
        p.set(&DataKey::YOf(seat.clone()), &y);
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
        p.set(&DataKey::Claims, &ys);
        Claimed { y }.publish(&env);
    }

    /// Recusa às claras, antes do seats, pela conta principal: o juror
    /// mostra `s`; seu `y` sai da lista e não entra mais (INV-07, INV-08 no
    /// modo selado: o próximo `y` da mesma lista toma o lugar).
    pub fn recuse(env: Env, juror: Address, s: U256) {
        juror.require_auth();
        let seal = Self::open_seal(&env);
        let p = env.storage().persistent();
        if p.has(&DataKey::Recused(juror.clone())) {
            panic_with_error!(&env, Error::AlreadyRecused);
        }
        let y = Self::y_of(&env, &juror, &s, &seal.seed);
        let mut ys = Self::claims(env.clone());
        if let Some(i) = ys.first_index_of(&y) {
            ys.remove(i);
            p.set(&DataKey::Claims, &ys);
        }
        p.set(&DataKey::YUsed(y.clone()), &true);
        p.set(&DataKey::Recused(juror.clone()), &true);
        Recused { juror, s, y }.publish(&env);
    }

    /// Fecha a window. Com `k` ou mais reivindicações, os `k` menores `y`
    /// sentam. Com menos, na primeira vez o threshold vai ao campo inteiro e a
    /// window reabre (devolve `None`); na segunda, senta quem há e o seats
    /// fica `complete = false`. Nunca um segundo sorteio. Depois disto nada
    /// muda (INV-16). Qualquer um pode chamar.
    pub fn finalize(env: Env) -> Option<Panel> {
        let mut seal = Self::open_seal(&env);
        if env.ledger().sequence() <= seal.deadline {
            panic_with_error!(&env, Error::WindowOpen);
        }
        let k = Self::closing(env.clone()).unwrap().k;
        let ys = Self::claims(env.clone());
        if ys.len() < k && !seal.widened {
            let window: u32 = env.storage().instance().get(&DataKey::Window).unwrap();
            seal.threshold = Self::r(&env);
            seal.deadline = env.ledger().sequence() + window;
            seal.widened = true;
            env.storage().instance().set(&DataKey::Seal, &seal);
            Widened {
                threshold: seal.threshold,
                deadline: seal.deadline,
            }
            .publish(&env);
            return None;
        }
        let seats = ys.slice(0..ys.len().min(k));
        let end = Panel {
            complete: seats.len() == k,
            seats,
            ledger: env.ledger().sequence(),
        };
        env.storage().persistent().set(&DataKey::Panel, &end);
        let target = Self::until_the_reveal(&env, &Self::closing(env.clone()).unwrap());
        env.storage().instance().extend_ttl(target, target);
        env.storage()
            .persistent()
            .extend_ttl(&DataKey::Panel, target, target);
        Finalized {
            seats: end.seats.clone(),
            complete: end.complete,
        }
        .publish(&env);
        Some(end)
    }

    /// Depois do seats e de `reveal_at`, o juror mostra `s`: o contrato
    /// confere `Poseidon(s) = c` e calcula `y = Poseidon(seed, s)`, e registra
    /// se aquele `y` sentou e se a conta ainda tem o ticket (INV-12 no modo
    /// selado). É proof, nunca correção: o seats não muda (INV-16).
    pub fn reveal(env: Env, juror: Address, s: U256) -> Revelation {
        juror.require_auth();
        let end = match Self::panel(env.clone()) {
            Some(f) => f,
            None => panic_with_error!(&env, Error::NotFinalized),
        };
        if env.ledger().sequence() < Self::closing(env.clone()).unwrap().reveal_at {
            panic_with_error!(&env, Error::TooEarlyToReveal);
        }
        let p = env.storage().persistent();
        if p.has(&DataKey::Revealed(juror.clone())) {
            panic_with_error!(&env, Error::AlreadyRevealed);
        }
        let seed = Self::seal_state(env.clone()).unwrap().seed;
        let y = Self::y_of(&env, &juror, &s, &seed);
        let seated = end.seats.contains(&y);
        let balance = token::TokenClient::new(&env, &Self::ticket(&env)).balance(&juror);
        let r = Revelation {
            y: y.clone(),
            seated,
            without_ticket: balance < ONE_TICKET,
        };
        p.set(&DataKey::Revealed(juror.clone()), &r);
        Revealed {
            juror,
            s,
            y,
            seated: r.seated,
            without_ticket: r.without_ticket,
        }
        .publish(&env);
        r
    }

    /// Quantos ledgers, a partir de agora, o que `reveal` lê precisa viver:
    /// até `reveal_at + window`, e nunca além do teto da rede (INV-21). Se a
    /// revelação passar do teto, alguém precisa estender de novo no caminho
    /// — qualquer conta pode, `ExtendFootprintTTLOp` não tem controle de acesso.
    fn until_the_reveal(env: &Env, f: &Closing) -> u32 {
        let window: u32 = env.storage().instance().get(&DataKey::Window).unwrap();
        f.reveal_at
            .saturating_add(window)
            .saturating_sub(env.ledger().sequence())
            .min(env.storage().max_ttl())
    }

    /// O seal, se há e o seats ainda não saiu.
    fn open_seal(env: &Env) -> Seal {
        if Self::panel(env.clone()).is_some() {
            panic_with_error!(env, Error::AlreadyFinalized);
        }
        match Self::seal_state(env.clone()) {
            Some(s) => s,
            None => panic_with_error!(env, Error::NotSealed),
        }
    }

    /// `Poseidon(seed, s)`, depois de conferir que `Poseidon(s)` é o
    /// compromisso do juror.
    fn y_of(env: &Env, juror: &Address, s: &U256, seed: &U256) -> U256 {
        let c = Self::commitment(env.clone(), juror.clone());
        if c.is_none() || c.unwrap() != pinakion_poseidon::hash1(env, s) {
            panic_with_error!(env, Error::SecretDoesNotMatch);
        }
        pinakion_poseidon::hash2(env, seed, s)
    }

    fn r(env: &Env) -> U256 {
        U256::from_be_bytes(env, &Bytes::from_array(env, &R_BN254))
    }

    /// A primeira round publicada estritamente depois do instante `t`:
    /// a round `r` sai em `genesis + (r-1)·period`, logo
    /// `r = ⌊(t − genesis)/period⌋ + 2`. Uma round publicada no mesmo
    /// segundo do fechamento já podia ser conhecida; por isso "estritamente".
    fn first_round_after(t: u64) -> u64 {
        t.saturating_sub(QUICKNET_GENESIS) / QUICKNET_PERIOD + 2
    }

    /// Põe `valor` na folha `index` e refaz o caminho até a root: uma
    /// Poseidon(2) por nível, com o irmão lido do armazenamento ou, se nunca
    /// escrito, a subárvore vazia daquele nível.
    fn per_leaf(env: &Env, index: u32, valor: U256) {
        let p = env.storage().persistent();
        let h = Sponge::t3(env);
        let mut pos = index;
        let mut current = valor;
        p.set(&DataKey::Node(0, pos), &current);
        for level in 0..DEPTH {
            let sibling = Self::no(env, level, pos ^ 1);
            current = if pos & 1 == 0 {
                h.hash(&vec![env, current, sibling])
            } else {
                h.hash(&vec![env, sibling, current])
            };
            pos >>= 1;
            p.set(&DataKey::Node(level + 1, pos), &current);
        }
    }

    fn no(env: &Env, level: u32, pos: u32) -> U256 {
        env.storage()
            .persistent()
            .get(&DataKey::Node(level, pos))
            .unwrap_or_else(|| Self::zero(env, level))
    }

    pub(crate) fn zero(env: &Env, level: u32) -> U256 {
        U256::from_be_bytes(env, &Bytes::from_array(env, &ZEROS[level as usize]))
    }

    /// A proof de seat vale? INV-15: contra a root que o fechamento
    /// guardou, com esta semente, este `y` e esta conta de seat. As
    /// entradas públicas são montadas aqui, na ordem do circuito. T-038 chama
    /// `claim` chama isto com a semente de `seal` e a conta que assina.
    pub(crate) fn prova_de_assento_vale(
        env: &Env,
        proof: &Proof,
        seed: &U256,
        y: &U256,
        seat: &Address,
    ) -> bool {
        let f = match Self::closing(env.clone()) {
            Some(f) => f,
            None => panic_with_error!(env, Error::RosterOpen),
        };
        let publics = vec![
            env,
            f.root,
            seed.clone(),
            y.clone(),
            Self::claimer(env, seat),
        ];
        VerifierClient::new(env, &Self::verifier(env.clone())).verify(proof, &publics)
    }

    /// `SHA-256(strkey da conta) >> 8`: um elemento do campo que amarra a
    /// proof à conta que vai enviá-la (DEC-024; `circuits/pinakion.mjs`).
    pub(crate) fn claimer(env: &Env, seat: &Address) -> U256 {
        let h = env.crypto().sha256(&seat.to_string().to_bytes()).to_array();
        let mut b = [0u8; 32];
        b[1..].copy_from_slice(&h[..31]);
        U256::from_be_bytes(env, &Bytes::from_array(env, &b))
    }

    fn clerk(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Clerk).unwrap()
    }

    fn ticket(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Ticket).unwrap()
    }

    // ---- cópias literais do beacon-verifier ----------------------------

    /// Verifica uma round do beacon contra a chave pinada da quicknet.
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

    /// A aleatoriedade da round, como a drand a publica: o sha256 da
    /// assinatura **compressed**.
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

        let mut y_is_the_larger = false;
        for i in 0..48 {
            if bytes[48 + i] != HALF_OF_P[i] {
                y_is_the_larger = bytes[48 + i] > HALF_OF_P[i];
                break;
            }
        }
        x[0] |= 0x80;
        if y_is_the_larger {
            x[0] |= 0x20;
        }

        let compressed = Bytes::from_array(&env, &x);
        env.crypto().sha256(&compressed).to_bytes()
    }
}

mod test;
