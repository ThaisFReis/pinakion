#![cfg(test)]
extern crate std;
use super::*;
use soroban_sdk::testutils::{
    Address as _, Events as _, IssuerFlags, Ledger as _, MockAuth, MockAuthInvoke,
};
use soroban_sdk::{token, Address, Bytes, BytesN, Env, Event, IntoVal, U256};
use std::borrow::ToOwned;
use std::cell::Cell;

// Vetores partilhados com contracts/beacon-verifier/src/test.rs. Mesma rodada,
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
/// Assinatura da rodada 1000000 feita por um par de chaves que não é o da
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

/// Mínimo do rol usado nos testes. Sete, como a mesa de um júri pequeno.
const MINIMO: u32 = 7;

/// Janela de reivindicação, em ledgers, nos testes.
const JANELA: u32 = 100;

struct Mesa {
    env: Env,
    bilhete: Address,
    id: Address,
    c: KleroterionClient<'static>,
    /// Próximo compromisso de teste: cada jurado recebe um `c` diferente.
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
    env.ledger().with_mut(|l| l.timestamp = GENESIS_QUICKNET);
    let escrivao = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(escrivao.clone());
    sac.issuer().set_flag(IssuerFlags::RequiredFlag);
    sac.issuer().set_flag(IssuerFlags::RevocableFlag);
    let bilhete = sac.address();
    let verificador = env.register(groth16_verifier::Groth16Verifier, ());
    let id = env.register(
        Kleroterion,
        (&escrivao, &bilhete, &minimo, &verificador, &JANELA),
    );
    token::StellarAssetClient::new(&env, &bilhete).set_admin(&id);
    let c = KleroterionClient::new(&env, &id);
    Mesa {
        env,
        bilhete,
        id,
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
    token::StellarAssetClient::new(&m.env, &m.bilhete)
}

fn bilhete(m: &Mesa) -> token::TokenClient<'static> {
    token::TokenClient::new(&m.env, &m.bilhete)
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
        bilhete(&m).balance(&j),
        UM_BILHETE,
        "um bilhete, nem mais nem menos"
    );
    assert_eq!(
        m.c.commitment(&j),
        Some(c),
        "o compromisso fica preso ao endereco"
    );
    assert!(sac(&m).authorized(&j), "inscrito sai autorizado");
    assert_eq!(m.c.roster(), soroban_sdk::vec![&m.env, j]);
}

#[test]
fn a_ordem_do_rol_e_a_de_inscricao() {
    let m = mesa();
    let js = jurados(&m, 3);
    inscrever_todos(&m, &js);
    let rol = m.c.roster();
    for (i, j) in js.iter().enumerate() {
        assert_eq!(rol.get(i as u32).unwrap(), *j, "posicao {}", i);
    }
}

#[test]
fn inscrever_repetido_e_recusado() {
    let m = mesa();
    let j = Address::generate(&m.env);
    inscrever(&m, &j);
    let r = m.c.try_enroll(&j, &novo_c(&m));
    assert_eq!(r, Err(Ok(Erro::JuradoRepetido.into())));
    assert_eq!(
        bilhete(&m).balance(&j),
        UM_BILHETE,
        "a recusa nao emite segundo bilhete"
    );
}

#[test]
fn inscrever_sem_escrivao_e_recusado() {
    let env = Env::default();
    env.ledger().with_mut(|l| l.timestamp = GENESIS_QUICKNET);
    let escrivao = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(escrivao.clone());
    sac.issuer().set_flag(IssuerFlags::RequiredFlag);
    let verificador = env.register(groth16_verifier::Groth16Verifier, ());
    let id = env.register(
        Kleroterion,
        (&escrivao, &sac.address(), &MINIMO, &verificador, &JANELA),
    );
    env.mock_all_auths();
    token::StellarAssetClient::new(&env, &sac.address()).set_admin(&id);
    let c = KleroterionClient::new(&env, &id);

    // Só o jurado assina; o escrivão não.
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
        "sem o escrivao ninguem entra no rol"
    );
}

// ---- fechamento ----------------------------------------------------------

/// Assentos usados nos testes, e o ledger de revelação: um dia de ledgers de
/// 5 s depois do zero, sem relação com os fechamentos dos testes.
const K: u32 = 3;
const REVELA_EM: u32 = 17_280;

/// Fecha num instante em que a rodada alvo é exatamente `rodada`: a primeira
/// publicada depois do fechamento é `rodada - MARGEM_DE_RODADAS`.
fn fechar_para(m: &Mesa, rodada: u64, k: u32) -> Fechamento {
    m.env.ledger().with_mut(|l| {
        l.timestamp = GENESIS_QUICKNET + (rodada - 2 - MARGEM_DE_RODADAS) * PERIODO_QUICKNET
    });
    m.c.lock(&k, &REVELA_EM)
}

