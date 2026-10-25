extern crate std;

use super::*;
use soroban_sdk::{Bytes, BytesN, Env, Vec, U256};
use std::borrow::ToOwned;

/// Uma prova do circuito do Pinakion (`circuits/pinakion_claim.circom`,
/// profundidade 6, públicos `[root, seed, y, claimer]`), com o setup de TESTE
/// do spike (`circuits/provar.sh`). Dados, não código.
const PROVA: &str = include_str!("../../../tests/prova-claim-spike.json");

/// A prova sob a chave do setup de T-036 (`circuits/setup/`), feita por
/// `pinakion prove` (`circuits/gerar_fixture.mjs`).
const PROVA_T036: &str = include_str!("../../../tests/prova-claim.json");

/// Custo de um `verify` válido no host de teste, `(cpu, mem)`. Não é número
/// de transação da testnet; esse está em docs/evidencia-testnet-spike.md.
const CUSTO_VERIFY: (u64, u64) = (29_128_335, 229_165);

fn bytes<const N: usize>(env: &Env, hex_str: &str) -> BytesN<N> {
    let raw = hex::decode(hex_str).unwrap();
    BytesN::from_array(env, &raw.try_into().unwrap())
}

fn load(env: &Env) -> (VerificationKey, Proof, Vec<U256>) {
    load_de(env, PROVA)
}

fn load_de(env: &Env, fonte: &str) -> (VerificationKey, Proof, Vec<U256>) {
    let j: serde_json::Value = serde_json::from_str(fonte).unwrap();
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

fn cliente(env: &Env, vk: VerificationKey) -> Groth16VerifierClient<'_> {
    let id = env.register(Groth16Verifier, (vk,));
    Groth16VerifierClient::new(env, &id)
}

#[test]
fn prova_valida_verifica_e_custo_medido() {
    let env = Env::default();
    let (vk, proof, public) = load(&env);
    assert_eq!(public.len(), 4, "[root, seed, y, claimer]");
    let c = cliente(&env, vk);
    env.cost_estimate().budget().reset_unlimited();
    assert!(c.verify(&proof, &public));
    let b = env.cost_estimate().budget();
    let custo = (b.cpu_instruction_cost(), b.memory_bytes_cost());
    std::println!("verify {:?}", custo);
    assert_eq!(custo, CUSTO_VERIFY, "verify (cpu, mem)");
}

/// Outro `y` com a mesma prova: falha. É o que impede trocar o bilhete.
#[test]
fn y_trocado_falha() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, mut public) = load(&env);
    public.set(2, U256::from_u32(&env, 1));
    assert!(!cliente(&env, vk).verify(&proof, &public));
}

/// Outra conta de assento com a mesma prova: falha. Uma prova copiada da
/// mempool não serve a quem a copiou.
#[test]
fn claimer_trocado_falha() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, mut public) = load(&env);
    public.set(3, public.get(3).unwrap().add(&U256::from_u32(&env, 1)));
    assert!(!cliente(&env, vk).verify(&proof, &public));
}

#[test]
fn entradas_permutadas_falham() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, public) = load(&env);
    let mut trocado = public.clone();
    trocado.set(0, public.get(1).unwrap());
    trocado.set(1, public.get(0).unwrap());
    assert!(!cliente(&env, vk).verify(&proof, &trocado));
}

/// Um byte alterado na prova: ou o ponto deixa de ser válido e o host recusa,
/// ou a equação de pareamento não fecha. Nunca `true`.
#[test]
fn um_byte_da_prova_alterado_nao_passa() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, public) = load(&env);
    let c = cliente(&env, vk);
    for (campo, i) in [("c", 63usize), ("a", 40)] {
        let mut p = proof.clone();
        let mut raw = match campo {
            "c" => p.c.to_array(),
            _ => p.a.to_array(),
        };
        raw[i] ^= 0x01;
        match campo {
            "c" => p.c = BytesN::from_array(&env, &raw),
            _ => p.a = BytesN::from_array(&env, &raw),
        }
        let r = c.try_verify(&p, &public);
        assert!(r != Ok(Ok(true)), "{} alterado passou", campo);
    }
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn entrada_fora_do_campo_e_recusada() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, mut public) = load(&env);
    let r = U256::from_be_bytes(&env, &Bytes::from_array(&env, &SCALAR_MODULUS));
    public.set(2, public.get(2).unwrap().add(&r));
    cliente(&env, vk).verify(&proof, &public);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn numero_de_entradas_errado_e_recusado() {
    let env = Env::default();
    env.cost_estimate().budget().reset_unlimited();
    let (vk, proof, mut public) = load(&env);
    public.pop_back();
    cliente(&env, vk).verify(&proof, &public);
}

/// A chave do setup público (ptau da Hermez, fase 2 de uma parte com beacon
/// drand) e uma prova da CLI: verifica, e com `y` trocado não verifica. O
/// custo é o mesmo de uma verificação qualquer de quatro entradas públicas.
#[test]
fn prova_do_setup_de_t036_verifica() {
    let env = Env::default();
    let (vk, proof, public) = load_de(&env, PROVA_T036);
    let c = cliente(&env, vk);
    env.cost_estimate().budget().reset_unlimited();
    assert!(c.verify(&proof, &public));
    let b = env.cost_estimate().budget();
    assert_eq!(
        (b.cpu_instruction_cost(), b.memory_bytes_cost()),
        CUSTO_VERIFY,
        "verify (cpu, mem)"
    );
    let mut errado = public.clone();
    errado.set(2, U256::from_u32(&env, 1));
    assert!(!c.verify(&proof, &errado));
}
