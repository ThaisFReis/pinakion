#![cfg(test)]
extern crate std;
use super::*;
use soroban_sdk::{Bytes, Env, U256};

/// tests/vetores-poseidon.json, gerado por circuits/vetores_poseidon.mjs a
/// partir da circomlibjs — a mesma Poseidon que o circuito usa (Q-22).
const VETORES: &str = include_str!("../../../tests/vetores-poseidon.json");

fn u256(env: &Env, h: &str) -> U256 {
    let h = h.trim_start_matches("0x");
    let mut b = [0u8; 32];
    for i in 0..32 {
        b[i] = u8::from_str_radix(&h[2 * i..2 * i + 2], 16).unwrap();
    }
    U256::from_be_bytes(env, &Bytes::from_array(env, &b))
}

fn vetores() -> std::vec::Vec<(std::vec::Vec<std::string::String>, std::string::String)> {
    let j: serde_json::Value = serde_json::from_str(VETORES).unwrap();
    j["vetores"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| {
            let e = v["entradas"]
                .as_array()
                .unwrap()
                .iter()
                .map(|x| x.as_str().unwrap().into())
                .collect();
            (e, v["saida"].as_str().unwrap().into())
        })
        .collect()
}

#[test]
fn ha_ao_menos_vinte_vetores_de_uma_e_duas_entradas() {
    let vs = vetores();
    let um = vs.iter().filter(|(e, _)| e.len() == 1).count();
    let dois = vs.iter().filter(|(e, _)| e.len() == 2).count();
    assert!(um + dois >= 20, "o SPEC pede pelo menos 20");
    assert!(um >= 1 && dois >= 1);
    assert_eq!(um + dois, vs.len());
}

/// Q-22: o sponge sobre a permutação do host devolve, byte a byte, o que a
/// circomlibjs devolve.
#[test]
fn bate_com_a_circomlib() {
    let env = Env::default();
    for (i, (e, s)) in vetores().iter().enumerate() {
        let obtido = match e.len() {
            1 => hash1(&env, &u256(&env, &e[0])),
            2 => hash2(&env, &u256(&env, &e[0]), &u256(&env, &e[1])),
            _ => unreachable!(),
        };
        assert_eq!(obtido, u256(&env, s), "vetor {} ({:?})", i, e);
    }
}

/// Uma entrada fora do campo é recusada, e não reduzida: sem isso `x` e
/// `x + r` dariam o mesmo hash.
#[test]
#[should_panic]
fn entrada_fora_do_campo_e_recusada() {
    let env = Env::default();
    let r = u256(
        &env,
        "0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001",
    );
    hash1(&env, &r);
}

/// Custo medido no host de teste (`env.cost_estimate().budget()`), não numa
/// transação da testnet. Mudou o custo, o teste fica vermelho (G2).
/// `(cpu, mem)`: uma hash1 com sponge novo, uma hash2 com sponge novo, e a
/// árvore inteira de profundidade 6 — 64 folhas Poseidon(1) e 63 nós
/// Poseidon(2), com os dois sponges construídos uma vez (T-035).
const CUSTO_HASH1: (u64, u64) = (1_001_368, 57_296);
const CUSTO_HASH2: (u64, u64) = (1_454_641, 81_832);
const CUSTO_ARVORE_6: (u64, u64) = (109_821_216, 3_373_240);

fn medir(env: &Env, f: impl FnOnce()) -> (u64, u64) {
    env.cost_estimate().budget().reset_unlimited();
    f();
    let b = env.cost_estimate().budget();
    (b.cpu_instruction_cost(), b.memory_bytes_cost())
}

#[test]
fn custo_medido() {
    let env = Env::default();
    let x = U256::from_u32(&env, 11);
    let h1 = medir(&env, || {
        hash1(&env, &x);
    });
    let h2 = medir(&env, || {
        hash2(&env, &x, &x);
    });
    let arvore = medir(&env, || {
        let s2 = Sponge::t2(&env);
        let s3 = Sponge::t3(&env);
        let mut nivel: std::vec::Vec<U256> = (0..64u32)
            .map(|i| s2.hash(&soroban_sdk::vec![&env, U256::from_u32(&env, i)]))
            .collect();
        while nivel.len() > 1 {
            nivel = nivel
                .chunks(2)
                .map(|p| s3.hash(&soroban_sdk::vec![&env, p[0].clone(), p[1].clone()]))
                .collect();
        }
    });
    std::println!("hash1 {:?}  hash2 {:?}  arvore6 {:?}", h1, h2, arvore);
    assert_eq!(h1, CUSTO_HASH1, "hash1 (cpu, mem)");
    assert_eq!(h2, CUSTO_HASH2, "hash2 (cpu, mem)");
    assert_eq!(
        arvore, CUSTO_ARVORE_6,
        "arvore de profundidade 6 (cpu, mem)"
    );
}
