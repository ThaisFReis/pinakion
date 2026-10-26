#![cfg(test)]
extern crate std;
use super::*;
use soroban_sdk::testutils::{
    Address as _, Events as _, IssuerFlags, Ledger as _, MockAuth, MockAuthInvoke,
};
use soroban_sdk::{token, Address, Bytes, BytesN, Env, Event, IntoVal, U256};
use std::borrow::ToOwned;
use std::cell::Cell;

// Vetores partilhados com contracts/beacon-verifier/src/test.rs. Mesma round,
// mesma assinatura, mesma aleatoriedade: é isso que mantém as cópias honestas.
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
const SIG_B: [u8; 96] = [
    0x11, 0x08, 0xdb, 0x9e, 0x91, 0x7f, 0x1c, 0xd5, 0x7f, 0x15, 0x33, 0x90, 0xdd, 0xb5, 0x35, 0x3c,
    0xd1, 0x1f, 0xe3, 0x6b, 0x05, 0x1c, 0x71, 0xf9, 0x97, 0x81, 0x86, 0xea, 0xdc, 0xa2, 0x50, 0x6f,
    0xd0, 0x1d, 0x28, 0x5f, 0xf0, 0xbc, 0x3e, 0x16, 0xd9, 0x94, 0x28, 0xe6, 0x68, 0xcd, 0x07, 0x3b,
    0x06, 0xfd, 0x5e, 0x36, 0x9d, 0x6f, 0x67, 0x4e, 0x7e, 0xc8, 0xfb, 0x7a, 0x08, 0xcb, 0xbc, 0xdd,
    0x54, 0xec, 0xc6, 0x87, 0x5e, 0x1d, 0x85, 0x68, 0x43, 0x4d, 0x53, 0xc8, 0x65, 0x2a, 0x68, 0xb4,
    0xd5, 0x6d, 0xd1, 0xed, 0xac, 0x00, 0xb0, 0xfa, 0xe5, 0xe0, 0x09, 0xc3, 0x72, 0xfc, 0x0a, 0x00,
];
/// Assinatura da round 1000000 feita por um par de chaves que não é o da
/// quicknet. Ela é válida — verifica sob a chave do adversário — e tem que ser
/// recusada por este contrato, que confia só na quicknet.
const SIG_FORJADA: [u8; 96] = [
    0x10, 0x9a, 0xf8, 0x13, 0x33, 0xbf, 0x09, 0xab, 0x23, 0x37, 0xbd, 0xca, 0xbc, 0xd1, 0x03, 0x5c,
    0xc5, 0xda, 0x3b, 0x72, 0x87, 0x3c, 0xf3, 0x3e, 0x91, 0x9d, 0xcd, 0x3f, 0x06, 0xa5, 0x26, 0x49,
    0x9c, 0x8e, 0xac, 0x74, 0x3c, 0x89, 0x0f, 0xa7, 0x6f, 0x9f, 0xe7, 0x88, 0xe7, 0x0a, 0x2f, 0xc0,
    0x16, 0xe4, 0xc1, 0x2f, 0x24, 0xd7, 0x47, 0x96, 0xf0, 0x14, 0xbd, 0x7f, 0x85, 0xba, 0xe8, 0x5c,
    0xa1, 0x54, 0x40, 0x19, 0x46, 0xf4, 0x67, 0xa8, 0xa8, 0x67, 0xc4, 0x0f, 0x57, 0x77, 0xc9, 0x6d,
    0x39, 0x27, 0x0f, 0x24, 0x05, 0x11, 0x8b, 0x58, 0xb5, 0xe0, 0x7d, 0x14, 0x61, 0x77, 0x0b, 0xf5,
];
const RANDOMNESS_A: [u8; 32] = [
    0xb2, 0x2a, 0xad, 0x47, 0x94, 0xf7, 0x45, 0x18, 0x96, 0xf7, 0xa3, 0x71, 0xaa, 0x46, 0x10, 0x6f,
    0xd8, 0x4d, 0x91, 0x9f, 0x3f, 0x56, 0x9a, 0xcd, 0x5b, 0x2f, 0xdd, 0xf1, 0xd1, 0x44, 0x0a, 0xf3,
];

/// Mínimo do roster usado nos testes. Sete, como a mesa de um júri pequeno.
const MINIMO: u32 = 7;

/// Window de reivindicação, em ledgers, nos testes.
const JANELA: u32 = 100;

/// Suspension da pessoa julgada, em ledgers, nos testes.
const AFASTAMENTO: u32 = 500;

/// O menor intervalo entre fechamento e revelação, em ledgers, nos testes.
const ESPERA: u32 = 1_000;

/// A pessoa julgada dos testes: um C-strkey fixo, para que o `Closed` do
/// vetor de ouro seja o mesmo em toda execução (scripts/gerar_vetor_selado.py
/// o deriva de `sha256("pinakion:fluxo:acusado")`). Não está inscrita, a não
/// ser onde o teste diz.
const ACUSADO: &str = "CD6FRVLLOLONSG6SFSJ72VGYKF7PWI5BEJC7FD6OF4VEF7EX5GSL277U";

/// A tabela de severity dos testes: o nível `g` senta `k = g`, de 1 a 10,
/// todos com o mesmo roster mínimo. Assim `lock(&K, …)` ainda fecha com `K`
/// assentos, e os testes de antes de T-045 continuam dizendo o que diziam.
fn niveis_de_teste(env: &Env, minimo: u32) -> soroban_sdk::Vec<Level> {
    let mut v = soroban_sdk::Vec::new(env);
    for k in 1..=10 {
        v.push_back(Level {
            k,
            roster_minimum: minimo,
        });
    }
    v
}

struct Mesa {
    env: Env,
    ticket: Address,
    id: Address,
    clerk: Address,
    accused: Address,
    /// Os eventos do construtor (`Constituted`), guardados antes da próxima
    /// chamada apagá-los.
    constituido: std::vec::Vec<soroban_sdk::xdr::ContractEvent>,
    c: KleroterionClient<'static>,
    /// Próximo compromisso de teste: cada juror recebe um `c` diferente.
    proximo_c: Cell<u32>,
}

/// Emissor com AUTH_REQUIRED e AUTH_REVOCABLE, SAC entregue ao contrato.
/// Quem fecha o emissor (master weight 0) é a sequência clássica de T-031;
/// o ambiente de teste não tem assinatura de conta para simular isso.
fn mesa() -> Mesa {
    mesa_com_minimo(MINIMO)
}

fn mesa_com_minimo(minimo: u32) -> Mesa {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = QUICKNET_GENESIS);
    let clerk = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(clerk.clone());
    sac.issuer().set_flag(IssuerFlags::RequiredFlag);
    sac.issuer().set_flag(IssuerFlags::RevocableFlag);
    let ticket = sac.address();
    let verificador = env.register(groth16_verifier::Groth16Verifier, ());
    let id = env.register(
        Kleroterion,
        (
            &clerk,
            &ticket,
            &verificador,
            &JANELA,
            niveis_de_teste(&env, minimo),
            &AFASTAMENTO,
            &ESPERA,
        ),
    );
    let constituido = env.events().all().filter_by_contract(&id).events().to_vec();
    token::StellarAssetClient::new(&env, &ticket).set_admin(&id);
    let c = KleroterionClient::new(&env, &id);
    let accused = Address::from_str(&env, ACUSADO);
    Mesa {
        env,
        ticket,
        id,
        clerk,
        accused,
        constituido,
        c,
        proximo_c: Cell::new(1000),
    }
}

/// Um compromisso de teste novo. Qualquer elemento do campo, não nulo, serve:
/// o contrato não sabe — nem precisa saber — que `c = Poseidon(s)`.
fn novo_c(m: &Mesa) -> U256 {
    let v = m.proximo_c.get();
    m.proximo_c.set(v + 1);
    U256::from_u32(&m.env, v)
}

fn inscrever(m: &Mesa, j: &Address) -> U256 {
    let c = novo_c(m);
    m.c.enroll(j, &c);
    c
}

fn u256(env: &Env, decimal: &str) -> U256 {
    // decimal -> 32 bytes big-endian, sem dependências
    let mut b = [0u8; 32];
    for d in decimal.bytes() {
        let mut vai = (d - b'0') as u32;
        for x in b.iter_mut().rev() {
            let v = (*x as u32) * 10 + vai;
            *x = v as u8;
            vai = v >> 8;
        }
    }
    U256::from_be_bytes(env, &Bytes::from_array(env, &b))
}

fn jurados(m: &Mesa, n: usize) -> std::vec::Vec<Address> {
    (0..n).map(|_| Address::generate(&m.env)).collect()
}

fn inscrever_todos(m: &Mesa, js: &[Address]) {
    for j in js {
        inscrever(m, j);
    }
}

fn sac(m: &Mesa) -> token::StellarAssetClient<'static> {
    token::StellarAssetClient::new(&m.env, &m.ticket)
}

fn ticket(m: &Mesa) -> token::TokenClient<'static> {
    token::TokenClient::new(&m.env, &m.ticket)
}

fn sig_a(m: &Mesa) -> BytesN<96> {
    BytesN::from_array(&m.env, &SIG_A)
}

// ---- inscrição -----------------------------------------------------------

