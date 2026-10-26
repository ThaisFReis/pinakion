#![no_std]
//! Verifier for a drand beacon round (quicknet) on Stellar.
//!
//! Scheme `bls-unchained-g1-rfc9380`: signature in G1, public key in G2. The
//! signed message is `sha256(round as u64 big-endian)` — checked empirically
//! against real rounds before this contract was written.
//!
//! Verification: e(H(msg), pk) · e(sig, -G2) == 1

use soroban_sdk::{
    contract, contracterror, contractimpl,
    crypto::bls12_381::{Bls12381G1Affine, Bls12381G2Affine},
    panic_with_error, vec, Bytes, BytesN, Env,
};

/// drand quicknet's domain separation tag.
const DST: &[u8] = b"BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_";

/// The G2 generator, negated, uncompressed encoding:
/// be(X_c1) || be(X_c0) || be(Y_c1) || be(Y_c0)
pub const NEG_G2_GENERATOR: [u8; 192] = *include_bytes!("neg_g2.bin");

/// quicknet's threshold-group public key, pinned.
///
/// It is **not** a parameter. If it were, anyone could pass their own key and
/// make any round verify — which empties the whole guarantee, because the seed
/// would stop coming from a beacon nobody controls.
///
/// Accepted consequence: this contract trusts a single drand chain. If the
/// quicknet group rotates its key, that is a new chain with a different hash,
/// and it needs a new contract. Swapping the key without swapping the contract
/// would be exactly the power this pinning exists to remove.
pub const BEACON_PUBLIC_KEY: [u8; 192] = *include_bytes!("quicknet_pk.bin");

/// The quicknet chain hash. Published by `beacon()` so anyone can check which
/// beacon this contract trusts without reading the Wasm.
pub const BEACON_CHAIN_HASH: [u8; 32] = *include_bytes!("quicknet_chain.bin");

/// (p-1)/2 for BLS12-381's base field, big-endian.
/// Used for Y's sign bit when compressing a G1 point.
const HALF_OF_P: [u8; 48] = [
    0x0d, 0x00, 0x88, 0xf5, 0x1c, 0xbf, 0xf3, 0x4d, 0x25, 0x8d, 0xd3, 0xdb, 0x21, 0xa5, 0xd6, 0x6b,
    0xb2, 0x3b, 0xa5, 0xc2, 0x79, 0xc2, 0x89, 0x5f, 0xb3, 0x98, 0x69, 0x50, 0x7b, 0x58, 0x7b, 0x12,
    0x0f, 0x55, 0xff, 0xff, 0x58, 0xa9, 0xff, 0xff, 0xdc, 0xff, 0x7f, 0xff, 0xff, 0xff, 0xd5, 0x55,
];

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The round asked for is earlier than the one the roster fixed (INV-06).
    RoundBeforeRoster = 1,
    /// The signature is not quicknet's.
    InvalidSignature = 2,
}

#[contract]
pub struct BeaconVerifier;

#[contractimpl]
impl BeaconVerifier {
    /// Verifies a beacon round against quicknet's pinned key.
    /// Returns `true` if the signature checks out.
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

    /// Which beacon this contract trusts. A public read with no argument: the
    /// answer is the same for everyone, forever.
    pub fn beacon(env: Env) -> BytesN<32> {
        BytesN::from_array(&env, &BEACON_CHAIN_HASH)
    }

    /// Draws `k` positions from a roster of size `n`, using the beacon's
    /// randomness as the seed. Fisher-Yates, deterministic and recomputable.
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

    /// Draws from a beacon round, refusing a round earlier than the one the
    /// roster fixed.
    ///
    /// This is the half of INV-06 that can be enforced without a roster
    /// contract: the caller states `minimum_round`, and the contract refuses
    /// any older round. The other half — making sure that `minimum_round` is
    /// really the roster's, and not a number chosen afterwards — needs the
    /// roster held on the ledger, which does not exist yet.
    ///
    /// The seed is not a parameter: it is derived from the verified signature,
    /// so the panel is tied to a round that is authentic and not earlier.
    pub fn draw_round(
        env: Env,
        minimum_round: u64,
        round: u64,
        sig: BytesN<96>,
        n: u32,
        k: u32,
    ) -> soroban_sdk::Vec<u32> {
        if round < minimum_round {
            panic_with_error!(&env, Error::RoundBeforeRoster);
        }
        if !Self::verify(env.clone(), round, sig.clone()) {
            panic_with_error!(&env, Error::InvalidSignature);
        }
        let seed = Self::randomness(env.clone(), sig);
        Self::draw(env, seed, n, k)
    }

    /// The round's randomness, as drand publishes it: the sha256 of the
    /// **compressed** signature.
    ///
    /// The signature arrives here uncompressed, because that is what the
    /// BLS12-381 host functions take. Compressing it back is only a matter of
    /// putting the flag bits back in X: 0x80 always, and 0x20 when Y is the
    /// larger of the two possible values. Without that the value returned
    /// would not be drand's, and the seed here would not match the independent
    /// verifier's.
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
