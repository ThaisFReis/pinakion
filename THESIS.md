# Pinakion

**Sealed jury selection for governance on Stellar.**

> In classical Athens the *pinakion* was a small bronze ticket, engraved with a citizen's name. On the morning of a trial it went into the *kleroterion*, a stone slab with slots and a tube, and the balls that fell decided who judged that day. Then came a second lot. Each juror drew an acorn lettered for one court, and walked into that court and no other (*Ath. Pol.* 63–65).
>
> Aristotle gives the purpose plainly. A juror goes to the court the lot gives him, not the one he chooses. Nobody can gather the jurors he wants into one court. The text says nothing about secrecy. Athens is cited here for **no choosing**, and the sealing is ours.

---

## The guarantee

**Nobody chooses who judges you. Nobody knows who until they have sat and judged. Afterwards, anyone recomputes the panel from the ledger, among those who revealed.**

Everything below follows from those three sentences. A design decision that does not follow from them is wrong.

- **The first sentence is "no choosing".** The roster is frozen before the case, and the drand round is fixed at the same moment. The draw uses that round once.
- **The second is "sealed".** A juror proves a seat in zero knowledge from a fresh account. On the ledger, a seat is a number `y`, never a name, until the verdict.
- **The third is "public afterwards".** Each juror reveals their secret. The panel can then be recomputed by anyone, among the jurors who revealed. Silence is on the ledger too: a juror who did not reveal is listed as such.

*Ninguém escolhe quem te julga. Ninguém sabe quem até que se sentem e julguem. Depois, qualquer um recalcula o painel a partir do ledger, entre quem revelou.*

---

## Who it is for

**An open-source project with a code of conduct that has to judge one of its own.**

**Rust, 2021.** The entire moderation team resigned. They said the Core Team had made itself answerable to no one but itself, which left them unable to enforce the code of conduct. Their own proposed fix was a rule about appointment: **the replacement moderators should be chosen by people who are not on the Core Team.** That request is, almost word for word, this module. The people who will judge are fixed by a rule set before the case, by lot, and nobody inside the dispute picks them.

**Node.js, 2017.** A member of the Technical Steering Committee was accused of violating the code of conduct. **The TSC voted on removing one of its own.** Four members resigned and the project forked.

Both reached for the same missing mechanism and had to settle for a social rule. In both, the only exits were resignation or a fork.

*It also serves* any governance that decides about a person rather than a parameter, and where the affected person never consented to the forum: a grant program revoking a grant, a protocol judging a slashed validator, a marketplace delisting a seller who disputes it. It is not for commercial arbitration, where choosing your arbitrator is a right rather than a defect.

---

## The ticket

The *pinakion* is literally a ticket here: **one unit of a classic Stellar asset**.

- The issuer sets `AUTH_REQUIRED`, `AUTH_REVOCABLE` and `AUTH_IMMUTABLE`.
- It hands the asset contract's admin to `kleroterion`.
- It then sets its own master weight to 0.

From that ledger on, nobody but the contract issues or authorizes a ticket, and the contract refuses any enrolment or impediment after `lock`.

| In Athens | Here |
|---|---|
| the bronze ticket with your name | one PINAKION on your trustline, created by the contract (SAC `trust`, Protocol 26), and with it your commitment `c = Poseidon(s)` |
| the slab closed before the lot | `lock`: the roster frozen, the drand round and `k` fixed from the ledger clock |
| a citizen struck off before the lot | `impede`, before `lock` only: the ticket stays and the leaf becomes 0 |
| the first lot | `seal`: the drand round verified on-chain, then the seed and the threshold fixed |
| the acorn that sends you to one court | `claim`: from a fresh account, a proof that your `y` is under the threshold. The ledger sees `y`, not you |
| sitting with the acorn and the staff, in public | `reveal`, after the verdict: `s` shown, `y` checked, the seat tied to the name |
| anyone counting the tickets | any Horizon: `GET /accounts?asset=PINAKION:issuer` |

---

## Why Stellar

These are specific reasons, each one measured or read from source. "Because blockchain" is not one of them.