#[test]
fn inscrever_da_bilhete_e_autorizacao() {
    let m = mesa();
    let j = Address::generate(&m.env);
    let c = inscrever(&m, &j);
    assert_eq!(
        ticket(&m).balance(&j),
        ONE_TICKET,
        "um ticket, nem mais nem menos"
    );
    assert_eq!(
        m.c.commitment(&j),
        Some(c),
        "o compromisso fica preso ao address"
    );
    assert!(sac(&m).authorized(&j), "inscrito sai autorizado");
    assert_eq!(m.c.roster(), soroban_sdk::vec![&m.env, j]);
}

#[test]
fn a_ordem_do_rol_e_a_de_inscricao() {
    let m = mesa();
    let js = jurados(&m, 3);
    inscrever_todos(&m, &js);
    let roster = m.c.roster();
    for (i, j) in js.iter().enumerate() {
        assert_eq!(roster.get(i as u32).unwrap(), *j, "posicao {}", i);
    }
}

#[test]
fn inscrever_repetido_e_recusado() {
    let m = mesa();
    let j = Address::generate(&m.env);
    inscrever(&m, &j);
    let r = m.c.try_enroll(&j, &novo_c(&m));
    assert_eq!(r, Err(Ok(Error::DuplicateJuror.into())));
    assert_eq!(
        ticket(&m).balance(&j),
        ONE_TICKET,
        "a recusa nao emite segundo ticket"
    );
}

#[test]
fn inscrever_sem_escrivao_e_recusado() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = QUICKNET_GENESIS);
    let clerk = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(clerk.clone());
    sac.issuer().set_flag(IssuerFlags::RequiredFlag);
    let verificador = env.register(groth16_verifier::Groth16Verifier, ());
    let id = env.register(
        Kleroterion,
        (
            &clerk,
            &sac.address(),
            &verificador,
            &JANELA,
            niveis_de_teste(&env, MINIMO),
            &AFASTAMENTO,
            &ESPERA,
        ),
    );
    env.mock_all_auths();
    token::StellarAssetClient::new(&env, &sac.address()).set_admin(&id);
    let c = KleroterionClient::new(&env, &id);

    // Só o juror assina; o escrivão não.
    let j = Address::generate(&env);
    let cj = U256::from_u32(&env, 7);
    env.mock_auths(&[MockAuth {
        address: &j,
        invoke: &MockAuthInvoke {
            contract: &id,
            fn_name: "enroll",
            args: (j.clone(), cj.clone()).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(
        c.try_enroll(&j, &cj).is_err(),
        "sem o clerk ninguem entra no roster"
    );
}

// ---- fechamento ----------------------------------------------------------

/// Assentos usados nos testes, e o ledger de revelação: um dia de ledgers de
/// 5 s depois do zero, sem relação com os fechamentos dos testes.
const K: u32 = 3;
const REVELA_EM: u32 = 17_280;

/// Fecha num instante em que a round target é exatamente `round`: a primeira
/// publicada depois do fechamento é `round - ROUND_MARGIN`.
fn fechar_para(m: &Mesa, round: u64, k: u32) -> Closing {
    m.env.ledger().with_mut(|l| {
        l.timestamp = QUICKNET_GENESIS + (round - 2 - ROUND_MARGIN) * QUICKNET_PERIOD
    });
    m.c.lock(&k, &REVELA_EM, &m.accused)
}

#[test]
fn fechar_abaixo_do_minimo_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, (MINIMO - 1) as usize));
    assert_eq!(
        m.c.try_lock(&K, &REVELA_EM, &m.accused),
        Err(Ok(Error::RosterBelowMinimum.into()))
    );
    assert_eq!(m.c.closing(), None);
}

#[test]
fn fechar_fixa_ledger_tempo_rodada_k_e_revelacao() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    m.env.ledger().with_mut(|l| {
        l.sequence_number = 4321;
        l.timestamp = QUICKNET_GENESIS + 6;
    });
    let f = m.c.lock(&K, &REVELA_EM, &m.accused);
    assert_eq!(f.ledger, 4321);
    assert_eq!(f.time, QUICKNET_GENESIS + 6);
    assert_eq!(
        f.round,
        4 + ROUND_MARGIN,
        "round 3 sai em genesis+6, no mesmo segundo; a 4 e a primeira depois, mais a margem"
    );
    assert_eq!(f.k, K);
    assert_eq!(f.reveal_at, REVELA_EM);
    assert_eq!(m.c.closing(), Some(f));
}

/// A margem mínima do SPEC (§9 T-033) é 10 rodadas; abaixo disso não compila.
const _: () = assert!(ROUND_MARGIN >= 10);

/// INV-13: a round é calculada do relógio do ledger, com margem, e não
/// passada por quem chama.
#[test]
fn a_rodada_vem_do_fechamento_com_margem() {
    for (dt, primeira) in [(0u64, 2u64), (1, 2), (2, 2), (3, 3), (4, 3), (5, 3), (6, 4)] {
        let m = mesa();
        inscrever_todos(&m, &jurados(&m, MINIMO as usize));
        m.env
            .ledger()
            .with_mut(|l| l.timestamp = QUICKNET_GENESIS + dt);
        assert_eq!(
            m.c.lock(&K, &REVELA_EM, &m.accused).round,
            primeira + ROUND_MARGIN,
            "dt = {}",
            dt
        );
    }
}

#[test]
fn fechar_com_mais_assentos_que_o_rol_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    assert_eq!(
        m.c.try_lock(&(MINIMO + 1), &REVELA_EM, &m.accused),
        Err(Ok(Error::InvalidSeatCount.into()))
    );
    assert_eq!(
        m.c.try_lock(&0u32, &REVELA_EM, &m.accused),
        Err(Ok(Error::InvalidSeverity.into()))
    );
    assert_eq!(m.c.closing(), None);
    // k = N é permitido.
    assert_eq!(m.c.lock(&MINIMO, &REVELA_EM, &m.accused).k, MINIMO);
}

#[test]
fn fechar_duas_vezes_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    m.c.lock(&K, &REVELA_EM, &m.accused);
    assert_eq!(
        m.c.try_lock(&K, &REVELA_EM, &m.accused),
        Err(Ok(Error::RosterClosed.into()))
    );
}

/// INV-11 e INV-14: o roster é comprometido antes do caso existir.
#[test]
fn inscrever_depois_de_fechar_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    m.c.lock(&K, &REVELA_EM, &m.accused);
    let tarde = Address::generate(&m.env);
    assert_eq!(
        m.c.try_enroll(&tarde, &novo_c(&m)),
        Err(Ok(Error::RosterClosed.into()))
    );
    assert_eq!(ticket(&m).balance(&tarde), 0);
    assert_eq!(m.c.roster().len(), MINIMO);
}

/// INV-14 (Q-21): depois do fechamento o escrivão não tira ninguém.
#[test]
fn impedir_depois_de_fechar_e_recusado() {
    let m = mesa();
    let js = jurados(&m, MINIMO as usize);
    inscrever_todos(&m, &js);
    m.c.lock(&K, &REVELA_EM, &m.accused);
    assert_eq!(m.c.try_impede(&js[0]), Err(Ok(Error::RosterClosed.into())));
    assert!(sac(&m).authorized(&js[0]), "continua autorizado");
}

// ---- o seal (T-038) ------------------------------------------------------

#[test]
fn selar_antes_de_fechar_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    assert_eq!(m.c.try_seal(&sig_a(&m)), Err(Ok(Error::RosterOpen.into())));
}

/// INV-13: só a round que o fechamento guardou serve. A assinatura de
/// qualquer outra round — mesmo autêntica, mesmo posterior — é recusada.
#[test]
fn selar_com_outra_rodada_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    assert_eq!(fechar_para(&m, ROUND_A, K).round, ROUND_A);
    let b = BytesN::from_array(&m.env, &SIG_B);
    assert!(m.c.verify(&ROUND_B, &b), "a assinatura B e autentica");
    assert_eq!(m.c.try_seal(&b), Err(Ok(Error::InvalidSignature.into())));
    assert_eq!(m.c.seal_state(), None);
}

#[test]
fn assinatura_forjada_e_recusada() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    fechar_para(&m, ROUND_A, K);
    let r = m.c.try_seal(&BytesN::from_array(&m.env, &SIG_FORJADA));
    assert_eq!(r, Err(Ok(Error::InvalidSignature.into())));
}

/// INV-13: não há segundo seal; o que o primeiro fixou não muda.
#[test]
fn selar_duas_vezes_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    fechar_para(&m, ROUND_A, K);
    let seal = m.c.seal(&sig_a(&m));
    assert_eq!(
        m.c.try_seal(&sig_a(&m)),
        Err(Ok(Error::AlreadySealed.into()))
    );
    assert_eq!(m.c.seal_state(), Some(seal));
}

/// N, no fechamento, conta só os authorized: cinco impedidos de sete deixam
/// N = 2, e três assentos são recusados já no `lock`.
#[test]
fn fechar_com_mais_assentos_que_autorizados_e_recusado() {
    let m = mesa();
    let js = jurados(&m, 7);
    inscrever_todos(&m, &js);
    for j in &js[..5] {
        m.c.impede(j);
    }
    m.env.ledger().with_mut(|l| {
        l.timestamp = QUICKNET_GENESIS + (ROUND_A - 2 - ROUND_MARGIN) * QUICKNET_PERIOD
    });
    assert_eq!(
        m.c.try_lock(&K, &REVELA_EM, &m.accused),
        Err(Ok(Error::InvalidSeatCount.into()))
    );
}

