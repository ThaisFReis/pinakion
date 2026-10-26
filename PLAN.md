# PLAN

Where the work stands, and in what order it happens next.

**What this file is for.** `docs/SPEC.md` §9 holds each task's acceptance criteria — what *done* means. This file holds the **order and the state** — what is finished, what is in flight, what is waiting on whom. Where the two disagree about criteria, the spec wins and this file is stale; fix it here and say so.

Last updated **2026-10-25**, after #40 merged (`c226483`), from `git log`, `gh issue list` and `gh pr list`. Every line below is checkable with those three commands.

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

Everything through **T-033** is on `main` (`c226483`). **Open pull requests:** [#41](https://github.com/ThaisFReis/pinakion/pull/41), T-034, and the one carrying this update. #9 to #29 and #40 are merged; #29's last commit, the T-033 line, reached `main` only through this later PR. #24 to #27 were each merged into their stacked base branch, not into `main`; their commits reached `main` only because #28 was cut from the top of that stack. Same trap as #13 and #15: the merge bar has to say *into main*, or the stack has to be merged from the top.

Open issues: [#8](https://github.com/ThaisFReis/pinakion/issues/8), the recording, and [#30](https://github.com/ThaisFReis/pinakion/issues/30) to [#39](https://github.com/ThaisFReis/pinakion/issues/39), one per M1 task (Phase 5). [#6](https://github.com/ThaisFReis/pinakion/issues/6) was closed on 2026-10-25, answered by T-029.

### Gates right now — all seven green on `main`

As on `main` after #40. #41 (T-034), once merged, brings G2 to 52 tests and G7 to 24 figures in 7 documents.

| # | Command | State |
|---|---|---|
| G1 | `cargo fmt --all --check` | PASS |
| G2 | `cargo test --locked --workspace` | PASS, 41 tests |
| G3 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | PASS |
| G4 | `stellar contract build` | PASS |
| G5 | `npm test` | PASS, 4 vectors |
| G6 | `python3 scripts/conferir_vetores.py` | PASS, 4 vectors + the tie to both `test.rs` |
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
| Landing page | built, fixed, and **published**: <https://thaisfreis.github.io/pinakion/web/index.html> | T-017, T-024, T-027, done |
| MVP spec — the demo path, nothing more | written and proven: `docs/DEMO.md` | T-016 |
| Video demo, 3 min | script written in `docs/DEMO.md`; **the recording is not made** | T-016 |
| Submission data | the name and one line exist; the path is **Genesis** with the **Stellar Unlocked** track, decided by the human on 25 Oct (DEC-013). Open: the portal still records `scale`, approved, and no public rule says a path may be changed — Q-17 | T-009 done |

| Order | Task | Issue | State |
|---|---|---|---|
| 7 | T-016 write the demo path | [#8](https://github.com/ThaisFReis/pinakion/issues/8) | `review` — `docs/DEMO.md`, rehearsed end to end in 11 s |
| 8 | **record the video** | [#8](https://github.com/ThaisFReis/pinakion/issues/8) | **not started — the only thing left before the submission** |

The video's four parts, from the deck: `0:00` the problem · `0:30` live demo on testnet · `2:00` why Stellar · `2:30` what is next.

The flow runs live on testnet in 11 seconds, measured on 2026-10-25. What is left is the recording itself, which is the human's to make.

**This is the highest-value item with zero work done.** The deck's own line is *judges score demos, not ideas*, and the organiser's judging page agrees: the async review looks at the demo video, the pitch deck, the GitHub repo and the submission answers, in that order. Live pitching is for finalists only, on Wed 28 Oct. **There is no pitch deck.**

---

## Phase 3b — the ticket as an asset · **done, on `main`**

Decided on 25 Oct, the clock set aside, so that the project fits *Stellar Unlocked* on the track's own terms (DEC-014). Four tasks, four stacked branches under [#23](https://github.com/ThaisFReis/pinakion/pull/23) (T-028, the research), all green on the seven gates. All four merged; they reached `main` through #28 (see *State of the tree*).

| Order | Task | Branch | State |
|---|---|---|---|
| 9 | T-029 `kleroterion`: enroll, lock, impede, draw_round over a classic asset | [#24](https://github.com/ThaisFReis/pinakion/pull/24) on `t-029-bilhete-como-ativo` | merged — 22 tests written first |
| 10 | T-030 the verifiers recompute the panel from the roster; fourth golden vector | [#25](https://github.com/ThaisFReis/pinakion/pull/25) on `t-030-verificadores-do-painel` | merged |
| 11 | T-031 the classic sequence on testnet, every hash in `docs/evidencia-testnet-bilhete.md` | [#26](https://github.com/ThaisFReis/pinakion/pull/26) on `t-031-sequencia-classica-na-testnet` | merged — off-chain panel equals on-chain, address for address |
| 12 | T-032 THESIS, README, DEMO, the page and this file say so; `docs/avaliacao-genesis-unlocked.md` scores the rubric and names the objection (Q-21) | [#27](https://github.com/ThaisFReis/pinakion/pull/27) on `t-032-o-bilhete-em-todo-lugar` | merged |

What it closes: INV-06's roster half (Q-15), INV-07, 08, 09; INV-11 and 12 are new. What it does not: the case lifecycle, roster renewal, `AUTH_IMMUTABLE`, costs of the new calls — the last two are now in T-040; and it leaves two doors open after `lock` (any round, any number of draws; `impede` at will), which T-033 closes.

## Phase 4 — after the event

Not before 26 Oct. These need contracts that do not exist yet.

| Task | Issue | Blocked on |
|---|---|---|
| ~~T-008, the roster half of INV-06~~ | [#6](https://github.com/ThaisFReis/pinakion/issues/6), closed | **closed by T-029**: `kleroterion::lock` computes the minimum from the ledger clock — Q-15 answered |
| The case lifecycle, the roster cycle, the recusal rules | — | designed in three documents, **587 lines of design with no implementation**. M1 narrows it: no `impede` after lock (T-033), public `recuse` before `finalize` (T-038); a case, a reviewer and a verdict stay out of scope (DEC-015) |
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
| Q-17 | The portal records this participant on **Scale**; DEC-013 submits on **Genesis** + *Stellar Unlocked*. Nothing public says a path may be changed. Ask `meridian@growwithguava.com` | the submission |
| Q-24 | The default for `reveal_after`: "after the verdict" or something else? With 0 the seal buys only the claim window | T-038, T-041 |
| Q-25 | Tessera's voter-list format for the export | T-042 |
| Q-07 | Who publishes the first roster, and under what authority? Unresolved across three design documents | Phase 4 |
| Q-08 | Documents are in Portuguese, root documents in English, code identifiers mixed. What is the convention? | nothing yet, but it gets worse with every file |
| Q-02 | Is the milestone goal in §1 correct as written? Still `[INFERRED]`; M1 adds non-goals to it | — |

Answered since the last update: Q-11 (the path is Genesis, DEC-013, superseding the Scale reading), Q-16 (moot under Genesis), Q-21 (closed by T-033, #40), Q-09 and Q-23 (400 M instructions per transaction on testnet, read from the network config — T-034, #41), Q-22 (22 circomlibjs vectors match the contract's sponge — T-034, #41).

---

## Phase 5 — M1, Plan B: sealed sortition · **done, on `main`**

Decided 2026-10-25 (DEC-015 to DEC-019): the juror registers a commitment at `enroll`, the seat is claimed with a Groth16 proof over BN254 and Poseidon (Protocol 25), the panel is the `k` smallest `y`, and it is revealed **after the verdict**. Acceptance criteria in `docs/SPEC.md` §9; the reasoning, the days per item and the limits in the doc *Pinakion — Plano B: sortição selada* (https://claude.ai/artifact/9WDXFX9eSJWjhWUdRmrh2N).

**It does not ship on 26 Oct.** About 10 days of work (an estimate, not a measurement); the hacking window closes at 12:00. The submission says M1 is designed and not built, and claims nothing from it (§2).

**Kill criterion.** If T-034 ends without one proof of our circuit verified **on testnet** within budget, M1 stops there: the product is the open draw with T-033 alone (`lock(k, reveal_after)` → `seal(sig)`, already on `main`), and nothing of Plan B is mentioned. A second cut at half the time given to M1: without `seal → claim → finalize → reveal` passing on testnet, stop the same day — half-built scores below the open draw done well.

| Order | Task | Deps | Days (est.) | Issue | Branch / PR | State |
|---|---|---|---|---|---|---|
| 13 | T-033 close the two doors: round and `k` fixed at `lock`, `draw_round` replaced by a one-shot `seal(sig)` (DEC-020), no `enroll`/`impede` after lock | T-029 | 0.5 | [#30](https://github.com/ThaisFReis/pinakion/issues/30) | [#40](https://github.com/ThaisFReis/pinakion/pull/40) on `t-033-close-the-doors` | merged — §9 still `review` until the human sets `done` |
| 13 | T-034 **spike, kill-or-go**: ≥ 20 Poseidon vectors host × circomlib; one proof verified on testnet, cost measured; CPU ceiling read from the docs | T-033 | 1.5 | [#31](https://github.com/ThaisFReis/pinakion/issues/31) | [#41](https://github.com/ThaisFReis/pinakion/pull/41) on `t-034-spike-kill-or-go` | review — **go** (DEC-022): 22/22 vectors, proof `true` on testnet at 7.7 % of the 400 M ceiling |
| 13 | T-041 first page and guarantee sentence — cheap, and what a judge reads; finished after T-040 | T-033 | 1, split | [#38](https://github.com/ThaisFReis/pinakion/issues/38) | — | todo |
| 14 | T-035 commitment at `enroll`, `used_c`, Poseidon tree built by the contract, root frozen at `lock` | T-034 | 1 | [#32](https://github.com/ThaisFReis/pinakion/issues/32) | — | todo |
| 14 | T-036 circuit ported, public ptau, single-party phase 2, `vk`, CLI `pinakion prove` | T-034 | 1 | [#33](https://github.com/ThaisFReis/pinakion/issues/33) | — | todo |
| 15 | T-037 BN254 verifier as a separate contract; inputs assembled by `kleroterion` | T-036 | 1 | [#34](https://github.com/ThaisFReis/pinakion/issues/34) | — | todo |
| 16 | T-038 `seal`, `claim`, `finalize`, `reveal`, `recuse`; widening when fewer than `k` claim | T-035, T-037 | 1.5 | [#35](https://github.com/ThaisFReis/pinakion/issues/35) | — | todo |
| 17 | T-039 independent recomputation from the ledger; new golden vectors (N = 8, k = 3) | T-038 | 1 | [#36](https://github.com/ThaisFReis/pinakion/issues/36) | — | todo |
| 18 | T-040 testnet sequence end to end (with `AUTH_IMMUTABLE`), cost table by a §8 command | T-038, T-039 | 1 | [#37](https://github.com/ThaisFReis/pinakion/issues/37) | — | todo |
| 19 | T-042 ticket page, README links, Tessera export, submission text | T-039, T-041 | 1 | [#39](https://github.com/ThaisFReis/pinakion/issues/39) | — | todo |

Same order number = can run in parallel. T-035 and T-036 are independent once the spike passes: one person on the contract, one on the circuit, about six calendar days for two. Every task: tests written first and seen failing, seven gates green, one branch and one PR **into `main`**.

**Done when:** a stranger runs `scripts/sequencia_bilhete.sh` and the verifier against testnet and gets *seated · revealed · not revealed · drawn-absent* matching the contract; the README's first page states the setup, the relayer, the reveal condition and the limits; every number in the docs is produced by a §8 command.

**Where it stands (2026-10-26):** built and merged, T-033 to T-043, PRs #28 to #51; the sealed run `t040b` is on testnet and checks out (77 checks). §9 of `docs/SPEC.md` is the authority on each task.

## Phase 6 — M2: the case around the draw · **done, on `main`**

| Task | What | PR |
|---|---|---|
| T-044 | study: severity and panel size, interim suspension, late reveal | [#52](https://github.com/ThaisFReis/pinakion/pull/52) |
| T-045 | severity table sets `k`; the party impeded at `lock`; `suspended` | [#53](https://github.com/ThaisFReis/pinakion/pull/53), [#55](https://github.com/ThaisFReis/pinakion/pull/55) |
| T-046 | a floor on the reveal; TTL that reaches it | [#54](https://github.com/ThaisFReis/pinakion/pull/54), [#55](https://github.com/ThaisFReis/pinakion/pull/55) |

## Phase 7 — M3: the case cycle, end to end, with a demo link

One branch, `t-047-case-cycle`, and one PR (DEC-034).

| Task | What |
|---|---|
| T-047 | `seated(assento)` in kleroterion; one seat per seat account |
| T-048 | `contracts/processo`: opening, attendance and quorum, the record, the Tessera vote bound to the attendees, the verdict |
| T-049 | `scripts/sequencia_processo.sh`: the whole case on testnet, the vote on Tessera |
| T-050 | `scripts/verificar_processo.py` and `web/processo.mjs`: the whole case checked from the record |
| T-051 | `web/processo.html`: the case page, with a presentation mode for the video |
| T-052 | landing, README, thesis, submission and `docs/DEMO.md` follow |