#[test]
fn fechar_abaixo_do_minimo_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, (MINIMO - 1) as usize));
    assert_eq!(
        m.c.try_lock(&K, &REVELA_EM),
        Err(Ok(Erro::RolAbaixoDoMinimo.into()))
    );
    assert_eq!(m.c.closing(), None);
}

#[test]
fn fechar_fixa_ledger_tempo_rodada_k_e_revelacao() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    m.env.ledger().with_mut(|l| {
        l.sequence_number = 4321;
        l.timestamp = GENESIS_QUICKNET + 6;
    });
    let f = m.c.lock(&K, &REVELA_EM);
    assert_eq!(f.ledger, 4321);
    assert_eq!(f.tempo, GENESIS_QUICKNET + 6);
    assert_eq!(
        f.rodada,
        4 + MARGEM_DE_RODADAS,
        "rodada 3 sai em genesis+6, no mesmo segundo; a 4 e a primeira depois, mais a margem"
    );
    assert_eq!(f.k, K);
    assert_eq!(f.revela_em, REVELA_EM);
    assert_eq!(m.c.closing(), Some(f));
}

/// A margem mínima do SPEC (§9 T-033) é 10 rodadas; abaixo disso não compila.
const _: () = assert!(MARGEM_DE_RODADAS >= 10);

/// INV-13: a rodada é calculada do relógio do ledger, com margem, e não
/// passada por quem chama.
#[test]
fn a_rodada_vem_do_fechamento_com_margem() {
    for (dt, primeira) in [(0u64, 2u64), (1, 2), (2, 2), (3, 3), (4, 3), (5, 3), (6, 4)] {
        let m = mesa();
        inscrever_todos(&m, &jurados(&m, MINIMO as usize));
        m.env
            .ledger()
            .with_mut(|l| l.timestamp = GENESIS_QUICKNET + dt);
        assert_eq!(
            m.c.lock(&K, &REVELA_EM).rodada,
            primeira + MARGEM_DE_RODADAS,
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
        m.c.try_lock(&(MINIMO + 1), &REVELA_EM),
        Err(Ok(Erro::AssentosInvalidos.into()))
    );
    assert_eq!(
        m.c.try_lock(&0u32, &REVELA_EM),
        Err(Ok(Erro::AssentosInvalidos.into()))
    );
    assert_eq!(m.c.closing(), None);
    // k = N é permitido.
    assert_eq!(m.c.lock(&MINIMO, &REVELA_EM).k, MINIMO);
}

#[test]
fn fechar_duas_vezes_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    m.c.lock(&K, &REVELA_EM);
    assert_eq!(
        m.c.try_lock(&K, &REVELA_EM),
        Err(Ok(Erro::RolFechado.into()))
    );
}

/// INV-11 e INV-14: o rol é comprometido antes do caso existir.
#[test]
fn inscrever_depois_de_fechar_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    m.c.lock(&K, &REVELA_EM);
    let tarde = Address::generate(&m.env);
    assert_eq!(
        m.c.try_enroll(&tarde, &novo_c(&m)),
        Err(Ok(Erro::RolFechado.into()))
    );
    assert_eq!(bilhete(&m).balance(&tarde), 0);
    assert_eq!(m.c.roster().len(), MINIMO);
}

/// INV-14 (Q-21): depois do fechamento o escrivão não tira ninguém.
#[test]
fn impedir_depois_de_fechar_e_recusado() {
    let m = mesa();
    let js = jurados(&m, MINIMO as usize);
    inscrever_todos(&m, &js);
    m.c.lock(&K, &REVELA_EM);
    assert_eq!(m.c.try_impede(&js[0]), Err(Ok(Erro::RolFechado.into())));
    assert!(sac(&m).authorized(&js[0]), "continua autorizado");
}

// ---- o selo (T-038) ------------------------------------------------------

#[test]
fn selar_antes_de_fechar_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    assert_eq!(m.c.try_seal(&sig_a(&m)), Err(Ok(Erro::RolAberto.into())));
}

/// INV-13: só a rodada que o fechamento guardou serve. A assinatura de
/// qualquer outra rodada — mesmo autêntica, mesmo posterior — é recusada.
#[test]
fn selar_com_outra_rodada_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    assert_eq!(fechar_para(&m, ROUND_A, K).rodada, ROUND_A);
    let b = BytesN::from_array(&m.env, &SIG_B);
    assert!(m.c.verify(&ROUND_B, &b), "a assinatura B e autentica");
    assert_eq!(m.c.try_seal(&b), Err(Ok(Erro::AssinaturaInvalida.into())));
    assert_eq!(m.c.seal_state(), None);
}

