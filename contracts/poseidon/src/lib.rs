#![no_std]
//! Poseidon sobre BN254, compatível com a circomlib, montado sobre a
//! permutação do host (CAP-75, Protocol 25).
//!
//! O host expõe só a permutação; o sponge — estado zerado, capacidade 1 na
//! posição 0, entradas nas posições seguintes, uma permutação, saída na
//! posição 0 — é o da circomlib (`poseidon.circom`), e os parâmetros são os
//! dela. `hash1` é Poseidon(1) (t = 2) e `hash2` é Poseidon(2) (t = 3), os
//! dois que o circuito usa: a folha `Poseidon(s)`, os nós da árvore e
//! `y = Poseidon(seed, s)`.
//!
//! Adaptado de stellar/rs-soroban-poseidon v28.0.0, `src/poseidon/sponge.rs`
//! (Apache-2.0, Copyright 2026 Stellar Development Foundation; ver
//! LICENSE-APACHE). Mudanças: só BN254 e t ∈ {2, 3}; sem o trait `Field`; a
//! versão publicada exige soroban-sdk 28 e este workspace está no 29 (DEC-021).

use soroban_sdk::{bytesn, symbol_short, vec, Bytes, Env, Vec, U256};

mod params;
use params::{get_mds_bn254_t_2, get_mds_bn254_t_3, get_rc_bn254_t_2, get_rc_bn254_t_3, SBOX_D};

/// Rodadas completas e parciais da circomlib para t = 2 e t = 3.
const ROUNDS_F: u32 = 8;
const ROUNDS_P_T2: u32 = 56;
const ROUNDS_P_T3: u32 = 57;

/// Módulo r do campo escalar do BN254.
fn modulo(env: &Env) -> U256 {
    let r: Bytes = bytesn!(
        env,
        0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001
    )
    .into();
    U256::from_be_bytes(env, &r)
}

/// Um sponge de largura `t`, com os parâmetros construídos uma vez. Para
/// muitas hashes seguidas (a árvore), crie um e reuse: montar as tabelas
/// como objetos do host custa a cada `new`.
pub struct Sponge {
    env: Env,
    t: u32,
    rounds_p: u32,
    mds: Vec<Vec<U256>>,
    rc: Vec<Vec<U256>>,
    r: U256,
}

impl Sponge {
    /// Poseidon(1): t = 2.
    pub fn t2(env: &Env) -> Self {
        Self {
            env: env.clone(),
            t: 2,
            rounds_p: ROUNDS_P_T2,
            mds: get_mds_bn254_t_2(env),
            rc: get_rc_bn254_t_2(env),
            r: modulo(env),
        }
    }

    /// Poseidon(2): t = 3.
    pub fn t3(env: &Env) -> Self {
        Self {
            env: env.clone(),
            t: 3,
            rounds_p: ROUNDS_P_T3,
            mds: get_mds_bn254_t_3(env),
            rc: get_rc_bn254_t_3(env),
            r: modulo(env),
        }
    }

    /// Hash de exatamente `t - 1` entradas, cada uma menor que r. Uma entrada
    /// fora do campo entra em pânico em vez de ser reduzida: sem isso `x` e
    /// `x + r` dariam o mesmo hash.
    pub fn hash(&self, entradas: &Vec<U256>) -> U256 {
        assert!(entradas.len() == self.t - 1, "Poseidon: t - 1 entradas");
        let mut estado = vec![&self.env, U256::from_u32(&self.env, 0)];
        for x in entradas.iter() {
            assert!(x < self.r, "Poseidon: entrada fora do campo");
            estado.push_back(x);
        }
        let saida = self.env.crypto_hazmat().poseidon_permutation(
            &estado,
            symbol_short!("BN254"),
            self.t,
            SBOX_D,
            ROUNDS_F,
            self.rounds_p,
            &self.mds,
            &self.rc,
        );
        saida.get_unchecked(0)
    }
}

/// Poseidon(a), como `Poseidon(1)` da circomlib.
pub fn hash1(env: &Env, a: &U256) -> U256 {
    Sponge::t2(env).hash(&vec![env, a.clone()])
}

/// Poseidon(a, b), como `Poseidon(2)` da circomlib.
pub fn hash2(env: &Env, a: &U256, b: &U256) -> U256 {
    Sponge::t3(env).hash(&vec![env, a.clone(), b.clone()])
}

#[cfg(test)]
mod test;
