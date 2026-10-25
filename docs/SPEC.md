# SPEC

Source of truth for this repository. Written by Bootstrap on 2026-10-25 from what was observed in the working tree. Items not observed are tagged `[INFERRED]` or raised in §11.

---

## §0 Rules for agents

**Approved:** _(not yet — the human writes `Approved: YYYY-MM-DD` here)_

- `CLAUDE.md` at the repository root governs how work is done. This file governs what is built.
- Only the human sets a task to `done` and approves this spec.
- **Document hierarchy for this event**, from the HackMeridian *Speedrun SHIP* deck (Wlad Mendes, Stellar, 25 Oct 2026):

  | File | Role | State |
  |---|---|---|
  | `THESIS.md` | **the root** — one line, who it is for, the problem, why Stellar. Change it here first, then the others | **missing** (T-010) |
  | `SPEC.md` | what to build and how to check it works | this file |
  | `DESIGN.md` | colours, type, logo, for every screen | **missing** (T-012) |
  | `AGENTS.md` | the mission, ground rules, scope, tools, working style, done-when | **missing** (T-011) |
  | `README.md` | how to run it; says what it is in line 1 | present, describes the withdrawn generation |

  The deck's rule for correcting course: a mentor saying *"nobody has this problem"* means fixing `THESIS.md`; *"cut that feature"* means `SPEC.md`; *"I can't tell what it does"* means `DESIGN.md`. Fix the file, not only the code.
- Project-specific additions to `CLAUDE.md`: none yet.

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
| A real drand round verifies on Stellar testnet | tx `9de19747…`, recorded in `docs/evidencia-testnet-sorteio.md` |
| A tampered round is rejected on-chain | tx `2ebda22b…` |
| One beacon verification costs 26,718,596 CPU instructions and 161,546 memory bytes | measured via `env.cost_estimate().budget()` in `contracts/beacon-verifier/src/test.rs`; test-host figure, **not** a figure from a testnet transaction |
| The draw is recomputable off-chain by an independent implementation | `scripts/verificar_sorteio.py` and `web/sorteio.mjs` reproduce the on-chain panel; locked by golden vectors in `bate_com_a_recomputacao_fora_da_cadeia` |
| `randomness = sha256(signature)` for drand quicknet | verified against live rounds; test `aleatoriedade_e_sha256_da_assinatura` |

**Must not claim** (no evidence today): secure, audited, production-ready, trustless, unbiasable. The entropy source is trusted (§7), the contract is unaudited, and nothing has run on mainnet.

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

### Components observed

| Path | What it is |
|---|---|
| `contracts/beacon-verifier/` | Soroban contract: `verify`, `randomness`, `draw` |
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
| INV-06 | No draw uses a beacon round earlier than the roster's definitive ledger | `[proposed]` |
| INV-07 | The attendance count never decreases because of an impediment (art. 463 §2) | `[proposed]` |
| INV-08 | A substitute always comes from the already-drawn order; there is no re-draw for substitution | `[proposed]` |
| INV-09 | The contract refuses to operate below the minimum roster size | `[proposed]` |
| INV-10 | The beacon public key is a pinned constant, not a parameter | `[proposed]` — **violated today**, see §11 Q-05 |

---

## §5 Interfaces and frozen areas

### Frozen by external definition

The beacon recipe is not ours to choose. It is fixed by drand and by the Soroban host:

- message = `sha256(round as u64 big-endian)`
- DST = `BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_`
- seed derivation = `HMAC-SHA256(SALT, seed)`, `SALT = sha256("Public Global Stellar Network ; September 2015")`
- shuffle = Fisher-Yates of `rand` 0.8.5 over `ChaCha20Rng` of `rand_chacha` 0.3.1

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
| rustc / cargo | 1.97.1 | installed toolchain; **no pin file in the repo** (§11 Q-06) |
| wasm target | `wasm32v1-none` | installed; `stellar contract build` requires it |
| Stellar CLI | 28.1.0 | installed; **not pinned** (§11 Q-06) |
| Node | 24.14.1 | installed; **no `.nvmrc`** (§11 Q-06) |
| `rand` (inside the Soroban host) | 0.8.5 | pinned by `rs-soroban-env`; the off-chain verifiers depend on this |
| `rand_chacha` (inside the host) | 0.3.1 | same |

