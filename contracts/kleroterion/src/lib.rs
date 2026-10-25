#![no_std]
//! Kleroterion: o rol de jurados como um ativo clássico da Stellar.
//!
//! O *pinakion* era um bilhete de bronze. Aqui ele é uma unidade de um ativo
//! clássico, com `AUTH_REQUIRED` no emissor: ter o bilhete e estar autorizado
//! é o que dá assento. O rol é a lista de trustlines autorizadas, legível por
//! qualquer Horizon em `/accounts?asset=`; a ordem do rol é a ordem de
//! inscrição guardada aqui; o compromisso é `lock()`, depois do qual ninguém
//! entra — e o emissor, com master weight 0, não emite nem autoriza mais nada.
//!
//! A entropia vem da drand quicknet, verificada pelas funções de host da
//! BLS12-381 (CAP-59); o embaralhamento é o PRNG do host. `verify`,
//! `randomness` e `draw` são cópias literais do `beacon-verifier`, e os
//! vetores de ouro mantêm as duas cópias honestas (DEC-014).

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype,
    crypto::bls12_381::{Bls12381G1Affine, Bls12381G2Affine},
    panic_with_error, token, vec, Address, Bytes, BytesN, Env, Vec,
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

/// Um bilhete: 1.0000000 PINAKION, em stroops. Ter menos que isto é não ter
/// bilhete (INV-12).
pub const UM_BILHETE: i128 = 10_000_000;

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
    /// A rodada pedida é anterior à que o fechamento fixou (INV-06).
    RodadaAnteriorAoRol = 1,
    /// A assinatura não é da quicknet.
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
}

/// O que `lock()` fixa, de uma vez por todas.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Fechamento {
    pub ledger: u32,
    pub tempo: u64,
    pub rodada_minima: u64,
}

#[contracttype]
pub enum DataKey {
    Escrivao,
    Bilhete,
    Minimo,
    Rol,
    Fechamento,
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
        let vazio: Vec<Address> = Vec::new(&env);
        env.storage().persistent().set(&DataKey::Rol, &vazio);
    }

    /// Inscreve um jurado: cria a trustline (SAC `trust`, CAP-73), autoriza e
    /// emite um bilhete. O escrivão e o jurado assinam — o jurado consente em
    /// servir; é a parte afetada pelo caso que nunca consentiu com o foro.
    pub fn enroll(env: Env, jurado: Address) {
        Self::escrivao(&env).require_auth();
        jurado.require_auth();
        if Self::closing(env.clone()).is_some() {
            panic_with_error!(&env, Erro::RolFechado);
        }
        let mut rol = Self::roster(env.clone());
        if rol.contains(&jurado) {
            panic_with_error!(&env, Erro::JuradoRepetido);
        }
        let sac = token::StellarAssetClient::new(&env, &Self::bilhete(&env));
        sac.trust(&jurado);
        sac.set_authorized(&jurado, &true);
        sac.mint(&jurado, &UM_BILHETE);
        rol.push_back(jurado);
        env.storage().persistent().set(&DataKey::Rol, &rol);
    }

    /// Fecha o rol: a partir daqui ninguém entra (INV-11), e a rodada mínima
    /// do beacon fica fixada pelo relógio do ledger, não por quem chama
    /// (INV-06). Recusa abaixo do mínimo (INV-09).
    pub fn lock(env: Env) -> Fechamento {
        Self::escrivao(&env).require_auth();
        if Self::closing(env.clone()).is_some() {
            panic_with_error!(&env, Erro::RolFechado);
        }
        let minimo: u32 = env.storage().instance().get(&DataKey::Minimo).unwrap();
        if Self::roster(env.clone()).len() < minimo {
            panic_with_error!(&env, Erro::RolAbaixoDoMinimo);
        }
        let tempo = env.ledger().timestamp();
        let f = Fechamento {
            ledger: env.ledger().sequence(),
            tempo,
            rodada_minima: Self::primeira_rodada_depois_de(tempo),
        };
        env.storage().instance().set(&DataKey::Fechamento, &f);
        f
    }

    /// Impedimento ou recusa (art. 463): tira a autorização do bilhete. O
    /// jurado continua no rol e na ordem; só não senta (INV-12), e o
    /// substituto é o próximo da ordem já sorteada (INV-08).
    pub fn impede(env: Env, jurado: Address) {
        Self::escrivao(&env).require_auth();
        token::StellarAssetClient::new(&env, &Self::bilhete(&env)).set_authorized(&jurado, &false);
    }

    /// O rol, na ordem de inscrição. É esta ordem que o sorteio embaralha.
    pub fn roster(env: Env) -> Vec<Address> {
        env.storage().persistent().get(&DataKey::Rol).unwrap()
    }

    /// O fechamento, se já houve.
    pub fn closing(env: Env) -> Option<Fechamento> {
        env.storage().instance().get(&DataKey::Fechamento)
    }

    /// Sorteia `k` assentos a partir de uma rodada do beacon: verifica a
    /// assinatura, deriva a semente, embaralha a ordem do rol e senta os `k`
    /// primeiros que têm bilhete e autorização.
    pub fn draw_round(env: Env, round: u64, sig: BytesN<96>, k: u32) -> Vec<Address> {
        let f = match Self::closing(env.clone()) {
            Some(f) => f,
            None => panic_with_error!(&env, Erro::RolAberto),
        };
        if round < f.rodada_minima {
            panic_with_error!(&env, Erro::RodadaAnteriorAoRol);
        }
        if !Self::verify(env.clone(), round, sig.clone()) {
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
            if painel.len() == k {
                break;
            }
            let j = rol.get(i).unwrap();
            if sac.authorized(&j) && saldo.balance(&j) >= UM_BILHETE {
                painel.push_back(j);
            }
        }
        if painel.len() < k {
            panic_with_error!(&env, Erro::ElegiveisInsuficientes);
        }
        painel
    }

    /// A primeira rodada publicada estritamente depois do instante `t`:
    /// a rodada `r` sai em `genesis + (r-1)·period`, logo
    /// `r = ⌊(t − genesis)/period⌋ + 2`. Uma rodada publicada no mesmo
    /// segundo do fechamento já podia ser conhecida; por isso "estritamente".
    fn primeira_rodada_depois_de(t: u64) -> u64 {
        t.saturating_sub(GENESIS_QUICKNET) / PERIODO_QUICKNET + 2
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
