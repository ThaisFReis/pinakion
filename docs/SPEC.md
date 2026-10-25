# SPEC

Source of truth for this repository. Written by Bootstrap on 2026-10-25 from what was observed in the working tree. Items not observed are tagged `[INFERRED]` or raised in §11.

---

## §0 Rules for agents

**Approved: 2026-10-25**

  Written after the fact, and the record says so: T-001 to T-024 were built before this line existed, each one on an explicit instruction. The rule in `CLAUDE.md` is that the agent does not choose the direction unasked; it was not relaxed and it was not quite followed either, and Q-14 records that rather than tidying it away.

- `CLAUDE.md` at the repository root governs how work is done. This file governs what is built.
- Only the human sets a task to `done` and approves this spec.
- **Document hierarchy for this event**, from the HackMeridian *Speedrun SHIP* deck (Wlad Mendes, Stellar, 25 Oct 2026):

  | File | Role | State |
  |---|---|---|
  | `THESIS.md` | **the root** — one line, who it is for, the problem, why Stellar. Change it here first, then the others | present (T-010) |
  | `SPEC.md` | what to build and how to check it works | this file |
  | `DESIGN.md` | colours, type, logo, for every screen | present (T-012) |
  | `AGENTS.md` | the mission, ground rules, scope, tools, working style, done-when | present (T-011) |
  | `README.md` | how to run it; says what it is in line 1 | present, rewritten for Pinakion (T-015) |
  | `PLAN.md` | the order and state of the work; this file keeps the acceptance criteria | present (DEC-006) |

  The deck's rule for correcting course: a mentor saying *"nobody has this problem"* means fixing `THESIS.md`; *"cut that feature"* means `SPEC.md`; *"I can't tell what it does"* means `DESIGN.md`. Fix the file, not only the code.
- Project-specific additions to `CLAUDE.md`:
  - **Git is written in English.** Every commit message, and every pull request title and body, is in English, whatever language the conversation is in (DEC-007).

---

## §1 Goals and non-goals

### Observed state

The repository holds **one project**: Pinakion. A previous generation — due process over asset clawback, called first Cautela and then Pignus — was withdrawn (DEC-001) and moved out of the tree on 2026-10-25 to the sibling folder `../Cautela-geracao-retirada/`, which carries its own note. Nothing was deleted.

| Built and verified on testnet | Designed, not built |
|---|---|
| `contracts/beacon-verifier/` · `scripts/verificar_sorteio.py` · `web/` | the case lifecycle, the roster cycle, the recusal rules |

### Goal of the current milestone `[INFERRED — confirm in §11 Q-02]`

A module that draws a deciding panel from a roster committed **before** the case existed, using entropy nobody controlled at commit time, such that **any third party can recompute the panel** and check it.

The guarantee, in one sentence: *nobody chooses who judges you.*

### Non-goals `[INFERRED — confirm in §11 Q-02]`

- Not a DAO, not a governance application, not a wallet. It is a module an existing governance calls.
- Does not decide who may be on the roster, what the panel decides, or what follows from the decision.
- Does not provide due process over asset clawback. That was the withdrawn generation.

---

## §2 Claims policy

A claim appears in a document, README or UI only if it is listed here with its evidence.

| May claim | Evidence |
|---|---|
| A real drand round verifies on Stellar testnet | tx `c74150ee…`, recorded in `docs/evidencia-testnet-sorteio.md` |
| A signature forged under a different keypair is rejected on-chain | tx `30de94fe…` — a genuinely valid signature from another secret key, not a flipped byte |
| The contract trusts one named beacon, and says which | `beacon()` returns the quicknet chain hash; tx `804ba7a1…` |
| One beacon verification costs 26,719,483 CPU instructions and 161,818 memory bytes | measured via `env.cost_estimate().budget()` in `contracts/beacon-verifier/src/test.rs`; test-host figure, **not** a figure from a testnet transaction |
| The draw is recomputable off-chain by an independent implementation | `scripts/verificar_sorteio.py` and `web/sorteio.mjs` reproduce the on-chain panel; locked by golden vectors in `bate_com_a_recomputacao_fora_da_cadeia` |
| `randomness = sha256(signature)` for drand quicknet | verified against live rounds; test `aleatoriedade_e_sha256_da_assinatura` |

**Must not claim** (no evidence today): secure, audited, production-ready, trustless, unbiasable. The entropy source is trusted (§7), the contract is unaudited, and nothing has run on mainnet.