/// Com 2k ≥ N o threshold é o campo inteiro: todo juror está abaixo dele.
#[test]
fn com_2k_maior_que_n_o_limiar_e_o_campo() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, 7));
    fechar_para(&m, ROUND_A, 4);
    let seal = m.c.seal(&sig_a(&m));
    assert_eq!(seal.threshold, u256(&m.env, R_DECIMAL));
}

const R_DECIMAL: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";

// ---- o fluxo selado, N = 8, k = 3 (T-038) ----------------------------------

/// tests/fluxo-n8.json: oito jurados, seus compromissos, contas de seat,
/// `y` e uma proof de cada um, sob a chave de circuits/setup/ e a semente que
/// `seal` deriva da round de teste (circuits/gerar_fluxo.mjs).
const FLUXO: &str = include_str!("../../../tests/fluxo-n8.json");

struct JuradoDoFluxo {
    principal: Address,
    seat: Address,
    s: U256,
    y: U256,
    abaixo: bool,
    proof: Proof,
}

struct Fluxo {
    m: Mesa,
    js: std::vec::Vec<JuradoDoFluxo>,
    seed: U256,
    threshold: U256,
}

fn fluxo() -> Fluxo {
    let j: serde_json::Value = serde_json::from_str(FLUXO).unwrap();
    let m = mesa_com_minimo(1);
    let txt = |v: &serde_json::Value| v.as_str().unwrap().to_owned();
    let mut js = std::vec::Vec::new();
    for x in j["jurados"].as_array().unwrap() {
        let principal = Address::generate(&m.env);
        m.c.enroll(&principal, &u256(&m.env, &txt(&x["c"])));
        let bn = |h: &str| hex::decode(h).unwrap();
        js.push(JuradoDoFluxo {
            principal,
            seat: Address::from_str(&m.env, &txt(&x["assento"])),
            s: u256(&m.env, &txt(&x["s"])),
            y: u256(&m.env, &txt(&x["y"])),
            abaixo: x["abaixo_do_limiar"].as_bool().unwrap(),
            proof: Proof {
                a: BytesN::from_array(&m.env, &bn(&txt(&x["proof"]["a"])).try_into().unwrap()),
                b: BytesN::from_array(&m.env, &bn(&txt(&x["proof"]["b"])).try_into().unwrap()),
                c: BytesN::from_array(&m.env, &bn(&txt(&x["proof"]["c"])).try_into().unwrap()),
            },
        });
    }
    let f = fechar_para(&m, ROUND_A, 3);
    assert_eq!(f.root, u256(&m.env, &txt(&j["raiz"])), "a root do fixture");
    m.env.ledger().with_mut(|l| l.sequence_number = 1000);
    let seed = u256(&m.env, &txt(&j["seed"]));
    let threshold = u256(&m.env, &txt(&j["limiar"]));
    Fluxo {
        m,
        js,
        seed,
        threshold,
    }
}

impl Fluxo {
    fn selar(&self) -> Seal {
        self.m.c.seal(&sig_a(&self.m))
    }
    fn reivindicar(&self, i: usize) {
        let j = &self.js[i];
        self.m.c.claim(&j.seat, &j.proof, &j.y);
    }
    fn tentar(&self, i: usize) -> Option<Error> {
        let j = &self.js[i];
        match self.m.c.try_claim(&j.seat, &j.proof, &j.y) {
            Ok(_) => None,
            Err(Ok(e)) => Some(Error::try_from(e).expect("erro do contrato")),
            Err(Err(e)) => panic!("erro do host: {:?}", e),
        }
    }
    fn passar_o_prazo(&self) {
        let deadline = self.m.c.seal_state().unwrap().deadline;
        self.m
            .env
            .ledger()
            .with_mut(|l| l.sequence_number = deadline + 1);
    }
    fn chegar_a_revelacao(&self) {
        self.m
            .env
            .ledger()
            .with_mut(|l| l.sequence_number = l.sequence_number.max(REVELA_EM));
    }
    /// Os `k` menores `y` entre os jurados `is`, em ordem.
    fn menores(&self, is: &[usize], k: usize) -> std::vec::Vec<U256> {
        let mut ys: std::vec::Vec<U256> = is.iter().map(|&i| self.js[i].y.clone()).collect();
        ys.sort();
        ys.truncate(k);
        ys
    }
}

/// O seal fixa a semente — a aleatoriedade da round, deslocada 8 bits para
/// caber no campo — e o threshold `T = ⌊r·2k/N⌋`, e abre a window.
#[test]
fn o_selo_fixa_semente_limiar_e_prazo() {
    let f = fluxo();
    let seal = f.selar();
    assert_eq!(seal.seed, f.seed, "randomness >> 8");
    assert_eq!(seal.threshold, f.threshold, "floor(r*6/8)");
    assert_eq!(seal.deadline, 1000 + JANELA);
    assert!(!seal.widened);
    let esperado = Sealed {
        seed: seal.seed.clone(),
        threshold: seal.threshold.clone(),
        deadline: seal.deadline,
    };
    assert_eq!(
        f.m.env.events().all().filter_by_contract(&f.m.id),
        [esperado.to_xdr(&f.m.env, &f.m.id)]
    );
}

#[test]
fn reivindicar_antes_do_selo_e_recusado() {
    let f = fluxo();
    assert_eq!(f.tentar(1), Some(Error::NotSealed));
}

/// INV-15: com a proof, o `y` entra na lista, em ordem crescente.
#[test]
fn reivindicacoes_entram_em_ordem() {
    let f = fluxo();
    f.selar();
    for i in [7, 1, 3, 6] {
        f.reivindicar(i);
    }
    assert_eq!(
        f.m.c.claims(),
        soroban_sdk::Vec::from_slice(&f.m.env, &f.menores(&[7, 1, 3, 6], 4))
    );
}

#[test]
fn y_acima_do_limiar_e_recusado() {
    let f = fluxo();
    f.selar();
    assert!(!f.js[4].abaixo);
    assert_eq!(f.tentar(4), Some(Error::YAboveThreshold));
}

/// INV-15: um `y` por folha. A mesma reivindicação duas vezes é recusada.
#[test]
fn y_repetido_e_recusado() {
    let f = fluxo();
    f.selar();
    f.reivindicar(1);
    assert_eq!(f.tentar(1), Some(Error::DuplicateY));
}

/// A proof de um juror com o `y` de outro, ou enviada de outra conta, não
/// vale.
#[test]
fn prova_com_outro_y_ou_de_outra_conta_e_recusada() {
    let f = fluxo();
    f.selar();
    let (a, b) = (&f.js[1], &f.js[2]);
    assert_eq!(
        f.m.c.try_claim(&a.seat, &a.proof, &b.y),
        Err(Ok(Error::InvalidProof.into())),
        "outro y"
    );
    assert_eq!(
        f.m.c.try_claim(&b.seat, &a.proof, &a.y),
        Err(Ok(Error::InvalidProof.into())),
        "outra conta"
    );
}

#[test]
fn reivindicar_depois_do_prazo_e_recusado() {
    let f = fluxo();
    f.selar();
    f.passar_o_prazo();
    assert_eq!(f.tentar(1), Some(Error::WindowClosed));
}

/// INV-17: o evento de uma reivindicação leva o `y` e nada mais — nenhum
/// endereço ao lado dele.
#[test]
fn nenhum_evento_liga_y_a_endereco() {
    let f = fluxo();
    f.selar();
    f.reivindicar(3);
    let esperado = Claimed {
        y: f.js[3].y.clone(),
    };
    assert_eq!(
        f.m.env.events().all().filter_by_contract(&f.m.id),
        [esperado.to_xdr(&f.m.env, &f.m.id)]
    );
}

#[test]
fn finalizar_antes_do_prazo_e_recusado() {
    let f = fluxo();
    f.selar();
    f.reivindicar(1);
    assert_eq!(f.m.c.try_finalize(), Err(Ok(Error::WindowOpen.into())));
}

/// Os `k` menores `y` sentam; o seats é público e imutável (INV-16).
#[test]
fn finalizar_senta_os_k_menores_e_nao_muda_mais() {
    let f = fluxo();
    f.selar();
    let quem = [1, 2, 3, 5, 6];
    for i in quem {
        f.reivindicar(i);
    }
    f.passar_o_prazo();
    let end = f.m.c.finalize().unwrap();
    assert!(end.complete);
    assert_eq!(
        end.seats,
        soroban_sdk::Vec::from_slice(&f.m.env, &f.menores(&quem, 3))
    );
    assert_eq!(
        f.m.c.try_finalize(),
        Err(Ok(Error::AlreadyFinalized.into()))
    );
    assert_eq!(f.tentar(7), Some(Error::AlreadyFinalized));
    assert_eq!(f.m.c.panel(), Some(end));
}

/// Menos de `k` reivindicações: o threshold vai para o campo inteiro, uma vez, e
/// a window reabre; quem estava acima do threshold pode entrar.
#[test]
fn com_menos_de_k_o_limiar_se_amplia_uma_vez() {
    let f = fluxo();
    f.selar();
    f.reivindicar(1);
    f.reivindicar(2);
    f.passar_o_prazo();
    assert_eq!(f.m.c.finalize(), None, "ampliou, nao finalizou");
    let seal = f.m.c.seal_state().unwrap();
    assert!(seal.widened);
    assert_eq!(seal.threshold, u256(&f.m.env, R_DECIMAL));
    f.reivindicar(4);
    f.passar_o_prazo();
    let end = f.m.c.finalize().unwrap();
    assert!(end.complete);
    assert_eq!(end.seats.len(), 3);
}

