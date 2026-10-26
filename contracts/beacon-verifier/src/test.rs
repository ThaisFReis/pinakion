#![cfg(test)]
extern crate std;
use super::*;
use soroban_sdk::{BytesN, Env};

const ROUND_A: u64 = 1000000;
const SIG_A: [u8; 96] = [
    0x03, 0xad, 0x29, 0xe4, 0xc4, 0x09, 0xf9, 0x47, 0x0f, 0xc2, 0xef, 0x02, 0xf9, 0x02, 0x14, 0xdf,
    0x49, 0xe0, 0x2b, 0x44, 0x1a, 0x1a, 0x24, 0x1a, 0x82, 0xd6, 0x22, 0xd9, 0xf6, 0x08, 0xef, 0x98,
    0xfd, 0x8b, 0x11, 0xa0, 0x29, 0xf1, 0xbe, 0xe9, 0xd9, 0xe8, 0x3b, 0x45, 0x08, 0x8a, 0xbe, 0x72,
    0x01, 0x77, 0x6f, 0xf7, 0x40, 0x8b, 0x39, 0xc5, 0xf6, 0xf9, 0xfa, 0x50, 0x74, 0x6e, 0xfd, 0x7e,
    0xea, 0x17, 0xfb, 0xb6, 0x1f, 0x2e, 0x7b, 0x9c, 0x84, 0x9f, 0xf0, 0x52, 0x8e, 0x5a, 0x3d, 0xee,
    0xed, 0xd0, 0x29, 0xd0, 0xdf, 0x34, 0x51, 0x99, 0x96, 0x3d, 0x75, 0xba, 0x93, 0xb5, 0xa0, 0x2a,
];
const ROUND_B: u64 = 33380727;
// Costs published in docs/evidencia-testnet-sorteio.md, README.md and
// THESIS.md. These tests fail if the cost changes; scripts/conferir_numeros.py
// fails if a document stops saying the same. It is the two sides together that
// close the door.
const VERIFY_CPU_COST: u64 = 26_719_483;
const VERIFY_MEM_COST: u64 = 161_818;
const DRAW_COST: [(u32, u64, u64); 3] = [
    (50, 128_952, 26_379),
    (200, 347_784, 192_579),
    (1000, 2_274_884, 4_118_979),
];

const SIG_B: [u8; 96] = [
    0x11, 0x08, 0xdb, 0x9e, 0x91, 0x7f, 0x1c, 0xd5, 0x7f, 0x15, 0x33, 0x90, 0xdd, 0xb5, 0x35, 0x3c,
    0xd1, 0x1f, 0xe3, 0x6b, 0x05, 0x1c, 0x71, 0xf9, 0x97, 0x81, 0x86, 0xea, 0xdc, 0xa2, 0x50, 0x6f,
    0xd0, 0x1d, 0x28, 0x5f, 0xf0, 0xbc, 0x3e, 0x16, 0xd9, 0x94, 0x28, 0xe6, 0x68, 0xcd, 0x07, 0x3b,
    0x06, 0xfd, 0x5e, 0x36, 0x9d, 0x6f, 0x67, 0x4e, 0x7e, 0xc8, 0xfb, 0x7a, 0x08, 0xcb, 0xbc, 0xdd,
    0x54, 0xec, 0xc6, 0x87, 0x5e, 0x1d, 0x85, 0x68, 0x43, 0x4d, 0x53, 0xc8, 0x65, 0x2a, 0x68, 0xb4,
    0xd5, 0x6d, 0xd1, 0xed, 0xac, 0x00, 0xb0, 0xfa, 0xe5, 0xe0, 0x09, 0xc3, 0x72, 0xfc, 0x0a, 0x00,
];
/// A signature for round 1000000 made by a keypair that is not quicknet's. It
/// is valid — it verifies under the adversary's key — and it has to be refused
/// by this contract, which trusts quicknet alone.
const SIG_FORGED: [u8; 96] = [
    0x10, 0x9a, 0xf8, 0x13, 0x33, 0xbf, 0x09, 0xab, 0x23, 0x37, 0xbd, 0xca, 0xbc, 0xd1, 0x03, 0x5c,
    0xc5, 0xda, 0x3b, 0x72, 0x87, 0x3c, 0xf3, 0x3e, 0x91, 0x9d, 0xcd, 0x3f, 0x06, 0xa5, 0x26, 0x49,
    0x9c, 0x8e, 0xac, 0x74, 0x3c, 0x89, 0x0f, 0xa7, 0x6f, 0x9f, 0xe7, 0x88, 0xe7, 0x0a, 0x2f, 0xc0,
    0x16, 0xe4, 0xc1, 0x2f, 0x24, 0xd7, 0x47, 0x96, 0xf0, 0x14, 0xbd, 0x7f, 0x85, 0xba, 0xe8, 0x5c,
    0xa1, 0x54, 0x40, 0x19, 0x46, 0xf4, 0x67, 0xa8, 0xa8, 0x67, 0xc4, 0x0f, 0x57, 0x77, 0xc9, 0x6d,
    0x39, 0x27, 0x0f, 0x24, 0x05, 0x11, 0x8b, 0x58, 0xb5, 0xe0, 0x7d, 0x14, 0x61, 0x77, 0x0b, 0xf5,
];