**Also must not claim**: *first*, *only*, or *novel* for verifying a drand signature on Soroban. Two Stellar projects already do it — `AshFrancis/splicers` (CAP-0059, same DST, 1st place at `scaffoldstellar`) and `NibrasD/Stellar-VRF` (SCF #44, $50,000) — logged in `docs/SOURCES.md` on 2026-10-25. What is unclaimed by either is the **sortition**: a panel drawn from a roster committed before the case, with the beacon key as a compile-time constant and three independent recomputations locked to golden vectors. Claim that, and say what came before.

---

## §3 Architecture

```
  drand quicknet  ──(round, BLS signature)──►  beacon-verifier (Soroban)
   public beacon                                 │ verify()  — pairing check
                                                 │ randomness() — sha256(sig)
                                                 │ draw()    — reseeded PRNG + shuffle
                                                 ▼
                                            panel (indices into the roster)
                                                 │
                   ┌─────────────────────────────┴───────────────────────┐
                   ▼                                                      ▼
    scripts/verificar_sorteio.py                              web/ (index.html + sorteio.mjs)
      independent recomputation, no deps                       the same recomputation, in a browser
```

### The ticket (T-029)

```
  court / clerk (escrivão)                   juror (G- or C-address)
      │ enroll(juror)                              │ signs (opt-in)
      ▼                                            ▼
  kleroterion (Soroban) ── SAC: trust · set_authorized(true) · mint 1 ──► PINAKION trustline
      │ lock()        ──► ledger, timestamp, minimum drand round; no enrol after
      │ impede(juror) ──► SAC: set_authorized(false)            (recusal, art. 463)
      │ draw_round(round, sig, k)
      │    verify (CAP-59) → randomness → shuffle (host PRNG) → order
      │    panel = first k of the order with SAC authorized() and balance ≥ 1
      ▼
  panel (addresses)             any Horizon: GET /accounts?asset=PINAKION:issuer
```

The issuer account sets `AUTH_REQUIRED` and `AUTH_REVOCABLE`, hands the SAC admin to the contract, and then sets its master weight to 0. After that no ticket can be issued or authorized except by the contract, which has no upgrade path and refuses enrolment after `lock()`.

### Components observed

| Path | What it is |
|---|---|
| `contracts/beacon-verifier/` | Soroban contract: `verify`, `randomness`, `draw` |
| `contracts/kleroterion/` | Soroban contract (T-029): the roster as a classic asset — `enroll`, `lock`, `impede`, `draw_round`; carries its own copy of `verify`/`randomness`/`draw` |
| `scripts/verificar_sorteio.py` | independent recomputation of the draw, no dependencies |
| `web/sorteio.mjs` | the same recomputation, for browser and Node |
| `web/index.html` | page that fetches a round and checks a panel |
| `docs/` | design documents and testnet evidence |

### Boundaries

- `B-01` The contract never depends on `scripts/` or `web/`. The off-chain verifiers depend on the contract's **observable behaviour**, never on its source.
- `B-02` `web/sorteio.mjs` is the single source of the browser recomputation. `web/index.html` imports it; it does not reimplement it.
- `B-03` No allowed exceptions today.

---

## §4 Invariants

Numbered; each with the test that enforces it, or marked `[proposed]` where nothing enforces it yet.

| ID | Statement | Enforced by |
|---|---|---|
| INV-01 | A valid drand round verifies; the same signature under a different round does not | `rodada_real_verifica_e_mede_custo`, `rodada_errada_falha`, `assinatura_trocada_falha` |
| INV-02 | The draw is deterministic: the same seed yields the same panel | `sorteio_e_deterministico` |
| INV-03 | A panel never repeats a position | `sorteio_sem_repeticao` |
| INV-04 | The on-chain panel equals the independent off-chain recomputation | `bate_com_a_recomputacao_fora_da_cadeia` (golden vectors) |
| INV-05 | `randomness = sha256(signature)` | `aleatoriedade_e_sha256_da_assinatura` |
| INV-06 | No draw uses a beacon round earlier than the roster's definitive ledger | `draw_round` (T-026) against a caller-given minimum in `beacon-verifier`; **fully** in `kleroterion` (T-029), where the minimum is computed at `lock()` from the ledger timestamp and is not a parameter: `rodada_minima_vem_do_fechamento`, `sorteio_com_rodada_anterior_ao_fechamento_e_recusado` |
| INV-07 | The attendance count never decreases because of an impediment (art. 463 §2) | `impedido_e_substituido_e_o_painel_fica_do_mesmo_tamanho` (T-029) |
| INV-08 | A substitute always comes from the already-drawn order; there is no re-draw for substitution | `impedido_e_substituido_e_o_painel_fica_do_mesmo_tamanho`, `substituto_e_o_proximo_da_ordem` (T-029) |
| INV-09 | The contract refuses to operate below the minimum roster size | `fechar_abaixo_do_minimo_e_recusado` (T-029) |
| INV-10 | The beacon public key is a pinned constant, not a parameter | `assinatura_de_outra_chave_nao_verifica`, `a_chave_do_beacon_nao_e_parametro` |
| INV-11 | The roster is committed before the case: after `lock()` nothing can be enrolled, and the issuer cannot mint or authorize once its master weight is 0 | `inscrever_depois_de_fechar_e_recusado` (T-029); the issuer half is a classic ledger fact, proven on testnet in T-031 |
| INV-12 | A juror sits only with ticket in hand and authorization on: `balance ≥ 1` and SAC `authorized()` at draw time | `sem_bilhete_nao_senta`, `impedido_e_substituido_e_o_painel_fica_do_mesmo_tamanho` (T-029) |

---

## §5 Interfaces and frozen areas

### Frozen by external definition

The beacon recipe is not ours to choose. It is fixed by drand and by the Soroban host:

- message = `sha256(round as u64 big-endian)`
- DST = `BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_`
- seed derivation = `HMAC-SHA256(SALT, seed)`, `SALT = sha256("Public Global Stellar Network ; September 2015")`
- shuffle = Fisher-Yates of `rand` 0.8.5 over `ChaCha20Rng` of `rand_chacha` 0.3.1
- quicknet genesis = `1692803367`, period = `3` s; round `r` is published at `genesis + (r-1)·period`; the minimum round after a lock at time `t` is `⌊(t−genesis)/period⌋ + 2`, the first round published strictly after `t` (checked 2026-10-25 against `api.drand.sh/<chain>/info`)

Changing any of these breaks every off-chain verifier. A change requires a DEC and regenerated golden vectors.

### Golden test data

`contracts/beacon-verifier/src/test.rs` — the vectors in `bate_com_a_recomputacao_fora_da_cadeia`, produced by `scripts/verificar_sorteio.py`. `contracts/*/test_snapshots/` are generated by the SDK.

### Frozen code

None declared yet. `[INFERRED]` candidate: nothing is stable enough to freeze before the spec is approved.

---

## §6 Dependencies and versions

| Component | Version | Source |
|---|---|---|
| `soroban-sdk` | 29.0.1 | `Cargo.toml` workspace |
| rustc / cargo | 1.97.1 | **pinned** in `rust-toolchain.toml`, with `rustfmt`, `clippy` and the `wasm32v1-none` target |
| wasm target | `wasm32v1-none` | **pinned** in `rust-toolchain.toml`; `stellar contract build` requires it |
| Stellar CLI | 28.1.0 | **not pinned** — it is a separate binary with no manifest rustup or nvm reads; the version is recorded here and in `README.md` (§11 Q-06) |
| Node | 24.14.1 | **pinned** in `.nvmrc`. No npm dependencies: `npm test` runs on builtins alone |
| `rand` (inside the Soroban host) | 0.8.5 | pinned by `rs-soroban-env`; the off-chain verifiers depend on this |
| `rand_chacha` (inside the host) | 0.3.1 | same |
| `py_ecc` | optional | only for `scripts/rodada.py`, which decompresses a drand signature for the demo. No gate depends on it, and `scripts/verificar_sorteio.py` keeps its zero-dependency promise |

Open `[VERIFY]` items: none outstanding — every external fact relied on today is logged in `docs/SOURCES.md`.

---

## §7 Security and secrets

### Trust model

| Party | What it can do | Mitigated how |
|---|---|---|
| drand threshold network | produce the entropy | public, scheduled, unpredictable before its round; a round earlier than the roster commitment is refused by INV-06 `[proposed]` |
| Stellar validators | bias `env.prng()` if it were used alone | not used alone: the PRNG is reseeded from the beacon value. The SDK states the base seed is under validator control and must not be relied on for unpredictability |
| whoever publishes the roster | choose who is eligible | public contestation window; **unresolved** — see §11 Q-07 |
| ~~whoever supplies the beacon public key~~ | — | **resolved (T-007)**: the key is a pinned constant and `verify` takes no key. The contract now trusts exactly one drand chain; a rotation of the quicknet group key would be a new chain and require a new contract |

### Secrets

No secrets in the repository. Testnet operations use the local Stellar CLI identity `cautela`; its key lives in the CLI's own config, outside this tree. No mainnet keys, endpoints or IDs anywhere.

### Environments

Testnet only. No mainnet target exists and none is in scope for this milestone.

---

## §8 Gates

Run from a clean checkout — **verified**: on 2026-10-25 a fresh clone ran G1 to G6 and all six passed, with `rust-toolchain.toml` making rustup select the pinned toolchain by itself. The baseline below was recorded on 2026-10-25 **before any change**, per `CLAUDE.md` Bootstrap step 3.

| # | Command | Baseline (25 Oct, before any change) | Now | Note |
|---|---|---|---|---|
| G1 | `cargo fmt --all --check` | FAIL | **PASS** | fixed by T-004 |
| G2 | `cargo test --locked --workspace` | PASS, 10 tests | **PASS, 16 tests** | +2 from T-007, +4 from T-026 |
| G3 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | FAIL, 5 deprecated aliases | **PASS** | fixed by T-004 |
| G4 | `stellar contract build` | PASS | **PASS** | beacon-verifier, 2,576 B optimised |
| G5 | `npm test` | FAIL | **PASS, 3 vectors** | was a placeholder; T-006 replaced it with `web/test.mjs` |
| G6 | `python3 scripts/conferir_vetores.py` | did not exist | **PASS, 3 vectors + the tie to `test.rs`** | added by T-005 |
| G7 | `python3 scripts/conferir_numeros.py` | did not exist | **PASS, 18 figures in 6 documents** | added by T-023 |

**Published numbers.** The cost figures are asserted in `test.rs` against constants, so a change in cost turns G2 red; G7 then checks that the six documents still print those same constants, so a document drifting turns G7 red. Both directions were proven to fail (checked 2026-10-25).

**Golden vectors.** `tests/vetores-de-ouro.json` holds them, and three implementations are held to it: the contract (G2, hard-coded in `test.rs`), the Python verifier (G6) and the JavaScript one (G5). G6 also checks that the JSON and `test.rs` still carry the same numbers, so the two cannot drift apart while each stays green on its own. Both gates were proven to fail: altering one number turns G5 and G6 red, with exit code 1 (checked 2026-10-25).

**Missing gates** (proposed as tasks, not added silently):

- Nothing runs `scripts/rodada.py`; it needs the network and an optional dependency, so it is exercised by the demo, not by a gate.
- The optimised contract size (2,576 B) is published in `docs/evidencia-testnet-sorteio.md` and in §8 above, and no gate checks it — it would need the build output, which G7 deliberately does not depend on.

---

## §9 Milestones and tasks

**Progress view:** [`PLAN.md`](../PLAN.md) at the root carries the order of the work, the branch and PR state, and the phases. This section stays the authority on each task's acceptance criteria (DEC-006).

**The clock.** The *Speedrun SHIP* deck puts the event at: 25 Oct build, 26 Oct ship — stop new features, record the video, submit one hour early. Today is **25 Oct**, and the first-hour checklist (thesis, spec, design, open repo with issues) is not done. The repository has **zero commits** and is not public, against the deck's "public repo on day 1".

**Current milestone:** M0 — put the repository on its own feet, and get the four documents the event expects. No feature work until this spec is approved.

**The four deliverables** the deck names, and what each needs:

| Deliverable | Needs | State |
|---|---|---|
| Landing page | thesis + design | `web/index.html` exists but is a verifier tool, not a landing page |
| MVP spec — the demo path, nothing more | spec + design | §9 below; the demo path is not yet written down as one path |
| Video demo, 3 min | spec + design | not started. Structure: 0:00 problem · 0:30 live demo on testnet · 2:00 why Stellar · 2:30 next |
| Submission data | thesis + spec | blocked on the name and the one-liner (Q-04, Q-10) |

**Phase gate:** the human approves this spec (§0) before T-001.

| ID | Title | Deps | Reads | Status | Acceptance criteria |
|---|---|---|---|---|---|
| T-000 | Bootstrap: spec, sources, gate baseline | — | — | review | SPEC.md and SOURCES.md exist; baseline recorded; branch `t-000-spec` |
| T-001 | Decide the fate of the withdrawn generation | T-000 | §1, §11 Q-03 | review | Moved to `../Cautela-geracao-retirada/` with a note; removed from this tree; §1, §3, §6 and §8 updated |
| T-002 | License the repository | T-000 | §11 Q-01 | review | MIT. `LICENSE` present; `package.json` renamed to `pinakion` and declares MIT |
| T-003 | Pin the toolchain ([#2](https://github.com/ThaisFReis/pinakion/issues/2)) | T-000 | §6, §11 Q-06 | review | `rust-toolchain.toml` and `.nvmrc` present; **G1–G6 run from a clean clone**, checked 2026-10-25, with rustup selecting the pinned toolchain on its own |
| T-004 | Green the baseline: fmt and clippy ([#3](https://github.com/ThaisFReis/pinakion/issues/3)) | — | §8 | review | G1 and G3 pass. Aliases renamed to `Bls12381G1Affine` / `Bls12381G2Affine`; rustfmt applied. No behaviour change: the golden vectors still match |
| T-005 | Test the Python verifier in the repository ([#4](https://github.com/ThaisFReis/pinakion/issues/4)) | T-003 | §4 INV-04 | review | G6 runs `scripts/conferir_vetores.py` against `tests/vetores-de-ouro.json` and also ties those numbers to `test.rs`; proven to fail when a vector is altered |
| T-006 | Test the JavaScript verifier in the repository ([#5](https://github.com/ThaisFReis/pinakion/issues/5)) | T-005 | §3 B-02 | review | `npm test` runs `web/test.mjs` against the same file and stops being a placeholder; proven to fail when a vector is altered |
| T-007 | Pin the beacon public key ([#1](https://github.com/ThaisFReis/pinakion/issues/1)) | — | §4 INV-10, §7 | review | Key pinned; `verify` takes no key; `beacon()` publishes the trusted chain hash; a forged signature from a real adversary keypair is rejected on testnet |
| T-008 | Enforce the roster's minimum round ([#6](https://github.com/ThaisFReis/pinakion/issues/6)) | T-007 | §4 INV-06 | review | `draw_round` refuses a round below the minimum and binds the seed to a verified signature. Proven on testnet: refusal is `Error(Contract, #1)`; the accepted draw reproduces the documented panel. The roster half stays open as Q-15 |

### Event track — from the Speedrun SHIP deck

Ordered by the clock, not by dependency. These block the submission; T-001 to T-008 do not.

| ID | Title | Deps | Reads | Status | Acceptance criteria |
|---|---|---|---|---|---|
| T-009 | Name the project and write the one line | — | §11 Q-04, Q-10 | review | **Pinakion** · *"Verifiable jury selection for governance on Stellar."* 7 words. Still to do: carry the same words into README line 1, the landing hero and the pitch opening (T-015, T-016) |
| T-010 | Write `THESIS.md` | T-009 | §1 | review | Present at the repository root: the guarantee, who it is for, the problem with two documented cases, why Stellar with measured figures, what is proven and what is not |
| T-011 | Write `AGENTS.md` | T-010 | §0 | review | Present at the root, in the deck's six sections. Defers to `CLAUDE.md` where the two disagree, and to §9 here for scope |
| T-012 | Write `DESIGN.md` | T-010 | — | review | Present at the root: tokens for both themes, three type roles with a scale, the mark as inline SVG, layout and voice. Derived from the thesis — cool stone, oxidised bronze, and the machine's own black and white balls as the semantic pair |
| T-017 | Civic design for the public page ([#7](https://github.com/ThaisFReis/pinakion/issues/7)) | T-012 | `DESIGN.md`, §2, §3 B-02 | review | User expanded the design brief on 2026-10-25: express governance legitimacy for all audiences and avoid crypto clichés. DESIGN.md and page hierarchy updated; responsive light/dark tokens applied; independent recomputation and evidence links retained; scope of comparison made visible. Local review only; see DEC-005 |
| T-013 | Public repository | T-011, T-015 | §11 Q-04 | review | Live at [github.com/ThaisFReis/pinakion](https://github.com/ThaisFReis/pinakion), public, MIT, one clean commit on `main` |
| T-014 | Spec tasks become issues | T-013 | §9 | review | Eight issues open, one per task still to do. Owners not yet assigned |
| T-015 | Rewrite `README.md` | T-010 | §2 | review | Opens with the one-liner; how to run it; every claim traced to §2; limits stated before the instructions |
| T-016 | Write the demo path and record the video ([#8](https://github.com/ThaisFReis/pinakion/issues/8)) | T-012, T-015 | §2 | review | `docs/DEMO.md`: one live path, proven end to end on 2026-10-25, plus the 3-minute script in the deck's four parts and the limits to state out loud. **The recording itself is still to do.** |
| T-018 | Write `PLAN.md`, the progress board | T-014 | §8, §9, §11 | review | Phases ordered by the clock; branch and PR state; what is out of scope before the ship date; §9 stays the authority on acceptance criteria (DEC-006) |

| T-019 | Put the git record in English | T-018 | §0, §11 Q-08 | review | Commits not yet on `main` rewritten in English; PR titles and bodies in English; the convention recorded in §0 and DEC-007 |

| T-020 | Make the live round reproducible | T-007 | §5, §7 | review | `scripts/rodada.py` fetches a round and decompresses its G1 signature to the `BytesN<96>` the contract takes; without it the headline demo cannot be reproduced from the repository |

| T-023 | Gate the published numbers | T-005 | §8 | review | `test.rs` asserts the measured costs; G7 checks every document still prints them. Proven to fail from both sides |

| T-027 | Publish the page, and point the README at it | T-017 | §2, §7 | review | GitHub Pages on `main`; the README's *Try it* resolves for a stranger. Checked live, including a cross-origin fetch of a drand round |

| T-028 | Read the event's own rules and settle the submission path | T-009 | §9, §11 Q-11 | review | The paths, the Genesis-only tracks, the rubric and its weights, the deadline and the prize terms read from `hackmeridian.com/build` and the portal MCP, logged in `docs/SOURCES.md` with both conflicts named. Q-11 answered, then superseded by the human on the same day: the path is **Genesis** with the **Stellar Unlocked** track (DEC-013). The Genesis requirements, the Unlocked wording and its four idea starters, the Genesis weights, and what the contract actually calls against them, all logged in `docs/SOURCES.md`. Q-16 moot; Q-17 opened for the one unknown. No code touched |

### The ticket as an asset — fitting *Stellar Unlocked* (DEC-014)

| ID | Title | Deps | Reads | Status | Acceptance criteria |
|---|---|---|---|---|---|
| T-029 | `kleroterion`: the roster as a classic asset | T-026, T-028 | §3, §4 INV-06..12, §5, DEC-014 | doing | New contract. `enroll` creates the trustline through SAC `trust`, authorizes and mints one ticket; `lock` records ledger and timestamp, computes the minimum round and refuses below the roster minimum; `impede` revokes authorization; `draw_round` refuses before lock and below the minimum, verifies the beacon, shuffles the roster order and seats the first `k` with ticket and authorization. Golden vectors still match. All tests written first and seen failing |
| T-030 | Verifiers recompute the panel from the roster | T-029 | §3 B-01, B-02, §5 | todo | `scripts/verificar_painel.py` and `web/sorteio.mjs` take the ordered roster, the impeded set and `k`, and return the same panel as the contract; a fourth golden vector with addresses; G5 and G6 cover it |
| T-031 | The classic sequence on testnet | T-029 | §7, DEC-014 | todo | Issuer with `AUTH_REQUIRED`+`AUTH_REVOCABLE`, SAC admin handed to the contract, master weight set to 0; seven jurors enrolled, lock, one impediment, one draw; every hash in `docs/evidencia-testnet-bilhete.md`; Horizon `/accounts?asset=` shows the roster; the sort key of that endpoint recorded (the `[VERIFY]` in `docs/SOURCES.md`) |
| T-032 | Say it everywhere the ticket is an asset | T-029 | §0, §2 | todo | `THESIS.md`, `README.md`, the landing and `docs/DEMO.md` describe the ticket, the lock and the two Unlocked bullets it answers; every claim traced to §2; no *first*/*novel* |

Statuses: `todo` → `doing` → `review` → `done` (human only), or `blocked` with a §11 link.

---

## §10 Decision log

### DEC-001: Withdraw the clawback due-process design (2026-10-25, pre-T-000)
**Context:** The design that this repository was created for — containment with scope, expiry and mandatory ratification over asset clawback — was found to duplicate an existing public project, item by item, including the state machine, the hypothesis taxonomy, the locked issuer and the permissionless settle.
**Decision:** Withdraw it. The human instructed that the new project has no relation to that work and does not cite it.
**Alternatives:** Differentiate within the same thesis — rejected: a second implementation of a design published a week earlier scores nothing on innovation and loses on execution.
**Consequences:** the whitepapers, the Groth16 verifier and the Circom circuit were superseded. Resolved by T-001 on the same day: moved to `../Cautela-geracao-retirada/` with a note, not deleted.

### DEC-002: drand quicknet as the entropy source (2026-10-25, pre-T-000)
**Context:** `env.prng()` alone cannot drive a draw. The SDK documents that its seed is under validator control and is not hard to bias.
**Decision:** Seed the PRNG from a drand quicknet round whose number is fixed before the roster and case are committed; verify the round's BLS signature on-chain.
**Alternatives:** A commit-reveal among participants — rejected: the last revealer controls the outcome. Pure `env.prng()` — rejected on the SDK's own warning.
**Consequences:** The contract depends on BLS12-381 host functions; the recipe in §5 is frozen by drand; the public key must be pinned (T-007).

### DEC-003: Off-chain recomputation is a product requirement (2026-10-25, pre-T-000)
**Context:** A deterministic draw is not a verifiable one. The guarantee requires that a third party recompute the panel.
**Decision:** Treat the agreement between contract and independent implementations as an invariant (INV-04), locked by golden vectors in the test suite.
**Alternatives:** Publishing the algorithm as documentation only — rejected: nothing would catch a host change.
**Consequences:** Two off-chain implementations exist; both need gates (T-005, T-006). A host change to the PRNG or shuffle turns the suite red, which is the intent.

### DEC-004: Pin one beacon key, and publish which (2026-10-25, T-007)
**Context:** `verify` accepted the drand public key as a call parameter, so anyone could pass their own key and have any round verify. That removes the entire guarantee, because the seed stops coming from a beacon nobody controls.
**Decision:** The quicknet group key is a constant compiled into the contract, and `verify` takes no key. A new read, `beacon()`, returns the chain hash the contract trusts, so the pinned trust is auditable on-chain without reading the Wasm.
**Alternatives:** A settable key behind an admin — rejected: it reintroduces the same party with an extra step. A set of accepted keys — rejected as unneeded complexity before a second beacon exists.
**Consequences:** The contract trusts exactly one drand chain. A quicknet group-key rotation is a new chain with a new hash and requires a new contract; changing the key without changing the contract is precisely the power this pinning removes. Verification cost rose by 887 instructions, the constant being loaded rather than passed. INV-10 is now enforced; §7 and Q-05 updated. `beacon()` is a new public function and §5 declares no frozen interface yet.

### DEC-005: Civic editorial identity for all governance audiences (2026-10-25, T-017)
**Context:** The human requested a stronger design focused on what governance should convey, avoiding crypto clichés, and specified that all listed audiences matter.
**Decision:** Use the stone/patina palette and serif/sans/data type roles in an editorial layout. Lead with procedure and people. Place the working comparison, its evidence and its limitations together. Put technical inputs behind native disclosure controls. A matching result describes agreement with the supplied data.
**Correction made while finishing the task:** the page's worked example pointed at contract `CCLWP7WO…` and two transactions that no document in this repository records. Both are real on testnet, but they belong to the pre-T-007 deployment, whose beacon key was still a call parameter. The example is now round 33382274 on `CBPHNZLG…`, with the two transactions `docs/evidencia-testnet-sorteio.md` lists.
**Consequences:** T-017 includes hierarchy and copy presentation as authorised in the conversation; general approval in §0 remains pending. No contract or independent algorithm change. The page is a local preview, not a public deployment. Empty signature input clears the previous result so an obsolete success state is not displayed.
**Validation (observed 2026-10-25):** G1–G4 passed with their §8 commands, including 12 Rust tests. G5 fails as the placeholder `docs/SPEC.md` §8 declares, which T-006 replaces — an earlier note here said `npm` was absent from PATH; it is present (v24.14.1, via nvm) and the failure is the declared one, not a missing tool. Browser checks at 800px and 375px, light and dark: the documented panel recomputes and matches, a deliberately altered panel is refused, no console errors, no horizontal overflow. These checks do not replace T-006 or establish new contract guarantees.

---

### DEC-006: A separate `PLAN.md` for order and state (2026-10-25, post-T-014)
**Context:** §9 holds both the acceptance criteria and the status of every task, plus the event's deliverables. Reading it to answer "what is next, and what is waiting on whom" means reading three tables and two prose paragraphs, and the branch and PR state is not in it at all.
**Decision:** `PLAN.md` at the root carries the order, the phases, the branch and PR state, and what is deliberately out of scope before the ship date. §9 keeps the acceptance criteria and remains the authority on them.
**Alternatives:** Keeping everything in §9 — rejected: the file a human checks hourly during an event should not be the file that also defines correctness. Generating `PLAN.md` from §9 — rejected: no generator exists and writing one before the video is the wrong order.
**Consequences:** Two files state task status, which can drift. `PLAN.md` states the rule for that: where the two disagree about criteria, the spec wins and `PLAN.md` is stale. Fixed the §0 table, which still listed `THESIS.md`, `DESIGN.md` and `AGENTS.md` as missing after T-010 to T-012 wrote them.

### DEC-007: Git is written in English (2026-10-25, T-019)
**Context:** The conversation runs in Portuguese and the commit messages followed it, as did two PR bodies. The repository is public and read by people who do not read Portuguese; the history and the PR record are part of what they see.
**Decision:** Every commit message and every pull request title and body is written in English. The language of the conversation does not decide the language of the repository's git record.
**Alternatives:** Bilingual messages — rejected as noise. Leaving it to each commit — rejected: that is how the inconsistency arose.
**Consequences:** Five Portuguese commits that had not reached `main` were rewritten in English on a fresh branch, giving the same tree in three commits instead of seven. Two Portuguese commits are already on `main` (`ca738dc`, `f0048e2`) and are left alone, because rewriting published history on a public default branch is the human's call, not the agent's. The merged PR bodies of #11 and #12 are part of the public record and were rewritten in place. Q-08 is now partly answered; the prose documents remain mixed.

### DEC-008: A second script, with a dependency (2026-10-25, T-020)
**Context:** `verify` takes the signature as `BytesN<96>` — the G1 point uncompressed. drand publishes 48 compressed bytes. The decompression existed only as throwaway code in a conversation and as hard-coded arrays in the test file, so the project's headline claim — *a live drand round verifies on Stellar* — could not be reproduced by anyone reading the repository.
**Decision:** Add `scripts/rodada.py`, which fetches a round and prints the uncompressed signature ready for the CLI. It depends on `py_ecc`.
**Alternatives:** Implementing BLS12-381 decompression by hand to keep the zero-dependency rule — rejected for now: it is a square root in F_p with a sign rule, and getting it subtly wrong would produce a point that simply fails the pairing, which looks like a broken demo rather than a broken script. Worth doing later. Decompressing in the contract — rejected: it would spend CPU on something that is safe to do outside, since a wrong point fails the pairing anyway.
**Consequences:** `scripts/verificar_sorteio.py` keeps its zero-dependency promise and is still the one that matters for the guarantee; `rodada.py` is a convenience for the demo and says so. The repository now has an optional Python dependency, declared in `docs/DEMO.md` and §6.
### DEC-009: One file of golden vectors, three implementations (2026-10-25, T-005/T-006)
**Context:** INV-04 says the contract and the off-chain recomputations agree. Until now that was asserted only from the Rust side, with the numbers hard-coded in `test.rs`; the Python verifier was checked by hand and the JavaScript one was validated outside the repository.
**Decision:** `tests/vetores-de-ouro.json` is the one set of numbers. G6 runs the Python verifier against it, `npm test` runs the JavaScript one, and G6 additionally parses `test.rs` and fails if its asserted panels and the JSON have diverged.
**Alternatives:** Generating the JSON from the Rust test — rejected: it needs a build step before a gate can run. Having the Rust test read the JSON — rejected for now: the contract crate is `no_std` and parsing JSON there costs more than the drift check does. Three separate copies of the vectors — rejected, since that is the failure being prevented.
**Consequences:** G5 stops being a deliberate failure and the §8 baseline records that. A host change to the PRNG or the shuffle now turns three gates red instead of one. `tests/` is a new directory holding data, not code.

### DEC-010: The published numbers are gated from both sides (2026-10-25, T-023)
**Context:** `CLAUDE.md` requires a published number to be reproducible by a gate. The cost figures were produced by G2 as console output that nothing read, and repeated by hand in six documents. Either side could change without the other noticing.
**Decision:** The tests assert the measured costs against constants in `test.rs`, and `scripts/conferir_numeros.py` (G7) checks that each document still prints those constants. Digit separators are stripped, because the same figure appears as `26.719.483` and as `26,719,483`.
**Alternatives:** Generating the documents' numbers from the tests — rejected: it would turn prose into a template for one figure. Asserting in the test against the documents — rejected: the contract crate is `no_std`, the same reason DEC-009 gives.
**Consequences:** A cost change turns G2 red and tells the author to update the documents; a document edited by hand turns G7 red. The contract's optimised size is still ungated, and §8 says so. Seven gates now.

### DEC-011: `randomness` returns what drand publishes (2026-10-25, T-026)
**Context:** Building `draw_round` exposed that the contract's `randomness` was `sha256` of the **uncompressed** signature, while drand publishes `sha256` of the **compressed** one. Different values. Everything else in the project — the page, both verifiers, every documented panel — seeds from drand's value, so the contract's own function agreed with nothing and was used nowhere. Its only test printed the result without asserting it.
**Decision:** `randomness` recompresses the point on-chain — the flag bits in X, `0x80` always and `0x20` when Y is the larger of the two roots — and hashes that. A test now asserts it equals the round's published randomness, so the two can never drift again.
**Alternatives:** Adding a second function and leaving the first — rejected: it would keep a function whose name promises the beacon's randomness and does not deliver it. Seeding from the uncompressed hash everywhere instead — rejected: it would break every published panel and lose the property that the browser's seed matches drand's.
**Consequences:** A public function changed behaviour. §5 declares no frozen interface yet, so no version bump is due, but it is recorded here as one. `(p-1)/2` is a new constant in the contract, checked against a real round before it was written down. Confirmed on testnet: `draw_round` at the minimum round reproduces the panel `docs/evidencia-testnet-sorteio.md` records.

### DEC-012: The page is published from `main` (2026-10-25, T-027)
**Context:** The README's first call to action — *try it in a browser* — pointed at a claude.ai artifact. Artifacts are private unless shared, so a reader following the repository's opening instruction could meet a sign-in wall; and that artifact was published before T-017 and T-024, so it was the old page, with the stale-comparison bug still in it.
**Decision:** Enable GitHub Pages on `main` at the repository root and point the README and `docs/DEMO.md` at <https://thaisfreis.github.io/pinakion/web/index.html>. What is on `main` is what runs there, so the page cannot drift from the code the way a copy does.
**Alternatives:** Re-sharing the artifact — rejected: it would still be a copy that ages. Telling the reader to run a local server — rejected as the headline path: *nothing to install* is the point, though the demo still prefers local, since one less network hop can fail on camera.
**Consequences:** The repository's whole tree is served publicly, which it already was by being a public repository. Checked live: the page loads, the documented example matches, fetching a round from the browser works across origins, and the console is clean.

---

### DEC-013: Submit on the Genesis path, Stellar Unlocked track (2026-10-25, T-028)
Context: Q-11 was answered from the portal, which records this participant on Scale, approved. The Scale path is judged first on milestone completion against pre-event intake goals, and this repository has no record of those goals (Q-16) — Pinakion is three days old and replaced a withdrawn thesis (DEC-001), so there is no product carried forward to show. The human decided on the Genesis path instead, with the *Stellar Unlocked* thematic track.
Decision: the submission is Genesis + Stellar Unlocked. The Unlocked case is that the contract stands on core-protocol primitives used for something they were not obviously built for: the BLS12-381 host functions (`hash_to_g1`, `pairing_check`) verify a public randomness beacon on-chain, and the host PRNG (`env.prng().seed`, `shuffle`) turns that beacon into a panel anyone can recompute. That is the *underrated toolbox* starter, not the *ZK, natively* one.
Alternatives: Scale, rejected because its first criterion cannot be evidenced (Q-16). *Stellar Composed*, rejected because the project composes no second ecosystem tool or project — it depends on drand, which is not a Stellar project. Entering both tracks, rejected for the same reason.
Consequences: the rubric that applies is the Genesis one (Track Alignment 20%, User Value & Problem 20%, Technical Execution 15%, Innovation 15%, Hackathon Progress 10%, Market & Ecosystem Potential 10%, Product Design & UX 10%). §9 T-028 restated; Q-11 superseded; Q-16 moot; Q-17 opened for the one thing this repository cannot check — whether the organiser allows the switch. No claim of ZK, of BN254 or of Poseidon may appear anywhere: §2 forbids it and the code does not do it.

### DEC-014: The ticket is a classic asset (2026-10-25, T-029)
Context: the submission is Genesis + *Stellar Unlocked* (DEC-013). The track's own test is a core-protocol feature used for something it was not obviously built for, by nobody before; the contract stood on the oldest primitive the track names, with two prior drand verifiers on record (Q-18, Q-19). The roster — the half of the guarantee that was 587 lines of design with no code — had no on-chain commitment (Q-15). The human instructed on 2026-10-25 that the project must fit the track, the clock notwithstanding.
Decision: the *pinakion* becomes literally a ticket: one unit of a classic Stellar asset. The issuer sets `AUTH_REQUIRED` and `AUTH_REVOCABLE`, makes the `kleroterion` contract the SAC admin, and locks itself (master weight 0). The contract enrols jurors through SAC `trust` (CAP-73), `set_authorized` and `mint`; `lock()` fixes the roster, its order, the ledger and the minimum drand round; `impede()` is `set_authorized(false)`; the draw seats the first `k` of the shuffled order who hold a ticket and are authorized. The roster is readable by any Horizon client through `/accounts?asset=`.
Alternatives: CAP-85 — rejected, it is an upgrade switch (Q-19). A Poseidon Merkle roster (CAP-75) — rejected, cosmetic without a circuit and it needs Poseidon in three languages. Ship as is — rejected by the human. Composing the deployed `beacon-verifier` cross-contract — rejected: `contractimpl` exports cannot be suppressed when a contract crate is linked as a library, so `verify`/`randomness`/`draw` are copied into `kleroterion` verbatim and the golden vectors keep both copies honest.
Consequences: §3 gains the ticket diagram and the `kleroterion` component; INV-06 becomes fully enforced; INV-07, 08, 09 leave `[proposed]`; INV-11 and INV-12 are new; §5 pins the quicknet genesis and period; T-029 to T-032 added; Q-15 and Q-20 answered. The trust model gains one named role, the clerk (`escrivão`) who enrols and locks — the same open question as Q-07, not a new one. Claims stay as §2 allows: nothing here is audited or on mainnet.

## §11 Open questions

| ID | Question | Blocks |
|---|---|---|
| Q-01 | ~~No `LICENSE`, and `package.json` declared ISC.~~ **Answered 2026-10-25: MIT.** It is the licence of the human's previous project and the ecosystem norm for contracts and applications (OpenZeppelin stellar-contracts, script3/soroban-governor). Apache-2.0, used by the SDF's own repositories, was the alternative and adds an express patent grant. | — |
| Q-02 | §1 states the goal as inferred. Is the milestone goal correct as written, and are the non-goals right? | T-001 |
| Q-03 | ~~What happens to the withdrawn generation?~~ **Answered 2026-10-25:** moved out of the repository to `../Cautela-geracao-retirada/`, with a note explaining that the thesis changed and the work did not fail. Nothing deleted; the Groth16 verifier still has green tests and a testnet deployment there. | — |
| Q-04 | ~~The active project has no name, and where does it live?~~ **Answered 2026-10-25: Pinakion**, in a new public repository at `ThaisFReis/pinakion`; the Cautela tree stays where it was. The πινάκιον was the bronze ticket bearing a citizen's name, slotted into the kleroterion before an Athenian jury draw. Chosen over *Kleroterion* because `kleros` is the root of an existing onchain jury protocol. | — |
| Q-05 | ~~`verify` takes the beacon public key as a parameter.~~ **Resolved 2026-10-25 by T-007.** Accepted consequence, worth confirming: the contract trusts a single drand chain, so a quicknet key rotation means a new chain and a new contract. | — |
| Q-06 | **Answered 2026-10-25 by T-003:** rustc 1.97.1 and Node 24.14.1, the versions everything here was built and checked with, pinned in `rust-toolchain.toml` and `.nvmrc`. The Stellar CLI (28.1.0) stays unpinned because no pin file governs it; that gap is stated rather than hidden. | — |
| Q-07 | Who publishes the first roster, and under what authority? This is unresolved across three design documents. | — |
| Q-08 | **Partly answered 2026-10-25:** git is in English — commit messages and PR titles and bodies, always (DEC-007). Still open for the repository's prose: `THESIS.md`, `README.md`, `DESIGN.md`, `AGENTS.md`, `PLAN.md` and `docs/SPEC.md` are in English, while `docs/desenho-*.md`, `docs/evidencia-*.md` and `web/index.html` are in Portuguese, and code identifiers are mixed. | — |
| Q-09 | `docs/whitepaper-v0.1.md` states the Soroban CPU ceiling as 100M. A measurement from a previous project of the human's, cited in conversation, gives 400M by bisection on testnet. The two have not been reconciled and no figure in §2 depends on either. | — |
| Q-10 | ~~What is the one line?~~ **Answered 2026-10-25:** *"Verifiable jury selection for governance on Stellar."* Seven words, in the deck's own shape — category plus where. *"Nobody chooses who judges you."* is kept as the guarantee: it opens the pitch and sits under the landing hero, but it is not the one-liner, because it does not say what the thing is. | — |
| Q-11 | ~~Which track and which category is the submission?~~ **Answered 2026-10-25, and the question was malformed:** Genesis and Scale are *paths*, not tracks, and the two thematic tracks — *Stellar Unlocked* and *Stellar Composed* — belong to the Genesis path alone. The portal records this submission on **Scale**, approved. A Scale project picks no track, so the Genesis eligibility worry is moot: it is scored on **Ecosystem Alignment** where a Genesis project is scored on Track Alignment, and judged *prototype → product* (milestone completion against the goals set at pre-event intake, SCF eligibility, production readiness, ecosystem impact). Logged in `docs/SOURCES.md`. **Superseded the same day:** on 2026-10-25 the human decided to submit on the **Genesis** path with the **Stellar Unlocked** track (DEC-013). The rubric that applies is therefore the Genesis one, Track Alignment at 20%, and Q-16 falls away with it; Q-17 takes its place. | — |
| Q-16 | ~~The first thing the Scale path is judged on is *milestone completion against goals set at pre-event intake*, and this repository has no record of what those intake goals were.~~ **Moot since DEC-013:** the submission goes on Genesis, which is not judged on intake goals. The honesty point behind it still stands and belongs in the submission answers: Pinakion is three days old and replaced a withdrawn thesis (DEC-001), and the write-up says so rather than implying a product carried forward. | the submission |
| Q-12 | ~~`AGENTS.md` versus `CLAUDE.md`.~~ **Resolved 2026-10-25 as proposed:** `AGENTS.md` carries the deck's six sections and states that `CLAUDE.md` wins where the two disagree. Neither duplicates the other. Confirm you are happy with that precedence. | — |
| Q-13 | **Addressed by the design request on 2026-10-25:** T-017 applies Cormorant Garamond and the stone/patina system in the civic direction of DEC-005. The implementation is for human review. | T-017 |
| Q-14 | The §0 approval line was written on 2026-10-25, after the work it governs. Everything was done on an explicit request, so the intent of the rule held, but the order it prescribes did not. Worth deciding before the next milestone whether the gate is a real gate or should be dropped from `CLAUDE.md`. | — |
| Q-15 | ~~`draw_round` takes `rodada_minima` as an argument.~~ **Answered by T-029 (DEC-014):** `kleroterion` computes the minimum at `lock()` from the ledger timestamp; it is not a parameter. `beacon-verifier` keeps the caller-given form as the stateless evidence contract. | — |
| Q-17 | The portal records this participant on **Scale**, `scale_status: "approved"`, and DEC-013 chooses **Genesis**. Nothing on the organiser's public pages says a path may be changed, and the portal exposes no tool to change it (`docs/SOURCES.md`, *Checked and not found*). **For the human: has the organiser confirmed the switch?** Until they have, the path in the submission form is a claim this repository cannot check. | the submission |
| Q-18 | The Raven pass on 2026-10-25 found that drand-BLS verification on Soroban has been done twice before, one of them SCF-funded at $50,000 and one a hackathon winner (`docs/SOURCES.md`). Nothing in this repository ever claimed otherwise, so there is nothing to retract — but **Innovation is 15% of the Genesis rubric and Track Alignment is 20%**, and the pitch has to be built on the sortition rather than on the beacon plumbing. §2 now forbids the first-ness claim. **For the human: does the video and the deck say what came before?** | the submission |
| Q-19 | ~~Build a CAP-85 roster?~~ **Withdrawn 2026-10-25, same day, after reading the spec:** CAP-85 is a beacon proxy — one owner contract repoints the executable of a whole fleet at once. The entry cannot be *removed*, but its *value is updatable*; that is an upgrade switch, the exact power *nobody chooses who judges you* exists to remove. CAP-86 is schema-migration tooling. Neither fits. The timeline finding stands: the track names recent primitives, and this project sits on the oldest one it names. The fit question moves to Q-20. | — |
| Q-20 | ~~How to fit *Stellar Unlocked* tightly, with what the research supports.** The track's own test is *a primitive used for something it was not obviously built for, by nobody before*. Three routes were costed against the thesis and the clock (`docs/SOURCES.md`, 2026-10-25): **(a) the ticket is an asset** — each juror holds one unit of a classic asset with `AUTH_REQUIRED`; the roster is the set of authorized trustlines, enumerable by anyone through Horizon `/accounts?asset=`; the commitment is the issuer setting its master weight to 0, after which no ticket can ever be issued; recusal is `set_authorized(false)` under `AUTH_REVOCABLE`; the draw contract checks each drawn juror with SAC `authorized()` — two bullets at once (*SAC trustlines*, *old-school assets, new tricks*), no new dependency, a small contract change, but the shuffle order must be fixed by a rule the verifiers share (account id ascending) and the lock ledger becomes the INV-06 minimum. **(b) Poseidon Merkle roster** (CAP-75, the *membership proofs* words of the ZK bullet) — needs `stellar/rs-soroban-poseidon`, a new dependency, and Poseidon in Python and JavaScript before the golden vectors hold; not feasible before 12:00 and cosmetic without a circuit. **(c) ship as is** and argue the sortition — honest, but on the track's oldest primitive with two prior drand verifiers. **For the human: (a), or (c)?** Either way the video and the deck are still unmade. **Answered 2026-10-25 by the human: route (a), the clock set aside — DEC-014, T-029 to T-032.** | — |
