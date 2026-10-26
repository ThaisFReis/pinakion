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

These are specific reasons, each one measured or read from source. "Because blockchain" is not one of them. They are in order of what only this project does.

**The roster is a classic asset whose issuer locks itself out.**
- The flags are from 2015: `AUTH_REQUIRED`, `AUTH_REVOCABLE`, `AUTH_IMMUTABLE`, and the issuer at master weight 0.
- The ecosystem reads these flags as an issuer's power over its holders. Here the issuer uses them to give that power away, and from then on nobody can issue or authorize a ticket except the contract.
- An `AUTH_REQUIRED` asset cannot be held without the issuer's say-so, and the issuer is now the contract.
- These are protocol rules, not contract code, and any Horizon client reads the roster without trusting us.
- No other use of these flags as a jury roster was visible in three searches (`docs/SOURCES.md`, *Checked and not found*). The indexes lag.

**The beacon verification is affordable on-chain.** A drand round is a BLS12-381 signature, and Stellar's host functions verify it (CAP-59).
- It costs **26,719,483 CPU instructions**, measured in the Soroban test host.
- It ran on testnet in [transaction `c74150ee…`](https://stellar.expert/explorer/testnet/tx/c74150eeb7b73d0a196d20b44ed851a140812b3ac67e579b670b87ee2c3ffeea).
- The open draw that `beacon-verifier` keeps costs 2,274,884 instructions for a roster of a thousand.
- Verifying drand on Soroban was done before us (`AshFrancis/splicers`, `NibrasD/Stellar-VRF`). We claim the use, not the verification.

**The randomness the chain offers is not good enough, and the chain says so.** The `soroban-sdk` documents that its PRNG seed is under validator control. So the entropy comes from drand, and the chain's job is to *verify* it.

**A seat can be proven without being named: this is what Protocol 25 unlocked.**
- Poseidon and BN254 became host functions in Protocol 25 (CAP-74 and CAP-75, January 2026).
- A Groth16 proof of seat over a Poseidon Merkle tree is one ordinary transaction: about 32 M instructions declared on testnet, **8 %** of the 400 M per-transaction ceiling.
- The contract builds the tree itself, one path per enrolment.

**Would this have existed a year ago?** The open draw would have, whole. Everything it needs was on mainnet in October 2025: the drand pairing, the PRNG reseeded by the beacon, Fisher-Yates over the roster, the asset flags and the locked issuer. The only loss would be CAP-73: each juror would send their own `changeTrust`, one transaction more and no guarantee lost. That is the fallback the plan wrote down as its kill path (T-033 alone). **The seal would not have existed.** A Groth16 path over BLS12-381 was open in theory (CAP-59), but circom targets BN254, and Poseidon would have had to run as Wasm inside the contract, at a cost nobody here measured. What is known is today's figure, about 32 M instructions. So the sealed seat is the part of the design that Protocol 25 made possible, and the second sentence of the guarantee rests on it.

---

## What it is, and is not

**It is** a module that an existing governance calls to run a draw. It keeps a roster, fixes a beacon round, lets the jurors prove their seats, publishes the panel, and emits a record anyone can recheck.

**Around the draw, it carries the case** (`processo`, since T-048):
- the complaint and the record, each document as its SHA-256 on the ledger, filed by the role that may file it — the clerk, the person judged (suspended, never silenced), a juror from a seat account;
- attendance and quorum, confirmed from the seat accounts, so the session opens without anyone learning who sits;
- the binding of the vote: the case accepts only a Tessera vote whose voter list is exactly the jurors who attended, recomputed on-chain under Tessera's own rule;
- the verdict: Tessera's published score under the level's rule, a majority of the whole panel or two thirds.

**It is not** a DAO, a court, a ballot or a dispute marketplace:
- it does not decide who may be on the roster;
- it does not decide what follows from the ruling: the sanction is the governance's;
- **it counts no votes.** The vote is **Tessera's**, the author's earlier secret-ballot module, used as it is;
- it holds no deliberation: deliberation happens off-chain, and only its minutes are hashed into the record.

**It is not Kleros.** Kleros answers *who will judge?* with a market of anonymous jurors selected by stake. Pinakion answers it with **the institution's own roster, fixed before the case**.

**It is not Tessera, and it uses Tessera.** Tessera is the author's earlier module, built for the Find Your Way hackathon. It hides *how one votes*, for good; Pinakion hides *who sits*, until they have judged. In the case cycle the vote runs on Tessera unchanged: its contract at commit `356c22f`, deployed on testnet, and its CLI at the same commit with a one-line client fix (`scripts/tessera-cli.patch`). What is new here is the binding — the case accepts a Tessera vote only when its voter list is exactly the attendees — and the verdict rule on top of Tessera's score. Neither module does the other's job.

---

## What is proven today

**On testnet, end to end, 2026-10-26** (`docs/evidencia-testnet-bilhete.md`):
- 20 jurors enrolled with commitments, one was impeded, and the roster locked with N = 19 and k = 3;
- a live drand round was verified on-chain;
- six seats were proven in zero knowledge from six fresh accounts, and a resent claim was refused;
- one juror recused openly;
- a complete panel came out, and 18 jurors revealed.

**Then the panel was recomputed from the ledger's events and drand alone**: `CONFERE`, 77 checks, 0 failures.

**Then a whole case, on testnet, the same day** (`docs/evidencia-processo.md`):
- a roster of 40, and a level-3 case with 7 seats;
- the person judged was on the roster, and the lock itself impeded them;
- the complaint, the evidence, the defence and the minutes were filed by their hashes;
- 6 of the 7 seated jurors attended from their seat accounts, against a quorum of 5;
- the vote was bound to exactly those 6, recomputed on-chain under Tessera's rule;
- the secret vote came out 5 to 1 on Tessera, upheld with 4 required;
- the jurors revealed afterwards.

**It was then checked from the record alone**: `scripts/verificar_processo.py` in a terminal, and `web/processo.html` in a browser.

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
- **The person judged is kept off their own panel in the code, not yet on testnet.** Since T-045 `lock` names them and impedes them if enrolled; the 2026-10-26 run predates that, and there the clerk had to do it by hand. The interim suspension only publishes a window (`suspended`); the host governance removes the real powers, or nothing happens.
- **Nothing has been audited, nothing has run on mainnet, and the run was one machine playing every juror.**
- **The vote's secrecy and tally are Tessera's**, and so are its limits: a floor of five confidential votes, and a committee that tallies. This project checks the binding and the rule, not the tally.
- **The case's records are public in the demo.** Confidential records with justified access are designed, not built.
- Below a roster of about sixteen the draw guarantees little (`docs/limite-inferior-do-rol.md`).

---

*This file is the root. When the thesis changes, it changes here first, then in `SPEC.md`, then in `DESIGN.md`, then on the landing page and in the pitch.*
