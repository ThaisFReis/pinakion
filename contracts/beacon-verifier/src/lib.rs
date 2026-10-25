#![no_std]
//! Verificador de rodada do beacon drand (quicknet) na Stellar.
//!
//! Esquema `bls-unchained-g1-rfc9380`: assinatura em G1, chave pública em G2.
//! A mensagem assinada é `sha256(round as u64 big-endian)` — verificado
//! empiricamente contra rodadas reais antes de escrever este contrato.
//!
//! Verificação: e(H(msg), pk) · e(sig, -G2) == 1

use soroban_sdk::{
    contract, contractimpl,
    crypto::bls12_381::{Bls12381G1Affine, Bls12381G2Affine},
    vec, Bytes, BytesN, Env,
};

/// Domain separation tag do drand quicknet.
const DST: &[u8] = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";

/// Gerador de G2 negado, codificação não comprimida:
/// be(X_c1) || be(X_c0) || be(Y_c1) || be(Y_c0)
pub const NEG_G2_GENERATOR: [u8; 192] = *include_bytes!("neg_g2.bin");

/// Chave pública do grupo de limiar da quicknet, pinada.
///
/// Ela **não** é parâmetro. Se fosse, qualquer pessoa passaria a própria chave
/// e faria qualquer rodada verificar — o que esvazia a garantia inteira, porque
/// a semente deixaria de vir de um beacon que ninguém controla.
///
/// Consequência aceita: este contrato confia em uma única chain da drand. Se o
/// grupo da quicknet rotacionar a chave, isso é uma chain nova, com outro hash,
/// e exige um contrato novo. Trocar a chave sem trocar o contrato seria
/// exatamente o poder que este pinning existe para remover.
pub const BEACON_PUBLIC_KEY: [u8; 192] = *include_bytes!("quicknet_pk.bin");

/// Hash da chain quicknet. Publicado por `beacon()` para que qualquer um
/// confira em que beacon este contrato confia, sem ler o Wasm.
pub const BEACON_CHAIN_HASH: [u8; 32] = *include_bytes!("quicknet_chain.bin");

#[contract]
pub struct BeaconVerifier;

#[contractimpl]
impl BeaconVerifier {
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

    /// A aleatoriedade da rodada é o sha256 da assinatura.
    pub fn randomness(env: Env, sig: BytesN<96>) -> BytesN<32> {
        let _ = &env;
        let b: Bytes = sig.into();
        env.crypto().sha256(&b).to_bytes()
    }
}

mod test;
