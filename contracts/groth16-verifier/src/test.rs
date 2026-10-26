extern crate std;

use super::*;
use soroban_sdk::{Bytes, BytesN, Env, Vec, U256};
use std::borrow::ToOwned;

/// A proof from Pinakion's circuit (`circuits/pinakion_claim.circom`, depth 6,
/// publics `[root, seed, y, claimer]`), under the spike's TEST setup
/// (`circuits/provar.sh`). Data, not code.
///
/// The fixture paths are still Portuguese: the vector and fixture files are
/// renamed in one step of T-058, together with every reader.
const SPIKE_PROOF: &str = include_str!("../../../tests/prova-claim-spike.json");

/// The proof under the key of the T-036 setup (`circuits/setup/`), made by
/// `pinakion prove` (`circuits/gerar_fixture.mjs`).
const CLI_PROOF: &str = include_str!("../../../tests/prova-claim.json");

/// Cost of one valid `verify` through the contract, with the embedded key
/// (T-037), in the test host, `(cpu, mem)`. Not a figure from a testnet
/// transaction. The spike measured 29,128,335 with the key read from storage;
/// that version of the contract no longer exists
/// (docs/evidencia-testnet-spike.md).
const VERIFY_COST: (u64, u64) = (29_105_202, 208_994);

fn bytes<const N: usize>(env: &Env, hex_str: &str) -> BytesN<N> {
    let raw = hex::decode(hex_str).unwrap();
    BytesN::from_array(env, &raw.try_into().unwrap())
}

fn load(env: &Env) -> (VerificationKey, Proof, Vec<U256>) {
    load_from(env, SPIKE_PROOF)
}

fn load_from(env: &Env, source: &str) -> (VerificationKey, Proof, Vec<U256>) {
    let j: serde_json::Value = serde_json::from_str(source).unwrap();
    let s = |v: &serde_json::Value| v.as_str().unwrap().to_owned();
    let mut ic = Vec::new(env);
    for p in j["vk"]["ic"].as_array().unwrap() {
        ic.push_back(bytes::<64>(env, &s(p)));
    }
    let vk = VerificationKey {
        alpha: bytes(env, &s(&j["vk"]["alpha"])),
        beta: bytes(env, &s(&j["vk"]["beta"])),
        gamma: bytes(env, &s(&j["vk"]["gamma"])),
        delta: bytes(env, &s(&j["vk"]["delta"])),
        ic,
    };
    let proof = Proof {
        a: bytes(env, &s(&j["proof"]["a"])),
        b: bytes(env, &s(&j["proof"]["b"])),
        c: bytes(env, &s(&j["proof"]["c"])),
    };
    let mut public = Vec::new(env);
    for p in j["public"].as_array().unwrap() {
        let raw = hex::decode(s(p)).unwrap();
        public.push_back(U256::from_be_bytes(env, &Bytes::from_slice(env, &raw)));
    }
    (vk, proof, public)
}

/// The spike's verifier: the pure `verify_proof` with the T-034 key, inside a
/// registered contract (the contract itself, since T-037, knows only the
/// T-036 key).
struct Spike<'a> {
    env: &'a Env,
    id: soroban_sdk::Address,
    vk: VerificationKey,
}

impl Spike<'_> {
    fn verify(&self, proof: &Proof, public: &Vec<U256>) -> bool {
        self.env
            .as_contract(&self.id, || verify_proof(self.env, &self.vk, proof, public))
    }
}

fn client(env: &Env, vk: VerificationKey) -> Spike<'_> {
    let id = env.register(Groth16Verifier, ());
    Spike { env, id, vk }
}

#[test]
fn the_spike_proof_verifies_under_the_spike_key() {
    let env = Env::default();
    let (vk, proof, public) = load(&env);
    assert_eq!(public.len(), 4, "[root, seed, y, claimer]");
    let c = client(&env, vk);
    env.cost_estimate().budget().reset_unlimited();
    assert!(c.verify(&proof, &public));
}