Open `[VERIFY]` items: none outstanding — every external fact relied on today is logged in `docs/SOURCES.md`.

---

## §7 Security and secrets

### Trust model

| Party | What it can do | Mitigated how |
|---|---|---|
| drand threshold network | produce the entropy | public, scheduled, unpredictable before its round; a round earlier than the roster commitment is refused by INV-06 `[proposed]` |
| Stellar validators | bias `env.prng()` if it were used alone | not used alone: the PRNG is reseeded from the beacon value. The SDK states the base seed is under validator control and must not be relied on for unpredictability |
| whoever publishes the roster | choose who is eligible | public contestation window; **unresolved** — see §11 Q-07 |
| whoever supplies the beacon public key | forge any round | **unresolved today**: the key is a call parameter, not a constant. See §11 Q-05 |

### Secrets

No secrets in the repository. Testnet operations use the local Stellar CLI identity `cautela`; its key lives in the CLI's own config, outside this tree. No mainnet keys, endpoints or IDs anywhere.

### Environments

Testnet only. No mainnet target exists and none is in scope for this milestone.

---

## §8 Gates

Run from a clean checkout. The baseline below was recorded on 2026-10-25 **before any change**, per `CLAUDE.md` Bootstrap step 3.

| # | Command | Baseline | Note |
|---|---|---|---|
| G1 | `cargo fmt --all --check` | **FAIL** | `contracts/beacon-verifier/src/test.rs` is not rustfmt-clean (generated code) |
| G2 | `cargo test --locked --workspace` | **PASS** | 10 tests, all in beacon-verifier |
| G3 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | **FAIL** | 5 uses of deprecated aliases `G1Affine` / `G2Affine` in `contracts/beacon-verifier/src/lib.rs` |
| G4 | `stellar contract build` | **PASS** | beacon-verifier, 2,182 B optimised |
| G5 | `npm test` | **FAIL** | deliberate placeholder until T-006 replaces it |

**Missing gates** (proposed as tasks, not added silently):

- No test runs `scripts/verificar_sorteio.py`. Its agreement with the contract is asserted only from the Rust side.
- No test runs `web/sorteio.mjs`. It was validated outside the repository and that validation is not reproducible here.
- `CLAUDE.md` requires every published number to be reproducible by a gate. The figures in `docs/evidencia-testnet-sorteio.md` are produced by G2 but are not checked against the document.

---

## §9 Milestones and tasks

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
| T-003 | Pin the toolchain | T-000 | §6, §11 Q-06 | todo | `rust-toolchain.toml` and `.nvmrc` present; G1–G4 pass from a clean checkout |
| T-004 | Green the baseline: fmt and clippy | T-003 | §8 | todo | G1 and G3 pass; no behaviour change; deprecated aliases replaced |
| T-005 | Test the Python verifier in the repository | T-003 | §4 INV-04 | todo | A gate runs `scripts/verificar_sorteio.py` against the golden vectors and fails if they diverge |
| T-006 | Test the JavaScript verifier in the repository | T-005 | §3 B-02 | todo | A gate runs `web/sorteio.mjs` against the same vectors; `npm test` stops being a placeholder (replaces G5) |
| T-007 | Pin the beacon public key | T-004 | §4 INV-10, §7 | todo | The key is a constant; `verify` no longer accepts it as a parameter; a test proves a wrong key cannot verify any round |
| T-008 | Enforce the roster's minimum round | T-007 | §4 INV-06 | todo | A draw against a round earlier than the roster's definitive ledger is rejected |

### Event track — from the Speedrun SHIP deck

Ordered by the clock, not by dependency. These block the submission; T-001 to T-008 do not.

