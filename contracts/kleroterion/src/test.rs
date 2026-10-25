#![cfg(test)]
extern crate std;
use super::*;
use soroban_sdk::testutils::{Address as _, IssuerFlags, Ledger as _, MockAuth, MockAuthInvoke};
use soroban_sdk::{token, Address, BytesN, Env, IntoVal};

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
    c: KleroterionClient<'static>,
}

/// Emissor com AUTH_REQUIRED e AUTH_REVOCABLE, SAC entregue ao contrato.
/// Quem fecha o emissor (master weight 0) é a sequência clássica de T-031;
/// o ambiente de teste não tem assinatura de conta para simular isso.
fn mesa() -> Mesa {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|l| l.timestamp = GENESIS_QUICKNET);
    let escrivao = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(escrivao.clone());
    sac.issuer().set_flag(IssuerFlags::RequiredFlag);
    sac.issuer().set_flag(IssuerFlags::RevocableFlag);
    let bilhete = sac.address();
    let id = env.register(Kleroterion, (&escrivao, &bilhete, &MINIMO));
    token::StellarAssetClient::new(&env, &bilhete).set_admin(&id);
    let c = KleroterionClient::new(&env, &id);
    Mesa { env, bilhete, c }
}

fn jurados(m: &Mesa, n: usize) -> std::vec::Vec<Address> {
    (0..n).map(|_| Address::generate(&m.env)).collect()
}

fn inscrever_todos(m: &Mesa, js: &[Address]) {
    for j in js {
        m.c.enroll(j);
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
    m.c.enroll(&j);
    assert_eq!(bilhete(&m).balance(&j), 1, "um bilhete, nem mais nem menos");
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
    m.c.enroll(&j);
    let r = m.c.try_enroll(&j);
    assert_eq!(r, Err(Ok(Erro::JuradoRepetido.into())));
    assert_eq!(
        bilhete(&m).balance(&j),
        1,
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
    env.mock_auths(&[MockAuth {
        address: &j,
        invoke: &MockAuthInvoke {
            contract: &id,
            fn_name: "enroll",
            args: (j.clone(),).into_val(&env),
            sub_invokes: &[],
        },
    }]);
    assert!(
        c.try_enroll(&j).is_err(),
        "sem o escrivao ninguem entra no rol"
    );
}

// ---- fechamento ----------------------------------------------------------

#[test]
fn fechar_abaixo_do_minimo_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, (MINIMO - 1) as usize));
    assert_eq!(m.c.try_lock(), Err(Ok(Erro::RolAbaixoDoMinimo.into())));
    assert_eq!(m.c.closing(), None);
}

#[test]
fn fechar_fixa_ledger_tempo_e_rodada_minima() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    m.env.ledger().with_mut(|l| {
        l.sequence_number = 4321;
        l.timestamp = GENESIS_QUICKNET + 6;
    });
    let f = m.c.lock();
    assert_eq!(f.ledger, 4321);
    assert_eq!(f.tempo, GENESIS_QUICKNET + 6);
    assert_eq!(
        f.rodada_minima, 4,
        "rodada 3 sai em genesis+6, no mesmo segundo; a 4 e a primeira depois"
    );
    assert_eq!(m.c.closing(), Some(f));
}

/// INV-06, a outra metade: a rodada mínima é calculada do relógio do ledger,
/// não passada por quem chama.
#[test]
fn rodada_minima_vem_do_fechamento() {
    for (dt, esperado) in [(0u64, 2u64), (1, 2), (2, 2), (3, 3), (4, 3), (5, 3), (6, 4)] {
        let m = mesa();
        inscrever_todos(&m, &jurados(&m, MINIMO as usize));
        m.env
            .ledger()
            .with_mut(|l| l.timestamp = GENESIS_QUICKNET + dt);
        assert_eq!(m.c.lock().rodada_minima, esperado, "dt = {}", dt);
    }
}

#[test]
fn fechar_duas_vezes_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    m.c.lock();
    assert_eq!(m.c.try_lock(), Err(Ok(Erro::RolFechado.into())));
}

/// INV-11: o rol é comprometido antes do caso existir.
#[test]
fn inscrever_depois_de_fechar_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    m.c.lock();
    let tarde = Address::generate(&m.env);
    assert_eq!(m.c.try_enroll(&tarde), Err(Ok(Erro::RolFechado.into())));
    assert_eq!(bilhete(&m).balance(&tarde), 0);
    assert_eq!(m.c.roster().len(), MINIMO);
}

// ---- sorteio -------------------------------------------------------------