#[test]
fn assinatura_forjada_e_recusada() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    fechar_para(&m, ROUND_A, K);
    let r = m.c.try_seal(&BytesN::from_array(&m.env, &SIG_FORJADA));
    assert_eq!(r, Err(Ok(Erro::AssinaturaInvalida.into())));
}

/// INV-13: não há segundo selo; o que o primeiro fixou não muda.
#[test]
fn selar_duas_vezes_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    fechar_para(&m, ROUND_A, K);
    let selo = m.c.seal(&sig_a(&m));
    assert_eq!(m.c.try_seal(&sig_a(&m)), Err(Ok(Erro::JaSelado.into())));
    assert_eq!(m.c.seal_state(), Some(selo));
}

/// N, no fechamento, conta só os autorizados: cinco impedidos de sete deixam
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
        l.timestamp = GENESIS_QUICKNET + (ROUND_A - 2 - MARGEM_DE_RODADAS) * PERIODO_QUICKNET
    });
    assert_eq!(
        m.c.try_lock(&K, &REVELA_EM),
        Err(Ok(Erro::AssentosInvalidos.into()))
    );
}

/// Com 2k ≥ N o limiar é o campo inteiro: todo jurado está abaixo dele.
#[test]
fn com_2k_maior_que_n_o_limiar_e_o_campo() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, 7));
    fechar_para(&m, ROUND_A, 4);
    let selo = m.c.seal(&sig_a(&m));
    assert_eq!(selo.limiar, u256(&m.env, R_DECIMAL));
}

const R_DECIMAL: &str =
    "21888242871839275222246405745257275088548364400416034343698204186575808495617";

// ---- o fluxo selado, N = 8, k = 3 (T-038) ----------------------------------

/// tests/fluxo-n8.json: oito jurados, seus compromissos, contas de assento,
/// `y` e uma prova de cada um, sob a chave de circuits/setup/ e a semente que
/// `seal` deriva da rodada de teste (circuits/gerar_fluxo.mjs).
const FLUXO: &str = include_str!("../../../tests/fluxo-n8.json");

struct JuradoDoFluxo {
    principal: Address,
    assento: Address,
    s: U256,
    y: U256,
    abaixo: bool,
    prova: Prova,
}

struct Fluxo {
    m: Mesa,
    js: std::vec::Vec<JuradoDoFluxo>,
    seed: U256,
    limiar: U256,
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
            assento: Address::from_str(&m.env, &txt(&x["assento"])),
            s: u256(&m.env, &txt(&x["s"])),
            y: u256(&m.env, &txt(&x["y"])),
            abaixo: x["abaixo_do_limiar"].as_bool().unwrap(),
            prova: Prova {
                a: BytesN::from_array(&m.env, &bn(&txt(&x["proof"]["a"])).try_into().unwrap()),
                b: BytesN::from_array(&m.env, &bn(&txt(&x["proof"]["b"])).try_into().unwrap()),
                c: BytesN::from_array(&m.env, &bn(&txt(&x["proof"]["c"])).try_into().unwrap()),
            },
        });
    }
    let f = fechar_para(&m, ROUND_A, 3);
    assert_eq!(f.raiz, u256(&m.env, &txt(&j["raiz"])), "a raiz do fixture");
    m.env.ledger().with_mut(|l| l.sequence_number = 1000);
    let seed = u256(&m.env, &txt(&j["seed"]));
    let limiar = u256(&m.env, &txt(&j["limiar"]));
    Fluxo {
        m,
        js,
        seed,
        limiar,
    }
}

