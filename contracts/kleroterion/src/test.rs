#![cfg(test)]
extern crate std;
use super::*;
use soroban_sdk::testutils::{
    Address as _, Events as _, IssuerFlags, Ledger as _, MockAuth, MockAuthInvoke,
};
use soroban_sdk::{token, Address, Bytes, BytesN, Env, Event, IntoVal, U256};
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
    let id = env.register(Kleroterion, (&escrivao, &bilhete, &minimo));
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
    let id = env.register(Kleroterion, (&escrivao, &sac.address(), &MINIMO));
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

// ---- sorteio -------------------------------------------------------------

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
    assert_eq!(m.c.panel(), None);
}

#[test]
fn assinatura_forjada_e_recusada() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    fechar_para(&m, ROUND_A, K);
    let r = m.c.try_seal(&BytesN::from_array(&m.env, &SIG_FORJADA));
    assert_eq!(r, Err(Ok(Erro::AssinaturaInvalida.into())));
}

/// INV-13: não há segundo sorteio. O painel guardado não muda.
#[test]
fn selar_duas_vezes_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    fechar_para(&m, ROUND_A, K);
    let painel = m.c.seal(&sig_a(&m));
    assert_eq!(m.c.try_seal(&sig_a(&m)), Err(Ok(Erro::JaSelado.into())));
    assert_eq!(m.c.panel(), Some(painel));
}

/// A ordem é o vetor de ouro "rol pequeno" (n = 7) do beacon-verifier:
/// [0, 1, 4, 2, 6, 5, 3]. O painel são os endereços nessas posições.
const ORDEM_DE_OURO: [u32; 7] = [0, 1, 4, 2, 6, 5, 3];

#[test]
fn painel_segue_a_ordem_de_ouro() {
    let m = mesa();
    let js = jurados(&m, 7);
    inscrever_todos(&m, &js);
    fechar_para(&m, ROUND_A, K);
    let painel = m.c.seal(&sig_a(&m));
    for i in 0..3 {
        assert_eq!(
            painel.get(i).unwrap(),
            js[ORDEM_DE_OURO[i as usize] as usize],
            "assento {}",
            i
        );
    }
}

/// INV-07 e INV-08: um impedido não encolhe a mesa, e o substituto é o
/// próximo da ordem sorteada — não há novo sorteio. O impedimento vem antes
/// do fechamento (INV-14).
#[test]
fn impedido_e_substituido_e_o_painel_fica_do_mesmo_tamanho() {
    let m = mesa();
    let js = jurados(&m, 7);
    inscrever_todos(&m, &js);
    let primeiro = &js[ORDEM_DE_OURO[0] as usize];
    m.c.impede(primeiro);
    assert!(!sac(&m).authorized(primeiro));
    fechar_para(&m, ROUND_A, K);

    let painel = m.c.seal(&sig_a(&m));
    assert_eq!(painel.len(), 3);
    assert!(!painel.contains(primeiro), "impedido nao senta");
    for i in 0..3 {
        assert_eq!(
            painel.get(i).unwrap(),
            js[ORDEM_DE_OURO[i as usize + 1] as usize]
        );
    }
}

#[test]
fn substituto_e_o_proximo_da_ordem() {
    let m = mesa();
    let js = jurados(&m, 7);
    inscrever_todos(&m, &js);
    // Impede o segundo da ordem: o painel pula só ele.
    m.c.impede(&js[ORDEM_DE_OURO[1] as usize]);
    fechar_para(&m, ROUND_A, K);
    let painel = m.c.seal(&sig_a(&m));
    let esperado = [ORDEM_DE_OURO[0], ORDEM_DE_OURO[2], ORDEM_DE_OURO[3]];
    for i in 0..3 {
        assert_eq!(painel.get(i).unwrap(), js[esperado[i as usize] as usize]);
    }
}

