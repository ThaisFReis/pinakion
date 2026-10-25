#![no_std]
//! Verificador de rodada do beacon drand (quicknet) na Stellar.
//!
//! Esquema `bls-unchained-g1-rfc9380`: assinatura em G1, chave pública em G2.
//! A mensagem assinada é `sha256(round as u64 big-endian)` — verificado
//! empiricamente contra rodadas reais antes de escrever este contrato.
//!
//! Verificação: e(H(msg), pk) · e(sig, -G2) == 1

use soroban_sdk::{
    contract, contracterror, contractimpl,
    crypto::bls12_381::{Bls12381G1Affine, Bls12381G2Affine},
    panic_with_error, vec, Bytes, BytesN, Env,
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

/// (p-1)/2 para o campo base da BLS12-381, big-endian.
/// Usado para o bit de sinal de Y na compressão de um ponto G1.
const METADE_DE_P: [u8; 48] = [
    0x0d, 0x00, 0x88, 0xf5, 0x1c, 0xbf, 0xf3, 0x4d, 0x25, 0x8d, 0xd3, 0xdb, 0x21, 0xa5, 0xd6, 0x6b,
    0xb2, 0x3b, 0xa5, 0xc2, 0x79, 0xc2, 0x89, 0x5f, 0xb3, 0x98, 0x69, 0x50, 0x7b, 0x58, 0x7b, 0x12,
    0x0f, 0x55, 0xff, 0xff, 0x58, 0xa9, 0xff, 0xff, 0xdc, 0xff, 0x7f, 0xff, 0xff, 0xff, 0xd5, 0x55,
];

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Erro {
    /// A rodada pedida é anterior à que o rol fixou (INV-06).
    RodadaAnteriorAoRol = 1,
    /// A assinatura não é da quicknet.
    AssinaturaInvalida = 2,
}

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

    /// Sorteia a partir de uma rodada do beacon, recusando rodada anterior à
    /// que o rol fixou.
    ///
    /// Esta é a metade de INV-06 que dá para impor sem um contrato de rol: o
    /// chamador informa `rodada_minima`, e o contrato recusa qualquer rodada
    /// mais antiga. A outra metade — garantir que essa `rodada_minima` é
    /// mesmo a do rol, e não um número escolhido depois — precisa do rol
    /// guardado na cadeia, que não existe ainda.
    ///
    /// A semente não é parâmetro: ela é derivada da assinatura verificada, de
    /// modo que o painel fica preso a uma rodada autêntica e não anterior.
    pub fn draw_round(
        env: Env,
        rodada_minima: u64,
        round: u64,
        sig: BytesN<96>,
        n: u32,
        k: u32,
    ) -> soroban_sdk::Vec<u32> {
        if round < rodada_minima {
            panic_with_error!(&env, Erro::RodadaAnteriorAoRol);
        }
        if !Self::verify(env.clone(), round, sig.clone()) {
            panic_with_error!(&env, Erro::AssinaturaInvalida);
        }
        let seed = Self::randomness(env.clone(), sig);
        Self::draw(env, seed, n, k)
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