impl Fluxo {
    fn selar(&self) -> Selo {
        self.m.c.seal(&sig_a(&self.m))
    }
    fn reivindicar(&self, i: usize) {
        let j = &self.js[i];
        self.m.c.claim(&j.assento, &j.prova, &j.y);
    }
    fn tentar(&self, i: usize) -> Option<Erro> {
        let j = &self.js[i];
        match self.m.c.try_claim(&j.assento, &j.prova, &j.y) {
            Ok(_) => None,
            Err(Ok(e)) => Some(Erro::try_from(e).expect("erro do contrato")),
            Err(Err(e)) => panic!("erro do host: {:?}", e),
        }
    }
    fn passar_o_prazo(&self) {
        let prazo = self.m.c.seal_state().unwrap().prazo;
        self.m
            .env
            .ledger()
            .with_mut(|l| l.sequence_number = prazo + 1);
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

/// O selo fixa a semente — a aleatoriedade da rodada, deslocada 8 bits para
/// caber no campo — e o limiar `T = ⌊r·2k/N⌋`, e abre a janela.
#[test]
fn o_selo_fixa_semente_limiar_e_prazo() {
    let f = fluxo();
    let selo = f.selar();
    assert_eq!(selo.seed, f.seed, "randomness >> 8");
    assert_eq!(selo.limiar, f.limiar, "floor(r*6/8)");
    assert_eq!(selo.prazo, 1000 + JANELA);
    assert!(!selo.ampliado);
    let esperado = Selado {
        seed: selo.seed.clone(),
        limiar: selo.limiar.clone(),
        prazo: selo.prazo,
    };
    assert_eq!(
        f.m.env.events().all().filter_by_contract(&f.m.id),
        [esperado.to_xdr(&f.m.env, &f.m.id)]
    );
}

#[test]
fn reivindicar_antes_do_selo_e_recusado() {
    let f = fluxo();
    assert_eq!(f.tentar(1), Some(Erro::NaoSelado));
}

/// INV-15: com a prova, o `y` entra na lista, em ordem crescente.
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
    assert_eq!(f.tentar(4), Some(Erro::YAcimaDoLimiar));
}

/// INV-15: um `y` por folha. A mesma reivindicação duas vezes é recusada.
#[test]
fn y_repetido_e_recusado() {
    let f = fluxo();
    f.selar();
    f.reivindicar(1);
    assert_eq!(f.tentar(1), Some(Erro::YRepetido));
}

/// A prova de um jurado com o `y` de outro, ou enviada de outra conta, não
/// vale.
#[test]
fn prova_com_outro_y_ou_de_outra_conta_e_recusada() {
    let f = fluxo();
    f.selar();
    let (a, b) = (&f.js[1], &f.js[2]);
    assert_eq!(
        f.m.c.try_claim(&a.assento, &a.prova, &b.y),
        Err(Ok(Erro::ProvaInvalida.into())),
        "outro y"
    );
    assert_eq!(
        f.m.c.try_claim(&b.assento, &a.prova, &a.y),
        Err(Ok(Erro::ProvaInvalida.into())),
        "outra conta"
    );
}

#[test]
fn reivindicar_depois_do_prazo_e_recusado() {
    let f = fluxo();
    f.selar();
    f.passar_o_prazo();
    assert_eq!(f.tentar(1), Some(Erro::JanelaFechada));
}

/// INV-17: o evento de uma reivindicação leva o `y` e nada mais — nenhum
/// endereço ao lado dele.
#[test]
fn nenhum_evento_liga_y_a_endereco() {
    let f = fluxo();
    f.selar();
    f.reivindicar(3);
    let esperado = Reivindicado {
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
    assert_eq!(f.m.c.try_finalize(), Err(Ok(Erro::JanelaAberta.into())));
}

/// Os `k` menores `y` sentam; o painel é público e imutável (INV-16).
#[test]
fn finalizar_senta_os_k_menores_e_nao_muda_mais() {
    let f = fluxo();
    f.selar();
    let quem = [1, 2, 3, 5, 6];
    for i in quem {
        f.reivindicar(i);
    }
    f.passar_o_prazo();
    let fim = f.m.c.finalize().unwrap();
    assert!(fim.completo);
    assert_eq!(
        fim.painel,
        soroban_sdk::Vec::from_slice(&f.m.env, &f.menores(&quem, 3))
    );
    assert_eq!(f.m.c.try_finalize(), Err(Ok(Erro::JaFinalizado.into())));
    assert_eq!(f.tentar(7), Some(Erro::JaFinalizado));
    assert_eq!(f.m.c.panel(), Some(fim));
}

/// Menos de `k` reivindicações: o limiar vai para o campo inteiro, uma vez, e
/// a janela reabre; quem estava acima do limiar pode entrar.
#[test]
fn com_menos_de_k_o_limiar_se_amplia_uma_vez() {
    let f = fluxo();
    f.selar();
    f.reivindicar(1);
    f.reivindicar(2);
    f.passar_o_prazo();
    assert_eq!(f.m.c.finalize(), None, "ampliou, nao finalizou");
    let selo = f.m.c.seal_state().unwrap();
    assert!(selo.ampliado);
    assert_eq!(selo.limiar, u256(&f.m.env, R_DECIMAL));
    f.reivindicar(4);
    f.passar_o_prazo();
    let fim = f.m.c.finalize().unwrap();
    assert!(fim.completo);
    assert_eq!(fim.painel.len(), 3);
}

/// Ampliado e ainda menos de `k`: o painel sai com quem há, marcado
/// incompleto — nunca um segundo sorteio.
#[test]
fn ampliado_e_ainda_menos_de_k_fica_incompleto() {
    let f = fluxo();
    f.selar();
    f.reivindicar(1);
    f.passar_o_prazo();
    assert_eq!(f.m.c.finalize(), None);
    f.passar_o_prazo();
    let fim = f.m.c.finalize().unwrap();
    assert!(!fim.completo);
    assert_eq!(fim.painel.len(), 1);
}

/// INV-07 e INV-08 no modo selado: quem se recusa sai e o próximo `y` da
/// mesma lista senta — o painel fica com `k`, sem novo sorteio.
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
    let esperado = Recusado {
        jurado: j.principal.clone(),
    };
    assert_eq!(
        f.m.env.events().all().filter_by_contract(&f.m.id),
        [esperado.to_xdr(&f.m.env, &f.m.id)]
    );
    f.passar_o_prazo();
    let fim = f.m.c.finalize().unwrap();
    let restantes: std::vec::Vec<usize> = quem.iter().copied().filter(|&i| i != saindo).collect();
    assert_eq!(
        fim.painel,
        soroban_sdk::Vec::from_slice(&f.m.env, &f.menores(&restantes, 3))
    );
}

#[test]
fn recusa_com_segredo_errado_ou_fora_de_hora_e_recusada() {
    let f = fluxo();
    let j = &f.js[1];
    assert_eq!(
        f.m.c.try_recuse(&j.principal, &j.s),
        Err(Ok(Erro::NaoSelado.into())),
        "antes do selo"
    );
    f.selar();
    assert_eq!(
        f.m.c.try_recuse(&j.principal, &f.js[2].s),
        Err(Ok(Erro::SegredoNaoConfere.into()))
    );
    f.m.c.recuse(&j.principal, &j.s);
    assert_eq!(
        f.m.c.try_recuse(&j.principal, &j.s),
        Err(Ok(Erro::JaRecusado.into()))
    );
    assert_eq!(
        f.tentar(1),
        Some(Erro::YRepetido),
        "recusado nao reivindica"
    );
    // Ninguém reivindicou: o primeiro finalize amplia, o segundo fecha.
    f.passar_o_prazo();
    assert_eq!(f.m.c.finalize(), None);
    f.passar_o_prazo();
    assert!(!f.m.c.finalize().unwrap().completo);
    let k = &f.js[2];
    assert_eq!(
        f.m.c.try_recuse(&k.principal, &k.s),
        Err(Ok(Erro::JaFinalizado.into())),
        "depois do painel"
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
        Err(Ok(Erro::NaoFinalizado.into()))
    );
    f.passar_o_prazo();
    f.m.c.finalize();
    f.passar_o_prazo();
    f.m.c.finalize();
    assert!(f.m.env.ledger().sequence() < REVELA_EM);
    assert_eq!(
        f.m.c.try_reveal(&j.principal, &j.s),
        Err(Ok(Erro::CedoParaRevelar.into()))
    );
}

/// `reveal` confere `Poseidon(s) = c` e `Poseidon(seed, s) = y`, e diz se
/// aquele `y` sentou. É prova, nunca correção: o painel não muda (INV-16).
#[test]
fn revelar_confere_c_e_y_e_nao_muda_o_painel() {
    let f = fluxo();
    f.selar();
    for i in [1, 2, 3, 5] {
        f.reivindicar(i);
    }
    f.passar_o_prazo();
    let fim = f.m.c.finalize().unwrap();
    f.chegar_a_revelacao();
    for i in [1, 2, 3, 5, 0] {
        let j = &f.js[i];
        assert_eq!(
            f.m.c.try_reveal(&j.principal, &f.js[(i + 1) % 8].s),
            Err(Ok(Erro::SegredoNaoConfere.into())),
            "segredo de outro"
        );
        let r = f.m.c.reveal(&j.principal, &j.s);
        assert_eq!(r.y, j.y);
        assert_eq!(r.sentado, fim.painel.contains(&j.y), "jurado {}", i);
        assert!(!r.sem_bilhete);
        assert_eq!(
            f.m.c.try_reveal(&j.principal, &j.s),
            Err(Ok(Erro::JaRevelado.into()))
        );
    }
    assert_eq!(f.m.c.panel(), Some(fim));
}

/// INV-12 no modo selado (decisão da humana, 2026-10-25): senta quem estava
/// autorizado no fechamento; o saldo não pode ser conferido sem identificar o
/// jurado. Quem sentou e já não tinha o bilhete fica registrado no `reveal`.
#[test]
fn sentou_sem_bilhete_fica_registrado_no_reveal() {
    let f = fluxo();
    f.selar();
    for i in [1, 2, 3] {
        f.reivindicar(i);
    }
    f.passar_o_prazo();
    let fim = f.m.c.finalize().unwrap();
    assert!(fim.completo);
    let j = &f.js[2];
    bilhete(&f.m).transfer(&j.principal, &f.js[0].principal, &UM_BILHETE);
    f.chegar_a_revelacao();
    let r = f.m.c.reveal(&j.principal, &j.s);
    assert!(r.sentado && r.sem_bilhete);
    let esperado = Revelado {
        jurado: j.principal.clone(),
        y: j.y.clone(),
        sentado: true,
        sem_bilhete: true,
    };
    assert_eq!(
        f.m.env.events().all().filter_by_contract(&f.m.id),
        [esperado.to_xdr(&f.m.env, &f.m.id)]
    );
}

/// O fluxo inteiro, N = 8, k = 3: o jurado 0 está abaixo do limiar e não
/// reivindica; o 4 está acima e é recusado; os outros seis reivindicam; um
/// se recusa; o painel sai com os três menores dos que ficaram; todos revelam
/// menos um. O que a cadeia guarda basta para dizer, de cada um: sentou,
/// revelou, não revelou, sorteado e ausente (T-039 recompõe isto).
#[test]
fn fluxo_completo_n8_k3() {
    let f = fluxo();
    let selo = f.selar();
    assert_eq!(selo.limiar, f.limiar);
    assert!(f.js[0].abaixo, "o 0 poderia, e nao reivindica");
    assert_eq!(f.tentar(4), Some(Erro::YAcimaDoLimiar));
    let quem = [1, 2, 3, 5, 6, 7];
    for i in quem {
        f.reivindicar(i);
    }
    let recusa = 6;
    f.m.c.recuse(&f.js[recusa].principal, &f.js[recusa].s);
    f.passar_o_prazo();
    let fim = f.m.c.finalize().unwrap();
    let ficaram: std::vec::Vec<usize> = quem.iter().copied().filter(|&i| i != recusa).collect();
    let esperado = f.menores(&ficaram, 3);
    assert!(fim.completo);
    assert_eq!(
        fim.painel,
        soroban_sdk::Vec::from_slice(&f.m.env, &esperado)
    );

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
        if r.sentado {
            sentados_revelados += 1;
        }
    }
    assert_eq!(sentados_revelados, 2, "tres sentados, um em silencio");
    assert_eq!(f.m.c.revelation(&f.js[silencioso].principal), None);
    assert_eq!(f.m.c.panel(), Some(fim), "INV-16");
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
        Err(Ok(Erro::CompromissoRepetido.into()))
    );
    assert_eq!(bilhete(&m).balance(&b), 0, "a recusa nao emite bilhete");
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
        Err(Ok(Erro::CompromissoRepetido.into()))
    );
}

