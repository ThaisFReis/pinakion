#![cfg(test)]
extern crate std;
use super::*;
use kleroterion::{Kleroterion as Klero, KleroterionClient as KleroClient, Nivel, Prova};
use soroban_sdk::testutils::{Address as _, IssuerFlags, Ledger as _};
use soroban_sdk::{contract, contractimpl, token, Address, Bytes, BytesN, Env, U256};
use std::borrow::ToOwned;

/// A rodada 1000000 da quicknet e a sua assinatura, as mesmas de
/// contracts/kleroterion/src/test.rs.
const ROUND_A: u64 = 1000000;
const SIG_A: [u8; 96] = [
    0x03, 0xad, 0x29, 0xe4, 0xc4, 0x09, 0xf9, 0x47, 0x0f, 0xc2, 0xef, 0x02, 0xf9, 0x02, 0x14, 0xdf,
    0x49, 0xe0, 0x2b, 0x44, 0x1a, 0x1a, 0x24, 0x1a, 0x82, 0xd6, 0x22, 0xd9, 0xf6, 0x08, 0xef, 0x98,
    0xfd, 0x8b, 0x11, 0xa0, 0x29, 0xf1, 0xbe, 0xe9, 0xd9, 0xe8, 0x3b, 0x45, 0x08, 0x8a, 0xbe, 0x72,
    0x01, 0x77, 0x6f, 0xf7, 0x40, 0x8b, 0x39, 0xc5, 0xf6, 0xf9, 0xfa, 0x50, 0x74, 0x6e, 0xfd, 0x7e,
    0xea, 0x17, 0xfb, 0xb6, 0x1f, 0x2e, 0x7b, 0x9c, 0x84, 0x9f, 0xf0, 0x52, 0x8e, 0x5a, 0x3d, 0xee,
    0xed, 0xd0, 0x29, 0xd0, 0xdf, 0x34, 0x51, 0x99, 0x96, 0x3d, 0x75, 0xba, 0x93, 0xb5, 0xa0, 0x2a,
];
const FLUXO: &str = include_str!("../../../tests/fluxo-n8.json");
const JANELA: u32 = 100;
const PRAZO_QUORUM: u32 = 200;

// ---- um Tessera de teste ---------------------------------------------------

/// Faz o papel do Tessera nos testes: guarda a proposta e o resultado que o
/// teste mandar. O Tessera real roda na testnet (scripts/sequencia_processo.sh).
#[contract]
pub struct TesseraDeTeste;

#[contracttype]
enum ChaveT {
    P(BytesN<32>),
    R(BytesN<32>),
}

#[contractimpl]
impl TesseraDeTeste {
    pub fn por_proposta(env: Env, id: BytesN<32>, p: Proposta) {
        env.storage().instance().set(&ChaveT::P(id), &p);
    }
    pub fn por_resultado(env: Env, id: BytesN<32>, r: Vec<u32>) {
        env.storage().instance().set(&ChaveT::R(id), &r);
    }
    pub fn proposta(env: Env, proposta: BytesN<32>) -> Option<Proposta> {
        env.storage().instance().get(&ChaveT::P(proposta))
    }
    pub fn resultado(env: Env, proposta: BytesN<32>) -> Option<Vec<u32>> {
        env.storage().instance().get(&ChaveT::R(proposta))
    }
}

// ---- a mesa: um kleroterion de verdade com o painel da fixture -------------

struct Mesa {
    env: Env,
    escrivao: Address,
    acusado: Address,
    klero: KleroClient<'static>,
    tessera: TesseraDeTesteClient<'static>,
    c: ProcessoClient<'static>,
    /// As contas de assento que sentaram, e as que reivindicaram e não.
    sentados: std::vec::Vec<Address>,
    de_fora: std::vec::Vec<Address>,
}