/// Another `y` with the same proof: fails. This is what stops a ticket being
/// swapped.
#[test]
fn a_swapped_y_fails() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, mut public) = load(&env);
    public.set(2, U256::from_u32(&env, 1));
    assert!(!client(&env, vk).verify(&proof, &public));
}

/// Another seat account with the same proof: fails. A proof copied from the
/// mempool is no use to whoever copied it.
#[test]
fn a_swapped_claimer_fails() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, mut public) = load(&env);
    public.set(3, public.get(3).unwrap().add(&U256::from_u32(&env, 1)));
    assert!(!client(&env, vk).verify(&proof, &public));
}

#[test]
fn permuted_inputs_fail() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, public) = load(&env);
    let mut swapped = public.clone();
    swapped.set(0, public.get(1).unwrap());
    swapped.set(1, public.get(0).unwrap());
    assert!(!client(&env, vk).verify(&proof, &swapped));
}

/// One byte changed in the proof: either the point stops being valid and the
/// host refuses it, or the pairing equation does not close. Never `true`.
#[test]
fn one_altered_proof_byte_does_not_pass() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, public) = load(&env);
    let c = client(&env, vk);
    for (field, i) in [("c", 63usize), ("a", 40)] {
        let mut p = proof.clone();
        let mut raw = match field {
            "c" => p.c.to_array(),
            _ => p.a.to_array(),
        };
        raw[i] ^= 0x01;
        match field {
            "c" => p.c = BytesN::from_array(&env, &raw),
            _ => p.a = BytesN::from_array(&env, &raw),
        }
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| c.verify(&p, &public)));
        assert!(!r.unwrap_or(false), "altered {} passed", field);
    }
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn an_input_outside_the_field_is_refused() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, mut public) = load(&env);
    let r = U256::from_be_bytes(&env, &Bytes::from_array(&env, &SCALAR_MODULUS));
    public.set(2, public.get(2).unwrap().add(&r));
    client(&env, vk).verify(&proof, &public);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn the_wrong_number_of_inputs_is_refused() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, mut public) = load(&env);
    public.pop_back();
    client(&env, vk).verify(&proof, &public);
}

/// The embedded key is the one in circuits/setup/vk.json, and the proof from
/// `pinakion prove` (tests/prova-claim.json) verifies through the contract —
/// and does not verify with `y` swapped. The cost is that of any verification
/// with four public inputs.
#[test]
fn the_embedded_key_is_the_setups_and_the_cli_proof_verifies() {
    let env = Env::default();
    let (fixture_vk, proof, public) = load_from(&env, CLI_PROOF);
    let id = env.register(Groth16Verifier, ());
    let c = Groth16VerifierClient::new(&env, &id);
    let vk = c.vk();
    assert!(vk.alpha == fixture_vk.alpha && vk.beta == fixture_vk.beta);
    assert!(vk.gamma == fixture_vk.gamma && vk.delta == fixture_vk.delta);
    assert_eq!(vk.ic, fixture_vk.ic);
    env.cost_estimate().budget().reset_unlimited();
    assert!(c.verify(&proof, &public));
    let b = env.cost_estimate().budget();
    assert_eq!(
        (b.cpu_instruction_cost(), b.memory_bytes_cost()),
        VERIFY_COST,
        "verify (cpu, mem)"
    );
    let mut wrong = public.clone();
    wrong.set(2, U256::from_u32(&env, 1));
    assert!(!c.verify(&proof, &wrong));
}

/// The spike's proof, made under another key, does not pass the contract.
#[test]
fn the_spike_proof_does_not_pass_the_embedded_key() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (_, proof, public) = load(&env);
    let id = env.register(Groth16Verifier, ());
    assert!(!Groth16VerifierClient::new(&env, &id).verify(&proof, &public));
}