#[test]
fn compromisso_zero_ou_fora_do_campo_e_recusado() {
    let m = mesa();
    let j = Address::generate(&m.env);
    let zero = U256::from_u32(&m.env, 0);
    assert_eq!(
        m.c.try_enroll(&j, &zero),
        Err(Ok(Erro::CompromissoInvalido.into())),
        "zero e a folha vazia"
    );
    let r = u256(
        &m.env,
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    );
    assert_eq!(
        m.c.try_enroll(&j, &r),
        Err(Ok(Erro::CompromissoInvalido.into())),
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
        Err(Ok(Erro::RolCheio.into()))
    );
}

#[test]
fn impedir_quem_nao_esta_no_rol_ou_duas_vezes_e_recusado() {
    let m = mesa();
    let a = Address::generate(&m.env);
    assert_eq!(m.c.try_impede(&a), Err(Ok(Erro::JuradoDesconhecido.into())));
    inscrever(&m, &a);
    m.c.impede(&a);
    assert_eq!(m.c.try_impede(&a), Err(Ok(Erro::JaImpedido.into())));
}

/// As raízes de subárvores vazias que o contrato embute são as da
/// circomlib (tests/vetores-arvore.json).
#[test]
fn os_zeros_sao_os_da_circomlib() {
    let env = Env::default();
    let j: serde_json::Value = serde_json::from_str(ARVORES).unwrap();
    let zeros = j["zeros"].as_array().unwrap();
    assert_eq!(zeros.len() as u32, PROFUNDIDADE + 1);
    for (d, z) in zeros.iter().enumerate() {
        assert_eq!(
            Kleroterion::zero(&env, d as u32),
            u256(&env, z.as_str().unwrap()),
            "nivel {}",
            d
        );
    }
}