/// The hash of drand's quicknet chain, the beacon this contract trusts.
const QUICKNET_CHAIN: [u8; 32] = [
    0x52, 0xdb, 0x9b, 0xa7, 0x0e, 0x0c, 0xc0, 0xf6, 0xea, 0xf7, 0x80, 0x3d, 0xd0, 0x74, 0x47, 0xa1,
    0xf5, 0x47, 0x77, 0x35, 0xfd, 0x3f, 0x66, 0x17, 0x92, 0xba, 0x94, 0x60, 0x0c, 0x84, 0xe9, 0x71,
];

const RANDOMNESS_A: [u8; 32] = [
    0xb2, 0x2a, 0xad, 0x47, 0x94, 0xf7, 0x45, 0x18, 0x96, 0xf7, 0xa3, 0x71, 0xaa, 0x46, 0x10, 0x6f,
    0xd8, 0x4d, 0x91, 0x9f, 0x3f, 0x56, 0x9a, 0xcd, 0x5b, 0x2f, 0xdd, 0xf1, 0xd1, 0x44, 0x0a, 0xf3,
];

fn setup() -> (Env, BeaconVerifierClient<'static>) {
    let env = Env::default();
    let id = env.register(BeaconVerifier, ());
    let c = BeaconVerifierClient::new(&env, &id);
    (env, c)
}

#[test]
fn a_real_round_verifies_and_the_cost_is_measured() {
    let (env, c) = setup();
    env.cost_estimate().budget().reset_unlimited();
    let ok = c.verify(&ROUND_A, &BytesN::from_array(&env, &SIG_A));
    let cpu = env.cost_estimate().budget().cpu_instruction_cost();
    let mem = env.cost_estimate().budget().memory_bytes_cost();
    std::println!("--- cost of verify() for round {} ---", ROUND_A);
    std::println!("cpu {} mem {}", cpu, mem);
    assert!(ok, "the real drand round should verify");
    assert_eq!(
        cpu, VERIFY_CPU_COST,
        "the cpu cost changed; update the documents and this constant"
    );
    assert_eq!(
        mem, VERIFY_MEM_COST,
        "the memory cost changed; update the documents and this constant"
    );
}

#[test]
fn a_second_real_round_verifies() {
    let (env, c) = setup();
    let ok = c.verify(&ROUND_B, &BytesN::from_array(&env, &SIG_B));
    assert!(ok, "the second real round should verify");
}

#[test]
fn the_wrong_round_fails() {
    let (env, c) = setup();
    let ok = c.verify(&(ROUND_A + 1), &BytesN::from_array(&env, &SIG_A));
    assert!(!ok, "round A's signature cannot hold for round A+1");
}

#[test]
fn a_swapped_signature_fails() {
    let (env, c) = setup();
    let ok = c.verify(&ROUND_A, &BytesN::from_array(&env, &SIG_B));
    assert!(!ok, "another round's signature cannot verify");
}

#[test]
fn the_randomness_is_what_drand_publishes() {
    let (env, c) = setup();
    let r = c.randomness(&BytesN::from_array(&env, &SIG_A));
    assert_eq!(
        r,
        BytesN::from_array(&env, &RANDOMNESS_A),
        "the randomness has to be the one drand publishes, not another sha256"
    );
}

fn seed(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &RANDOMNESS_A)
}

#[test]
fn the_draw_is_deterministic() {
    let (env, c) = setup();
    let a = c.draw(&seed(&env), &200u32, &25u32);
    let b = c.draw(&seed(&env), &200u32, &25u32);
    assert_eq!(a, b, "the same seed has to give the same draw");
    std::println!("draw (200 -> 25): {:?}", a);
}

#[test]
fn different_seeds_give_different_draws() {
    let (env, c) = setup();
    let mut other = RANDOMNESS_A;
    other[31] ^= 1;
    let a = c.draw(&seed(&env), &200u32, &25u32);
    let b = c.draw(&BytesN::from_array(&env, &other), &200u32, &25u32);
    assert_ne!(a, b, "different seeds should diverge");
}

#[test]
fn the_draw_has_no_repeats() {
    let (env, c) = setup();
    let a = c.draw(&seed(&env), &200u32, &25u32);
    for i in 0..a.len() {
        for j in (i + 1)..a.len() {
            assert_ne!(
                a.get(i).unwrap(),
                a.get(j).unwrap(),
                "a juror repeated in the draw"
            );
        }
    }
}

