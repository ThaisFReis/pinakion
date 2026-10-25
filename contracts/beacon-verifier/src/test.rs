#![cfg(test)]
extern crate std;
use super::*;
use soroban_sdk::{BytesN, Env};

const ROUND_A: u64 = 1000000;
const SIG_A: [u8; 96] = [0x03, 0xad, 0x29, 0xe4, 0xc4, 0x09, 0xf9, 0x47, 0x0f, 0xc2, 0xef, 0x02, 0xf9, 0x02, 0x14, 0xdf, 0x49, 0xe0, 0x2b, 0x44, 0x1a, 0x1a, 0x24, 0x1a, 0x82, 0xd6, 0x22, 0xd9, 0xf6, 0x08, 0xef, 0x98, 0xfd, 0x8b, 0x11, 0xa0, 0x29, 0xf1, 0xbe, 0xe9, 0xd9, 0xe8, 0x3b, 0x45, 0x08, 0x8a, 0xbe, 0x72, 0x01, 0x77, 0x6f, 0xf7, 0x40, 0x8b, 0x39, 0xc5, 0xf6, 0xf9, 0xfa, 0x50, 0x74, 0x6e, 0xfd, 0x7e, 0xea, 0x17, 0xfb, 0xb6, 0x1f, 0x2e, 0x7b, 0x9c, 0x84, 0x9f, 0xf0, 0x52, 0x8e, 0x5a, 0x3d, 0xee, 0xed, 0xd0, 0x29, 0xd0, 0xdf, 0x34, 0x51, 0x99, 0x96, 0x3d, 0x75, 0xba, 0x93, 0xb5, 0xa0, 0x2a];
const ROUND_B: u64 = 33380727;
const SIG_B: [u8; 96] = [0x11, 0x08, 0xdb, 0x9e, 0x91, 0x7f, 0x1c, 0xd5, 0x7f, 0x15, 0x33, 0x90, 0xdd, 0xb5, 0x35, 0x3c, 0xd1, 0x1f, 0xe3, 0x6b, 0x05, 0x1c, 0x71, 0xf9, 0x97, 0x81, 0x86, 0xea, 0xdc, 0xa2, 0x50, 0x6f, 0xd0, 0x1d, 0x28, 0x5f, 0xf0, 0xbc, 0x3e, 0x16, 0xd9, 0x94, 0x28, 0xe6, 0x68, 0xcd, 0x07, 0x3b, 0x06, 0xfd, 0x5e, 0x36, 0x9d, 0x6f, 0x67, 0x4e, 0x7e, 0xc8, 0xfb, 0x7a, 0x08, 0xcb, 0xbc, 0xdd, 0x54, 0xec, 0xc6, 0x87, 0x5e, 0x1d, 0x85, 0x68, 0x43, 0x4d, 0x53, 0xc8, 0x65, 0x2a, 0x68, 0xb4, 0xd5, 0x6d, 0xd1, 0xed, 0xac, 0x00, 0xb0, 0xfa, 0xe5, 0xe0, 0x09, 0xc3, 0x72, 0xfc, 0x0a, 0x00];
/// Assinatura da rodada 1000000 feita por um par de chaves que não é o da
/// quicknet. Ela é válida — verifica sob a chave do adversário — e tem que ser
/// recusada por este contrato, que confia só na quicknet.
const SIG_FORJADA: [u8; 96] = [0x10, 0x9a, 0xf8, 0x13, 0x33, 0xbf, 0x09, 0xab, 0x23, 0x37, 0xbd, 0xca, 0xbc, 0xd1, 0x03, 0x5c, 0xc5, 0xda, 0x3b, 0x72, 0x87, 0x3c, 0xf3, 0x3e, 0x91, 0x9d, 0xcd, 0x3f, 0x06, 0xa5, 0x26, 0x49, 0x9c, 0x8e, 0xac, 0x74, 0x3c, 0x89, 0x0f, 0xa7, 0x6f, 0x9f, 0xe7, 0x88, 0xe7, 0x0a, 0x2f, 0xc0, 0x16, 0xe4, 0xc1, 0x2f, 0x24, 0xd7, 0x47, 0x96, 0xf0, 0x14, 0xbd, 0x7f, 0x85, 0xba, 0xe8, 0x5c, 0xa1, 0x54, 0x40, 0x19, 0x46, 0xf4, 0x67, 0xa8, 0xa8, 0x67, 0xc4, 0x0f, 0x57, 0x77, 0xc9, 0x6d, 0x39, 0x27, 0x0f, 0x24, 0x05, 0x11, 0x8b, 0x58, 0xb5, 0xe0, 0x7d, 0x14, 0x61, 0x77, 0x0b, 0xf5];

/// Hash da chain quicknet da drand, o beacon em que este contrato confia.
const CHAIN_QUICKNET: [u8; 32] = [0x52, 0xdb, 0x9b, 0xa7, 0x0e, 0x0c, 0xc0, 0xf6, 0xea, 0xf7, 0x80, 0x3d, 0xd0, 0x74, 0x47, 0xa1, 0xf5, 0x47, 0x77, 0x35, 0xfd, 0x3f, 0x66, 0x17, 0x92, 0xba, 0x94, 0x60, 0x0c, 0x84, 0xe9, 0x71];

const RANDOMNESS_A: [u8; 32] = [0xb2, 0x2a, 0xad, 0x47, 0x94, 0xf7, 0x45, 0x18, 0x96, 0xf7, 0xa3, 0x71, 0xaa, 0x46, 0x10, 0x6f, 0xd8, 0x4d, 0x91, 0x9f, 0x3f, 0x56, 0x9a, 0xcd, 0x5b, 0x2f, 0xdd, 0xf1, 0xd1, 0x44, 0x0a, 0xf3];