| ID | Title | Deps | Reads | Status | Acceptance criteria |
|---|---|---|---|---|---|
| T-009 | Name the project and write the one line | — | §11 Q-04, Q-10 | review | **Pinakion** · *"Verifiable jury selection for governance on Stellar."* 7 words. Still to do: carry the same words into README line 1, the landing hero and the pitch opening (T-015, T-016) |
| T-010 | Write `THESIS.md` | T-009 | §1 | review | Present at the repository root: the guarantee, who it is for, the problem with two documented cases, why Stellar with measured figures, what is proven and what is not |
| T-011 | Write `AGENTS.md` | T-010 | §0 | review | Present at the root, in the deck's six sections. Defers to `CLAUDE.md` where the two disagree, and to §9 here for scope |
| T-012 | Write `DESIGN.md` | T-010 | — | review | Present at the root: tokens for both themes, three type roles with a scale, the mark as inline SVG, layout and voice. Derived from the thesis — cool stone, oxidised bronze, and the machine's own black and white balls as the semantic pair |
| T-017 | Bring `web/index.html` onto the design tokens | T-012 | `DESIGN.md` | todo | The page uses the `DESIGN.md` fonts and `:root` tokens; no literal colours; both themes checked. Declared divergence in `DESIGN.md` is removed |
| T-013 | Public repository | T-011, T-015 | §11 Q-04 | todo | All six documents and `LICENSE` present (done); remote created and pushed (**not done** — no remote exists) |
| T-014 | Spec tasks become issues | T-013 | §9 | todo | Each open task is a GitHub issue with an owner; one branch and one PR per issue |
| T-015 | Rewrite `README.md` | T-010 | §2 | review | Opens with the one-liner; how to run it; every claim traced to §2; limits stated before the instructions |
| T-016 | Write the demo path and record the video | T-012, T-015 | §2 | todo | One flow, live on testnet, with the video as backup. 3 minutes in the deck's four parts |

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

---

## §11 Open questions

| ID | Question | Blocks |
|---|---|---|
| Q-01 | ~~No `LICENSE`, and `package.json` declared ISC.~~ **Answered 2026-10-25: MIT.** It is the licence of the human's previous project and the ecosystem norm for contracts and applications (OpenZeppelin stellar-contracts, script3/soroban-governor). Apache-2.0, used by the SDF's own repositories, was the alternative and adds an express patent grant. | — |
| Q-02 | §1 states the goal as inferred. Is the milestone goal correct as written, and are the non-goals right? | T-001 |
| Q-03 | ~~What happens to the withdrawn generation?~~ **Answered 2026-10-25:** moved out of the repository to `../Cautela-geracao-retirada/`, with a note explaining that the thesis changed and the work did not fail. Nothing deleted; the Groth16 verifier still has green tests and a testnet deployment there. | — |
| Q-04 | ~~The active project has no name.~~ **Answered 2026-10-25: Pinakion.** The πινάκιον was the bronze ticket bearing a citizen's name, slotted into the kleroterion before an Athenian jury draw. Chosen over *Kleroterion* because `kleros` is the root of an existing onchain jury protocol. Still open: does this repository get renamed, or does Pinakion start a new one? | T-013 |
| Q-05 | `verify` takes the beacon public key as a parameter. Anyone can pass their own key and have any round verify. Confirm this is pinned before anything depends on it. | T-007 |
| Q-06 | No toolchain pins exist (`rust-toolchain.toml`, `.nvmrc`). Which versions are the project's? | T-003 |
| Q-07 | Who publishes the first roster, and under what authority? This is unresolved across three design documents. | — |
| Q-08 | Documents are written in Portuguese; code identifiers are mixed Portuguese and English. What is the convention? | — |
| Q-09 | `docs/whitepaper-v0.1.md` states the Soroban CPU ceiling as 100M. A measurement from a previous project of the human's, cited in conversation, gives 400M by bisection on testnet. The two have not been reconciled and no figure in §2 depends on either. | — |
| Q-10 | ~~What is the one line?~~ **Answered 2026-10-25:** *"Verifiable jury selection for governance on Stellar."* Seven words, in the deck's own shape — category plus where. *"Nobody chooses who judges you."* is kept as the guarantee: it opens the pitch and sits under the landing hero, but it is not the one-liner, because it does not say what the thing is. | — |
| Q-11 | Which track and which category is the submission? The eligibility question raised in the human's own research — Genesis describes itself as for builders with limited Stellar experience, and the human has a previously awarded Soroban contract — was never answered by the organisers. | T-009 |
| Q-12 | ~~`AGENTS.md` versus `CLAUDE.md`.~~ **Resolved 2026-10-25 as proposed:** `AGENTS.md` carries the deck's six sections and states that `CLAUDE.md` wins where the two disagree. Neither duplicates the other. Confirm you are happy with that precedence. | — |
| Q-13 | `DESIGN.md` chooses Cormorant Garamond for display, replacing the Newsreader used by `web/index.html`. That page was built first and now diverges from the system. Confirm the direction before T-017 changes a page that works. | T-017 |