**The beacon verification is affordable on-chain.** A drand round is a BLS12-381 signature, and Stellar's host functions verify it (CAP-59).
- It costs **26,719,483 CPU instructions**, measured in the Soroban test host.
- It ran on testnet in [transaction `c74150ee…`](https://stellar.expert/explorer/testnet/tx/c74150eeb7b73d0a196d20b44ed851a140812b3ac67e579b670b87ee2c3ffeea).
- The open draw that `beacon-verifier` keeps costs 2,274,884 instructions for a roster of a thousand.

**The roster can live in the classic ledger, frozen by the ledger's own rules.**
- A locked issuer cannot sign.
- An `AUTH_REQUIRED` asset cannot be held without the issuer's say-so, and the issuer is now the contract.
- These are protocol rules, not contract code, and any Horizon client reads the result without trusting us.

**The randomness the chain offers is not good enough, and the chain says so.** The `soroban-sdk` documents that its PRNG seed is under validator control. So the entropy comes from drand, and the chain's job is to *verify* it.

**A seat can be proven without being named, and that only became possible this year.**
- Poseidon and BN254 became host functions in Protocol 25 (CAP-74 and CAP-75, January 2026).
- A Groth16 proof of seat over a Poseidon Merkle tree is one ordinary transaction: about 32 M instructions declared on testnet, **8 %** of the 400 M per-transaction ceiling.
- The contract builds the tree itself, one path per enrolment.
- This is the only piece of the design that was impossible a year ago, and it is the piece that makes the second sentence of the guarantee true.

---

## What it is, and is not

**It is** a module that an existing governance calls to run a draw. It keeps a roster, fixes a beacon round, lets the jurors prove their seats, publishes the panel, and emits a record anyone can recheck.

**It is not** a DAO, a court, a ballot or a dispute marketplace:
- it does not decide who may be on the roster;
- it does not decide what the panel rules, or what follows from the ruling;
- it counts no votes;
- it holds no deliberation;
- it models no case.

**It is not Kleros.** Kleros answers *who will judge?* with a market of anonymous jurors selected by stake. Pinakion answers it with **the institution's own roster, fixed before the case**.

**It is not Tessera.** Tessera is the author's earlier module. It hides *how one votes*, for good; Pinakion hides *who sits*, until they have judged. The panel's seat accounts can be handed to Tessera as its voter list, and the two modules compose without either one doing the other's job.

---

## What is proven today

**On testnet, end to end, 2026-10-26** (`docs/evidencia-testnet-bilhete.md`):
- 20 jurors enrolled with commitments, one was impeded, and the roster locked with N = 19 and k = 3;
- a live drand round was verified on-chain;
- six seats were proven in zero knowledge from six fresh accounts, and a resent claim was refused;
- one juror recused openly;
- a complete panel came out, and 18 jurors revealed.

**Then the panel was recomputed from the ledger's events and drand alone**: `CONFERE`, 77 checks, 0 failures.

**The arithmetic is held to circomlib.**
- The contract's Poseidon equals circomlib's on 22 vectors.
- Its commitment tree equals circomlib's on four rosters.
- Three implementations agree: the contract, Python and JavaScript.

**Verifying a drand round on Soroban is not new.** `AshFrancis/splicers` and `NibrasD/Stellar-VRF` did it before us with the same host functions, and we say so.

## What is not

- **The setup is single-party.** Whoever ran `circuits/setup.sh` could have kept its entropy and forged seats. The drand beacon at the end only shows that the last step was not chosen.
- **Unlinkability rests on the seat account.** It is the claim transaction's source, so it must be funded with no trail back to the juror. Here every account came from friendbot; in real use that funding needs a relayer, and none is built.
- **Losing `s` loses the seat.** Nothing recovers a secret.
- **The panel is recomputable only among the jurors who revealed.** A juror who never reveals stays unaccounted for, by name, on the ledger.
- **The seal ends at the ledger.** Deliberation off-chain — a chat, a call — breaks it the moment the jurors meet.
- **Nothing has been audited, nothing has run on mainnet, and the run was one machine playing every juror.**
- Below a roster of about sixteen the draw guarantees little (`docs/limite-inferior-do-rol.md`).

---

*This file is the root. When the thesis changes, it changes here first, then in `SPEC.md`, then in `DESIGN.md`, then on the landing page and in the pitch.*
