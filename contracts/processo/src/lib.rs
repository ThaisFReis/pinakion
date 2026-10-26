#![no_std]
//! Proceeding: the case around the draw, from complaint to verdict (T-048).
//!
//! `kleroterion` decides **who sits**. This contract runs the rest, and
//! decides nothing that a rule fixed before the case does not decide:
//!
//! 1. `open` — with the roster already closed to judge someone, the clerk
//!    files the complaint. The person judged, the severity level and `k` come
//!    from `kleroterion`, not from the caller.
//! 2. `attend` — each seated juror confirms attendance **from the seat
//!    account**, without saying who they are. The contract only asks
//!    `kleroterion` whether that account sat. With the quorum the session
//!    opens; without it, once the deadline passes, the case is `adjourn`ed.
//! 3. `file` — the record: the clerk's evidence and minutes, the defence of
//!    the person judged (suspended, but never silenced), minutes from
//!    attending jurors. Each record is the SHA-256 of a document that stays
//!    off the ledger; whoever holds the document checks it.
//! 4. `bind_vote` — the vote is **Tessera's**, the same author's earlier
//!    secret-ballot module, used unchanged. The contract accepts a Tessera
//!    vote only if its voter list is exactly the attending jurors: it
//!    recomputes the Merkle root under Tessera's rule and compares it with
//!    the one Tessera stored.
//! 5. `decide` — reads the score Tessera published and applies the level's
//!    rule: a majority of the panel, or two thirds from the level fixed in
//!    the constructor.
//!
//! Interim suspension ends at the verdict: `suspended` is `kleroterion`'s
//! while the case runs, and false once it is decided.

use soroban_sdk::{
    contract, contractclient, contracterror, contractevent, contractimpl, contracttype,
    panic_with_error, xdr::ToXdr, Address, Bytes, BytesN, Env, Vec, U256,
};

/// Domain separation for Tessera's voter tree (`tessera/core/src/merkle.rs` at
/// commit 356c22f): leaf, node, empty. The tag bytes and the empty-leaf string
/// are Tessera's, so they keep its spelling.
const DOM_LEAF: u8 = 0x00;
const DOM_NODE: u8 = 0x01;
const DOM_EMPTY: u8 = 0x02;
// check-english: foreign-interface begin (Tessera 356c22f)
const EMPTY: &[u8] = b"TESSERA-V1-MERKLE-VAZIO";
// check-english: foreign-interface end

// ---- what is read from kleroterion -----------------------------------------

/// Mirror of `kleroterion::Closing`: same fields, same XDR.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Closing {
    pub ledger: u32,
    pub time: u64,
    pub round: u64,
    pub k: u32,
    pub reveal_at: u32,
    pub root: U256,
    pub n: u32,
    pub severity: u32,
    pub accused: Address,
    pub suspended_until: u32,
}

/// Mirror of `kleroterion::Panel`.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Panel {
    pub seats: Vec<U256>,
    pub complete: bool,
    pub ledger: u32,
}

#[contractclient(name = "KleroterionClient")]
pub trait Kleroterion {
    fn closing(env: Env) -> Option<Closing>;
    fn panel(env: Env) -> Option<Panel>;
    fn seated(env: Env, seat: Address) -> bool;
    fn suspended(env: Env, address: Address) -> bool;
}

// ---- what is read from Tessera ---------------------------------------------

/// Mirror of `tessera::tipos::Pergunta` (commit 356c22f). Soroban matches
/// struct field names on the wire, so these keep Tessera's spelling: the
/// contract is deployed unchanged and renaming them would break the call.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TesseraQuestion {
    // check-english: foreign-interface begin (Tessera 356c22f)
    pub opcoes: u32,
    pub confidencial: bool,
    // check-english: foreign-interface end
}

