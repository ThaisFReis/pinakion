#![no_std]
//! Groth16 verifier over BN254 using the Protocol 25 host functions.
//!
//! Ported from the author's withdrawn generation (`groth16-verifier`, written
//! 2026-10-25 13:02 UTC, inside the hacking window; DEC-017). Unchanged in
//! logic. Since T-037 the verification key is embedded (`vk.rs`, generated
//! from `circuits/setup/vk.json`): the contract verifies proofs of
//! `pinakion_claim` and of nothing else, and nobody deploying it can hand it
//! another key — the same reason the beacon key is pinned (INV-10).
//!
//! Checks e(-A, B) · e(alpha, beta) · e(vk_x, gamma) · e(C, delta) == 1,
//! where vk_x = IC[0] + Σ public[i] · IC[i + 1].

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype,
    crypto::bn254::{Bn254Fr, Bn254G1Affine, Bn254G2Affine},
    panic_with_error, vec, Bytes, BytesN, Env, Vec, U256,
};

/// BN254 base field modulus p, big-endian.
const FIELD_MODULUS: [u8; 32] = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
    0x97, 0x81, 0x6a, 0x91, 0x68, 0x71, 0xca, 0x8d, 0x3c, 0x20, 0x8c, 0x16, 0xd8, 0x7c, 0xfd, 0x47,
];

/// BN254 scalar field modulus r, big-endian.
const SCALAR_MODULUS: [u8; 32] = [
    0x30, 0x64, 0x4e, 0x72, 0xe1, 0x31, 0xa0, 0x29, 0xb8, 0x50, 0x45, 0xb6, 0x81, 0x81, 0x58, 0x5d,
    0x28, 0x33, 0xe8, 0x48, 0x79, 0xb9, 0x70, 0x91, 0x43, 0xe1, 0xf5, 0x93, 0xf0, 0x00, 0x00, 0x01,
];

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    PublicInputCount = 1,
    PublicInputOutOfField = 2,
}

#[contracttype]
#[derive(Clone)]
pub struct VerificationKey {
    pub alpha: BytesN<64>,
    pub beta: BytesN<128>,
    pub gamma: BytesN<128>,
    pub delta: BytesN<128>,
    pub ic: Vec<BytesN<64>>,
}

#[contracttype]
#[derive(Clone)]
pub struct Proof {
    pub a: BytesN<64>,
    pub b: BytesN<128>,
    pub c: BytesN<64>,
}

mod vk;

#[contract]
pub struct Groth16Verifier;

#[contractimpl]
impl Groth16Verifier {
    /// Checks a proof of `pinakion_claim` with public inputs
    /// `[root, seed, y, claimer]`, in that order.
    pub fn verify(env: Env, proof: Proof, public: Vec<U256>) -> bool {
        verify_proof(&env, &Self::vk(env.clone()), &proof, &public)
    }

    /// The embedded key, for anyone to compare with `circuits/setup/vk.json`.
    pub fn vk(env: Env) -> VerificationKey {
        let mut ic = Vec::new(&env);
        for p in vk::IC.iter() {
            ic.push_back(BytesN::from_array(&env, p));
        }
        VerificationKey {
            alpha: BytesN::from_array(&env, &vk::ALPHA),
            beta: BytesN::from_array(&env, &vk::BETA),
            gamma: BytesN::from_array(&env, &vk::GAMMA),
            delta: BytesN::from_array(&env, &vk::DELTA),
            ic,
        }
    }
}

pub fn verify_proof(env: &Env, vk: &VerificationKey, proof: &Proof, public: &Vec<U256>) -> bool {
    if public.len() + 1 != vk.ic.len() {
        panic_with_error!(env, Error::PublicInputCount);
    }
    // Reject non-canonical inputs, so x and x + r cannot both pass as "the same" value.
    let r = U256::from_be_bytes(env, &Bytes::from_array(env, &SCALAR_MODULUS));
    for input in public.iter() {
        if input >= r {
            panic_with_error!(env, Error::PublicInputOutOfField);
        }
    }
    let bn = env.crypto().bn254();

    let mut vk_x = Bn254G1Affine::from_bytes(vk.ic.get_unchecked(0));
    for i in 0..public.len() {
        let point = Bn254G1Affine::from_bytes(vk.ic.get_unchecked(i + 1));
        let scalar = Bn254Fr::from_u256(public.get_unchecked(i));
        vk_x = bn.g1_add(&vk_x, &bn.g1_mul(&point, &scalar));
    }

    let g1 = vec![
        env,
        negate_g1(env, &proof.a),
        Bn254G1Affine::from_bytes(vk.alpha.clone()),
        vk_x,
        Bn254G1Affine::from_bytes(proof.c.clone()),
    ];
    let g2 = vec![
        env,
        Bn254G2Affine::from_bytes(proof.b.clone()),
        Bn254G2Affine::from_bytes(vk.beta.clone()),
        Bn254G2Affine::from_bytes(vk.gamma.clone()),
        Bn254G2Affine::from_bytes(vk.delta.clone()),
    ];
    bn.pairing_check(g1, g2)
}

/// -(x, y) = (x, p - y); the point at infinity (all zeros) is its own negation.
fn negate_g1(env: &Env, point: &BytesN<64>) -> Bn254G1Affine {
    let bytes: Bytes = point.clone().into();
    let x = bytes.slice(0..32);
    let y = U256::from_be_bytes(env, &bytes.slice(32..64));
    let zero = U256::from_u32(env, 0);
    if y == zero && U256::from_be_bytes(env, &x) == zero {
        return Bn254G1Affine::from_bytes(point.clone());
    }
    let p = U256::from_be_bytes(env, &Bytes::from_array(env, &FIELD_MODULUS));
    let mut out = x;
    out.append(&p.sub(&y).to_be_bytes());
    Bn254G1Affine::from_bytes(out.try_into().unwrap())
}

#[cfg(test)]
mod test;