fn u256(env: &Env, decimal: &str) -> U256 {
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

/// Inscreve os oito jurados da fixture, fecha no nível `gravidade` (na tabela
/// de teste o nível `g` senta `k = g`) contra uma parte fora do rol, sela com
/// a rodada 1000000, reivindica `quem` e finaliza. Depois constitui o
/// processo, sem abrir o caso.
fn mesa(gravidade: u32, quem: &[usize], votos_minimos: u32, dois_tercos_desde: u32) -> Mesa {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger()
        .with_mut(|l| l.timestamp = kleroterion::GENESIS_QUICKNET);
    let escrivao = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(escrivao.clone());
    sac.issuer().set_flag(IssuerFlags::RequiredFlag);
    sac.issuer().set_flag(IssuerFlags::RevocableFlag);
    let verificador = env.register(groth16_verifier::Groth16Verifier, ());
    let mut niveis = soroban_sdk::Vec::new(&env);
    for k in 1..=8 {
        niveis.push_back(Nivel { k, rol_minimo: 1 });
    }
    let kid = env.register(
        Klero,
        (
            &escrivao,
            &sac.address(),
            &verificador,
            &JANELA,
            niveis,
            &10_000u32,
            &1_000u32,
        ),
    );
    token::StellarAssetClient::new(&env, &sac.address()).set_admin(&kid);
    let klero = KleroClient::new(&env, &kid);

    let j: serde_json::Value = serde_json::from_str(FLUXO).unwrap();
    let txt = |v: &serde_json::Value| v.as_str().unwrap().to_owned();
    let js = j["jurados"].as_array().unwrap();
    for x in js {
        klero.enroll(&Address::generate(&env), &u256(&env, &txt(&x["c"])));
    }
    let acusado = Address::generate(&env);
    env.ledger().with_mut(|l| {
        l.timestamp = kleroterion::GENESIS_QUICKNET
            + (ROUND_A - 2 - kleroterion::MARGEM_DE_RODADAS) * kleroterion::PERIODO_QUICKNET
    });
    klero.lock(&gravidade, &20_000, &acusado);
    env.ledger().with_mut(|l| l.sequence_number = 1000);
    klero.seal(&BytesN::from_array(&env, &SIG_A));
    let bn = |h: &str| hex::decode(h).unwrap();
    let mut assentos = std::vec::Vec::new();
    for &i in quem {
        let x = &js[i];
        let prova = Prova {
            a: BytesN::from_array(&env, &bn(&txt(&x["proof"]["a"])).try_into().unwrap()),
            b: BytesN::from_array(&env, &bn(&txt(&x["proof"]["b"])).try_into().unwrap()),
            c: BytesN::from_array(&env, &bn(&txt(&x["proof"]["c"])).try_into().unwrap()),
        };
        let assento = Address::from_str(&env, &txt(&x["assento"]));
        klero.claim(&assento, &prova, &u256(&env, &txt(&x["y"])));
        assentos.push(assento);
    }
    env.ledger()
        .with_mut(|l| l.sequence_number = 1000 + JANELA + 1);
    klero.finalize().expect("painel");
    let (sentados, de_fora): (std::vec::Vec<_>, std::vec::Vec<_>) =
        assentos.into_iter().partition(|a| klero.seated(a));

    let tid = env.register(TesseraDeTeste, ());
    let tessera = TesseraDeTesteClient::new(&env, &tid);
    let id = env.register(
        Processo,
        (
            &escrivao,
            &kid,
            &tid,
            &PRAZO_QUORUM,
            &votos_minimos,
            &dois_tercos_desde,
        ),
    );
    let c = ProcessoClient::new(&env, &id);
    Mesa {
        env,
        escrivao,
        acusado,
        klero,
        tessera,
        c,
        sentados,
        de_fora,
    }
}

/// Nível 3, `k = 3`, cinco reivindicações: três sentam, duas ficam de fora.
fn mesa_padrao() -> Mesa {
    mesa(3, &[1, 2, 3, 5, 7], 2, 4)
}

fn h(env: &Env, b: u8) -> BytesN<32> {
    BytesN::from_array(env, &[b; 32])
}

fn id(env: &Env) -> BytesN<32> {
    h(env, 0xee)
}

/// A votação do Tessera que o processo aceita: uma pergunta sigilosa de duas
/// opções, aptos = os presentes, mesa de presentes, aberta agora.
fn proposta_boa(m: &Mesa) -> Proposta {
    let presentes = m.c.presentes();
    Proposta {
        perguntas: soroban_sdk::vec![
            &m.env,
            Pergunta {
                opcoes: 2,
                confidencial: true,
            }
        ],
        raiz_aptos: m.c.raiz_dos_aptos(&presentes),
        mesa: presentes.slice(0..2),
        limiar: 2,
        abre_em: m.env.ledger().sequence(),
        fecha_em: m.env.ledger().sequence() + 30,
        rodada: 0,
        rodada_abertura: 0,
        anel: false,
        secoes: 1,
        limite_secao: 0,
    }
}

/// Abre o caso, faz os sentados comparecerem, vincula a votação boa.
fn ate_a_votacao(m: &Mesa) {
    m.c.abrir(&h(&m.env, 1));
    for a in &m.sentados {
        m.c.comparecer(a);
    }
    m.tessera.por_proposta(&id(&m.env), &proposta_boa(m));
    m.c.vincular_votacao(&id(&m.env));
}

// ---- a raiz dos aptos é a do Tessera ---------------------------------------

/// A votação `smk2` que o Tessera (commit 356c22f, CLI corrigida) abriu na
/// testnet em 2026-10-26, contrato CDWBWAHUZ3MC3BJXJIW5O74MNUXH76VCGBIVFQU3XJJFGML3VKFIXQT7:
/// cinco aptos, nesta ordem, e a raiz que a CLI do Tessera gravou. Se a regra
/// daqui divergir da do Tessera, nenhuma votação real vincula.
#[test]
fn raiz_dos_aptos_bate_com_a_do_tessera() {
    let env = Env::default();
    let id = env.register(
        Processo,
        (
            &Address::generate(&env),
            &Address::generate(&env),
            &Address::generate(&env),
            &1u32,
            &1u32,
            &4u32,
        ),
    );
    let c = ProcessoClient::new(&env, &id);
    let mut aptos = soroban_sdk::Vec::new(&env);
    for a in [
        "GCD6PXQMFVLVDXIOGGPO66RUDXZZFBCVYMVDM52SGNSJBW7DLBJD5TOZ",
        "GCRVDJBXUB6S6VSI7IEFFJ5EM3BLHR7JCPRXYFBXSXCMDLIU3JGL3HG2",
        "GAFFG5TXKN5RO2D5CFHELQIXIN6FPGOEBB2RR5L7XOULTVP3VG7EITOL",
        "GBCUXSSZTGGYR6BET4XESF4JF35YBKU3UX7ALOGPQRIHRH2YYVFHXSV3",
        "GDHNTWPV5Q7PRTCEQJYKPJM2EZ4N5TTMOF5YEXS2D4RQRK7GNI2JIURY",
    ] {
        aptos.push_back(Address::from_str(&env, a));
    }
    let esperado: [u8; 32] =
        hex::decode("52c7847cf9b3b968457a378074c9dd768f7d5d105891119093645b75f3376048")
            .unwrap()
            .try_into()
            .unwrap();
    assert_eq!(c.raiz_dos_aptos(&aptos).to_array(), esperado);
}

// ---- abrir -----------------------------------------------------------------

#[test]
fn abrir_le_o_caso_do_kleroterion_e_registra_a_denuncia() {
    let m = mesa_padrao();
    let caso = m.c.abrir(&h(&m.env, 1));
    assert_eq!(caso.acusado, m.acusado);
    assert_eq!(
        (caso.gravidade, caso.k, caso.quorum, caso.exigidos),
        (3, 3, 2, 2)
    );
    assert_eq!(m.c.fase(), Some(Fase::Aberto));
    let r = m.c.registros().get(0).unwrap();
    assert_eq!(
        (r.seq, r.tipo, r.papel),
        (0, Tipo::Denuncia, Papel::Escrivao)
    );
    assert_eq!(r.hash, h(&m.env, 1));
    assert_eq!(m.c.try_abrir(&h(&m.env, 2)), Err(Ok(Erro::JaAberto.into())));
}

#[test]
fn abrir_antes_de_o_rol_fechar_e_recusado() {
    let env = Env::default();
    env.mock_all_auths();
    let escrivao = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(escrivao.clone());
    let mut niveis = soroban_sdk::Vec::new(&env);
    niveis.push_back(Nivel {
        k: 1,
        rol_minimo: 1,
    });
    let kid = env.register(
        Klero,
        (
            &escrivao,
            &sac.address(),
            &Address::generate(&env),
            &JANELA,
            niveis,
            &1u32,
            &1u32,
        ),
    );
    let id = env.register(
        Processo,
        (
            &escrivao,
            &kid,
            &Address::generate(&env),
            &1u32,
            &1u32,
            &4u32,
        ),
    );
    let c = ProcessoClient::new(&env, &id);
    assert_eq!(
        c.try_abrir(&BytesN::from_array(&env, &[1; 32])),
        Err(Ok(Erro::RolAberto.into()))
    );
}

/// Com o piso de sigilo do Tessera (5) e `k = 3`, a votação nunca teria placar.
#[test]
fn painel_menor_que_o_sigilo_e_recusado() {
    let m = mesa(3, &[1, 2, 3, 5, 7], 5, 4);
    assert_eq!(
        m.c.try_abrir(&h(&m.env, 1)),
        Err(Ok(Erro::PainelPequenoParaOSigilo.into()))
    );
}

/// O quórum é a maioria do painel, nunca abaixo do piso de votos; a decisão
/// exige dois terços a partir do nível configurado. Com `k = 5`: maioria 3,
/// dois terços 4.
#[test]
fn quorum_e_votos_exigidos_seguem_a_regra() {
    let m = mesa(5, &[0, 1, 2, 3, 5, 6, 7], 2, 5);
    let caso = m.c.abrir(&h(&m.env, 1));
    assert_eq!((caso.k, caso.quorum, caso.exigidos), (5, 3, 4));
    let m = mesa(5, &[0, 1, 2, 3, 5, 6, 7], 4, 6);
    let caso = m.c.abrir(&h(&m.env, 1));
    assert_eq!((caso.quorum, caso.exigidos), (4, 3));
}

// ---- comparecer ------------------------------------------------------------

#[test]
fn so_quem_sentou_comparece_e_uma_vez() {
    let m = mesa_padrao();
    m.c.abrir(&h(&m.env, 1));
    assert_eq!(m.sentados.len(), 3);
    assert_eq!(m.de_fora.len(), 2);
    assert_eq!(
        m.c.try_comparecer(&m.de_fora[0]),
        Err(Ok(Erro::NaoSentou.into()))
    );
    assert_eq!(
        m.c.try_comparecer(&m.acusado),
        Err(Ok(Erro::NaoSentou.into()))
    );
    m.c.comparecer(&m.sentados[0]);
    assert_eq!(
        m.c.try_comparecer(&m.sentados[0]),
        Err(Ok(Erro::JaCompareceu.into()))
    );
}

#[test]
fn o_quorum_instala_a_sessao_e_quem_chega_depois_conta() {
    let m = mesa_padrao();
    m.c.abrir(&h(&m.env, 1));
    m.c.comparecer(&m.sentados[0]);
    assert_eq!(m.c.fase(), Some(Fase::Aberto));
    m.c.comparecer(&m.sentados[1]);
    assert_eq!(m.c.fase(), Some(Fase::Sessao), "quorum 2");
    // Depois do prazo, a sessão já instalada ainda recebe quem chega.
    m.env
        .ledger()
        .with_mut(|l| l.sequence_number = 1000 + JANELA + 1 + PRAZO_QUORUM + 1);
    assert_eq!(m.c.comparecer(&m.sentados[2]), 3);
}

#[test]
fn sem_quorum_no_prazo_o_caso_e_adiado() {
    let m = mesa_padrao();
    m.c.abrir(&h(&m.env, 1));
    m.c.comparecer(&m.sentados[0]);
    assert_eq!(m.c.try_adiar(), Err(Ok(Erro::PrazoDoQuorumAberto.into())));
    m.env
        .ledger()
        .with_mut(|l| l.sequence_number = 1000 + JANELA + 1 + PRAZO_QUORUM + 1);
    assert_eq!(
        m.c.try_comparecer(&m.sentados[1]),
        Err(Ok(Erro::PrazoDoQuorumVencido.into()))
    );
    m.c.adiar();
    assert_eq!(m.c.fase(), Some(Fase::Adiado));
    assert!(
        !m.c.suspended(&m.acusado),
        "o adiamento encerra o afastamento"
    );
}

// ---- os autos --------------------------------------------------------------

#[test]
fn cada_papel_registra_so_o_que_lhe_cabe() {
    let m = mesa_padrao();
    m.c.abrir(&h(&m.env, 1));
    m.c.comparecer(&m.sentados[0]);
    let sem_papel = Err(Ok(Erro::SemPapel.into()));
    // A pessoa julgada: só a defesa.
    assert_eq!(m.c.registrar(&m.acusado, &Tipo::Defesa, &h(&m.env, 2)), 1);
    assert_eq!(
        m.c.try_registrar(&m.acusado, &Tipo::Ata, &h(&m.env, 3)),
        sem_papel
    );
    // Um jurado presente: só atas. Um sentado ausente, nada.
    assert_eq!(m.c.registrar(&m.sentados[0], &Tipo::Ata, &h(&m.env, 4)), 2);
    assert_eq!(
        m.c.try_registrar(&m.sentados[0], &Tipo::Prova, &h(&m.env, 5)),
        sem_papel
    );
    assert_eq!(
        m.c.try_registrar(&m.sentados[1], &Tipo::Ata, &h(&m.env, 5)),
        sem_papel
    );
    // O escrivão: provas e atas, nunca a defesa nem outra denúncia.
    assert_eq!(m.c.registrar(&m.escrivao, &Tipo::Prova, &h(&m.env, 6)), 3);
    assert_eq!(
        m.c.try_registrar(&m.escrivao, &Tipo::Defesa, &h(&m.env, 7)),
        sem_papel
    );
    assert_eq!(
        m.c.try_registrar(&m.escrivao, &Tipo::Denuncia, &h(&m.env, 7)),
        sem_papel
    );
    // Um estranho, nada.
    assert_eq!(
        m.c.try_registrar(&Address::generate(&m.env), &Tipo::Ata, &h(&m.env, 8)),
        sem_papel
    );
    let rs = m.c.registros();
    assert_eq!(rs.len(), 4);
    assert_eq!(rs.get(1).unwrap().papel, Papel::Acusado);
    assert_eq!(rs.get(2).unwrap().papel, Papel::Jurado);
}

#[test]
fn depois_da_votacao_nao_entram_provas_e_depois_do_veredito_so_a_ata_do_escrivao() {
    let m = mesa_padrao();
    ate_a_votacao(&m);
    assert_eq!(
        m.c.try_registrar(&m.escrivao, &Tipo::Prova, &h(&m.env, 9)),
        Err(Ok(Erro::SemPapel.into()))
    );
    m.c.registrar(&m.acusado, &Tipo::Defesa, &h(&m.env, 9));
    m.tessera
        .por_resultado(&id(&m.env), &soroban_sdk::vec![&m.env, 2, 1]);
    m.c.decidir();
    assert_eq!(
        m.c.try_registrar(&m.acusado, &Tipo::Defesa, &h(&m.env, 10)),
        Err(Ok(Erro::SemPapel.into()))
    );
    m.c.registrar(&m.escrivao, &Tipo::Ata, &h(&m.env, 11));
}

// ---- a votação é a dos presentes -------------------------------------------

#[test]
fn a_votacao_vinculada_tem_de_ser_dos_presentes() {
    let m = mesa_padrao();
    m.c.abrir(&h(&m.env, 1));
    m.c.comparecer(&m.sentados[0]);
    assert_eq!(
        m.c.try_vincular_votacao(&id(&m.env)),
        Err(Ok(Erro::ForaDeFase.into())),
        "antes da sessao"
    );
    m.c.comparecer(&m.sentados[1]);
    assert_eq!(
        m.c.try_vincular_votacao(&id(&m.env)),
        Err(Ok(Erro::SemVotacao.into()))
    );
    let boa = proposta_boa(&m);

    let mut p = boa.clone();
    let mut outros = m.c.presentes();
    outros.push_back(m.de_fora[0].clone());
    p.raiz_aptos = m.c.raiz_dos_aptos(&outros);
    m.tessera.por_proposta(&id(&m.env), &p);
    assert_eq!(
        m.c.try_vincular_votacao(&id(&m.env)),
        Err(Ok(Erro::AptosNaoSaoOsPresentes.into())),
        "um aptos a mais"
    );

    let mut p = boa.clone();
    p.mesa = soroban_sdk::vec![&m.env, m.de_fora[0].clone()];
    m.tessera.por_proposta(&id(&m.env), &p);
    assert_eq!(
        m.c.try_vincular_votacao(&id(&m.env)),
        Err(Ok(Erro::MesaDeFora.into()))
    );

    let mut p = boa.clone();
    p.perguntas = soroban_sdk::vec![
        &m.env,
        Pergunta {
            opcoes: 2,
            confidencial: false,
        }
    ];
    m.tessera.por_proposta(&id(&m.env), &p);
    assert_eq!(
        m.c.try_vincular_votacao(&id(&m.env)),
        Err(Ok(Erro::CedulaErrada.into())),
        "voto aberto"
    );

    let mut p = boa.clone();
    p.abre_em = 1;
    m.tessera.por_proposta(&id(&m.env), &p);
    assert_eq!(
        m.c.try_vincular_votacao(&id(&m.env)),
        Err(Ok(Erro::VotacaoAnteriorASessao.into()))
    );

    m.tessera.por_proposta(&id(&m.env), &boa);
    m.c.vincular_votacao(&id(&m.env));
    assert_eq!(m.c.fase(), Some(Fase::Votacao));
    assert_eq!(m.c.votacao(), Some(id(&m.env)));
    assert_eq!(
        m.c.try_comparecer(&m.sentados[2]),
        Err(Ok(Erro::ForaDeFase.into())),
        "depois da votacao ninguem mais comparece"
    );
}

// ---- o veredito ------------------------------------------------------------

#[test]
fn sem_placar_nao_ha_veredito() {
    let m = mesa_padrao();
    ate_a_votacao(&m);
    assert_eq!(m.c.try_decidir(), Err(Ok(Erro::SemResultado.into())));
    m.tessera
        .por_resultado(&id(&m.env), &soroban_sdk::vec![&m.env, 1, 1, 1]);
    assert_eq!(
        m.c.try_decidir(),
        Err(Ok(Erro::SemResultado.into())),
        "tres opcoes"
    );
}

/// `k = 5`: com a maioria, 3 votos sim bastam; com dois terços, são 4.
/// A ausência não conta a favor: o que se exige é do painel, não dos presentes.
#[test]
fn a_maioria_e_os_dois_tercos_contam_sobre_o_painel() {
    for (desde, procedente) in [(6u32, true), (5, false)] {
        let m = mesa(5, &[0, 1, 2, 3, 5, 6, 7], 2, desde);
        ate_a_votacao(&m);
        m.tessera
            .por_resultado(&id(&m.env), &soroban_sdk::vec![&m.env, 3, 0]);
        let d = m.c.decidir();
        assert_eq!((d.sim, d.nao, d.procedente), (3, 0, procedente));
        assert_eq!(m.c.fase(), Some(Fase::Decidido));
        assert_eq!(m.c.decisao(), Some(d));
    }
}

#[test]
fn o_afastamento_termina_no_veredito() {
    let m = mesa_padrao();
    ate_a_votacao(&m);
    assert!(m.klero.suspended(&m.acusado));
    assert!(m.c.suspended(&m.acusado));
    m.tessera
        .por_resultado(&id(&m.env), &soroban_sdk::vec![&m.env, 0, 3]);
    assert!(!m.c.decidir().procedente);
    assert!(m.klero.suspended(&m.acusado), "o kleroterion so ve o prazo");
    assert!(!m.c.suspended(&m.acusado), "o processo ve o veredito");
}