const ARVORES: &str = include_str!("../../../tests/vetores-arvore.json");

/// T-035: a árvore é construída pelo contrato, evento a evento, e a raiz
/// congelada no `lock` é a que a circomlib — o circuito — calcula para os
/// mesmos compromissos. Cada evento emitido é o que o vetor lista, então
/// quem lê só os eventos recompõe a raiz (scripts/arvore.py, web/arvore.mjs).
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
            let jurado = &js[i as usize];
            match ev["evento"].as_str().unwrap() {
                "enroll" => {
                    let c = u256(&m.env, ev["c"].as_str().unwrap());
                    m.c.enroll(jurado, &c);
                    let esperado = Inscrito {
                        jurado: jurado.clone(),
                        indice: i,
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
                    m.c.impede(jurado);
                    let esperado = Impedido {
                        jurado: jurado.clone(),
                        indice: i,
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
        let raiz = u256(&m.env, caso["raiz"].as_str().unwrap());
        assert_eq!(m.c.root(), raiz, "{}: raiz antes do fechamento", nome);
        let f = m.c.lock(&1u32, &REVELA_EM);
        assert_eq!(f.raiz, raiz, "{}: raiz congelada", nome);
        assert_eq!(
            f.n as u64,
            caso["autorizados"].as_u64().unwrap(),
            "{}: N",
            nome
        );
    }
}

/// O fechamento publica o que fixou, e a raiz não muda mais: depois dele
/// ninguém entra nem sai (INV-14).
#[test]
fn fechar_publica_raiz_n_rodada_k() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    let raiz = m.c.root();
    let f = fechar_para(&m, ROUND_A, K);
    assert_eq!(f.raiz, raiz);
    assert_eq!(f.n, MINIMO);
    let esperado = Fechado {
        raiz: f.raiz.clone(),
        n: f.n,
        rodada: f.rodada,
        k: f.k,
        revela_em: f.revela_em,
    };
    assert_eq!(
        m.env.events().all().filter_by_contract(&m.id),
        [esperado.to_xdr(&m.env, &m.id)]
    );
}

// ---- a prova de assento (T-037) ----------------------------------------

/// tests/prova-claim.json: uma prova de `pinakion prove` para o jurado 3 da
/// árvore de 17 de tests/vetores-arvore.json, sob a chave de circuits/setup/.
const PROVA: &str = include_str!("../../../tests/prova-claim.json");

struct ProvaDeAssento {
    prova: Prova,
    seed: U256,
    y: U256,
    claimer: U256,
    assento: Address,
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
    let assento = j["_"].as_str().unwrap();
    let assento = &assento[assento.find("seat account ").unwrap() + 13..][..56];
    ProvaDeAssento {
        prova: Prova {
            a: b64(&txt(&j["proof"]["a"])),
            b: b128(&txt(&j["proof"]["b"])),
            c: b64(&txt(&j["proof"]["c"])),
        },
        seed: u256(env, &txt(&j["decimal"]["seed"])),
        y: u256(env, &txt(&j["decimal"]["y"])),
        claimer: u256(env, &txt(&j["decimal"]["claimer"])),
        assento: Address::from_str(env, assento),
    }
}

/// A mesa da árvore de 17 dos vetores: os mesmos compromissos, o mesmo
/// impedido, fechada. A raiz é a da prova.
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

fn vale(m: &Mesa, p: &Prova, seed: &U256, y: &U256, assento: &Address) -> bool {
    m.env.as_contract(&m.id, || {
        Kleroterion::prova_de_assento_vale(&m.env, p, seed, y, assento)
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
        .as_contract(&m.id, || Kleroterion::claimer(&m.env, &p.assento));
    assert_eq!(c, p.claimer);
}

/// INV-15: a prova vale contra a raiz que o contrato guarda, com a semente
/// dada e a conta de onde vem a chamada; as entradas públicas são montadas
/// pelo contrato, nunca recebidas prontas.
#[test]
fn a_prova_da_cli_vale_contra_a_raiz_do_contrato() {
    let m = mesa_da_arvore_de_17();
    let p = prova_da_cli(&m.env);
    assert!(vale(&m, &p.prova, &p.seed, &p.y, &p.assento));
}

#[test]
fn outro_y_outra_semente_ou_outra_conta_nao_vale() {
    let m = mesa_da_arvore_de_17();
    let p = prova_da_cli(&m.env);
    let um = U256::from_u32(&m.env, 1);
    assert!(!vale(&m, &p.prova, &p.seed, &p.y.add(&um), &p.assento), "y");
    assert!(
        !vale(&m, &p.prova, &p.seed.add(&um), &p.y, &p.assento),
        "seed"
    );
    let outra = Address::generate(&m.env);
    assert!(!vale(&m, &p.prova, &p.seed, &p.y, &outra), "conta");
    // seed e y trocados de lugar
    assert!(!vale(&m, &p.prova, &p.y, &p.seed, &p.assento), "permutadas");
}

#[test]
fn um_byte_da_prova_alterado_nao_vale() {
    let m = mesa_da_arvore_de_17();
    let p = prova_da_cli(&m.env);
    let mut c = p.prova.c.to_array();
    c[63] ^= 0x01;
    let alterada = Prova {
        c: BytesN::from_array(&m.env, &c),
        ..p.prova.clone()
    };
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vale(&m, &alterada, &p.seed, &p.y, &p.assento)
    }));
    assert!(!r.unwrap_or(false));
}

