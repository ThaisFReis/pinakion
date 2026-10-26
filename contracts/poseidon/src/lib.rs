#![no_std]
//! Poseidon over BN254, circomlib-compatible, built on the host permutation
//! (CAP-75, Protocol 25).
//!
//! The host exposes only the permutation; the sponge — zeroed state, capacity 1
//! at position 0, inputs in the positions after it, one permutation, output at
//! position 0 — is circomlib's (`poseidon.circom`), and so are the parameters.
//! `hash1` is Poseidon(1) (t = 2) and `hash2` is Poseidon(2) (t = 3), the two
//! the circuit uses: the leaf `Poseidon(s)`, the tree's nodes and
//! `y = Poseidon(seed, s)`.
//!
//! Adapted from stellar/rs-soroban-poseidon v28.0.0, `src/poseidon/sponge.rs`
//! (Apache-2.0, Copyright 2026 Stellar Development Foundation; see
//! LICENSE-APACHE). Changes: BN254 and t ∈ {2, 3} only; no `Field` trait; the
//! published version requires soroban-sdk 28 and this workspace is on 29
//! (DEC-021).

use soroban_sdk::{bytesn, symbol_short, vec, Bytes, Env, Vec, U256};

mod params;
use params::{get_mds_bn254_t_2, get_mds_bn254_t_3, get_rc_bn254_t_2, get_rc_bn254_t_3, SBOX_D};

/// circomlib's full and partial rounds for t = 2 and t = 3.
const ROUNDS_F: u32 = 8;
const ROUNDS_P_T2: u32 = 56;
const ROUNDS_P_T3: u32 = 57;

/// The modulus r of BN254's scalar field.
fn field_modulus(env: &Env) -> U256 {
    let r: Bytes = bytesn!(
        env,
        0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001
    )
    .into();
    U256::from_be_bytes(env, &r)
}

/// A sponge of width `t`, with the parameters built once. For many hashes in a
/// row (the tree), make one and reuse it: building the tables as host objects
/// costs on every `new`.
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
            r: field_modulus(env),
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
            r: field_modulus(env),
        }
    }

    /// Hash of exactly `t - 1` inputs, each smaller than r. An input outside
    /// the field panics instead of being reduced: without that, `x` and `x + r`
    /// would give the same hash.
    pub fn hash(&self, inputs: &Vec<U256>) -> U256 {
        assert!(inputs.len() == self.t - 1, "Poseidon: t - 1 inputs");
        let mut state = vec![&self.env, U256::from_u32(&self.env, 0)];
        for x in inputs.iter() {
            assert!(x < self.r, "Poseidon: input outside the field");
            state.push_back(x);
        }
        let output = self.env.crypto_hazmat().poseidon_permutation(
            &state,
            symbol_short!("BN254"),
            self.t,
            SBOX_D,
            ROUNDS_F,
            self.rounds_p,
            &self.mds,
            &self.rc,
        );
        output.get_unchecked(0)
    }
}

/// Poseidon(a), as circomlib's `Poseidon(1)`.
pub fn hash1(env: &Env, a: &U256) -> U256 {
    Sponge::t2(env).hash(&vec![env, a.clone()])
}

/// Poseidon(a, b), as circomlib's `Poseidon(2)`.
pub fn hash2(env: &Env, a: &U256, b: &U256) -> U256 {
    Sponge::t3(env).hash(&vec![env, a.clone(), b.clone()])
}

#[cfg(test)]
mod test;