/// Widened e ainda menos de `k`: o seats sai com quem há, marcado
/// incompleto — nunca um segundo sorteio.
#[test]
fn ampliado_e_ainda_menos_de_k_fica_incompleto() {
    let f = fluxo();
    f.selar();
    f.reivindicar(1);
    f.passar_o_prazo();
    assert_eq!(f.m.c.finalize(), None);
    f.passar_o_prazo();
    let end = f.m.c.finalize().unwrap();
    assert!(!end.complete);
    assert_eq!(end.seats.len(), 1);
}

/// INV-07 e INV-08 no modo selado: quem se recusa sai e o próximo `y` da
/// mesma lista senta — o seats fica com `k`, sem novo sorteio.
#[test]
fn recusar_tira_o_y_e_o_proximo_senta() {
    let f = fluxo();
    f.selar();
    let quem = [1, 2, 3, 5, 6, 7];
    for i in quem {
        f.reivindicar(i);
    }
    let menores = f.menores(&quem, 3);
    let saindo = quem
        .iter()
        .copied()
        .find(|&i| f.js[i].y == menores[0])
        .unwrap();
    let j = &f.js[saindo];
    f.m.c.recuse(&j.principal, &j.s);
    let esperado = Recused {
        juror: j.principal.clone(),
        s: j.s.clone(),
        y: j.y.clone(),
    };
    assert_eq!(
        f.m.env.events().all().filter_by_contract(&f.m.id),
        [esperado.to_xdr(&f.m.env, &f.m.id)]
    );
    f.passar_o_prazo();
    let end = f.m.c.finalize().unwrap();
    let restantes: std::vec::Vec<usize> = quem.iter().copied().filter(|&i| i != saindo).collect();
    assert_eq!(
        end.seats,
        soroban_sdk::Vec::from_slice(&f.m.env, &f.menores(&restantes, 3))
    );
}

#[test]
fn recusa_com_segredo_errado_ou_fora_de_hora_e_recusada() {
    let f = fluxo();
    let j = &f.js[1];
    assert_eq!(
        f.m.c.try_recuse(&j.principal, &j.s),
        Err(Ok(Error::NotSealed.into())),
        "antes do seal"
    );
    f.selar();
    assert_eq!(
        f.m.c.try_recuse(&j.principal, &f.js[2].s),
        Err(Ok(Error::SecretDoesNotMatch.into()))
    );
    f.m.c.recuse(&j.principal, &j.s);
    assert_eq!(
        f.m.c.try_recuse(&j.principal, &j.s),
        Err(Ok(Error::AlreadyRecused.into()))
    );
    assert_eq!(
        f.tentar(1),
        Some(Error::DuplicateY),
        "recusado nao reivindica"
    );
    // Ninguém reivindicou: o primeiro finalize amplia, o segundo fecha.
    f.passar_o_prazo();
    assert_eq!(f.m.c.finalize(), None);
    f.passar_o_prazo();
    assert!(!f.m.c.finalize().unwrap().complete);
    let k = &f.js[2];
    assert_eq!(
        f.m.c.try_recuse(&k.principal, &k.s),
        Err(Ok(Error::AlreadyFinalized.into())),
        "depois do seats"
    );
}

#[test]
fn revelar_antes_do_painel_ou_do_prazo_e_recusado() {
    let f = fluxo();
    f.selar();
    f.reivindicar(1);
    let j = &f.js[1];
    assert_eq!(
        f.m.c.try_reveal(&j.principal, &j.s),
        Err(Ok(Error::NotFinalized.into()))
    );
    f.passar_o_prazo();
    f.m.c.finalize();
    f.passar_o_prazo();
    f.m.c.finalize();
    assert!(f.m.env.ledger().sequence() < REVELA_EM);
    assert_eq!(
        f.m.c.try_reveal(&j.principal, &j.s),
        Err(Ok(Error::TooEarlyToReveal.into()))
    );
}

/// `reveal` confere `Poseidon(s) = c` e `Poseidon(seed, s) = y`, e diz se
/// aquele `y` sentou. É proof, nunca correção: o seats não muda (INV-16).
#[test]
fn revelar_confere_c_e_y_e_nao_muda_o_painel() {
    let f = fluxo();
    f.selar();
    for i in [1, 2, 3, 5] {
        f.reivindicar(i);
    }
    f.passar_o_prazo();
    let end = f.m.c.finalize().unwrap();
    f.chegar_a_revelacao();
    for i in [1, 2, 3, 5, 0] {
        let j = &f.js[i];
        assert_eq!(
            f.m.c.try_reveal(&j.principal, &f.js[(i + 1) % 8].s),
            Err(Ok(Error::SecretDoesNotMatch.into())),
            "segredo de outro"
        );
        let r = f.m.c.reveal(&j.principal, &j.s);
        assert_eq!(r.y, j.y);
        assert_eq!(r.seated, end.seats.contains(&j.y), "juror {}", i);
        assert!(!r.without_ticket);
        assert_eq!(
            f.m.c.try_reveal(&j.principal, &j.s),
            Err(Ok(Error::AlreadyRevealed.into()))
        );
    }
    assert_eq!(f.m.c.panel(), Some(end));
}

/// INV-12 no modo selado (decisão da humana, 2026-10-25): senta quem estava
/// autorizado no fechamento; o balance não pode ser conferido sem identificar o
/// juror. Quem sentou e já não tinha o ticket fica registrado no `reveal`.
#[test]
fn sentou_sem_bilhete_fica_registrado_no_reveal() {
    let f = fluxo();
    f.selar();
    for i in [1, 2, 3] {
        f.reivindicar(i);
    }
    f.passar_o_prazo();
    let end = f.m.c.finalize().unwrap();
    assert!(end.complete);
    let j = &f.js[2];
    ticket(&f.m).transfer(&j.principal, &f.js[0].principal, &ONE_TICKET);
    f.chegar_a_revelacao();
    let r = f.m.c.reveal(&j.principal, &j.s);
    assert!(r.seated && r.without_ticket);
    let esperado = Revealed {
        juror: j.principal.clone(),
        s: j.s.clone(),
        y: j.y.clone(),
        seated: true,
        without_ticket: true,
    };
    assert_eq!(
        f.m.env.events().all().filter_by_contract(&f.m.id),
        [esperado.to_xdr(&f.m.env, &f.m.id)]
    );
}

/// O fluxo inteiro, N = 8, k = 3: o juror 0 está abaixo do threshold e não
/// reivindica; o 4 está acima e é recusado; os outros seis reivindicam; um
/// se recusa; o seats sai com os três menores dos que ficaram; todos revelam
/// menos um. O que a cadeia guarda basta para dizer, de cada um: sentou,
/// revelou, não revelou, sorteado e ausente (T-039 recompõe isto).
#[test]
fn fluxo_completo_n8_k3() {
    let f = fluxo();
    let seal = f.selar();
    assert_eq!(seal.threshold, f.threshold);
    assert!(f.js[0].abaixo, "o 0 poderia, e nao reivindica");
    assert_eq!(f.tentar(4), Some(Error::YAboveThreshold));
    let quem = [1, 2, 3, 5, 6, 7];
    for i in quem {
        f.reivindicar(i);
    }
    let recusa = 6;
    f.m.c.recuse(&f.js[recusa].principal, &f.js[recusa].s);
    f.passar_o_prazo();
    let end = f.m.c.finalize().unwrap();
    let ficaram: std::vec::Vec<usize> = quem.iter().copied().filter(|&i| i != recusa).collect();
    let esperado = f.menores(&ficaram, 3);
    assert!(end.complete);
    assert_eq!(end.seats, soroban_sdk::Vec::from_slice(&f.m.env, &esperado));

    f.chegar_a_revelacao();
    let silencioso = ficaram
        .iter()
        .copied()
        .find(|&i| esperado.contains(&f.js[i].y))
        .unwrap();
    let mut sentados_revelados = 0;
    for i in [0, 1, 2, 3, 4, 5, 7] {
        if i == silencioso {
            continue;
        }
        let r = f.m.c.reveal(&f.js[i].principal, &f.js[i].s);
        if r.seated {
            sentados_revelados += 1;
        }
    }
    assert_eq!(sentados_revelados, 2, "tres sentados, um em silencio");
    assert_eq!(f.m.c.revelation(&f.js[silencioso].principal), None);
    assert_eq!(f.m.c.panel(), Some(end), "INV-16");
}

// ---- compromissos e árvore (T-035) ---------------------------------------

#[test]
fn compromisso_repetido_e_recusado() {
    let m = mesa();
    let a = Address::generate(&m.env);
    let b = Address::generate(&m.env);
    let c = inscrever(&m, &a);
    assert_eq!(
        m.c.try_enroll(&b, &c),
        Err(Ok(Error::DuplicateCommitment.into()))
    );
    assert_eq!(ticket(&m).balance(&b), 0, "a recusa nao emite ticket");
}