/// Outra raiz — o mesmo rol sem o impedimento — e a mesma prova não vale.
#[test]
fn a_prova_nao_vale_em_outro_rol() {
    let m = mesa_com_minimo(1);
    inscrever_todos(&m, &jurados(&m, 17));
    fechar_para(&m, ROUND_A, K);
    let p = prova_da_cli(&m.env);
    assert!(!vale(&m, &p.prova, &p.seed, &p.y, &p.assento));
}

#[test]
fn antes_de_fechar_nao_ha_raiz_para_provar() {
    let m = mesa();
    let p = prova_da_cli(&m.env);
    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        vale(&m, &p.prova, &p.seed, &p.y, &p.assento)
    }));
    assert!(r.is_err(), "sem fechamento, recusa");
}

#[test]
fn o_verificador_e_o_do_construtor() {
    let m = mesa();
    let v = m.c.verifier();
    let vk = groth16_verifier::Groth16VerifierClient::new(&m.env, &v).vk();
    assert_eq!(vk.ic.len(), 5, "quatro entradas publicas");
}

/// Custo, no host de teste, de conferir uma prova pelo kleroterion: montar
/// as entradas, o SHA-256 do endereço e a chamada ao verificador.
const CUSTO_PROVA: (u64, u64) = (29_200_539, 273_827);

