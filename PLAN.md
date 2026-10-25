# PLAN

Where the work stands, and in what order it happens next.

**What this file is for.** `docs/SPEC.md` §9 holds each task's acceptance criteria — what *done* means. This file holds the **order and the state** — what is finished, what is in flight, what is waiting on whom. Where the two disagree about criteria, the spec wins and this file is stale; fix it here and say so.

Last updated **2026-10-25**, after T-024 merged, from `git log`, `gh issue list` and `gh pr list`. Every line below is checkable with those three commands.

---

## The clock

From the *Speedrun SHIP* deck (Wlad Mendes, Stellar, 25 Oct 2026):

| | |
|---|---|
| **25 Oct** | build |
| **26 Oct** | ship — stop new features, record the video, submit one hour early |

Today is **25 Oct**. Everything in Phase 3 below is submission-critical; everything in Phase 4 is after the event.

---

## State of the tree

Everything through T-024 is on `main`. **No open pull requests.** PRs #9 to #19 are merged; #11, #12, #14 and #16 were first merged into their own base branches rather than `main`, which is why #13 and #15 exist — worth remembering next time: the merge bar has to say *into main*.

Two issues remain: [#8](https://github.com/ThaisFReis/pinakion/issues/8), the recording, and [#6](https://github.com/ThaisFReis/pinakion/issues/6), which is Phase 4.

### Gates right now — all seven green on `main`

| # | Command | State |
|---|---|---|
| G1 | `cargo fmt --all --check` | PASS |
| G2 | `cargo test --locked --workspace` | PASS, 12 tests |
| G3 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| G4 | `stellar contract build` | PASS |
| G5 | `npm test` | PASS, 3 vectors |
| G6 | `python3 scripts/conferir_vetores.py` | PASS, 3 vectors + the tie to `test.rs` |
| G7 | `python3 scripts/conferir_numeros.py` | PASS, 18 figures in 6 documents |

---

## Phase 0 — the gate that blocked everything · **done**

`docs/SPEC.md` §0 carries **Approved: 2026-10-25**.

Written after the work it governs, not before. Every task was done on an explicit instruction, so the intent of the rule held — the agent did not choose the direction unasked — but the order `CLAUDE.md` prescribes did not. §11 Q-14 records that instead of tidying it away, and it is a real question for the next milestone: either the gate means something or it should leave the file.

---

## Phase 1 — land what is already proven · **done**

Four testnet transactions already show the thing working. None of that is on `main`.

| Order | Task | Issue | State | Note |
|---|---|---|---|---|
| 1 | T-007 pin the beacon key | [#1](https://github.com/ThaisFReis/pinakion/issues/1) | `review` → merge #9 | the key is a constant; a forged signature from a real adversary keypair is rejected on testnet |
| 2 | T-004 green fmt and clippy | [#3](https://github.com/ThaisFReis/pinakion/issues/3) | `review` → merge #10 | five deprecated aliases renamed, `rustfmt` applied; golden vectors still match, so no behaviour changed |

Done: `main` carries both, and G1–G4 pass there.

---

## Phase 2 — make the repository survive a stranger · **done**

A judge clones it and runs two commands. Today the Python and JavaScript verifiers — the whole basis of the claim *anyone can recompute the panel* — are exercised by **nothing**.

| Order | Task | Issue | State | Why it matters |
|---|---|---|---|---|
| 3 | T-017 civic identity for the public page | [#7](https://github.com/ThaisFReis/pinakion/issues/7) | `review` | done and merged into the stack; the worked example now matches `docs/evidencia-testnet-sorteio.md` |
| 4 | T-003 pin the toolchain | [#2](https://github.com/ThaisFReis/pinakion/issues/2) | `review` | `rust-toolchain.toml` and `.nvmrc`; a clean clone runs all six gates green |
| 5 | T-005 gate the Python verifier | [#4](https://github.com/ThaisFReis/pinakion/issues/4) | `review` | G6 runs it against `tests/vetores-de-ouro.json` and ties those numbers to `test.rs` |
| 6 | T-006 gate the JavaScript verifier | [#5](https://github.com/ThaisFReis/pinakion/issues/5) | `review` | `npm test` runs `web/test.mjs` against the same file; the placeholder is gone |

Done when: G1–G6 all pass, and each of the three implementations is checked against the same golden vectors by a command in §8. **G1–G6 all pass as of 2026-10-25, from a clean clone.** This phase is done.

---

## Phase 3 — the submission · **one thing left**

The deck names four deliverables. Two are written, one is half-built, one has not been started.

| Deliverable | State | Task |
|---|---|---|
| Landing page | built, and the stale-comparison bug the rehearsal found is fixed | T-017, T-024, done |
| MVP spec — the demo path, nothing more | written and proven: `docs/DEMO.md` | T-016 |
| Video demo, 3 min | script written in `docs/DEMO.md`; **the recording is not made** | T-016 |
| Submission data | the name and one line exist; track still unknown (Q-11) | T-009 done, Q-11 open |

| Order | Task | Issue | State |
|---|---|---|---|
| 7 | T-016 write the demo path | [#8](https://github.com/ThaisFReis/pinakion/issues/8) | `review` — `docs/DEMO.md`, rehearsed end to end in 11 s |
| 8 | **record the video** | [#8](https://github.com/ThaisFReis/pinakion/issues/8) | **not started — the only thing left before the submission** |

The video's four parts, from the deck: `0:00` the problem · `0:30` live demo on testnet · `2:00` why Stellar · `2:30` what is next.

The flow runs live on testnet in 11 seconds, measured on 2026-10-25. What is left is the recording itself, which is the human's to make.

**This is the highest-value item with zero work done.** The deck's own line is *judges score demos, not ideas*.

---

## Phase 4 — after the event

Not before 26 Oct. These need contracts that do not exist yet.

| Task | Issue | Blocked on |
|---|---|---|
| T-008 enforce the roster's minimum beacon round (INV-06) | [#6](https://github.com/ThaisFReis/pinakion/issues/6) | there is no roster contract to hold the minimum |
| The case lifecycle, the roster cycle, the recusal rules | — | designed in three documents, **587 lines of design with no implementation** |
| Confidential case files | — | `docs/desenho-sorteio-e-sigilo.md`; threshold decryption per access |

---

## Explicitly not in this plan before 26 Oct

Stated so that nobody mistakes an omission for an oversight:

- **No mainnet.** `CLAUDE.md` non-negotiable 7.
- **No audit.** Nothing here has been reviewed by anyone outside this repository, and the README says so.
- **No new feature.** The deck's ship-day rule, and the reason Phase 4 exists.
- **No roster or case contract.** Building half of one before the video would leave both unfinished.

---

## Open questions that block work

Full list in `docs/SPEC.md` §11. These are the ones with something waiting on them:

| ID | Question | Blocks |
|---|---|---|
| Q-02 | Is the milestone goal in §1 correct as written, and are the non-goals right? Still `[INFERRED]`, and the spec is now approved around it | — |
| Q-06 | **Answered**: rustc 1.97.1 and Node 24.14.1, pinned. The Stellar CLI stays unpinned and §6 says why | — |
| Q-11 | Which track and category is the submission? Genesis eligibility was never answered by the organisers | T-009's last step |
| Q-13 | **Answered** by the civic direction, recorded with T-017. The implementation itself is still for human review | nothing |
| Q-07 | Who publishes the first roster, and under what authority? Unresolved across three design documents | Phase 4 |
| Q-08 | Documents are in Portuguese, root documents in English, code identifiers mixed. What is the convention? | nothing yet, but it gets worse with every file |
| Q-09 | The Soroban CPU ceiling: 100M in one document, 400M measured by bisection in a previous project. Unreconciled | nothing in §2 depends on either |