/// INV-17: um `c` visto uma vez nunca mais é aceito, nem depois de o dono
/// ter sido impedido.
#[test]
fn compromisso_de_impedido_nao_volta() {
    let m = mesa();
    let a = Address::generate(&m.env);
    let c = inscrever(&m, &a);
    m.c.impede(&a);
    let b = Address::generate(&m.env);
    assert_eq!(
        m.c.try_enroll(&b, &c),
        Err(Ok(Error::DuplicateCommitment.into()))
    );
}

#[test]
fn compromisso_zero_ou_fora_do_campo_e_recusado() {
    let m = mesa();
    let j = Address::generate(&m.env);
    let zero = U256::from_u32(&m.env, 0);
    assert_eq!(
        m.c.try_enroll(&j, &zero),
        Err(Ok(Error::InvalidCommitment.into())),
        "zero e a folha vazia"
    );
    let r = u256(
        &m.env,
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    );
    assert_eq!(
        m.c.try_enroll(&j, &r),
        Err(Ok(Error::InvalidCommitment.into())),
        "r nao e elemento do campo"
    );
}

#[test]
fn a_arvore_tem_64_folhas() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, 64));
    let j = Address::generate(&m.env);
    assert_eq!(
        m.c.try_enroll(&j, &novo_c(&m)),
        Err(Ok(Error::RosterFull.into()))
    );
}

#[test]
fn impedir_quem_nao_esta_no_rol_ou_duas_vezes_e_recusado() {
    let m = mesa();
    let a = Address::generate(&m.env);
    assert_eq!(m.c.try_impede(&a), Err(Ok(Error::UnknownJuror.into())));
    inscrever(&m, &a);
    m.c.impede(&a);
    assert_eq!(m.c.try_impede(&a), Err(Ok(Error::AlreadyImpeded.into())));
}

/// As raízes de subárvores vazias que o contrato embute são as da
/// circomlib (tests/vetores-arvore.json).
#[test]
fn os_zeros_sao_os_da_circomlib() {
    let env = Env::default();
    let j: serde_json::Value = serde_json::from_str(ARVORES).unwrap();
    let zeros = j["zeros"].as_array().unwrap();
    assert_eq!(zeros.len() as u32, DEPTH + 1);
    for (d, z) in zeros.iter().enumerate() {
        assert_eq!(
            Kleroterion::zero(&env, d as u32),
            u256(&env, z.as_str().unwrap()),
            "level {}",
            d
        );
    }
}

const ARVORES: &str = include_str!("../../../tests/vetores-arvore.json");

/// T-035: a árvore é construída pelo contrato, evento a evento, e a root
/// congelada no `lock` é a que a circomlib — o circuito — calcula para os
/// mesmos compromissos. Cada evento emitido é o que o vetor lista, então
/// quem lê só os eventos recompõe a root (scripts/arvore.py, web/arvore.mjs).
#[test]
fn a_raiz_bate_com_os_vetores_da_circomlib() {
    let j: serde_json::Value = serde_json::from_str(ARVORES).unwrap();
    for caso in j["arvores"].as_array().unwrap() {
        let nome = caso["nome"].as_str().unwrap();
        let n = caso["n"].as_u64().unwrap() as usize;
        let m = mesa_com_minimo(1);
        let js = jurados(&m, n);
        for ev in caso["eventos"].as_array().unwrap() {
            let i = ev["indice"].as_u64().unwrap() as u32;
            let juror = &js[i as usize];
            match ev["evento"].as_str().unwrap() {
                "enroll" => {
                    let c = u256(&m.env, ev["c"].as_str().unwrap());
                    m.c.enroll(juror, &c);
                    let esperado = Enrolled {
                        juror: juror.clone(),
                        index: i,
                        c,
                    };
                    assert_eq!(
                        m.env.events().all().filter_by_contract(&m.id),
                        [esperado.to_xdr(&m.env, &m.id)],
                        "{}: evento de inscricao {}",
                        nome,
                        i
                    );
                }
                "impede" => {
                    m.c.impede(juror);
                    let esperado = Impeded {
                        juror: juror.clone(),
                        index: i,
                    };
                    assert_eq!(
                        m.env.events().all().filter_by_contract(&m.id),
                        [esperado.to_xdr(&m.env, &m.id)],
                        "{}: evento de impedimento",
                        nome
                    );
                }
                outro => panic!("evento desconhecido {}", outro),
            }
        }
        let root = u256(&m.env, caso["raiz"].as_str().unwrap());
        assert_eq!(m.c.root(), root, "{}: root antes do fechamento", nome);
        let f = m.c.lock(&1u32, &REVELA_EM, &m.accused);
        assert_eq!(f.root, root, "{}: root congelada", nome);
        assert_eq!(
            f.n as u64,
            caso["autorizados"].as_u64().unwrap(),
            "{}: N",
            nome
        );
    }
}

/// O fechamento publica o que fixou, e a root não muda mais: depois dele
/// ninguém entra nem sai (INV-14).
#[test]
fn fechar_publica_raiz_n_rodada_k() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    let root = m.c.root();
    let f = fechar_para(&m, ROUND_A, K);
    assert_eq!(f.root, root);
    assert_eq!(f.n, MINIMO);
    let esperado = Closed {
        accused: m.accused.clone(),
        root: f.root.clone(),
        n: f.n,
        round: f.round,
        k: f.k,
        reveal_at: f.reveal_at,
        severity: K,
        suspended_until: f.ledger + AFASTAMENTO,
    };
    assert_eq!(
        m.env.events().all().filter_by_contract(&m.id),
        [esperado.to_xdr(&m.env, &m.id)]
    );
}

// ---- a severity, a parte e o suspension (T-045) -----------------------

/// A tabela padrão do estudo (docs/desenho-severity-suspension-revelacao.md
/// §1, aprovada em 2026-10-26): `k` 3/5/7/9, roster mínimo 16/40/40/100.
fn niveis_padrao(env: &Env) -> soroban_sdk::Vec<Level> {
    let mut v = soroban_sdk::Vec::new(env);
    for (k, roster_minimum) in [(3, 16), (5, 40), (7, 40), (9, 100)] {
        v.push_back(Level { k, roster_minimum });
    }
    v
}

/// Uma mesa com a tabela padrão em vez da de teste.
fn mesa_padrao() -> Mesa {
    let m = mesa();
    let verificador = m.c.verifier();
    let sac = m.env.register_stellar_asset_contract_v2(m.clerk.clone());
    sac.issuer().set_flag(IssuerFlags::RequiredFlag);
    sac.issuer().set_flag(IssuerFlags::RevocableFlag);
    let ticket = sac.address();
    let id = m.env.register(
        Kleroterion,
        (
            &m.clerk,
            &ticket,
            &verificador,
            &JANELA,
            niveis_padrao(&m.env),
            &AFASTAMENTO,
            &ESPERA,
        ),
    );
    token::StellarAssetClient::new(&m.env, &ticket).set_admin(&id);
    let c = KleroterionClient::new(&m.env, &id);
    Mesa {
        ticket,
        id,
        c,
        constituido: std::vec::Vec::new(),
        ..m
    }
}

/// INV-19: `k` vem do nível, não de quem fecha. Na tabela padrão, o nível 2
/// senta 5 e o fechamento guarda o nível.
#[test]
fn k_vem_da_gravidade() {
    let m = mesa_padrao();
    assert_eq!(m.c.levels(), niveis_padrao(&m.env));
    inscrever_todos(&m, &jurados(&m, 40));
    let f = m.c.lock(&2u32, &REVELA_EM, &m.accused);
    assert_eq!((f.severity, f.k), (2, 5));
}

/// INV-09 por nível: 39 inscritos fecham no nível 1, não no 2 nem no 4.
#[test]
fn rol_abaixo_do_minimo_do_nivel_e_recusado() {
    let m = mesa_padrao();
    inscrever_todos(&m, &jurados(&m, 39));
    for g in [2u32, 3, 4] {
        assert_eq!(
            m.c.try_lock(&g, &REVELA_EM, &m.accused),
            Err(Ok(Error::RosterBelowMinimum.into())),
            "level {}",
            g
        );
    }
    assert_eq!(m.c.lock(&1u32, &REVELA_EM, &m.accused).k, 3);
}

#[test]
fn gravidade_fora_da_tabela_e_recusada() {
    let m = mesa_padrao();
    inscrever_todos(&m, &jurados(&m, 16));
    for g in [0u32, 5, u32::MAX] {
        assert_eq!(
            m.c.try_lock(&g, &REVELA_EM, &m.accused),
            Err(Ok(Error::InvalidSeverity.into())),
            "level {}",
            g
        );
    }
    assert_eq!(m.c.closing(), None);
}

/// Uma tabela vazia, ou com um nível de zero assentos, não constitui nada.
#[test]
fn tabela_vazia_ou_k_zero_e_recusada() {
    let env = Env::default();
    let clerk = Address::generate(&env);
    let ticket = Address::generate(&env);
    let verificador = Address::generate(&env);
    let mut zero = niveis_padrao(&env);
    zero.set(
        1,
        Level {
            k: 0,
            roster_minimum: 40,
        },
    );
    for levels in [soroban_sdk::Vec::<Level>::new(&env), zero] {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            env.register(
                Kleroterion,
                (
                    &clerk,
                    &ticket,
                    &verificador,
                    &JANELA,
                    levels,
                    &AFASTAMENTO,
                    &ESPERA,
                ),
            )
        }));
        assert!(r.is_err(), "a tabela invalida e recusada no construtor");
    }
}