#[test]
fn custo_medido_da_prova() {
    let m = mesa_da_arvore_de_17();
    let p = prova_da_cli(&m.env);
    m.env.cost_estimate().budget().reset_unlimited();
    assert!(vale(&m, &p.prova, &p.seed, &p.y, &p.assento));
    let b = m.env.cost_estimate().budget();
    let custo = (b.cpu_instruction_cost(), b.memory_bytes_cost());
    assert_eq!(custo, CUSTO_PROVA, "prova (cpu, mem)");
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
/// a inscrição do 64.º jurado (o caminho inteiro da árvore e as três chamadas
/// ao SAC), um impedimento e o fechamento. Mudou, fica vermelho (G2); G7
/// confere que docs/SPEC.md ainda diz estes números.
const CUSTO_ENROLL: (u64, u64) = (10_481_144, 1_725_965);
const CUSTO_IMPEDE: (u64, u64) = (8_684_358, 1_048_886);
const CUSTO_LOCK: (u64, u64) = (606_903, 291_011);

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
    assert_eq!(enroll, CUSTO_ENROLL, "enroll (cpu, mem)");
    assert_eq!(impede, CUSTO_IMPEDE, "impede (cpu, mem)");
    assert_eq!(lock, CUSTO_LOCK, "lock (cpu, mem)");
}

/// Custo no host de teste, `(cpu, mem)`, das chamadas do fluxo selado com
/// N = 8, k = 3: o selo (verificação BLS12-381 e o limiar), a sexta
/// reivindicação (a prova e a inserção), uma recusa, o `finalize` e um
/// `reveal`. Mudou, fica vermelho (G2); G7 confere docs/SPEC.md.
const CUSTO_SEAL: (u64, u64) = (26_894_868, 229_645);
const CUSTO_CLAIM: (u64, u64) = (29_514_466, 376_981);
const CUSTO_RECUSE: (u64, u64) = (2_832_768, 303_478);
const CUSTO_FINALIZE: (u64, u64) = (184_731, 100_873);
const CUSTO_REVEAL: (u64, u64) = (2_942_518, 370_368);

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
    assert_eq!(seal, CUSTO_SEAL, "seal");
    assert_eq!(claim, CUSTO_CLAIM, "claim");
    assert_eq!(recuse, CUSTO_RECUSE, "recuse");
    assert_eq!(finalize, CUSTO_FINALIZE, "finalize");
    assert_eq!(reveal, CUSTO_REVEAL, "reveal");
}