#[test]
fn sortear_antes_de_fechar_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    let r = m.c.try_draw_round(&ROUND_A, &sig_a(&m), &3u32);
    assert_eq!(r, Err(Ok(Erro::RolAberto.into())));
}

#[test]
fn sorteio_com_rodada_anterior_ao_fechamento_e_recusado() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    // Fecha no segundo em que a rodada A sai: a mínima passa a ser A+1.
    m.env
        .ledger()
        .with_mut(|l| l.timestamp = GENESIS_QUICKNET + (ROUND_A - 1) * PERIODO_QUICKNET);
    assert_eq!(m.c.lock().rodada_minima, ROUND_A + 1);
    let r = m.c.try_draw_round(&ROUND_A, &sig_a(&m), &3u32);
    assert_eq!(r, Err(Ok(Erro::RodadaAnteriorAoRol.into())));
    // Uma rodada bem posterior passa.
    let painel =
        m.c.draw_round(&ROUND_B, &BytesN::from_array(&m.env, &SIG_B), &3u32);
    assert_eq!(painel.len(), 3);
}

#[test]
fn assinatura_forjada_e_recusada() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, MINIMO as usize));
    m.c.lock();
    let r =
        m.c.try_draw_round(&ROUND_A, &BytesN::from_array(&m.env, &SIG_FORJADA), &3u32);
    assert_eq!(r, Err(Ok(Erro::AssinaturaInvalida.into())));
}

/// A ordem é o vetor de ouro "rol pequeno" (n = 7) do beacon-verifier:
/// [0, 1, 4, 2, 6, 5, 3]. O painel são os endereços nessas posições.
const ORDEM_DE_OURO: [u32; 7] = [0, 1, 4, 2, 6, 5, 3];

#[test]
fn painel_segue_a_ordem_de_ouro() {
    let m = mesa();
    let js = jurados(&m, 7);
    inscrever_todos(&m, &js);
    m.c.lock();
    let painel = m.c.draw_round(&ROUND_A, &sig_a(&m), &3u32);
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
/// próximo da ordem já sorteada — não há novo sorteio.
#[test]
fn impedido_e_substituido_e_o_painel_fica_do_mesmo_tamanho() {
    let m = mesa();
    let js = jurados(&m, 7);
    inscrever_todos(&m, &js);
    m.c.lock();
    let primeiro = &js[ORDEM_DE_OURO[0] as usize];
    m.c.impede(primeiro);
    assert!(!sac(&m).authorized(primeiro));

    let painel = m.c.draw_round(&ROUND_A, &sig_a(&m), &3u32);
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
    m.c.lock();
    // Impede o segundo da ordem: o painel pula só ele.
    m.c.impede(&js[ORDEM_DE_OURO[1] as usize]);
    let painel = m.c.draw_round(&ROUND_A, &sig_a(&m), &3u32);
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
    m.c.lock();
    let doador = &js[ORDEM_DE_OURO[0] as usize];
    let receptor = &js[ORDEM_DE_OURO[6] as usize];
    bilhete(&m).transfer(doador, receptor, &1);
    assert_eq!(bilhete(&m).balance(doador), 0);
    assert!(
        sac(&m).authorized(doador),
        "continua autorizado, mas sem bilhete"
    );

    let painel = m.c.draw_round(&ROUND_A, &sig_a(&m), &3u32);
    assert!(!painel.contains(doador));
    assert_eq!(painel.get(0).unwrap(), js[ORDEM_DE_OURO[1] as usize]);
}

#[test]
fn elegiveis_insuficientes_e_recusado() {
    let m = mesa();
    let js = jurados(&m, 7);
    inscrever_todos(&m, &js);
    m.c.lock();
    for j in &js[..5] {
        m.c.impede(j);
    }
    let r = m.c.try_draw_round(&ROUND_A, &sig_a(&m), &3u32);
    assert_eq!(r, Err(Ok(Erro::ElegiveisInsuficientes.into())));
    let ok = m.c.draw_round(&ROUND_A, &sig_a(&m), &2u32);
    assert_eq!(ok.len(), 2);
}

#[test]
fn o_sorteio_e_deterministico() {
    let m = mesa();
    inscrever_todos(&m, &jurados(&m, 7));
    m.c.lock();
    let a = m.c.draw_round(&ROUND_A, &sig_a(&m), &5u32);
    let b = m.c.draw_round(&ROUND_A, &sig_a(&m), &5u32);
    assert_eq!(a, b);
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
    m.c.lock();
    m.c.impede(&js[ORDEM_DE_OURO[0] as usize]);
    let painel = m.c.draw_round(&ROUND_A, &sig_a(&m), &3u32);
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