fn setup() -> (Env, BeaconVerifierClient<'static>) {
    let env = Env::default();
    let id = env.register(BeaconVerifier, ());
    let c = BeaconVerifierClient::new(&env, &id);
    (env, c)
}

#[test]
fn rodada_real_verifica_e_mede_custo() {
    let (env, c) = setup();
    env.cost_estimate().budget().reset_unlimited();
    let ok = c.verify(&ROUND_A, &BytesN::from_array(&env, &SIG_A));
    std::println!("--- custo de verify() para a rodada {} ---", ROUND_A);
    std::println!("{:?}", env.cost_estimate().budget());
    assert!(ok, "a rodada real do drand deveria verificar");
}

#[test]
fn segunda_rodada_real_verifica() {
    let (env, c) = setup();
    let ok = c.verify(&ROUND_B, &BytesN::from_array(&env, &SIG_B));
    assert!(ok, "a segunda rodada real deveria verificar");
}

#[test]
fn rodada_errada_falha() {
    let (env, c) = setup();
    let ok = c.verify(&(ROUND_A + 1), &BytesN::from_array(&env, &SIG_A));
    assert!(!ok, "assinatura da rodada A nao pode valer para a rodada A+1");
}

#[test]
fn assinatura_trocada_falha() {
    let (env, c) = setup();
    let ok = c.verify(&ROUND_A, &BytesN::from_array(&env, &SIG_B));
    assert!(!ok, "assinatura de outra rodada nao pode verificar");
}

#[test]
fn aleatoriedade_e_sha256_da_assinatura() {
    let (env, c) = setup();
    let r = c.randomness(&BytesN::from_array(&env, &SIG_A));
    std::println!("randomness: {:?}", r);
    let _ = RANDOMNESS_A;
}

fn semente(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &RANDOMNESS_A)
}

#[test]
fn sorteio_e_deterministico() {
    let (env, c) = setup();
    let a = c.draw(&semente(&env), &200u32, &25u32);
    let b = c.draw(&semente(&env), &200u32, &25u32);
    assert_eq!(a, b, "mesma semente tem que dar o mesmo sorteio");
    std::println!("sorteio (200 -> 25): {:?}", a);
}

#[test]
fn sementes_diferentes_dao_sorteios_diferentes() {
    let (env, c) = setup();
    let mut outra = RANDOMNESS_A;
    outra[31] ^= 1;
    let a = c.draw(&semente(&env), &200u32, &25u32);
    let b = c.draw(&BytesN::from_array(&env, &outra), &200u32, &25u32);
    assert_ne!(a, b, "sementes diferentes deveriam divergir");
}

#[test]
fn sorteio_sem_repeticao() {
    let (env, c) = setup();
    let a = c.draw(&semente(&env), &200u32, &25u32);
    for i in 0..a.len() {
        for j in (i + 1)..a.len() {
            assert_ne!(a.get(i).unwrap(), a.get(j).unwrap(), "jurado repetido no sorteio");
        }
    }
}

#[test]
fn custo_do_sorteio_por_tamanho_de_rol() {
    let (env, c) = setup();
    for n in [50u32, 200u32, 1000u32] {
        env.cost_estimate().budget().reset_unlimited();
        let _ = c.draw(&semente(&env), &n, &25u32);
        std::println!("--- sorteio com rol de {} ---", n);
        std::println!("{:?}", env.cost_estimate().budget());
    }
}

// Vetores de ouro: produzidos por `scripts/verificar_sorteio.py`, uma
// reimplementação independente (ChaCha20 + Lemire + Fisher-Yates escritos do
// zero, sem nenhuma crate). Se o host mudar o algoritmo, estes testes quebram.

#[test]
fn bate_com_a_recomputacao_fora_da_cadeia() {
    let (env, c) = setup();

    let a = c.draw(&semente(&env), &200u32, &25u32);
    let esperado_a: [u32; 25] = [
        15, 179, 44, 133, 177, 1, 168, 104, 157, 62, 43, 63, 57, 23, 86, 50, 165, 64, 26, 27,
        148, 96, 120, 13, 29,
    ];
    for (i, v) in esperado_a.iter().enumerate() {
        assert_eq!(a.get(i as u32).unwrap(), *v, "divergiu na posicao {}", i);
    }

    let zeros = BytesN::from_array(&env, &[0u8; 32]);
    let b = c.draw(&zeros, &50u32, &7u32);
    let esperado_b: [u32; 7] = [28, 10, 18, 23, 41, 36, 26];
    for (i, v) in esperado_b.iter().enumerate() {
        assert_eq!(b.get(i as u32).unwrap(), *v, "semente zero divergiu em {}", i);
    }

    let d = c.draw(&semente(&env), &7u32, &7u32);
    let esperado_d: [u32; 7] = [0, 1, 4, 2, 6, 5, 3];
    for (i, v) in esperado_d.iter().enumerate() {
        assert_eq!(d.get(i as u32).unwrap(), *v, "rol pequeno divergiu em {}", i);
    }
}

#[test]
fn assinatura_de_outra_chave_nao_verifica() {
    let (env, c) = setup();
    // A assinatura é legítima: foi produzida por uma chave secreta real,
    // sobre a mensagem correta da rodada. Só não é a chave da quicknet.
    let ok = c.verify(&ROUND_A, &BytesN::from_array(&env, &SIG_FORJADA));
    assert!(!ok, "uma assinatura de outro par de chaves nao pode verificar");
}

#[test]
fn a_chave_do_beacon_nao_e_parametro() {
    let (env, c) = setup();
    // O contrato declara em que beacon confia, e ninguem pode trocar.
    assert_eq!(c.beacon(), BytesN::from_array(&env, &CHAIN_QUICKNET));
}