/// Mirror of `tessera::tipos::Proposta` (commit 356c22f).
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TesseraProposal {
    // check-english: foreign-interface begin (Tessera 356c22f)
    pub perguntas: Vec<TesseraQuestion>,
    pub raiz_aptos: BytesN<32>,
    pub mesa: Vec<Address>,
    pub limiar: u32,
    pub abre_em: u32,
    pub fecha_em: u32,
    pub rodada: u64,
    pub rodada_abertura: u64,
    pub anel: bool,
    pub secoes: u32,
    pub limite_secao: u32,
    // check-english: foreign-interface end
}

#[contractclient(name = "TesseraClient")]
pub trait Tessera {
    // check-english: foreign-interface begin (Tessera 356c22f)
    fn proposta(env: Env, id: BytesN<32>) -> Option<TesseraProposal>;
    fn resultado(env: Env, id: BytesN<32>) -> Option<Vec<u32>>;
    // check-english: foreign-interface end
}

// ---- the proceeding's own types --------------------------------------------

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    /// The case has already been opened.
    AlreadyOpen = 1,
    /// `kleroterion`'s roster has not closed yet: there is nobody to judge.
    RosterOpen = 2,
    /// `k` below the minimum number of votes the secret ballot requires.
    PanelTooSmallForSecrecy = 3,
    /// The case is not in the phase this call asks for.
    OutOfPhase = 4,
    /// The panel is not out yet.
    NoPanel = 5,
    /// This account did not sit on the panel.
    DidNotSit = 6,
    /// This account has already attended.
    AlreadyAttended = 7,
    /// The quorum deadline has passed.
    QuorumDeadlinePassed = 8,
    /// The quorum deadline has not passed yet.
    QuorumDeadlineOpen = 9,
    /// The caller has no role for this record.
    NoRole = 10,
    /// Tessera does not know this vote.
    NoVote = 11,
    /// The ballot is not one confidential two-option question.
    WrongBallot = 12,
    /// The vote's voter list is not the attending jurors.
    VotersAreNotAttendees = 13,
    /// A member of the vote's bench is not an attending juror.
    BenchOutsider = 14,
    /// The vote opened before the session was installed.
    VoteBeforeSession = 15,
    /// Tessera has not published the result yet.
    NoResult = 16,
}

/// Where the case stands.
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Phase {
    /// Complaint filed; the panel forms and the jurors attend.
    Open = 0,
    /// Quorum reached: the session is installed.
    Session = 1,
    /// Tessera's vote is bound to the case.
    Voting = 2,
    /// The verdict is out.
    Decided = 3,
    /// The quorum deadline passed without a quorum.
    Adjourned = 4,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Kind {
    Complaint = 0,
    Evidence = 1,
    Defence = 2,
    Minutes = 3,
}

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Role {
    Clerk = 0,
    Accused = 1,
    Juror = 2,
}

/// The case: who is judged, at which level, how many seats and the quorum.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Case {
    pub accused: Address,
    pub severity: u32,
    pub k: u32,
    pub quorum: u32,
    pub required: u32,
    pub opened_at: u32,
}

/// One entry in the record: a document's hash, who filed it and when.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Record {
    pub seq: u32,
    pub kind: Kind,
    pub role: Role,
    pub author: Address,
    pub hash: BytesN<32>,
    pub ledger: u32,
}

/// The verdict: Tessera's score and the level's rule.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decision {
    pub yes: u32,
    pub no: u32,
    pub required: u32,
    pub upheld: bool,
    pub ledger: u32,
}

#[contracttype]
enum Key {
    Clerk,
    Kleroterion,
    Tessera,
    QuorumDeadline,
    MinimumVotes,
    TwoThirdsFrom,
    Case,
    Phase,
    Attendees,
    Records,
    SessionAt,
    Vote,
    Decision,
}

// ---- events ----------------------------------------------------------------