#[test]
fn draw_cost_by_roster_size() {
    let (env, c) = setup();
    for (n, expected_cpu, expected_mem) in DRAW_COST {
        env.cost_estimate().budget().reset_unlimited();
        let _ = c.draw(&seed(&env), &n, &25u32);
        let cpu = env.cost_estimate().budget().cpu_instruction_cost();
        let mem = env.cost_estimate().budget().memory_bytes_cost();
        std::println!(
            "--- draw with a roster of {} --- cpu {} mem {}",
            n,
            cpu,
            mem
        );
        assert_eq!(
            cpu, expected_cpu,
            "the cpu of a draw with a roster of {} changed",
            n
        );
        assert_eq!(
            mem, expected_mem,
            "the memory of a draw with a roster of {} changed",
            n
        );
    }
}

// Golden vectors: produced by `scripts/verificar_sorteio.py`, an independent
// reimplementation (ChaCha20 + Lemire + Fisher-Yates written from scratch, with
// no crate at all). If the host changes the algorithm, these tests break.

#[test]
fn matches_the_off_chain_recomputation() {
    let (env, c) = setup();

    let a = c.draw(&seed(&env), &200u32, &25u32);
    let expected_a: [u32; 25] = [
        15, 179, 44, 133, 177, 1, 168, 104, 157, 62, 43, 63, 57, 23, 86, 50, 165, 64, 26, 27, 148,
        96, 120, 13, 29,
    ];
    for (i, v) in expected_a.iter().enumerate() {
        assert_eq!(a.get(i as u32).unwrap(), *v, "diverged at position {}", i);
    }

    let zeros = BytesN::from_array(&env, &[0u8; 32]);
    let b = c.draw(&zeros, &50u32, &7u32);
    let expected_b: [u32; 7] = [28, 10, 18, 23, 41, 36, 26];
    for (i, v) in expected_b.iter().enumerate() {
        assert_eq!(
            b.get(i as u32).unwrap(),
            *v,
            "the zero seed diverged at {}",
            i
        );
    }

    let d = c.draw(&seed(&env), &7u32, &7u32);
    let expected_d: [u32; 7] = [0, 1, 4, 2, 6, 5, 3];
    for (i, v) in expected_d.iter().enumerate() {
        assert_eq!(
            d.get(i as u32).unwrap(),
            *v,
            "the small roster diverged at {}",
            i
        );
    }
}

#[test]
fn a_signature_from_another_key_does_not_verify() {
    let (env, c) = setup();
    // The signature is legitimate: it was produced by a real secret key, over
    // the round's correct message. It just is not quicknet's key.
    let ok = c.verify(&ROUND_A, &BytesN::from_array(&env, &SIG_FORGED));
    assert!(!ok, "a signature from another keypair cannot verify");
}

#[test]
fn the_beacon_key_is_not_a_parameter() {
    let (env, c) = setup();
    // The contract states which beacon it trusts, and nobody can swap it.
    assert_eq!(c.beacon(), BytesN::from_array(&env, &QUICKNET_CHAIN));
}

/// INV-06, the half that can be checked today: a draw cannot use a round
/// earlier than the one the roster fixed. Without that, whoever already knows
/// a past round's result builds the roster afterwards and picks the round that
/// suits them.
#[test]
fn a_draw_with_a_round_before_the_minimum_is_refused() {
    let (env, c) = setup();
    let sig = BytesN::from_array(&env, &SIG_A);
    let r = c.try_draw_round(&(ROUND_A + 1), &ROUND_A, &sig, &200u32, &25u32);
    assert_eq!(
        r,
        Err(Ok(Error::RoundBeforeRoster.into())),
        "a round before the minimum had to be refused"
    );
}

#[test]
fn a_draw_with_the_minimum_round_itself_is_accepted() {
    let (env, c) = setup();
    let sig = BytesN::from_array(&env, &SIG_A);
    let panel = c.draw_round(&ROUND_A, &ROUND_A, &sig, &200u32, &25u32);
    let direct = c.draw(&seed(&env), &200u32, &25u32);
    assert_eq!(panel, direct, "the panel has to be the same as draw()'s");
}

#[test]
fn a_draw_with_a_later_round_is_accepted() {
    let (env, c) = setup();
    let sig = BytesN::from_array(&env, &SIG_B);
    let panel = c.draw_round(&ROUND_A, &ROUND_B, &sig, &50u32, &7u32);
    assert_eq!(panel.len(), 7);
}

/// The minimum round alone is not enough: the signature still has to be
/// quicknet's. Otherwise it would be enough to invent a high round.
#[test]
fn a_draw_with_a_forged_signature_is_refused() {
    let (env, c) = setup();
    let sig = BytesN::from_array(&env, &SIG_FORGED);
    let r = c.try_draw_round(&1u64, &1000000u64, &sig, &200u32, &25u32);
    assert_eq!(
        r,
        Err(Ok(Error::InvalidSignature.into())),
        "a signature from another key cannot draw"
    );
}
