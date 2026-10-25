# PLAN

Where the work stands, and in what order it happens next.

**What this file is for.** `docs/SPEC.md` §9 holds each task's acceptance criteria — what *done* means. This file holds the **order and the state** — what is finished, what is in flight, what is waiting on whom. Where the two disagree about criteria, the spec wins and this file is stale; fix it here and say so.

Last updated **2026-10-25**, from `git log`, `gh issue list` and `gh pr list`. Every line below is checkable with those three commands.

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

`main` has two commits. Nothing has been merged into it since the repository was published.

| Branch | Ahead of `main` | PR | State |
|---|---|---|---|
| `t-007-pin-beacon-key` | 1 commit | [#9](https://github.com/ThaisFReis/pinakion/pull/9) → `main` | open, **awaiting human review** |
| `t-004-green-gates` | 2 commits | [#10](https://github.com/ThaisFReis/pinakion/pull/10) → `t-007-pin-beacon-key` | open, stacked on #9 |
| `t-017-civic-design` | 2 commits | — | **in flight, uncommitted**: `web/index.html` modified, `web/style.css` new |

The stack is linear: #9 merges first, then #10. `CLAUDE.md` forbids the agent merging its own PRs, so both wait on the human.

### Gates right now

| # | Command | State |
|---|---|---|
| G1 | `cargo fmt --all --check` | PASS (on `t-004-green-gates`) |
| G2 | `cargo test --locked --workspace` | PASS, 12 tests |
| G3 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS (on `t-004-green-gates`) |
| G4 | `stellar contract build` | PASS |
| G5 | `npm test` | **FAIL** — deliberate placeholder until T-006 replaces it |

---

## Phase 0 — the gate that blocks everything

| | |
|---|---|
| **Who** | the human, in `docs/SPEC.md` §0 |
| **What** | write `Approved: 2026-10-25` on the approval line |
| **Why it blocks** | `CLAUDE.md` Bootstrap step 5: *no feature work until the human approves the spec* |

Done when: the line is written. Everything below is formally blocked until then.

---

## Phase 1 — land what is already proven

Four testnet transactions already show the thing working. None of that is on `main`.

| Order | Task | Issue | State | Note |
|---|---|---|---|---|
| 1 | T-007 pin the beacon key | [#1](https://github.com/ThaisFReis/pinakion/issues/1) | `review` → merge #9 | the key is a constant; a forged signature from a real adversary keypair is rejected on testnet |
| 2 | T-004 green fmt and clippy | [#3](https://github.com/ThaisFReis/pinakion/issues/3) | `review` → merge #10 | five deprecated aliases renamed, `rustfmt` applied; golden vectors still match, so no behaviour changed |

Done when: `main` carries both, and G1–G4 pass on `main`.

---

## Phase 2 — make the repository survive a stranger

A judge clones it and runs two commands. Today the Python and JavaScript verifiers — the whole basis of the claim *anyone can recompute the panel* — are exercised by **nothing**.

| Order | Task | Issue | State | Why it matters |
|---|---|---|---|---|
| 3 | T-017 bring `web/index.html` onto the design tokens | [#7](https://github.com/ThaisFReis/pinakion/issues/7) | **`doing`**, uncommitted | the page a judge opens; finish it before starting anything new. Q-13 was answered by the civic direction in DEC-005, so it no longer blocks |
| 4 | T-003 pin the toolchain | [#2](https://github.com/ThaisFReis/pinakion/issues/2) | `todo` | without `rust-toolchain.toml` and `.nvmrc`, "works from a clean checkout" is untested |
| 5 | T-005 gate the Python verifier | [#4](https://github.com/ThaisFReis/pinakion/issues/4) | `todo` | INV-04 is asserted only from the Rust side; nothing catches the Python drifting |
| 6 | T-006 gate the JavaScript verifier | [#5](https://github.com/ThaisFReis/pinakion/issues/5) | `todo` | replaces the G5 placeholder; `web/sorteio.mjs` was validated **outside** this repository and that validation is not reproducible here |

Done when: G1–G5 all pass, and each of the three implementations is checked against the same golden vectors by a command in §8.

---

## Phase 3 — the submission

The deck names four deliverables. Two are written, one is half-built, one has not been started.

| Deliverable | State | Task |
|---|---|---|
| Landing page | `web/index.html` is a verifier tool being reshaped into a page | T-017, above |
| MVP spec — the demo path, nothing more | **not written as one path** | T-016 |
| Video demo, 3 min | **not started** | T-016 |
| Submission data | the name and one line exist; track still unknown (Q-11) | T-009 done, Q-11 open |

| Order | Task | Issue | State |
|---|---|---|---|
| 7 | T-016 write the demo path and record the video | [#8](https://github.com/ThaisFReis/pinakion/issues/8) | `todo` |

The video's four parts, from the deck: `0:00` the problem · `0:30` live demo on testnet · `2:00` why Stellar · `2:30` what is next.

Done when: one flow runs live on testnet, with the recording as backup, in three minutes.

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
| Q-02 | Is the milestone goal in §1 correct as written, and are the non-goals right? | spec approval |
| Q-06 | Which toolchain versions are the project's? | T-003 |
| Q-11 | Which track and category is the submission? Genesis eligibility was never answered by the organisers | T-009's last step |
| Q-13 | **Answered** by the civic direction (DEC-005). The T-017 implementation itself is still for human review | nothing |
| Q-07 | Who publishes the first roster, and under what authority? Unresolved across three design documents | Phase 4 |
| Q-08 | Documents are in Portuguese, root documents in English, code identifiers mixed. What is the convention? | nothing yet, but it gets worse with every file |
| Q-09 | The Soroban CPU ceiling: 100M in one document, 400M measured by bisection in a previous project. Unreconciled | nothing in §2 depends on either |