/// O construtor publica a regra antes de qualquer caso.
#[test]
fn constituir_publica_a_tabela_e_o_afastamento() {
    let m = mesa();
    let esperado = Constituted {
        levels: niveis_de_teste(&m.env, MINIMO),
        suspension: AFASTAMENTO,
        minimum_wait: ESPERA,
    };
    assert_eq!(m.constituido, [esperado.to_xdr(&m.env, &m.id)]);
}

/// INV-18: a pessoa julgada, inscrita e autorizada, é impedida no próprio
/// fechamento — antes de a root ser fixada. A folha dela vira zero, `N`
/// desce, o ticket fica desautorizado, e o `Impeded` vem antes do `Closed`.
#[test]
fn o_acusado_inscrito_e_impedido_no_fechamento() {
    let m = mesa();
    let js = jurados(&m, MINIMO as usize);
    inscrever_todos(&m, &js);
    let accused = js[2].clone();

    // A root esperada: a do mesmo roster com a folha 2 zerada por `impede`.
    let gemea = mesa();
    let gjs = jurados(&gemea, MINIMO as usize);
    inscrever_todos(&gemea, &gjs);
    gemea.c.impede(&gjs[2]);
    let raiz_sem_a_parte = gemea.c.root();

    let f = m.c.lock(&K, &REVELA_EM, &accused);
    assert_eq!(f.root, raiz_sem_a_parte, "a folha da parte e zero");
    assert_eq!(f.n, MINIMO - 1);
    let esperados = [
        Impeded {
            juror: accused.clone(),
            index: 2,
        }
        .to_xdr(&m.env, &m.id),
        Closed {
            accused: accused.clone(),
            root: f.root.clone(),
            n: f.n,
            round: f.round,
            k: K,
            reveal_at: REVELA_EM,
            severity: K,
            suspended_until: f.ledger + AFASTAMENTO,
        }
        .to_xdr(&m.env, &m.id),
    ];
    let todos = m.env.events().all().filter_by_contract(&m.id);
    let nossos: std::vec::Vec<_> = todos
        .events()
        .iter()
        .filter(|e| esperados.contains(e))
        .cloned()
        .collect();
    assert_eq!(nossos, esperados, "Impeded e depois Closed");
    // Depois de ler os eventos: outra chamada os apagaria.
    assert!(!sac(&m).authorized(&accused), "o ticket dela foi recolhido");
}

/// Uma parte já impedida antes do fechamento não é impedida de novo, e uma
/// parte que nunca se inscreveu não muda nada no roster.
#[test]
fn acusado_ja_impedido_ou_fora_do_rol_nao_muda_o_rol() {
    let m = mesa();
    let js = jurados(&m, MINIMO as usize + 1);
    inscrever_todos(&m, &js);
    m.c.impede(&js[0]);
    let root = m.c.root();
    let f = m.c.lock(&K, &REVELA_EM, &js[0]);
    assert_eq!((f.root, f.n), (root, MINIMO));

    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    let root = m.c.root();
    let f = fechar_para(&m, ROUND_A, K);
    assert_eq!((f.root, f.n), (root, MINIMO), "ACUSADO nao esta no roster");
}

/// INV-18: quem fecha o roster não fecha o próprio caso.
#[test]
fn escrivao_acusado_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    assert_eq!(
        m.c.try_lock(&K, &REVELA_EM, &m.clerk),
        Err(Ok(Error::ClerkIsParty.into()))
    );
    assert_eq!(m.c.closing(), None);
}

/// INV-18, de ponta a ponta: no fluxo de N = 8 da fixture, a parte é um
/// juror que estaria abaixo do threshold. Ela é impedida no fechamento, e a
/// proof que antes valia deixa de valer contra a nova root.
#[test]
fn a_parte_nao_prova_assento_no_proprio_caso() {
    let j: serde_json::Value = serde_json::from_str(FLUXO).unwrap();
    let m = mesa_com_minimo(1);
    let txt = |v: &serde_json::Value| v.as_str().unwrap().to_owned();
    let mut principais = std::vec::Vec::new();
    for x in j["jurados"].as_array().unwrap() {
        let principal = Address::generate(&m.env);
        m.c.enroll(&principal, &u256(&m.env, &txt(&x["c"])));
        principais.push(principal);
    }
    let x = &j["jurados"][1];
    assert!(x["abaixo_do_limiar"].as_bool().unwrap());
    m.env.ledger().with_mut(|l| {
        l.timestamp = QUICKNET_GENESIS + (ROUND_A - 2 - ROUND_MARGIN) * QUICKNET_PERIOD
    });
    let f = m.c.lock(&3u32, &REVELA_EM, &principais[1]);
    assert_ne!(f.root, u256(&m.env, &txt(&j["raiz"])), "a root mudou");
    m.env.ledger().with_mut(|l| l.sequence_number = 1000);
    m.c.seal(&sig_a(&m));
    let bn = |h: &str| hex::decode(h).unwrap();
    let proof = Proof {
        a: BytesN::from_array(&m.env, &bn(&txt(&x["proof"]["a"])).try_into().unwrap()),
        b: BytesN::from_array(&m.env, &bn(&txt(&x["proof"]["b"])).try_into().unwrap()),
        c: BytesN::from_array(&m.env, &bn(&txt(&x["proof"]["c"])).try_into().unwrap()),
    };
    let seat = Address::from_str(&m.env, &txt(&x["assento"]));
    assert_eq!(
        m.c.try_claim(&seat, &proof, &u256(&m.env, &txt(&x["y"]))),
        Err(Ok(Error::InvalidProof.into()))
    );
}

/// INV-20: afastada só a parte, só do fechamento até `suspended_until`.
#[test]
fn afastado_so_para_o_acusado_e_so_no_prazo() {
    let m = mesa();
    let js = jurados(&m, MINIMO as usize);
    inscrever_todos(&m, &js);
    assert!(!m.c.suspended(&m.accused), "antes do caso, ninguem");
    m.env.ledger().with_mut(|l| l.sequence_number = 2000);
    let f = m.c.lock(&K, &REVELA_EM, &m.accused);
    assert_eq!(f.suspended_until, 2000 + AFASTAMENTO);
    assert!(m.c.suspended(&m.accused));
    assert!(!m.c.suspended(&js[0]), "um juror nao esta afastado");
    assert!(!m.c.suspended(&m.clerk));
    m.env
        .ledger()
        .with_mut(|l| l.sequence_number = f.suspended_until - 1);
    assert!(m.c.suspended(&m.accused), "ate o ultimo ledger do deadline");
    m.env
        .ledger()
        .with_mut(|l| l.sequence_number = f.suspended_until);
    assert!(!m.c.suspended(&m.accused), "o deadline vence sozinho");
}

// ---- quem sentou, para o processo (T-047) -------------------------------

/// `seated` é falso antes do seats, verdadeiro depois para quem sentou, e
/// falso para quem se recusou ou nunca foi seat.
#[test]
fn seated_so_depois_do_painel_e_so_para_quem_sentou() {
    let f = fluxo();
    f.selar();
    for i in [1, 2, 3, 5, 6, 7] {
        f.reivindicar(i);
    }
    let seat = |i: usize| f.js[i].seat.clone();
    assert!(!f.m.c.seated(&seat(1)), "antes do seats, ninguem");
    f.m.c.recuse(&f.js[6].principal, &f.js[6].s);
    f.passar_o_prazo();
    let end = f.m.c.finalize().unwrap();
    for i in [1usize, 2, 3, 5, 7] {
        assert_eq!(
            f.m.c.seated(&seat(i)),
            end.seats.contains(&f.js[i].y),
            "juror {}",
            i
        );
    }
    assert!(!f.m.c.seated(&seat(6)), "quem se recusou nao sentou");
    assert!(
        !f.m.c.seated(&f.js[0].principal),
        "conta principal nao e seat"
    );
}

/// Uma conta de seat reivindica um seat só.
#[test]
fn a_mesma_conta_de_assento_nao_reivindica_duas_vezes() {
    let f = fluxo();
    f.selar();
    f.reivindicar(1);
    let j = &f.js[2];
    assert_eq!(
        f.m.c.try_claim(&f.js[1].seat, &j.proof, &j.y),
        Err(Ok(Error::DuplicateSeat.into()))
    );
}

// ---- a revelação tardia (T-046) -----------------------------------------

/// INV-21: o `lock` recusa `reveal_at` antes de `ledger + minimum_wait`, e
/// aceita exatamente no limite.
#[test]
fn revelar_antes_da_espera_minima_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    m.env.ledger().with_mut(|l| l.sequence_number = 5_000);
    assert_eq!(
        m.c.try_lock(&K, &(5_000 + ESPERA - 1), &m.accused),
        Err(Ok(Error::RevealTooEarly.into()))
    );
    assert_eq!(m.c.closing(), None);
    assert_eq!(m.c.lock(&K, &(5_000 + ESPERA), &m.accused).reveal_at, 6_000);
}

fn ttl_persistente(m: &Mesa, chave: &DataKey) -> u32 {
    use soroban_sdk::testutils::storage::Persistent as _;
    m.env
        .as_contract(&m.id, || m.env.storage().persistent().get_ttl(chave))
}