/// INV-12: sem o bilhete na mão, não há assento — mesmo autorizado.
#[test]
fn sem_bilhete_nao_senta() {
    let m = mesa();
    let js = jurados(&m, 7);
    inscrever_todos(&m, &js);
    fechar_para(&m, ROUND_A, K);
    let doador = &js[ORDEM_DE_OURO[0] as usize];
    let receptor = &js[ORDEM_DE_OURO[6] as usize];
    bilhete(&m).transfer(doador, receptor, &UM_BILHETE);
    assert_eq!(bilhete(&m).balance(doador), 0);
    assert!(
        sac(&m).authorized(doador),
        "continua autorizado, mas sem bilhete"
    );

    let painel = m.c.seal(&sig_a(&m));
    assert!(!painel.contains(doador));
    assert_eq!(painel.get(0).unwrap(), js[ORDEM_DE_OURO[1] as usize]);
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

/// Depois do fechamento um jurado ainda pode passar o bilhete adiante; se
/// sobram menos de `k` com bilhete, o sorteio é recusado.
#[test]
fn elegiveis_insuficientes_e_recusado() {
    let m = mesa();
    let js = jurados(&m, 7);
    inscrever_todos(&m, &js);
    fechar_para(&m, ROUND_A, 7);
    bilhete(&m).transfer(&js[0], &js[1], &UM_BILHETE);
    let r = m.c.try_seal(&sig_a(&m));
    assert_eq!(r, Err(Ok(Erro::ElegiveisInsuficientes.into())));
    assert_eq!(m.c.panel(), None);
}

#[test]
fn com_dois_elegiveis_dois_assentos_sentam() {
    let m = mesa();
    let js = jurados(&m, 7);
    inscrever_todos(&m, &js);
    for j in &js[..5] {
        m.c.impede(j);
    }
    fechar_para(&m, ROUND_A, 2);
    assert_eq!(m.c.seal(&sig_a(&m)).len(), 2);
}

/// O quarto vetor de tests/vetores-de-ouro.json: sete endereços fixos, o
/// primeiro da ordem impedido, painel de três. Os verificadores em Python e
/// JavaScript devolvem estes mesmos endereços, nesta ordem.
const ROL_DE_OURO: [&str; 7] = [
    "CAR7NKBMHGHGH7NSGHE46UHR7DK6OFUKIJZ5LVGCF5LZ5BSLOOMFZKP2",
    "CBNZ6GW7SJP4RWRMHXME65XUHZCTBVXA27PPO4KR7IDMJH2UZL7HPXPP",
    "CBGK2LCVUESQU52MUQM2XYGAGBJ2GKTYAJUID4WQGMUD2ANMYTYNIYPH",
    "CA222NMTAQQHVODIF3RSN476HUP7DW4NN6TXDKUMUCR35Z323Q6FURPR",
    "CBYYRHFKSY5VVAACXXJS44HXYQ47CPF65JALEPTSUL7EHPCP5SCEL4TK",
    "CAIQB4VFZUJH7LOCYT36YRZOH6ZMGK3WHVZX6JEOI7CTHPQONUI26AOG",
    "CBSA33VZ5Q7FHWOGACNR6MXRFQIPCPCEXM3Z6ZE22OP2UM7FZHFQSUJI",
];

#[test]
fn bate_com_o_vetor_de_enderecos() {
    let m = mesa();
    let js: std::vec::Vec<Address> = ROL_DE_OURO
        .iter()
        .map(|s| Address::from_str(&m.env, s))
        .collect();
    inscrever_todos(&m, &js);
    m.c.impede(&js[ORDEM_DE_OURO[0] as usize]);
    fechar_para(&m, ROUND_A, K);
    let painel = m.c.seal(&sig_a(&m));
    let esperado = [
        "CBNZ6GW7SJP4RWRMHXME65XUHZCTBVXA27PPO4KR7IDMJH2UZL7HPXPP",
        "CBYYRHFKSY5VVAACXXJS44HXYQ47CPF65JALEPTSUL7EHPCP5SCEL4TK",
        "CBGK2LCVUESQU52MUQM2XYGAGBJ2GKTYAJUID4WQGMUD2ANMYTYNIYPH",
    ];
    for (i, s) in esperado.iter().enumerate() {
        assert_eq!(
            painel.get(i as u32).unwrap(),
            Address::from_str(&m.env, s),
            "assento {}",
            i
        );
    }
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

// ---- as cópias do beacon-verifier ---------------------------------------

/// As mesmas asserções de beacon-verifier::bate_com_a_recomputacao_fora_da_cadeia,
/// contra tests/vetores-de-ouro.json. Se uma cópia divergir, isto acusa.
#[test]
fn bate_com_a_recomputacao_fora_da_cadeia() {
    let m = mesa();
    let semente = BytesN::from_array(&m.env, &RANDOMNESS_A);

    let a = m.c.draw(&semente, &200u32, &25u32);
    let esperado_a: [u32; 25] = [
        15, 179, 44, 133, 177, 1, 168, 104, 157, 62, 43, 63, 57, 23, 86, 50, 165, 64, 26, 27, 148,
        96, 120, 13, 29,
    ];
    for (i, v) in esperado_a.iter().enumerate() {
        assert_eq!(a.get(i as u32).unwrap(), *v, "divergiu na posicao {}", i);
    }

    let zeros = BytesN::from_array(&m.env, &[0u8; 32]);
    let b = m.c.draw(&zeros, &50u32, &7u32);
    let esperado_b: [u32; 7] = [28, 10, 18, 23, 41, 36, 26];
    for (i, v) in esperado_b.iter().enumerate() {
        assert_eq!(
            b.get(i as u32).unwrap(),
            *v,
            "semente zero divergiu em {}",
            i
        );
    }

    let d = m.c.draw(&semente, &7u32, &7u32);
    for (i, v) in ORDEM_DE_OURO.iter().enumerate() {
        assert_eq!(
            d.get(i as u32).unwrap(),
            *v,
            "rol pequeno divergiu em {}",
            i
        );
    }
}

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
const CUSTO_ENROLL: (u64, u64) = (10_472_755, 1_714_763);
const CUSTO_IMPEDE: (u64, u64) = (8_656_396, 1_039_588);
const CUSTO_LOCK: (u64, u64) = (587_464, 283_073);

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
    std::println!("enroll {:?} impede {:?} lock {:?}", enroll, impede, lock);
    assert_eq!(enroll, CUSTO_ENROLL, "enroll (cpu, mem)");
    assert_eq!(impede, CUSTO_IMPEDE, "impede (cpu, mem)");
    assert_eq!(lock, CUSTO_LOCK, "lock (cpu, mem)");
}
