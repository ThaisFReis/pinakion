#![cfg(test)]
extern crate std;
use super::*;
use kleroterion::{Kleroterion as Klero, KleroterionClient as KleroClient, Level, Proof};
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
const WINDOW: u32 = 100;
const QUORUM_DEADLINE: u32 = 200;

// ---- um Tessera de teste ---------------------------------------------------

/// Faz o role do Tessera nos testes: guarda a proposta e o resultado que o
/// teste mandar. O Tessera real roda na testnet (scripts/sequencia_processo.sh).
#[contract]
pub struct TestTessera;

#[contracttype]
enum TestKey {
    P(BytesN<32>),
    R(BytesN<32>),
}

#[contractimpl]
impl TestTessera {
    pub fn put_proposal(env: Env, id: BytesN<32>, p: TesseraProposal) {
        env.storage().instance().set(&TestKey::P(id), &p);
    }
    pub fn put_result(env: Env, id: BytesN<32>, r: Vec<u32>) {
        env.storage().instance().set(&TestKey::R(id), &r);
    }
    pub fn proposta(env: Env, proposta: BytesN<32>) -> Option<TesseraProposal> {
        env.storage().instance().get(&TestKey::P(proposta))
    }
    pub fn resultado(env: Env, proposta: BytesN<32>) -> Option<Vec<u32>> {
        env.storage().instance().get(&TestKey::R(proposta))
    }
}

// ---- a bench: um kleroterion de verdade com o painel da fixture -------------

struct Bench {
    env: Env,
    clerk: Address,
    accused: Address,
    klero: KleroClient<'static>,
    tessera: TestTesseraClient<'static>,
    c: ProceedingClient<'static>,
    /// As contas de seat que sentaram, e as que reivindicaram e não.
    seated: std::vec::Vec<Address>,
    outsiders: std::vec::Vec<Address>,
}

fn u256(env: &Env, decimal: &str) -> U256 {
    let mut b = [0u8; 32];
    for d in decimal.bytes() {
        let mut carry = (d - b'0') as u32;
        for x in b.iter_mut().rev() {
            let v = (*x as u32) * 10 + carry;
            *x = v as u8;
            carry = v >> 8;
        }
    }
    U256::from_be_bytes(env, &Bytes::from_array(env, &b))
}