fn ttl_da_instancia(m: &Mesa) -> u32 {
    use soroban_sdk::testutils::storage::Instance as _;
    m.env
        .as_contract(&m.id, || m.env.storage().instance().get_ttl())
}

/// INV-21: depois do `lock` e do `finalize`, o que `reveal` lê — a instância,
/// o compromisso de cada inscrito e o seats — vive até `reveal_at + window`.
/// Sem a extensão, o compromisso morreria em `min_persistent_entry_ttl`
/// (4.096 ledgers no host de teste), bem antes da revelação.
#[test]
fn o_que_o_reveal_le_vive_ate_a_revelacao() {
    let f = fluxo();
    let m = &f.m;
    let reveal_at = m.c.closing().unwrap().reveal_at;
    let ate = |agora: u32| reveal_at + JANELA - agora;
    let agora = m.env.ledger().sequence();
    for j in &f.js {
        assert!(ttl_persistente(m, &DataKey::Commitment(j.principal.clone())) >= ate(agora));
    }
    assert!(ttl_da_instancia(m) >= ate(agora));
    f.selar();
    for i in [1, 2, 3, 5, 7] {
        f.reivindicar(i);
    }
    f.passar_o_prazo();
    f.m.c.finalize();
    let agora = m.env.ledger().sequence();
    assert!(ttl_persistente(m, &DataKey::Panel) >= ate(agora));
    assert!(ttl_da_instancia(m) >= ate(agora));
    f.chegar_a_revelacao();
    assert!(m.c.reveal(&f.js[1].principal, &f.js[1].s).seated);
}

/// INV-21, o teto: uma revelação além do máximo da rede leva o TTL ao
/// máximo, não além; o resto do caminho é de quem estender depois.
#[test]
fn a_extensao_para_no_teto_da_rede() {
    let m = mesa();
    let js = jurados(&m, MINIMO as usize);
    inscrever_todos(&m, &js);
    m.env.ledger().with_mut(|l| l.max_entry_ttl = 20_000);
    m.c.lock(&K, &1_000_000, &m.accused);
    let teto = m.env.as_contract(&m.id, || m.env.storage().max_ttl());
    assert_eq!(
        ttl_persistente(&m, &DataKey::Commitment(js[0].clone())),
        teto
    );
    assert_eq!(ttl_da_instancia(&m), teto);
}

// ---- a proof de seat (T-037) ----------------------------------------

/// tests/prova-claim.json: uma proof de `pinakion prove` para o juror 3 da
/// árvore de 17 de tests/vetores-arvore.json, sob a chave de circuits/setup/.
const PROVA: &str = include_str!("../../../tests/prova-claim.json");

struct ProvaDeAssento {
    proof: Proof,
    seed: U256,
    y: U256,
    claimer: U256,
    seat: Address,
}

fn prova_da_cli(env: &Env) -> ProvaDeAssento {
    let j: serde_json::Value = serde_json::from_str(PROVA).unwrap();
    let b64 = |h: &str| -> BytesN<64> {
        BytesN::from_array(env, &hex::decode(h).unwrap().try_into().unwrap())
    };
    let b128 = |h: &str| -> BytesN<128> {
        BytesN::from_array(env, &hex::decode(h).unwrap().try_into().unwrap())
    };
    let txt = |v: &serde_json::Value| v.as_str().unwrap().to_owned();
    let seat = j["_"].as_str().unwrap();
    let seat = &seat[seat.find("seat account ").unwrap() + 13..][..56];
    ProvaDeAssento {
        proof: Proof {
            a: b64(&txt(&j["proof"]["a"])),
            b: b128(&txt(&j["proof"]["b"])),
            c: b64(&txt(&j["proof"]["c"])),
        },
        seed: u256(env, &txt(&j["decimal"]["seed"])),
        y: u256(env, &txt(&j["decimal"]["y"])),
        claimer: u256(env, &txt(&j["decimal"]["claimer"])),
        seat: Address::from_str(env, seat),
    }
}

/// A mesa da árvore de 17 dos vetores: os mesmos compromissos, o mesmo
/// impedido, fechada. A root é a da proof.
fn mesa_da_arvore_de_17() -> Mesa {
    let j: serde_json::Value = serde_json::from_str(ARVORES).unwrap();
    let caso = j["arvores"]
        .as_array()
        .unwrap()
        .iter()
        .find(|a| a["n"] == 17)
        .unwrap()
        .clone();
    let m = mesa_com_minimo(1);
    let js = jurados(&m, 17);
    for ev in caso["eventos"].as_array().unwrap() {
        let i = ev["indice"].as_u64().unwrap() as usize;
        match ev["evento"].as_str().unwrap() {
            "enroll" => m.c.enroll(&js[i], &u256(&m.env, ev["c"].as_str().unwrap())),
            _ => m.c.impede(&js[i]),
        }
    }
    fechar_para(&m, ROUND_A, K);
    m
}

fn vale(m: &Mesa, p: &Proof, seed: &U256, y: &U256, seat: &Address) -> bool {
    m.env.as_contract(&m.id, || {
        Kleroterion::prova_de_assento_vale(&m.env, p, seed, y, seat)
    })
}

/// `claimer = SHA-256(strkey) >> 8`, o mesmo que `circuits/pinakion.mjs`
/// calcula fora da cadeia (DEC-024).
#[test]
fn claimer_e_o_da_cli() {
    let m = mesa();
    let p = prova_da_cli(&m.env);
    let c = m
        .env
        .as_contract(&m.id, || Kleroterion::claimer(&m.env, &p.seat));
    assert_eq!(c, p.claimer);
}

/// INV-15: a proof vale contra a root que o contrato guarda, com a semente
/// dada e a conta de onde vem a chamada; as entradas públicas são montadas
/// pelo contrato, nunca recebidas prontas.
#[test]
fn a_prova_da_cli_vale_contra_a_raiz_do_contrato() {
    let m = mesa_da_arvore_de_17();
    let p = prova_da_cli(&m.env);
    assert!(vale(&m, &p.proof, &p.seed, &p.y, &p.seat));
}

#[test]
fn outro_y_outra_semente_ou_outra_conta_nao_vale() {
    let m = mesa_da_arvore_de_17();
    let p = prova_da_cli(&m.env);
    let um = U256::from_u32(&m.env, 1);
    assert!(!vale(&m, &p.proof, &p.seed, &p.y.add(&um), &p.seat), "y");
    assert!(!vale(&m, &p.proof, &p.seed.add(&um), &p.y, &p.seat), "seed");
    let outra = Address::generate(&m.env);
    assert!(!vale(&m, &p.proof, &p.seed, &p.y, &outra), "conta");
    // seed e y trocados de lugar
    assert!(!vale(&m, &p.proof, &p.y, &p.seed, &p.seat), "permutadas");
}

#[test]
fn um_byte_da_prova_alterado_nao_vale() {
    let m = mesa_da_arvore_de_17();
    let p = prova_da_cli(&m.env);
    let mut c = p.proof.c.to_array();
    c[63] ^= 0x01;
    let alterada = Proof {
        c: BytesN::from_array(&m.env, &c),
        ..p.proof.clone()
    };
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vale(&m, &alterada, &p.seed, &p.y, &p.seat)
    }));
    assert!(!r.unwrap_or(false));
}

/// Outra root — o mesmo roster sem o impedimento — e a mesma proof não vale.
#[test]
fn a_prova_nao_vale_em_outro_rol() {
    let m = mesa_com_minimo(1);
    inscrever_todos(&m, &jurados(&m, 17));
    fechar_para(&m, ROUND_A, K);
    let p = prova_da_cli(&m.env);
    assert!(!vale(&m, &p.proof, &p.seed, &p.y, &p.seat));
}

#[test]
fn antes_de_fechar_nao_ha_raiz_para_provar() {
    let m = mesa();
    let p = prova_da_cli(&m.env);
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vale(&m, &p.proof, &p.seed, &p.y, &p.seat)
    }));
    assert!(r.is_err(), "sem fechamento, recusa");
}

#[test]
fn o_verificador_e_o_do_construtor() {
    let m = mesa();
    let v = m.c.verifier();
    let vk = groth16_verifier::Groth16VerifierClient::new(&m.env, &v).vk();
    assert_eq!(vk.ic.len(), 5, "quatro entradas publics");
}

/// Custo, no host de teste, de conferir uma proof pelo kleroterion: montar
/// as entradas, o SHA-256 do endereço e a chamada ao verificador.
const PROOF_COST: (u64, u64) = (29_284_586, 331_248);

#[test]
fn custo_medido_da_prova() {
    let m = mesa_da_arvore_de_17();
    let p = prova_da_cli(&m.env);
    m.env.cost_estimate().budget().reset_unlimited();
    assert!(vale(&m, &p.proof, &p.seed, &p.y, &p.seat));
    let b = m.env.cost_estimate().budget();
    let custo = (b.cpu_instruction_cost(), b.memory_bytes_cost());
    assert_eq!(custo, PROOF_COST, "proof (cpu, mem)");
}

// ---- as cópias do beacon-verifier: verify e randomness -------------------

#[test]
fn a_aleatoriedade_e_a_que_a_drand_publica() {
    let m = mesa();
    assert_eq!(m.c.randomness(&sig_a(&m)).to_array(), RANDOMNESS_A);
}