/// The rule, published before any case.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Instituted {
    pub kleroterion: Address,
    pub tessera: Address,
    pub quorum_deadline: u32,
    pub minimum_votes: u32,
    pub two_thirds_from: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseOpened {
    #[topic]
    pub accused: Address,
    pub severity: u32,
    pub k: u32,
    pub quorum: u32,
    pub required: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Filed {
    pub seq: u32,
    pub kind: Kind,
    pub role: Role,
    pub author: Address,
    pub hash: BytesN<32>,
}

/// A seat account attended. It carries the account, not the name: the name
/// appears only when the juror reveals in `kleroterion`.
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attended {
    pub seat: Address,
    pub attendees: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionOpened {
    pub attendees: u32,
    pub quorum: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Adjourned {
    pub attendees: u32,
    pub quorum: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VoteBound {
    pub proposal: BytesN<32>,
    pub voter_root: BytesN<32>,
    pub voters: u32,
    pub closes_at: u32,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Decided {
    pub yes: u32,
    pub no: u32,
    pub required: u32,
    pub upheld: bool,
    pub severity: u32,
}

#[contract]
pub struct Proceeding;

#[contractimpl]
impl Proceeding {
    /// `clerk` opens the case, files evidence and minutes and binds the vote;
    /// `kleroterion` is this case's roster; `tessera` is the voting contract;
    /// `quorum_deadline` is how many ledgers after the panel the jurors have
    /// to attend; `minimum_votes` is the vote's secrecy floor (Tessera's is 5:
    /// below that it publishes no score); from level `two_thirds_from` the
    /// decision requires two thirds of the panel.
    pub fn __constructor(
        env: Env,
        clerk: Address,
        kleroterion: Address,
        tessera: Address,
        quorum_deadline: u32,
        minimum_votes: u32,
        two_thirds_from: u32,
    ) {
        let s = env.storage().instance();
        s.set(&Key::Clerk, &clerk);
        s.set(&Key::Kleroterion, &kleroterion);
        s.set(&Key::Tessera, &tessera);
        s.set(&Key::QuorumDeadline, &quorum_deadline);
        s.set(&Key::MinimumVotes, &minimum_votes);
        s.set(&Key::TwoThirdsFrom, &two_thirds_from);
        s.set(&Key::Attendees, &Vec::<Address>::new(&env));
        s.set(&Key::Records, &Vec::<Record>::new(&env));
        Instituted {
            kleroterion,
            tessera,
            quorum_deadline,
            minimum_votes,
            two_thirds_from,
        }
        .publish(&env);
    }

    /// Opens the case with the complaint, once the roster has closed. The
    /// person judged, the level and `k` are `kleroterion`'s. The quorum is a
    /// majority of the panel, and never less than `minimum_votes`; the
    /// decision requires a majority, or two thirds from `two_thirds_from`.
    pub fn open(env: Env, complaint: BytesN<32>) -> Case {
        let clerk = Self::clerk(&env);
        clerk.require_auth();
        if Self::phase(env.clone()).is_some() {
            panic_with_error!(&env, Error::AlreadyOpen);
        }
        let f = match Self::klero(&env).closing() {
            Some(f) => f,
            None => panic_with_error!(&env, Error::RosterOpen),
        };
        let i = env.storage().instance();
        let minimum: u32 = i.get(&Key::MinimumVotes).unwrap();
        if f.k < minimum {
            panic_with_error!(&env, Error::PanelTooSmallForSecrecy);
        }
        let from: u32 = i.get(&Key::TwoThirdsFrom).unwrap();
        let case = Case {
            accused: f.accused.clone(),
            severity: f.severity,
            k: f.k,
            quorum: (f.k / 2 + 1).max(minimum),
            required: if f.severity >= from {
                (2 * f.k).div_ceil(3)
            } else {
                f.k / 2 + 1
            },
            opened_at: env.ledger().sequence(),
        };
        i.set(&Key::Case, &case);
        i.set(&Key::Phase, &Phase::Open);
        CaseOpened {
            accused: case.accused.clone(),
            severity: case.severity,
            k: case.k,
            quorum: case.quorum,
            required: case.required,
        }
        .publish(&env);
        Self::note(&env, Kind::Complaint, Role::Clerk, clerk, complaint);
        Self::keep_alive(&env);
        case
    }

    /// A juror confirms attendance from the seat account. Only an account that
    /// sat, only once, and before the quorum deadline passes while the session
    /// is not installed. With the quorum the session opens; after that,
    /// latecomers still count, until the vote is bound.
    pub fn attend(env: Env, seat: Address) -> u32 {
        seat.require_auth();
        let phase = Self::phase_or_fail(&env);
        if phase != Phase::Open && phase != Phase::Session {
            panic_with_error!(&env, Error::OutOfPhase);
        }
        let k = Self::klero(&env);
        let panel = match k.panel() {
            Some(p) => p,
            None => panic_with_error!(&env, Error::NoPanel),
        };
        if phase == Phase::Open && env.ledger().sequence() > Self::quorum_deadline(&env, &panel) {
            panic_with_error!(&env, Error::QuorumDeadlinePassed);
        }
        if !k.seated(&seat) {
            panic_with_error!(&env, Error::DidNotSit);
        }
        let i = env.storage().instance();
        let mut attendees = Self::attendees(env.clone());
        if attendees.contains(&seat) {
            panic_with_error!(&env, Error::AlreadyAttended);
        }
        attendees.push_back(seat.clone());
        i.set(&Key::Attendees, &attendees);
        let n = attendees.len();
        Attended { seat, attendees: n }.publish(&env);
        let case = Self::case(env.clone()).unwrap();
        if phase == Phase::Open && n >= case.quorum {
            i.set(&Key::Phase, &Phase::Session);
            i.set(&Key::SessionAt, &env.ledger().sequence());
            SessionOpened {
                attendees: n,
                quorum: case.quorum,
            }
            .publish(&env);
        }
        Self::keep_alive(&env);
        n
    }

    /// Once the deadline passes without a quorum, the case is adjourned.
    /// Anyone may call it. A new draw is a new roster, and stays outside this
    /// contract.
    pub fn adjourn(env: Env) {
        if Self::phase_or_fail(&env) != Phase::Open {
            panic_with_error!(&env, Error::OutOfPhase);
        }
        let panel = match Self::klero(&env).panel() {
            Some(p) => p,
            None => panic_with_error!(&env, Error::NoPanel),
        };
        if env.ledger().sequence() <= Self::quorum_deadline(&env, &panel) {
            panic_with_error!(&env, Error::QuorumDeadlineOpen);
        }
        env.storage().instance().set(&Key::Phase, &Phase::Adjourned);
        Adjourned {
            attendees: Self::attendees(env.clone()).len(),
            quorum: Self::case(env.clone()).unwrap().quorum,
        }
        .publish(&env);
    }

    /// Puts a document in the record, by its hash. Who may do what:
    /// - the clerk: evidence, until the vote; minutes, at any time;
    /// - the person judged: the defence, until the verdict — suspended, never
    ///   silenced;
    /// - an attending juror, from the seat account: minutes, until the verdict.
    pub fn file(env: Env, author: Address, kind: Kind, hash: BytesN<32>) -> u32 {
        author.require_auth();
        let phase = Self::phase_or_fail(&env);
        let before_the_verdict = phase != Phase::Decided && phase != Phase::Adjourned;
        let before_the_vote = phase == Phase::Open || phase == Phase::Session;
        let case = Self::case(env.clone()).unwrap();
        let role = if author == Self::clerk(&env) {
            Role::Clerk
        } else if author == case.accused {
            Role::Accused
        } else if Self::attendees(env.clone()).contains(&author) {
            Role::Juror
        } else {
            panic_with_error!(&env, Error::NoRole)
        };
        let allowed = match (role, kind) {
            (Role::Clerk, Kind::Evidence) => before_the_vote,
            (Role::Clerk, Kind::Minutes) => true,
            (Role::Accused, Kind::Defence) => before_the_verdict,
            (Role::Juror, Kind::Minutes) => before_the_verdict,
            _ => false,
        };
        if !allowed {
            panic_with_error!(&env, Error::NoRole);
        }
        let seq = Self::note(&env, kind, role, author, hash);
        Self::keep_alive(&env);
        seq
    }

    /// Binds Tessera's vote to the case. It accepts only a vote of one
    /// confidential question with two options (yes, no), opened after the
    /// session was installed, whose voter list is **exactly** the attending
    /// jurors in the order they attended, and whose bench is made of
    /// attendees.
    pub fn bind_vote(env: Env, proposal: BytesN<32>) {
        Self::clerk(&env).require_auth();
        if Self::phase_or_fail(&env) != Phase::Session {
            panic_with_error!(&env, Error::OutOfPhase);
        }
        let p = match Self::tessera(&env).proposta(&proposal) {
            Some(p) => p,
            None => panic_with_error!(&env, Error::NoVote),
        };
        let one_confidential_of_two = p.perguntas.len() == 1
            && p.perguntas.get(0).unwrap()
                == TesseraQuestion {
                    opcoes: 2,
                    confidencial: true,
                };
        if !one_confidential_of_two {
            panic_with_error!(&env, Error::WrongBallot);
        }
        let attendees = Self::attendees(env.clone());
        if p.raiz_aptos != Self::voter_root(env.clone(), attendees.clone()) {
            panic_with_error!(&env, Error::VotersAreNotAttendees);
        }
        if p.mesa.iter().any(|m| !attendees.contains(&m)) {
            panic_with_error!(&env, Error::BenchOutsider);
        }
        let session: u32 = env.storage().instance().get(&Key::SessionAt).unwrap();
        if p.abre_em < session {
            panic_with_error!(&env, Error::VoteBeforeSession);
        }
        let i = env.storage().instance();
        i.set(&Key::Vote, &proposal);
        i.set(&Key::Phase, &Phase::Voting);
        VoteBound {
            proposal,
            voter_root: p.raiz_aptos,
            voters: attendees.len(),
            closes_at: p.fecha_em,
        }
        .publish(&env);
        Self::keep_alive(&env);
    }

    /// Reads Tessera's score and applies the level's rule: upheld if `yes`
    /// reaches the required votes, counted over the whole panel — absence is
    /// not a vote in favour. Anyone may call it.
    pub fn decide(env: Env) -> Decision {
        if Self::phase_or_fail(&env) != Phase::Voting {
            panic_with_error!(&env, Error::OutOfPhase);
        }
        let proposal: BytesN<32> = env.storage().instance().get(&Key::Vote).unwrap();
        let score = match Self::tessera(&env).resultado(&proposal) {
            Some(r) if r.len() == 2 => r,
            _ => panic_with_error!(&env, Error::NoResult),
        };
        let case = Self::case(env.clone()).unwrap();
        let yes = score.get(0).unwrap();
        let d = Decision {
            yes,
            no: score.get(1).unwrap(),
            required: case.required,
            upheld: yes >= case.required,
            ledger: env.ledger().sequence(),
        };
        let i = env.storage().instance();
        i.set(&Key::Decision, &d);
        i.set(&Key::Phase, &Phase::Decided);
        Decided {
            yes: d.yes,
            no: d.no,
            required: d.required,
            upheld: d.upheld,
            severity: case.severity,
        }
        .publish(&env);
        Self::keep_alive(&env);
        d
    }

    /// Is the person suspended? It is `kleroterion`'s suspension while the
    /// case runs; it ends at the verdict or at the adjournment, and what
    /// follows — the sanction — belongs to whoever calls the module.
    pub fn suspended(env: Env, address: Address) -> bool {
        match Self::phase(env.clone()) {
            Some(Phase::Decided) | Some(Phase::Adjourned) => false,
            _ => Self::klero(&env).suspended(&address),
        }
    }

    /// The Merkle root Tessera computes for this voter list, all of weight 1:
    /// leaf `SHA-256(0x00 ‖ xdr(address) ‖ weight_be)`, node
    /// `SHA-256(0x01 ‖ left ‖ right)`, padded to a power of two with the empty
    /// leaf `SHA-256(0x02 ‖ "TESSERA-V1-MERKLE-VAZIO")`.
    pub fn voter_root(env: Env, voters: Vec<Address>) -> BytesN<32> {
        let mut level: Vec<BytesN<32>> = Vec::new(&env);
        for a in voters.iter() {
            let mut b = Bytes::from_array(&env, &[DOM_LEAF]);
            b.append(&a.to_xdr(&env));
            b.extend_from_array(&1u32.to_be_bytes());
            level.push_back(env.crypto().sha256(&b).to_bytes());
        }
        if level.is_empty() {
            return Self::empty(&env);
        }
        let target = level.len().next_power_of_two();
        while level.len() < target {
            level.push_back(Self::empty(&env));
        }
        while level.len() > 1 {
            let mut above = Vec::new(&env);
            for i in (0..level.len()).step_by(2) {
                let mut b = Bytes::from_array(&env, &[DOM_NODE]);
                b.extend_from_array(&level.get(i).unwrap().to_array());
                b.extend_from_array(&level.get(i + 1).unwrap().to_array());
                above.push_back(env.crypto().sha256(&b).to_bytes());
            }
            level = above;
        }
        level.get(0).unwrap()
    }

    pub fn case(env: Env) -> Option<Case> {
        env.storage().instance().get(&Key::Case)
    }

    pub fn phase(env: Env) -> Option<Phase> {
        env.storage().instance().get(&Key::Phase)
    }

    /// The seat accounts that attended, in the order they attended.
    pub fn attendees(env: Env) -> Vec<Address> {
        env.storage().instance().get(&Key::Attendees).unwrap()
    }

    pub fn records(env: Env) -> Vec<Record> {
        env.storage().instance().get(&Key::Records).unwrap()
    }

    pub fn vote(env: Env) -> Option<BytesN<32>> {
        env.storage().instance().get(&Key::Vote)
    }

    pub fn decision(env: Env) -> Option<Decision> {
        env.storage().instance().get(&Key::Decision)
    }

    // ---- internal ------------------------------------------------------

    fn note(env: &Env, kind: Kind, role: Role, author: Address, hash: BytesN<32>) -> u32 {
        let i = env.storage().instance();
        let mut rs = Self::records(env.clone());
        let seq = rs.len();
        rs.push_back(Record {
            seq,
            kind,
            role,
            author: author.clone(),
            hash: hash.clone(),
            ledger: env.ledger().sequence(),
        });
        i.set(&Key::Records, &rs);
        Filed {
            seq,
            kind,
            role,
            author,
            hash,
        }
        .publish(env);
        seq
    }

    /// The record lives in the instance: a case has dozens of entries, not
    /// thousands, and the instance holds 64 KiB. Every write takes the TTL to
    /// the network's ceiling, so the record does not archive mid-case.
    fn keep_alive(env: &Env) {
        let ceiling = env.storage().max_ttl();
        env.storage().instance().extend_ttl(ceiling, ceiling);
    }

    fn quorum_deadline(env: &Env, panel: &Panel) -> u32 {
        let deadline: u32 = env.storage().instance().get(&Key::QuorumDeadline).unwrap();
        panel.ledger.saturating_add(deadline)
    }

    fn phase_or_fail(env: &Env) -> Phase {
        match Self::phase(env.clone()) {
            Some(f) => f,
            None => panic_with_error!(env, Error::OutOfPhase),
        }
    }

    fn empty(env: &Env) -> BytesN<32> {
        let mut b = Bytes::from_array(env, &[DOM_EMPTY]);
        b.extend_from_slice(EMPTY);
        env.crypto().sha256(&b).to_bytes()
    }

    fn clerk(env: &Env) -> Address {
        env.storage().instance().get(&Key::Clerk).unwrap()
    }

    fn klero(env: &Env) -> KleroterionClient<'_> {
        let a: Address = env.storage().instance().get(&Key::Kleroterion).unwrap();
        KleroterionClient::new(env, &a)
    }

    fn tessera(env: &Env) -> TesseraClient<'_> {
        let a: Address = env.storage().instance().get(&Key::Tessera).unwrap();
        TesseraClient::new(env, &a)
    }
}

mod test;