/// Inscreve os oito jurors da fixture, fecha no nível `severity` (na tabela
/// de teste o nível `g` senta `k = g`) contra uma parte fora do rol, sela com
/// a rodada 1000000, reivindica `who` e finaliza. Depois constitui o
/// processo, sem open o caso.
fn bench(severity: u32, who: &[usize], minimum_votes: u32, two_thirds_from: u32) -> Bench {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger()
        .with_mut(|l| l.timestamp = kleroterion::QUICKNET_GENESIS);
    let clerk = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(clerk.clone());
    sac.issuer().set_flag(IssuerFlags::RequiredFlag);
    sac.issuer().set_flag(IssuerFlags::RevocableFlag);
    let verifier = env.register(groth16_verifier::Groth16Verifier, ());
    let mut levels = soroban_sdk::Vec::new(&env);
    for k in 1..=8 {
        levels.push_back(Level {
            k,
            roster_minimum: 1,
        });
    }
    let kid = env.register(
        Klero,
        (
            &clerk,
            &sac.address(),
            &verifier,
            &WINDOW,
            levels,
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
    let accused = Address::generate(&env);
    env.ledger().with_mut(|l| {
        l.timestamp = kleroterion::QUICKNET_GENESIS
            + (ROUND_A - 2 - kleroterion::ROUND_MARGIN) * kleroterion::QUICKNET_PERIOD
    });
    klero.lock(&severity, &20_000, &accused);
    env.ledger().with_mut(|l| l.sequence_number = 1000);
    klero.seal(&BytesN::from_array(&env, &SIG_A));
    let bn = |h: &str| hex::decode(h).unwrap();
    let mut seats = std::vec::Vec::new();
    for &i in who {
        let x = &js[i];
        let proof = Proof {
            a: BytesN::from_array(&env, &bn(&txt(&x["proof"]["a"])).try_into().unwrap()),
            b: BytesN::from_array(&env, &bn(&txt(&x["proof"]["b"])).try_into().unwrap()),
            c: BytesN::from_array(&env, &bn(&txt(&x["proof"]["c"])).try_into().unwrap()),
        };
        let seat = Address::from_str(&env, &txt(&x["assento"]));
        klero.claim(&seat, &proof, &u256(&env, &txt(&x["y"])));
        seats.push(seat);
    }
    env.ledger()
        .with_mut(|l| l.sequence_number = 1000 + WINDOW + 1);
    klero.finalize().expect("painel");
    let (seated, outsiders): (std::vec::Vec<_>, std::vec::Vec<_>) =
        seats.into_iter().partition(|a| klero.seated(a));

    let tid = env.register(TestTessera, ());
    let tessera = TestTesseraClient::new(&env, &tid);
    let id = env.register(
        Proceeding,
        (
            &clerk,
            &kid,
            &tid,
            &QUORUM_DEADLINE,
            &minimum_votes,
            &two_thirds_from,
        ),
    );
    let c = ProceedingClient::new(&env, &id);
    Bench {
        env,
        clerk,
        accused,
        klero,
        tessera,
        c,
        seated,
        outsiders,
    }
}

/// Nível 3, `k = 3`, cinco reivindicações: três sentam, duas ficam de fora.
fn default_bench() -> Bench {
    bench(3, &[1, 2, 3, 5, 7], 2, 4)
}

fn h(env: &Env, b: u8) -> BytesN<32> {
    BytesN::from_array(env, &[b; 32])
}

fn id(env: &Env) -> BytesN<32> {
    h(env, 0xee)
}

/// A votação do Tessera que o processo aceita: uma pergunta sigilosa de duas
/// opções, aptos = os attendees, bench de attendees, aberta agora.
fn good_proposal(m: &Bench) -> TesseraProposal {
    let attendees = m.c.attendees();
    TesseraProposal {
        perguntas: soroban_sdk::vec![
            &m.env,
            TesseraQuestion {
                opcoes: 2,
                confidencial: true,
            }
        ],
        raiz_aptos: m.c.voter_root(&attendees),
        mesa: attendees.slice(0..2),
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

/// Abre o caso, faz os seated comparecerem, vincula a votação boa.
fn up_to_the_vote(m: &Bench) {
    m.c.open(&h(&m.env, 1));
    for a in &m.seated {
        m.c.attend(a);
    }
    m.tessera.put_proposal(&id(&m.env), &good_proposal(m));
    m.c.bind_vote(&id(&m.env));
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
        Proceeding,
        (
            &Address::generate(&env),
            &Address::generate(&env),
            &Address::generate(&env),
            &1u32,
            &1u32,
            &4u32,
        ),
    );
    let c = ProceedingClient::new(&env, &id);
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
    let expected: [u8; 32] =
        hex::decode("52c7847cf9b3b968457a378074c9dd768f7d5d105891119093645b75f3376048")
            .unwrap()
            .try_into()
            .unwrap();
    assert_eq!(c.voter_root(&aptos).to_array(), expected);
}

// ---- open -----------------------------------------------------------------

#[test]
fn abrir_le_o_caso_do_kleroterion_e_registra_a_denuncia() {
    let m = default_bench();
    let caso = m.c.open(&h(&m.env, 1));
    assert_eq!(caso.accused, m.accused);
    assert_eq!(
        (caso.severity, caso.k, caso.quorum, caso.required),
        (3, 3, 2, 2)
    );
    assert_eq!(m.c.phase(), Some(Phase::Open));
    let r = m.c.records().get(0).unwrap();
    assert_eq!((r.seq, r.kind, r.role), (0, Kind::Complaint, Role::Clerk));
    assert_eq!(r.hash, h(&m.env, 1));
    assert_eq!(
        m.c.try_open(&h(&m.env, 2)),
        Err(Ok(Error::AlreadyOpen.into()))
    );
}

#[test]
fn abrir_antes_de_o_rol_fechar_e_recusado() {
    let env = Env::default();
    env.mock_all_auths();
    let clerk = Address::generate(&env);
    let sac = env.register_stellar_asset_contract_v2(clerk.clone());
    let mut levels = soroban_sdk::Vec::new(&env);
    levels.push_back(Level {
        k: 1,
        roster_minimum: 1,
    });
    let kid = env.register(
        Klero,
        (
            &clerk,
            &sac.address(),
            &Address::generate(&env),
            &WINDOW,
            levels,
            &1u32,
            &1u32,
        ),
    );
    let id = env.register(
        Proceeding,
        (&clerk, &kid, &Address::generate(&env), &1u32, &1u32, &4u32),
    );
    let c = ProceedingClient::new(&env, &id);
    assert_eq!(
        c.try_open(&BytesN::from_array(&env, &[1; 32])),
        Err(Ok(Error::RosterOpen.into()))
    );
}

/// Com o piso de sigilo do Tessera (5) e `k = 3`, a votação nunca teria placar.
#[test]
fn painel_menor_que_o_sigilo_e_recusado() {
    let m = bench(3, &[1, 2, 3, 5, 7], 5, 4);
    assert_eq!(
        m.c.try_open(&h(&m.env, 1)),
        Err(Ok(Error::PanelTooSmallForSecrecy.into()))
    );
}

/// O quórum é a maioria do painel, nunca abaixo do piso de votos; a decisão
/// exige dois terços a partir do nível configurado. Com `k = 5`: maioria 3,
/// dois terços 4.
#[test]
fn quorum_e_votos_exigidos_seguem_a_regra() {
    let m = bench(5, &[0, 1, 2, 3, 5, 6, 7], 2, 5);
    let caso = m.c.open(&h(&m.env, 1));
    assert_eq!((caso.k, caso.quorum, caso.required), (5, 3, 4));
    let m = bench(5, &[0, 1, 2, 3, 5, 6, 7], 4, 6);
    let caso = m.c.open(&h(&m.env, 1));
    assert_eq!((caso.quorum, caso.required), (4, 3));
}

// ---- attend ------------------------------------------------------------

#[test]
fn so_quem_sentou_comparece_e_uma_vez() {
    let m = default_bench();
    m.c.open(&h(&m.env, 1));
    assert_eq!(m.seated.len(), 3);
    assert_eq!(m.outsiders.len(), 2);
    assert_eq!(
        m.c.try_attend(&m.outsiders[0]),
        Err(Ok(Error::DidNotSit.into()))
    );
    assert_eq!(m.c.try_attend(&m.accused), Err(Ok(Error::DidNotSit.into())));
    m.c.attend(&m.seated[0]);
    assert_eq!(
        m.c.try_attend(&m.seated[0]),
        Err(Ok(Error::AlreadyAttended.into()))
    );
}

#[test]
fn o_quorum_instala_a_sessao_e_quem_chega_depois_conta() {
    let m = default_bench();
    m.c.open(&h(&m.env, 1));
    m.c.attend(&m.seated[0]);
    assert_eq!(m.c.phase(), Some(Phase::Open));
    m.c.attend(&m.seated[1]);
    assert_eq!(m.c.phase(), Some(Phase::Session), "quorum 2");
    // Depois do prazo, a sessão já instalada ainda recebe who chega.
    m.env
        .ledger()
        .with_mut(|l| l.sequence_number = 1000 + WINDOW + 1 + QUORUM_DEADLINE + 1);
    assert_eq!(m.c.attend(&m.seated[2]), 3);
}

#[test]
fn sem_quorum_no_prazo_o_caso_e_adiado() {
    let m = default_bench();
    m.c.open(&h(&m.env, 1));
    m.c.attend(&m.seated[0]);
    assert_eq!(m.c.try_adjourn(), Err(Ok(Error::QuorumDeadlineOpen.into())));
    m.env
        .ledger()
        .with_mut(|l| l.sequence_number = 1000 + WINDOW + 1 + QUORUM_DEADLINE + 1);
    assert_eq!(
        m.c.try_attend(&m.seated[1]),
        Err(Ok(Error::QuorumDeadlinePassed.into()))
    );
    m.c.adjourn();
    assert_eq!(m.c.phase(), Some(Phase::Adjourned));
    assert!(
        !m.c.suspended(&m.accused),
        "o adiamento encerra o afastamento"
    );
}

// ---- os autos --------------------------------------------------------------

#[test]
fn cada_papel_registra_so_o_que_lhe_cabe() {
    let m = default_bench();
    m.c.open(&h(&m.env, 1));
    m.c.attend(&m.seated[0]);
    let sem_papel = Err(Ok(Error::NoRole.into()));
    // A pessoa julgada: só a defesa.
    assert_eq!(m.c.file(&m.accused, &Kind::Defence, &h(&m.env, 2)), 1);
    assert_eq!(
        m.c.try_file(&m.accused, &Kind::Minutes, &h(&m.env, 3)),
        sem_papel
    );
    // Um jurado presente: só atas. Um sentado ausente, nada.
    assert_eq!(m.c.file(&m.seated[0], &Kind::Minutes, &h(&m.env, 4)), 2);
    assert_eq!(
        m.c.try_file(&m.seated[0], &Kind::Evidence, &h(&m.env, 5)),
        sem_papel
    );
    assert_eq!(
        m.c.try_file(&m.seated[1], &Kind::Minutes, &h(&m.env, 5)),
        sem_papel
    );
    // O escrivão: provas e atas, nunca a defesa nem outra denúncia.
    assert_eq!(m.c.file(&m.clerk, &Kind::Evidence, &h(&m.env, 6)), 3);
    assert_eq!(
        m.c.try_file(&m.clerk, &Kind::Defence, &h(&m.env, 7)),
        sem_papel
    );
    assert_eq!(
        m.c.try_file(&m.clerk, &Kind::Complaint, &h(&m.env, 7)),
        sem_papel
    );
    // Um estranho, nada.
    assert_eq!(
        m.c.try_file(&Address::generate(&m.env), &Kind::Minutes, &h(&m.env, 8)),
        sem_papel
    );
    let rs = m.c.records();
    assert_eq!(rs.len(), 4);
    assert_eq!(rs.get(1).unwrap().role, Role::Accused);
    assert_eq!(rs.get(2).unwrap().role, Role::Juror);
}

#[test]
fn depois_da_votacao_nao_entram_provas_e_depois_do_veredito_so_a_ata_do_escrivao() {
    let m = default_bench();
    up_to_the_vote(&m);
    assert_eq!(
        m.c.try_file(&m.clerk, &Kind::Evidence, &h(&m.env, 9)),
        Err(Ok(Error::NoRole.into()))
    );
    m.c.file(&m.accused, &Kind::Defence, &h(&m.env, 9));
    m.tessera
        .put_result(&id(&m.env), &soroban_sdk::vec![&m.env, 2, 1]);
    m.c.decide();
    assert_eq!(
        m.c.try_file(&m.accused, &Kind::Defence, &h(&m.env, 10)),
        Err(Ok(Error::NoRole.into()))
    );
    m.c.file(&m.clerk, &Kind::Minutes, &h(&m.env, 11));
}

// ---- a votação é a dos attendees -------------------------------------------

#[test]
fn a_votacao_vinculada_tem_de_ser_dos_presentes() {
    let m = default_bench();
    m.c.open(&h(&m.env, 1));
    m.c.attend(&m.seated[0]);
    assert_eq!(
        m.c.try_bind_vote(&id(&m.env)),
        Err(Ok(Error::OutOfPhase.into())),
        "antes da sessao"
    );
    m.c.attend(&m.seated[1]);
    assert_eq!(
        m.c.try_bind_vote(&id(&m.env)),
        Err(Ok(Error::NoVote.into()))
    );
    let boa = good_proposal(&m);

    let mut p = boa.clone();
    let mut outros = m.c.attendees();
    outros.push_back(m.outsiders[0].clone());
    p.raiz_aptos = m.c.voter_root(&outros);
    m.tessera.put_proposal(&id(&m.env), &p);
    assert_eq!(
        m.c.try_bind_vote(&id(&m.env)),
        Err(Ok(Error::VotersAreNotAttendees.into())),
        "um aptos a mais"
    );

    let mut p = boa.clone();
    p.mesa = soroban_sdk::vec![&m.env, m.outsiders[0].clone()];
    m.tessera.put_proposal(&id(&m.env), &p);
    assert_eq!(
        m.c.try_bind_vote(&id(&m.env)),
        Err(Ok(Error::BenchOutsider.into()))
    );

    let mut p = boa.clone();
    p.perguntas = soroban_sdk::vec![
        &m.env,
        TesseraQuestion {
            opcoes: 2,
            confidencial: false,
        }
    ];
    m.tessera.put_proposal(&id(&m.env), &p);
    assert_eq!(
        m.c.try_bind_vote(&id(&m.env)),
        Err(Ok(Error::WrongBallot.into())),
        "voto aberto"
    );

    let mut p = boa.clone();
    p.abre_em = 1;
    m.tessera.put_proposal(&id(&m.env), &p);
    assert_eq!(
        m.c.try_bind_vote(&id(&m.env)),
        Err(Ok(Error::VoteBeforeSession.into()))
    );

    m.tessera.put_proposal(&id(&m.env), &boa);
    m.c.bind_vote(&id(&m.env));
    assert_eq!(m.c.phase(), Some(Phase::Voting));
    assert_eq!(m.c.vote(), Some(id(&m.env)));
    assert_eq!(
        m.c.try_attend(&m.seated[2]),
        Err(Ok(Error::OutOfPhase.into())),
        "depois da votacao ninguem mais comparece"
    );
}

// ---- o veredito ------------------------------------------------------------

#[test]
fn sem_placar_nao_ha_veredito() {
    let m = default_bench();
    up_to_the_vote(&m);
    assert_eq!(m.c.try_decide(), Err(Ok(Error::NoResult.into())));
    m.tessera
        .put_result(&id(&m.env), &soroban_sdk::vec![&m.env, 1, 1, 1]);
    assert_eq!(
        m.c.try_decide(),
        Err(Ok(Error::NoResult.into())),
        "tres opcoes"
    );
}

/// `k = 5`: com a maioria, 3 votos yes bastam; com dois terços, são 4.
/// A ausência não conta a favor: o que se exige é do painel, não dos attendees.
#[test]
fn a_maioria_e_os_dois_tercos_contam_sobre_o_painel() {
    for (desde, upheld) in [(6u32, true), (5, false)] {
        let m = bench(5, &[0, 1, 2, 3, 5, 6, 7], 2, desde);
        up_to_the_vote(&m);
        m.tessera
            .put_result(&id(&m.env), &soroban_sdk::vec![&m.env, 3, 0]);
        let d = m.c.decide();
        assert_eq!((d.yes, d.no, d.upheld), (3, 0, upheld));
        assert_eq!(m.c.phase(), Some(Phase::Decided));
        assert_eq!(m.c.decision(), Some(d));
    }
}

#[test]
fn o_afastamento_termina_no_veredito() {
    let m = default_bench();
    up_to_the_vote(&m);
    assert!(m.klero.suspended(&m.accused));
    assert!(m.c.suspended(&m.accused));
    m.tessera
        .put_result(&id(&m.env), &soroban_sdk::vec![&m.env, 0, 3]);
    assert!(!m.c.decide().upheld);
    assert!(m.klero.suspended(&m.accused), "o kleroterion so ve o prazo");
    assert!(!m.c.suspended(&m.accused), "o processo ve o veredito");
}