#[test]
fn rodada_real_verifica() {
    let m = mesa();
    assert!(m.c.verify(&ROUND_A, &sig_a(&m)));
    assert!(!m.c.verify(&(ROUND_A + 1), &sig_a(&m)));
}

/// Custo no host de teste, `(cpu, mem)`, não numa transação da testnet:
/// a inscrição do 64.º juror (o caminho inteiro da árvore e as três chamadas
/// ao SAC), um impedimento e o fechamento. Mudou, fica vermelho (G2); G7
/// confere que docs/SPEC.md ainda diz estes números.
const ENROLL_COST: (u64, u64) = (10_593_604, 1_783_415);
const IMPEDE_COST: (u64, u64) = (8_797_260, 1_106_822);
const LOCK_COST: (u64, u64) = (10_775_151, 3_780_028);

#[test]
fn custo_medido_de_enroll_impede_lock() {
    let m = mesa();
    let js = jurados(&m, 64);
    inscrever_todos(&m, &js[..63]);
    let medir = |f: &dyn Fn()| {
        m.env.cost_estimate().budget().reset_unlimited();
        f();
        let b = m.env.cost_estimate().budget();
        (b.cpu_instruction_cost(), b.memory_bytes_cost())
    };
    let c = novo_c(&m);
    let enroll = medir(&|| m.c.enroll(&js[63], &c));
    let impede = medir(&|| m.c.impede(&js[0]));
    let lock = medir(&|| {
        fechar_para(&m, ROUND_A, K);
    });
    assert_eq!(enroll, ENROLL_COST, "enroll (cpu, mem)");
    assert_eq!(impede, IMPEDE_COST, "impede (cpu, mem)");
    assert_eq!(lock, LOCK_COST, "lock (cpu, mem)");
}

/// Custo no host de teste, `(cpu, mem)`, das chamadas do fluxo selado com
/// N = 8, k = 3: o seal (verificação BLS12-381 e o threshold), a sexta
/// reivindicação (a proof e a inserção), uma recusa, o `finalize` e um
/// `reveal`. Mudou, fica vermelho (G2); G7 confere docs/SPEC.md.
const SEAL_COST: (u64, u64) = (27_019_754, 300_471);
const CLAIM_COST: (u64, u64) = (29_669_385, 464_867);
const RECUSE_COST: (u64, u64) = (2_916_231, 364_009);
const FINALIZE_COST: (u64, u64) = (343_745, 182_644);
const REVEAL_COST: (u64, u64) = (2_972_874, 408_181);

#[test]
fn custo_medido_do_fluxo_selado() {
    let f = fluxo();
    let medir = |g: &dyn Fn()| {
        f.m.env.cost_estimate().budget().reset_unlimited();
        g();
        let b = f.m.env.cost_estimate().budget();
        (b.cpu_instruction_cost(), b.memory_bytes_cost())
    };
    let seal = medir(&|| {
        f.selar();
    });
    for i in [1, 2, 3, 5, 6] {
        f.reivindicar(i);
    }
    let claim = medir(&|| f.reivindicar(7));
    let recuse = medir(&|| f.m.c.recuse(&f.js[6].principal, &f.js[6].s));
    f.passar_o_prazo();
    let finalize = medir(&|| {
        f.m.c.finalize();
    });
    f.chegar_a_revelacao();
    let reveal = medir(&|| {
        f.m.c.reveal(&f.js[1].principal, &f.js[1].s);
    });
    assert_eq!(seal, SEAL_COST, "seal");
    assert_eq!(claim, CLAIM_COST, "claim");
    assert_eq!(recuse, RECUSE_COST, "recuse");
    assert_eq!(finalize, FINALIZE_COST, "finalize");
    assert_eq!(reveal, REVEAL_COST, "reveal");
}

// ---- o vetor de ouro selado (T-039) --------------------------------------

const OURO: &str = include_str!("../../../tests/vetores-de-ouro.json");

/// O vetor selado de tests/vetores-de-ouro.json é a lista exata de eventos
/// que este contrato emite no fluxo complete: refaz o fluxo e confere cada
/// evento. Os verificadores independentes (G5, G6) partem desses eventos.
#[test]
fn bate_com_o_vetor_selado() {
    let ouro: serde_json::Value = serde_json::from_str(OURO).unwrap();
    let vetor = &ouro["selados"][0];
    let eventos = vetor["eventos"].as_array().unwrap();
    let fluxo: serde_json::Value = serde_json::from_str(FLUXO).unwrap();
    let fj = fluxo["jurados"].as_array().unwrap();

    let m = mesa_com_minimo(1);
    let env = &m.env;
    let txt = |v: &serde_json::Value| v.as_str().unwrap().to_owned();
    let n = |v: &serde_json::Value| u256(env, &txt(v));
    let addr = |v: &serde_json::Value| Address::from_str(env, &txt(v));
    let bn = |v: &serde_json::Value| hex::decode(txt(v)).unwrap();
    let mut e = eventos.iter();
    let confere = |m: &Mesa, esperado: soroban_sdk::xdr::ContractEvent, o_que: &str| {
        assert_eq!(
            m.env.events().all().filter_by_contract(&m.id),
            [esperado],
            "{}",
            o_que
        );
    };
    let mut proximo = || e.next().unwrap().clone();

    let v = proximo();
    let mut levels = soroban_sdk::Vec::new(env);
    for x in v["niveis"].as_array().unwrap() {
        levels.push_back(Level {
            k: x["k"].as_u64().unwrap() as u32,
            roster_minimum: x["rol_minimo"].as_u64().unwrap() as u32,
        });
    }
    let esperado = Constituted {
        levels,
        suspension: v["afastamento"].as_u64().unwrap() as u32,
        minimum_wait: v["espera_minima"].as_u64().unwrap() as u32,
    };
    assert_eq!(m.constituido, [esperado.to_xdr(env, &m.id)], "Constituted");

    for _ in 0..8 {
        let v = proximo();
        let juror = addr(&v["jurado"]);
        m.c.enroll(&juror, &n(&v["c"]));
        let esperado = Enrolled {
            juror,
            index: v["indice"].as_u64().unwrap() as u32,
            c: n(&v["c"]),
        };
        confere(&m, esperado.to_xdr(env, &m.id), "Enrolled");
    }
    let v = proximo();
    let f = fechar_para(&m, ROUND_A, v["k"].as_u64().unwrap() as u32);
    let esperado = Closed {
        accused: addr(&v["acusado"]),
        root: n(&v["raiz"]),
        n: v["n"].as_u64().unwrap() as u32,
        round: v["rodada"].as_u64().unwrap(),
        k: f.k,
        reveal_at: v["revela_em"].as_u64().unwrap() as u32,
        severity: v["gravidade"].as_u64().unwrap() as u32,
        suspended_until: v["afastado_ate"].as_u64().unwrap() as u32,
    };
    confere(&m, esperado.to_xdr(env, &m.id), "Closed");

    m.env.ledger().with_mut(|l| l.sequence_number = 1000);
    let v = proximo();
    m.c.seal(&sig_a(&m));
    let esperado = Sealed {
        seed: n(&v["seed"]),
        threshold: n(&v["limiar"]),
        deadline: v["prazo"].as_u64().unwrap() as u32,
    };
    confere(&m, esperado.to_xdr(env, &m.id), "Sealed");

    for i in [1usize, 2, 3, 5, 6, 7] {
        let v = proximo();
        let j = &fj[i];
        let proof = Proof {
            a: BytesN::from_array(env, &bn(&j["proof"]["a"]).try_into().unwrap()),
            b: BytesN::from_array(env, &bn(&j["proof"]["b"]).try_into().unwrap()),
            c: BytesN::from_array(env, &bn(&j["proof"]["c"]).try_into().unwrap()),
        };
        m.c.claim(&addr(&j["assento"]), &proof, &n(&v["y"]));
        let esperado = Claimed { y: n(&v["y"]) };
        confere(&m, esperado.to_xdr(env, &m.id), "Claimed");
    }
    let v = proximo();
    m.c.recuse(&addr(&v["jurado"]), &n(&v["s"]));
    let esperado = Recused {
        juror: addr(&v["jurado"]),
        s: n(&v["s"]),
        y: n(&v["y"]),
    };
    confere(&m, esperado.to_xdr(env, &m.id), "Recused");

    let deadline = m.c.seal_state().unwrap().deadline;
    m.env
        .ledger()
        .with_mut(|l| l.sequence_number = deadline + 1);
    let v = proximo();
    m.c.finalize();
    let mut seats = soroban_sdk::Vec::new(env);
    for y in v["painel"].as_array().unwrap() {
        seats.push_back(n(y));
    }
    let esperado = Finalized {
        seats,
        complete: v["completo"].as_bool().unwrap(),
    };
    confere(&m, esperado.to_xdr(env, &m.id), "Finalized");

    m.env.ledger().with_mut(|l| l.sequence_number = REVELA_EM);
    for v in e.by_ref() {
        m.c.reveal(&addr(&v["jurado"]), &n(&v["s"]));
        let esperado = Revealed {
            juror: addr(&v["jurado"]),
            s: n(&v["s"]),
            y: n(&v["y"]),
            seated: v["sentado"].as_bool().unwrap(),
            without_ticket: v["sem_bilhete"].as_bool().unwrap(),
        };
        confere(&m, esperado.to_xdr(env, &m.id), "Revealed");
    }
}
