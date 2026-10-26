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
- Does not count votes, run a ballot or deliberate: the vote is the author's earlier module, Tessera (DEC-019, DEC-036). Since M3 it **does** carry the case around the draw — the opening, the record, attendance and quorum, the binding of a Tessera vote, and the verdict under the level's rule (`contracts/processo`, DEC-035). It still does not decide the sanction or apply it. Plan B (DEC-015) adds *who sits is sealed until the verdict and public afterwards*; it never adds *how the panel votes*.

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
| The roster is a classic asset frozen by a locked issuer; the contract created each trustline through SAC `trust` | txs `b104d57e…` (flags), `7f9279a7…` (admin to the contract), `77808909…` (master weight 0), seven `enroll` txs; `docs/evidencia-testnet-bilhete.md` |
| `lock()` fixes the beacon round from the ledger clock, not from the caller | T-029, on testnet: tx `8a98bcf6…` → `rodada_minima 33385833` from `tempo 1792960862` (a minimum, under the pre-T-033 contract). Since T-033 the round is exact — first round after the lock plus `MARGEM_DE_RODADAS` = 10 — and `seal` uses it once: tests `a_rodada_vem_do_fechamento_com_margem`, `selar_com_outra_rodada_e_recusado`, `selar_duas_vezes_e_recusado`. **Not yet on testnet** (T-040) |
| An impeded juror is skipped and the next in the drawn order sits; the off-chain panel equals the on-chain one, address for address | txs `c1a2f592…`, `fc4a5f93…`; `scripts/verificar_painel.py`; golden vector 4 |
| A seat is proven by a Groth16 proof (BN254, Poseidon) verified on testnet, and the claim event carries no address | run `t040b`, 2026-10-26: six `claim` txs from six fresh accounts (`docs/evidencia-testnet-bilhete.md`); test `nenhum_evento_liga_y_a_endereco`. Approved by the human 2026-10-26 (DEC-029) |
| The sealed panel is recomputable from the ledger and drand alone, **among those who revealed** | `scripts/verificar_painel.py` on that run's events: `CONFERE`, 77 checks; G6 re-checks the committed record offline. Approved 2026-10-26 |
| Before reveal, no seat is tied to a roster address on the ledger: seats are **indistinguishable within the panel of k**, as far as the seat account's funding leaves no trail | the run's events (`docs/evidencia-testnet-registro.json`); INV-17. The qualifier is part of the claim. Approved 2026-10-26 |
| A seat claim costs about 32 M instructions declared on testnet, 8 % of the 400 M per-transaction ceiling | the cost table from the 53 transactions (G9); ceiling read with `stellar network settings` (`docs/SOURCES.md`). Approved 2026-10-26 |
| Verifying a drand round on Soroban was done before this project | `AshFrancis/splicers`, `NibrasD/Stellar-VRF` — stated wherever the beacon is claimed |
| A whole case ran on testnet, from the roster to the verdict: the party impeded at lock, the record filed by hash, attendance from seat accounts, a Tessera vote bound to the attendees, the verdict from Tessera's score | run `caso1`, 2026-10-26 (`docs/evidencia-processo.md`, `docs/evidencia-processo-hashes.tsv`); `scripts/verificar_processo.py` on the recorded events, re-checked offline by G6 and G5 (T-049, T-050). Asked for by the human, M3 |
| The vote runs on Tessera, the author's earlier module, used as it is; the case accepts a vote only if Tessera's voter list is exactly the attendees | Tessera contract at `356c22f` deployed unchanged (`CDWBWAHU…`); CLI with `scripts/tessera-cli.patch`; `vincular_votacao` recomputes Tessera's root on-chain (`raiz_dos_aptos_bate_com_a_do_tessera`, INV-25). **Must be stated wherever the vote appears** (DEC-036) |

**Must not claim** (no evidence today): secure, audited, production-ready, trustless, unbiasable. Under Plan B (DEC-015), also never *anonymous* — the word is *indistinguishable within the panel of k* — and never *full recomputation*: the panel is recomputable **among those who revealed**, and the silence of those who did not is itself on the ledger. Plan B ran on testnet on 2026-10-26 (T-040); the four Plan B rows above are what may be said of it, and nothing beyond them — never *anonymous*, never *full recomputation*, never *first* Groth16 on Soroban (Tessera measured one before; `splicers` and `Stellar-VRF` verified drand before). The entropy source is trusted (§7), the contract is unaudited, and nothing has run on mainnet.

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
      │ enroll(juror, c = Poseidon(s))             │ signs (opt-in); keeps s
      ▼                                            ▼
  kleroterion (Soroban) ── SAC: trust · set_authorized(true) · mint 1 ──► PINAKION trustline
      │    └─► leaf i := c; path to the root rebuilt (Poseidon, depth 6)   (T-035)
      │ impede(juror) ──► SAC: set_authorized(false); leaf := 0   (recusal, art. 463; before lock only)
      │ lock(level, reveal_after, party) ──► ledger, timestamp, drand round, k (from the level), root, N;
      │                        the party impeded if enrolled; suspended(party) until a window; nobody in or out after (T-045)
      │ seal(sig)     ──► once, with the round lock stored: seed = randomness >> 8, T = ⌊r·2k/N⌋
      │ claim(seat, proof, y)  from a fresh seat account: y < T, unseen, proof valid  (T-038)
      │ recuse(juror, s)       openly, before finalize: y out, next y moves up
      │ finalize()             after the window: the k smallest y sit; < k widens T once, then incomplete
      │ reveal(juror, s)       after reveal_after: Poseidon(s) = c, y = Poseidon(seed, s); seated? ticket?
      ▼
  panel (addresses)             any Horizon: GET /accounts?asset=PINAKION:issuer
```

The issuer account sets `AUTH_REQUIRED` and `AUTH_REVOCABLE`, hands the SAC admin to the contract, and then sets its master weight to 0. After that no ticket can be issued or authorized except by the contract, which has no upgrade path and refuses enrolment and impediment after `lock()`.

### Components observed

| Path | What it is |
|---|---|
| `contracts/beacon-verifier/` | Soroban contract: `verify`, `randomness`, `draw` |
| `contracts/kleroterion/` | Soroban contract (T-029, T-033, T-035, T-037, T-038): the roster as a classic asset, drawn sealed — `enroll(juror, c)`, `impede`, `lock(gravidade, reveal_after, acusado)` (T-045; `lock(k, reveal_after)` before), `suspended`, `levels`, `seal(sig)`, `panel`, `commitment`, `root`, `verifier`, `seal_state`, `claim`, `claims`, `recuse`, `finalize`, `reveal`, `revelation`; the open `draw` copy is gone (DEC-016); maintains the Poseidon commitment tree and emits `Inscrito`, `Impedido`, `Fechado`; assembles a seat proof's public inputs and calls the verifier fixed at construction (T-037); carries its own copy of `verify`/`randomness` |
| `contracts/poseidon/` | library crate (T-034): Poseidon(1) and Poseidon(2) over BN254, circomlib-compatible, on the CAP-75 host permutation; adapted from `stellar/rs-soroban-poseidon` (Apache-2.0, DEC-021) |
| `contracts/groth16-verifier/` | Soroban contract (T-034, T-037): the author's BN254 Groth16 verifier, ported (DEC-017), with the T-036 `vk` embedded (`src/vk.rs`, generated by `scripts/gerar_vk_rs.py`); `verify(proof, public)`, `vk()` |
| `circuits/` | **GPL-3.0** (DEC-021): `pinakion_claim.circom` (public `[root, seed, y, claimer]`, depth 6); `setup/` — r1cs, witness wasm, final zkey, `vk.json` and `TRANSCRICAO.md` of the phase-2 setup (T-036, DEC-024); `setup.sh` that produced them; `pinakion.mjs`, the `prove` CLI; `test.mjs` (G8); the vector and fixture generators; the superseded spike prover `provar.sh` |
| `scripts/verificar_sorteio.py` | independent recomputation of the draw, no dependencies |
| `scripts/verificar_painel.py` | the sealed panel recomputed from the contract's events and drand alone: root, seed, threshold, every revealed `y`, the order, the panel, and each juror as *sentou · revelou, nao sentou · sorteado e ausente · nao revelou · recusado · impedido* (T-039); the pre-T-038 open-draw mode is kept for the old testnet evidence |
| `web/sorteio.mjs` | the same recomputation, for browser and Node; `verificarSelado` is the sealed one (T-039) |
| `web/bilhetes.html` | read-only page (T-042): loads the committed testnet record, runs `verificarSelado` in the browser, shows each roster account as *inscrito · impedido · recusou · sentou · revelou · em silêncio* with tx links, and the root, round, seed, `T` and the claims with their seat accounts |
| `scripts/exportar_tessera.py` | the panel's seat accounts in Tessera's `--aptos` format (T-042) |
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
| INV-04 | The on-chain panel equals the independent off-chain recomputation | `beacon-verifier`: `bate_com_a_recomputacao_fora_da_cadeia` (golden vectors). `kleroterion` no longer draws in the open since T-038: its form is INV-04′ |
| INV-05 | `randomness = sha256(signature)` | `aleatoriedade_e_sha256_da_assinatura` |
| INV-06 | No draw uses a beacon round earlier than the roster's definitive ledger | `draw_round` (T-026) against a caller-given minimum in `beacon-verifier`; **fully** in `kleroterion`, where the round is computed at `lock()` from the ledger timestamp and is not a parameter (T-029; exact since T-033, see INV-13): `a_rodada_vem_do_fechamento_com_margem`, `selar_com_outra_rodada_e_recusado` |
| INV-07 | The attendance count never decreases because of an impediment (art. 463 §2) | sealed mode (T-038): an impediment happens before `lock` and only lowers `N`; a recusal removes one `y` and `finalize` still seats `k` while `k` claims remain — `recusar_tira_o_y_e_o_proximo_senta`; fewer than `k` widens `T` once (`com_menos_de_k_o_limiar_se_amplia_uma_vez`), then the panel is recorded incomplete (`ampliado_e_ainda_menos_de_k_fica_incompleto`) |
| INV-08 | A substitute always comes from the already-drawn order; there is no re-draw for substitution | sealed mode: the order is the ascending `y` fixed by the seed; a recusal hands the seat to the next `y` of the same list — `recusar_tira_o_y_e_o_proximo_senta`, `fluxo_completo_n8_k3` (T-038) |
| INV-09 | The contract refuses to operate below the minimum roster size | `fechar_abaixo_do_minimo_e_recusado` (T-029) |
| INV-10 | The beacon public key is a pinned constant, not a parameter | `assinatura_de_outra_chave_nao_verifica`, `a_chave_do_beacon_nao_e_parametro` |
| INV-11 | The roster is committed before the case: after `lock()` nothing can be enrolled, and the issuer cannot mint or authorize once its master weight is 0 | `inscrever_depois_de_fechar_e_recusado` (T-029); the issuer half is a classic ledger fact, proven on testnet in T-031 |
| INV-12 | A juror sits only if authorized at `lock()`: the leaf is the juror's `c`, an impediment zeroes it, and no proof exists for a zero leaf. *Rewritten by the human, 2026-10-25 (DEC-026):* under Plan B the ticket balance cannot be checked at claim without naming the juror, so it is no longer a condition to sit; a seated juror who no longer holds the ticket is recorded as such at `reveal` | `o_impedido (folha 5) nao tem assento a provar` (G8), `a_prova_nao_vale_em_outro_rol`, `sentou_sem_bilhete_fica_registrado_no_reveal` (T-038). Before T-038: `balance ≥ 1` and SAC `authorized()` at draw time, `sem_bilhete_nao_senta` — retired with the open draw |
| INV-13 | The beacon round is fixed at `lock()` from the ledger clock and `seal` accepts that round once; no caller chooses a round and no second draw exists | `a_rodada_vem_do_fechamento_com_margem`, `fechar_fixa_ledger_tempo_rodada_k_e_revelacao`, `selar_com_outra_rodada_e_recusado`, `assinatura_forjada_e_recusada`, `selar_duas_vezes_e_recusado` (T-033). `seal` here is the open draw; T-038 makes it sealed |
| INV-14 | Nothing changes in the roster after `lock()`: no `enroll`, no `impede` | `inscrever_depois_de_fechar_e_recusado`, `impedir_depois_de_fechar_e_recusado` (T-033; Q-21 closed) |
| INV-15 | A seat is claimed only with a valid proof against the root and seed the contract holds; one `y` per leaf per seed, so at most one seat per juror | `a_prova_da_cli_vale_contra_a_raiz_do_contrato`, `outro_y_outra_semente_ou_outra_conta_nao_vale`, `um_byte_da_prova_alterado_nao_vale`, `a_prova_nao_vale_em_outro_rol` (T-037); `prova_com_outro_y_ou_de_outra_conta_e_recusada`, `y_repetido_e_recusado`, `y_acima_do_limiar_e_recusado` (T-038 — the seed now comes from `seal`) |
| INV-16 | The panel is immutable after `finalize()`; `reveal` is evidence, never a correction | `finalizar_senta_os_k_menores_e_nao_muda_mais`, `revelar_confere_c_e_y_e_nao_muda_o_painel`, `fluxo_completo_n8_k3` (T-038) |
| INV-17 | No event links a `y` to a G-address before `reveal`; a commitment `c` is never accepted twice, in any lock | `nenhum_evento_liga_y_a_endereco` — `Reivindicado` carries `y` alone; `Recusado` and `Revelado` carry the address with `s` and `y`, because each is the juror showing themself — a recused `y` never sits (T-038, T-039); `compromisso_repetido_e_recusado`, `compromisso_de_impedido_nao_volta` (T-035). The seat account is still the transaction's source: unlinkability rests on it being fresh and funded without a trail (README, T-041) |
| INV-04′ | Under Plan B, INV-04 splits: each seat is verifiable at `claim`; the panel is recomputable **among those who revealed**, and the non-revealers are listed by address | `scripts/verificar_painel.py` (G6) and `web/sorteio.mjs` `verificarSelado` (G5) recompute root, seed, threshold, every recused and revealed `y`, the order and the panel from the events and the drand randomness alone, and reach the sealed golden vector's `esperado`; `bate_com_o_vetor_selado` proves the contract emits exactly those events (T-039). Altering one `y` turns G2, G5 and G6 red (checked 2026-10-25) |
| INV-18 | The person judged never sits on their own case: `lock` names them and, if enrolled and authorized, impedes them in the same call, before the root is fixed; the clerk cannot be the party. *Approved by the human 2026-10-26 (Q-28).* | `o_acusado_inscrito_e_impedido_no_fechamento`, `a_parte_nao_prova_assento_no_proprio_caso`, `escrivao_acusado_e_recusado`, `acusado_ja_impedido_ou_fora_do_rol_nao_muda_o_rol` (T-045); both verifiers check that an enrolled party has an `Impedido` before `Fechado` and never sits, and a mutated vector fails by name (G5, G6) |
| INV-19 | `k` comes from the severity table fixed at construction; nobody chooses `k` at `lock`; below the level's roster floor, `lock` is refused (INV-09 per level). *Approved 2026-10-26.* | `k_vem_da_gravidade`, `rol_abaixo_do_minimo_do_nivel_e_recusado`, `gravidade_fora_da_tabela_e_recusada`, `tabela_vazia_ou_k_zero_e_recusada`, `constituir_publica_a_tabela_e_o_afastamento` (T-045); the verifiers check `k` against `Constituido` |
| INV-20 | `suspended(a)` is true only for the party, from `lock` to `lock + afastamento` (exclusive), and nothing extends it. *Approved 2026-10-26.* | `afastado_so_para_o_acusado_e_so_no_prazo` (T-045); the verifiers check `afastado_ate` against `Constituido` when the record carries the lock's ledger |
| INV-21 | `revela_em ≥ lock + espera_minima`; what `reveal` reads lives at least until `revela_em + janela`, or the network maximum if sooner. *Approved 2026-10-26.* | `revelar_antes_da_espera_minima_e_recusado`, `o_que_o_reveal_le_vive_ate_a_revelacao` (red without the extension, checked 2026-10-26), `a_extensao_para_no_teto_da_rede` (T-046); the verifiers check `revela_em` against `Constituido.espera_minima` when the record has the lock's ledger |
| INV-22 | A case opens only after the roster locked, for the lock's party, level and `k`; never twice; never with `k` below the vote's privacy floor | `abrir_le_o_caso_do_kleroterion_e_registra_a_denuncia`, `abrir_antes_de_o_rol_fechar_e_recusado`, `painel_menor_que_o_sigilo_e_recusado` (T-048); the verifiers check the opening against `Fechado` |
| INV-23 | Only a seat account that sat attends, once; the session opens exactly when the count reaches the quorum (the panel's majority, never below the privacy floor); without it in time, the case is adjourned | `so_quem_sentou_comparece_e_uma_vez`, `o_quorum_instala_a_sessao_e_quem_chega_depois_conta`, `sem_quorum_no_prazo_o_caso_e_adiado`, `quorum_e_votos_exigidos_seguem_a_regra`; `seated_so_depois_do_painel_e_so_para_quem_sentou`, `a_mesma_conta_de_assento_nao_reivindica_duas_vezes` (T-047) |
| INV-24 | Each role files only what it may: the clerk evidence (before the vote) and minutes; the person judged their defence (until the verdict); an attending juror minutes (until the verdict); nobody else | `cada_papel_registra_so_o_que_lhe_cabe`, `depois_da_votacao_nao_entram_provas_e_depois_do_veredito_so_a_ata_do_escrivao` |
| INV-25 | A vote binds only if it is a Tessera proposal of one secret two-option question, whose voter root is exactly the attendees' (in order, under Tessera's rule, recomputed on-chain), whose committee are attendees, opened after the session | `a_votacao_vinculada_tem_de_ser_dos_presentes`, `raiz_dos_aptos_bate_com_a_do_tessera` (the rule against a root Tessera's CLI wrote on testnet) |
| INV-26 | The verdict is Tessera's published score under the level's rule — upheld iff yes reaches a majority of the whole panel, or two thirds from the configured level; absence never counts in favour; the interim suspension ends at the verdict | `a_maioria_e_os_dois_tercos_contam_sobre_o_painel`, `sem_placar_nao_ha_veredito`, `o_afastamento_termina_no_veredito` |

---

## §5 Interfaces and frozen areas

### Frozen by external definition

The beacon recipe is not ours to choose. It is fixed by drand and by the Soroban host:

- message = `sha256(round as u64 big-endian)`
- DST = `BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_`
- seed derivation = `HMAC-SHA256(SALT, seed)`, `SALT = sha256("Public Global Stellar Network ; September 2015")`
- shuffle = Fisher-Yates of `rand` 0.8.5 over `ChaCha20Rng` of `rand_chacha` 0.3.1
- quicknet genesis = `1692803367`, period = `3` s; round `r` is published at `genesis + (r-1)·period`; the first round published strictly after a lock at time `t` is `⌊(t−genesis)/period⌋ + 2` (checked 2026-10-25 against `api.drand.sh/<chain>/info`); `kleroterion` draws with that round plus `MARGEM_DE_RODADAS` = 10 (T-033, DEC-020)

Changing any of these breaks every off-chain verifier. A change requires a DEC and regenerated golden vectors.

### Golden test data

`contracts/beacon-verifier/src/test.rs` — the vectors in `bate_com_a_recomputacao_fora_da_cadeia`, produced by `scripts/verificar_sorteio.py`. `contracts/*/test_snapshots/` are generated by the SDK.

Since T-034/T-035, the reference is circomlib — the circuit's own arithmetic — and three implementations are held to it: `tests/vetores-poseidon.json` (22 Poseidon vectors, `circuits/vetores_poseidon.mjs`) and `tests/vetores-arvore.json` (4 commitment trees, N ∈ {1, 2, 17, 64}, as event lists and roots, `circuits/vetores_arvore.mjs`), read by the contract's tests (G2), `scripts/arvore.py` (G6) and `web/arvore.mjs` (G5). Since T-039, `tests/vetores-de-ouro.json` also holds `selados`: the exact event list of the N = 8, k = 3 flow (`scripts/gerar_vetor_selado.py`), replayed by the contract (`bate_com_o_vetor_selado`) and recomputed by both verifiers. `tests/poseidon-bn254.json` is the contract's constants exported for the off-chain code; G6 re-derives it from `params.rs` so the two cannot drift. Altering one root turns G2, G5 and G6 red (checked 2026-10-25).

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
| Node | 24.14.1 | **pinned** in `.nvmrc`. `npm test` still runs on builtins alone; the devDependencies below serve `circuits/` only |
| `soroban-sdk` feature `hazmat-crypto` | 29.0.1 | `contracts/poseidon` only — it exposes `poseidon_permutation` (CAP-75) |
| circom (compiler) | 2.2.2 | **not pinned** — a separate binary, like the Stellar CLI; recorded here. GPL-3.0 |
| circomlib, circomlibjs, snarkjs | 2.0.5, 0.1.7, 0.7.6 | **pinned** exact in `package.json` and `package-lock.json` (`npm ci`). GPL-3.0, used by `circuits/` only (DEC-021) |
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
| G2 | `cargo test --locked --workspace` | PASS, 10 tests | **PASS, 114 tests** | +2 from T-007, +4 from T-026, +22 from T-029, +3 from T-033, +11 from T-034, +10 from T-035, +1 from T-036, +9 from T-037, +12 net from T-038, +1 from T-039, +10 from T-045, +3 from T-046, +2 from T-047, +14 from T-048 (`processo`) (16 `beacon-verifier`, 56 `kleroterion`, 9 `groth16-verifier`, 4 `pinakion-poseidon`) |
| G3 | `cargo clippy --workspace --all-targets --locked -- -D warnings` | FAIL, 5 deprecated aliases | **PASS** | fixed by T-004 |
| G4 | `stellar contract build` | PASS | **PASS** | beacon-verifier, 2,576 B optimised; since T-048 also `processo` (17,130 B optimised) |
| G5 | `npm test` | FAIL | **PASS, 4 vectors, 22 Poseidon, 4 trees, 1 sealed** | was a placeholder; T-006 replaced it with `web/test.mjs`; T-035 adds Poseidon and the commitment root; T-045 adds two mutations of the sealed vector that must fail by name; T-050 adds the recorded case (`docs/evidencia-processo-registro.json`, the same checks as the Python verifier) and three mutations of it that must fail by name |
| G6 | `python3 scripts/conferir_vetores.py` | did not exist | **PASS, 4 vectors + the tie to `beacon-verifier`'s `test.rs`, 22 Poseidon, 4 trees, constants = `params.rs`, the N = 8 flow fixture, 1 sealed vector, the recorded testnet run (77 checks), the Tessera export = `docs/evidencia-testnet-aptos.txt`** | added by T-005; T-035 adds Poseidon, the root and the constants check; T-044 adds the six tables of the panel-size study; T-045 adds the same two mutations; T-050 adds the recorded case and the same three mutations; T-049 adds that `docs/evidencia-processo.md` is what `scripts/evidencia_processo.py` prints |
| G7 | `python3 scripts/conferir_numeros.py` | did not exist | **PASS, 34 figures in 7 documents** | added by T-023; T-035 adds the costs of `enroll`, `impede`, `lock` in this file; T-034 adds the `(cpu, mem)` constants of `contracts/poseidon` and `contracts/groth16-verifier` and `docs/evidencia-testnet-spike.md` |
| G8 | `node circuits/test.mjs` (`npm run test:circuito`) | did not exist | **PASS, 19 checks** | added by T-036 (T-037 adds: the key embedded in the verifier is `vk.json`): the CLI's path reaches circomlib's roots; a real claim verifies; wrong root, seed, claimer or another juror's `y` are refused; no witness exists for a false root or `y`; an impeded or unknown `s` gets no proof; the Rust fixture uses this setup's `vk`. Needs `npm ci`; no circom, no ptau |
| G9 | `python3 scripts/custos_testnet.py docs/evidencia-testnet-hashes.tsv` | did not exist | **network, not per task** | added by T-040: reprints the cost table in `docs/evidencia-testnet-bilhete.md` from the 53 testnet transactions (declared resources, fee charged). Needs the Stellar CLI and RPC access; run when that page changes |

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
| T-029 | `kleroterion`: the roster as a classic asset | T-026, T-028 | §3, §4 INV-06..12, §5, DEC-014 | review | New contract. `enroll` creates the trustline through SAC `trust`, authorizes and mints one ticket; `lock` records ledger and timestamp, computes the minimum round and refuses below the roster minimum; `impede` revokes authorization; `draw_round` refuses before lock and below the minimum, verifies the beacon, shuffles the roster order and seats the first `k` with ticket and authorization. Golden vectors still match. All tests written first and seen failing |
| T-030 | Verifiers recompute the panel from the roster | T-029 | §3 B-01, B-02, §5 | review | `scripts/verificar_painel.py` and `web/sorteio.mjs` take the ordered roster, the impeded set and `k`, and return the same panel as the contract; a fourth golden vector with addresses; G5 and G6 cover it |
| T-031 | The classic sequence on testnet | T-029 | §7, DEC-014 | review | Issuer with `AUTH_REQUIRED`+`AUTH_REVOCABLE`, SAC admin handed to the contract, master weight set to 0; seven jurors enrolled, lock, one impediment, one draw; every hash in `docs/evidencia-testnet-bilhete.md`; Horizon `/accounts?asset=` shows the roster; the sort key of that endpoint recorded (the `[VERIFY]` in `docs/SOURCES.md`) |
| T-032 | Say it everywhere the ticket is an asset | T-029 | §0, §2 | review | `THESIS.md`, `README.md`, the landing and `docs/DEMO.md` describe the ticket, the lock and the two Unlocked bullets it answers; every claim traced to §2; no *first*/*novel* |

### M1 — Plan B: sealed sortition, public after the verdict (DEC-015)

The plan is written up in full, ordered from most to least important, in the doc *Pinakion — Plano B: sortição selada* (https://claude.ai/artifact/9WDXFX9eSJWjhWUdRmrh2N, 2026-10-25); this board holds the acceptance criteria. Estimated at about 10 days of work (an estimate, not a measurement); the hacking window closes 2026-10-26 12:00, so M1 does not ship in this submission — the submission says it is designed and not built. **Kill criterion:** without one proof of our circuit verified on testnet within budget at the end of T-034, M1 stops and the product is the open draw with T-033 only.

| ID | Title | Deps | Reads | Status | Acceptance criteria |
|---|---|---|---|---|---|
| T-033 | Close the two doors | T-029 | §4 INV-13, INV-14, §11 Q-21 | review | `lock(k, reveal_after)` computes and stores the target drand round (first round published strictly after the ledger time, plus a margin ≥ 10 rounds) and refuses `N < k`; `enroll` and `impede` refused after lock; `draw_round(round, sig, k)` replaced in `kleroterion` by `seal(sig)`, which takes no round and no `k`, uses the stored ones, runs once and stores the panel (DEC-020, amended by the human 2026-10-25; the open draw with a caller-given round stays in `beacon-verifier`); tests written first and seen failing; G1–G7 green |
| T-034 | Spike kill-or-go: Poseidon compatibility and one proof on testnet | T-033 | §11 Q-22, Q-09 | review | ≥ 20 Poseidon(1)/Poseidon(2) vectors from circomlib JS match the contract's sponge over the host permutation (CAP-75); the ported BN254 verifier verifies one proof of the current circuit **on testnet**, cost measured and recorded with the tx hash; the per-tx CPU ceiling checked at `developers.stellar.org` and logged; a DEC records go or kill |
| T-035 | Juror commitment and Poseidon tree | T-034 | §3, §4 INV-17, §5 | review | `enroll(juror, c)` keeps today's SAC path and stores `c` bound to the address; duplicate `c` in a lock and any `c` seen in a previous lock (`used_c`) refused; Merkle tree of depth 6 over authorized holders built by the contract, never a posted root; root frozen at `lock` with `N`, round and `reveal_after`; root recomputed in Python and JS from events for N ∈ {1, 2, 17, 64} with one `impede` before lock |
| T-036 | Circuit and Groth16 setup | T-034 | §5, DEC-017 | review | `pinakion_claim.circom` ported from the author's 2026-10-25 `role_membership.circom`: public `[root, seed, y, claimer]`, private `s` + path, depth 6, `y = Poseidon(seed, s)`; public ptau 2^12, single-party phase 2 with published transcript, `vk` exported; CLI `pinakion prove` on snarkjs; negatives (wrong root/seed/claimer) rejected; origin logged in `docs/SOURCES.md` *Copied code* |
| T-037 | On-chain verifier | T-036 | §3 B-01, §5, §8 | review | The 2026-10-25 BN254 Groth16 verifier ported as a separate contract with the new `vk`; `kleroterion` assembles the public inputs itself (`root`, `seed` from state, `claimer = SHA-256(address) >> 8`); valid proof passes, one altered byte fails, swapped inputs fail; cost asserted in `test.rs` as today's costs are (G7) |
| T-038 | `seal`, `claim`, `finalize`, `reveal`, `recuse` | T-035, T-037 | §3, §4 INV-15..17 | review | `seal(sig)` verifies the stored round once and fixes `seed` and `T = ⌊p·2k/N⌋`; `claim(proof, y)` from a seat account: `y < T`, `y` unseen, ordered insert; `finalize` seats the `k` smallest `y`, widens to `T := p` once if fewer than `k`, then records *incomplete*; `reveal(s)` after `reveal_after` checks `Poseidon(s) = c` and `Poseidon(seed, s) = y`; `recuse` public, before `finalize`, by the main account; no event carries a G-address beside a `y` before reveal; full flow in a unit test with N = 8, k = 3, one non-claimant, one non-revealer |
| T-039 | Independent recomputation and golden vectors | T-038 | §3 B-02, §4 INV-04′, §8 | review | `scripts/verificar_painel.py` and `web/sorteio.mjs` read only the ledger and drand, recompute root, every revealed `y`, the order and the panel, and print *seated · revealed · not revealed · drawn-absent* plus match/mismatch; new vectors in `tests/vetores-de-ouro.json` (N = 8, k = 3); altering a `y` turns G5 and G6 red; Fisher-Yates vectors stay with `beacon-verifier` |
| T-040 | Testnet sequence and measured costs | T-038, T-039 | §7, §8 | review | `scripts/sequencia_bilhete.sh` runs issuer (with `AUTH_IMMUTABLE` set before weight 0) → 20 enrolls with commitments → one impede → lock → seal with a live drand round → claims from friendbot seat accounts, one duplicate refused → finalize → 18 reveals → one recuse → verifier output; a cost table per operation from the transactions' consumed resources, produced by a §8 command; `docs/evidencia-testnet-bilhete.md` rewritten with the new hashes |
| T-041 | Thesis, spec, README, claims | T-033 | §0, §1, §2, §4, §11 | review | `THESIS.md` carries the three-clause guarantee, one user (OSS project with a code of conduct; Rust 2021 lead), Athens corrected to *Ath. Pol.* 63–65 (two lots, no choosing — no claim of secrecy), the fourth *why Stellar* (CAP-74/75); the "regulated issuer" row removed; `README.md` first page states single-party setup, relayer, loss of `s`, reveal completeness, off-chain deliberation breaks the seal, testnet only; *Related work* names Tessera as the author's prior and complementary module; *Prior work* dates the ported code; G7 green; no word forbidden by §2 |
| T-042 | Ticket page, Tessera export, submission text | T-039, T-041 | §2, `DESIGN.md` | review | `web/` shows per person *enrolled · recused · seated · revealed · silent* with tx hashes, root, round, `seed`, `T`, read-only; README links stellar.expert and Horizon `/accounts?asset=`; seat-account list exported in Tessera's voter format (`[VERIFY: --aptos format]`); submission text follows the spine problem → user → what → why Stellar → what is not done |
| T-043 | Landing page and DESIGN tell the sealed draw | T-041, T-042 | `THESIS.md`, `DESIGN.md`, §2 | review | Asked by the human 2026-10-26. `DESIGN.md` follows `THESIS.md` (§0 order): audience led by the OSS project with a code of conduct, the regulated-institution audience removed; reading order purpose → the three sentences → the sealed testnet run → the open draw it began with → limits; a vocabulary for *inscrito · impedido · recusou · sentou · revelou · em silêncio* in shape and word, never colour alone; the rule that no screen shows a name beside a seat before reveal. `web/index.html`: hero and principles are the guarantee; a section on the 2026-10-26 run linking `bilhetes.html`; the Fisher-Yates check kept and labelled as the open draw; page-one limits; no word §2 forbids; checked in a browser in light, dark and at phone width; G1–G8 green |

### M2 — the case around the draw: severity, the party, late reveal (proposed 2026-10-26)

Asked by the human on 2026-10-26: a council decides more than expulsion, so severity should size the panel; the person judged should lose the powers to interfere while the case runs; the jurors should stay hidden longer (option B, *reveal late*). The study is `docs/desenho-gravidade-afastamento-revelacao.md`. **Phase gate:** the human approves INV-18 to INV-21 and the defaults (Q-28) before T-045.

| ID | Title | Deps | Reads | Status | Acceptance criteria |
|---|---|---|---|---|---|
| T-044 | Study: severity and panel size, interim suspension, late reveal | — | §4, `docs/limite-inferior-do-rol.md`, `docs/desenho-sorteio-e-sigilo.md` | review | `docs/desenho-gravidade-afastamento-revelacao.md` answers the three requests; every table is printed by `scripts/tamanho_do_painel.py` and G6 fails when one figure drifts; the re-check of `limite-inferior-do-rol.md` §4 matches; proposed invariants, defaults and the two build tasks written down; limits section; nothing built |
| T-045 | Severity table, the party impeded at `lock`, `afastado` | T-044, Q-28 | §4 INV-18..20, §5, the study §1–§2 | review | constructor takes the level table (`k`, roster floor) and `afastamento`, and emits `Constituido`; `lock(gravidade, revela_em, acusado)` takes `k` from the table, refuses below the level's floor and `acusado == escrivão`, impedes an enrolled party in the same call; `Fechado` carries `gravidade`, `acusado`, `afastado_ate`; `afastado(endereco)` true only for the party, only in the window; Python and JS verifiers check `k` against `Constituido` and that the party never sits; sealed golden vector regenerated; costs re-pinned; G1–G8 green |
| T-046 | Late reveal: a floor on `revela_em`, TTL to match | T-045, Q-28 | §4 INV-21, §6, the study §3 | review | constructor takes `espera_minima`; `lock` refuses `revela_em < lock + espera_minima`; `lock` and `finalize` extend the TTL of what `reveal` reads to `revela_em + janela`, capped at `max_ttl()`; a test advances the ledger past `min_persistent_ttl` and reveals; the two `[VERIFY]` of the study's §3 settled; `bilhetes.html` shows *sealed until ledger X*; G1–G8 green |

### M3 — the case cycle, end to end, with a demo link (2026-10-26)

Asked by the human on 2026-10-26: *build the full cycle — the opening of the case, the quorum, the records and minutes, and the vote — end to end, tested, good for a video and for a dapp link; deploy it yourself; do not stop to ask.* The human also asked that the vote be stated to run on Tessera, so the link to the author's earlier module is disclosed, not discovered (DEC-034, DEC-036). One branch and one PR for the milestone (DEC-034).

| ID | Title | Deps | Reads | Status | Acceptance criteria |
|---|---|---|---|---|---|
| T-047 | `seated(assento)` in kleroterion | T-046 | §4 INV-15, INV-17 | review | `claim` records the seat account's `y` and refuses a second claim from the same account; `seated` is true only after the panel and only for a seat account whose `y` sat; tests; costs re-pinned |
| T-048 | The `processo` contract | T-047 | §4 INV-22..26 | review | `abrir` (after `lock`, reading party, level and `k`; refuses `k` below the vote's privacy floor) · `comparecer` (seated seat accounts only, once, within the quorum window; the session opens at the quorum) · `adiar` · `registrar` (each role files only its types: clerk evidence and minutes, the party its defence, an attending juror minutes) · `vincular_votacao` (a Tessera proposal of one secret two-option question whose voter root is the attendees' under Tessera's rule, recomputed on-chain, with a committee of attendees, opened after the session) · `decidir` (Tessera's score; upheld iff yes ≥ the level's required votes over the whole panel) · `suspended` ends at the verdict; tests against a real kleroterion panel from the N = 8 fixture and a Tessera stand-in; the voter-root rule checked against a root Tessera's own CLI produced on testnet |
| T-049 | The whole case on testnet | T-048 | §7 | review | `scripts/sequencia_processo.sh` runs roster → lock against an enrolled party → complaint, evidence, defence → seal → claims → panel → attendance and quorum → juror's minutes → Tessera vote bound → secret votes → Tessera tally and its own verifier → verdict → minutes → reveals; every hash recorded; `docs/evidencia-processo.md` |
| T-050 | Check the case from the ledger | T-049 | §4 | review | `scripts/verificar_processo.py` and `web/processo.mjs` reach the same checks from the record alone: the draw (with `verificar_painel`), the opening, the record against the documents, attendance against the panel, Tessera's voter root against the attendees, the verdict against Tessera's score; G6 and G5 check the recorded run and fail on mutations |
| T-051 | The case page | T-050 | `DESIGN.md` | review | `web/processo.html`: verdict first, the case in five figures, nine phases each with what happened, how it is checked and its transactions, documents opened and hashed in the browser, the Tessera disclosure in the vote phase, a presentation mode (one phase per screen, keys, auto-advance) for the video; light, dark and phone width checked in a browser |
| T-052 | Landing, README, thesis, demo script, submission | T-051 | §2 | review | the landing page leads to the case; `THESIS.md`, `README.md`, `docs/SUBMISSAO.md` say what the cycle is and that the vote is Tessera's; `docs/DEMO.md` is the video script for the case; G1–G8 green |
| T-053 | The live case, run from the browser | T-052 | §7, `DESIGN.md` | review | Asked by the human 2026-10-26 (*a demo looks like a pitch deck; make it a dapp*). `web/vivo/`: one click opens a whole case on testnet from the tab — accounts, the ticket, both contracts deployed, 40 enrolments, the lock against an enrolled party, the record typed by the visitor, the drand seal, seat proofs made in the browser, the panel, attendance, a Tessera vote bound to the attendees, the visitor choosing the ballots, Tessera's time-locked tally, the verdict, the reveals; then the record is read back from the ledger and checked with `web/processo.mjs`; every step's transaction is linked as it lands; the case page opens the live case; the same engine runs from Node against testnet |
| T-054 | Study: how real councils meet and produce documents | — | §4 INV-23..26, `docs/desenho-sorteio-e-sigilo.md`, `docs/desenho-rol-sorteio-recusas.md` | review | `docs/desenho-como-os-conselhos-funcionam.md` answers how five families of council meet, who writes the record and what is published, each fact read in the primary source and logged in `docs/SOURCES.md` with its date; the eight divergences from what the contract already does are named; five proposals (P-1..P-5) written down, none built; the positive test for *a council that commits documents by hash* and for *an asynchronous rite* run and recorded in *Checked and not found*; limits section; nothing built, no gate added |
| T-056 | The case page's UX pass | T-051, T-053 | `DESIGN.md`, `PRODUCT.md`, `docs/DEMO.md` | review | Asked by the human 2026-10-26 (*melhore a UX/UI da demo*). `web/processo.html`: a live line under the verdict says what the browser did (record read, documents hashed, checks and their time on this machine); three ways in from the hero (the phases, Present, the live case); each phase's checks under one count; the seats phase draws every claimed `y` on the field against the threshold, filled where it sat; the quorum table names the attendance that installed the session; the verdict ruler has a legend; the rail compacts below 1100px and keeps the current phase in view; Present ends on the limits as a tenth screen, and its controls wrap on a phone; a failing check is named in Portuguese. `PRODUCT.md` written from the thesis, the design and §1–§2. No check, no number and no claim changes; `processo.mjs` untouched; G5–G7 green; verified in the browser in light and dark, at desktop and phone widths. No gate covers the page's markup; that gap is stated, not closed |

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

### DEC-015: Sealed sortition, revealed after the verdict (2026-10-25, M1)
Context: the judges read *Stellar Unlocked* as primitives that did not exist a year ago (the written bullets also reward old assets; both readings are in `docs/SOURCES.md`). Of what Pinakion uses, only CAP-73 is younger than a year, and it is a convenience. The only primitive on the table that makes something impossible a year ago possible is Groth16 on BN254 with Poseidon (CAP-74/75, Protocol 25, 2026-01-22). A roundtable (Tyler, Nicole, Justin; four rounds) tested where it is structural.
Decision: the juror registers `c = Poseidon(s)` at `enroll`; the contract builds the Poseidon Merkle root at `lock`; `seal` fixes the seed from the stored drand round; a juror with `y = Poseidon(seed, s) < T` proves membership and `y` in Groth16 from a fresh seat account; `finalize` seats the `k` smallest `y`; after `reveal_after` — set at `lock`, intended to fall **after the verdict** — each juror reveals `s`, and the panel is recomputable among those who revealed, with non-revealers listed. The guarantee gains two clauses: *nobody knows who until they have judged; everyone knows afterwards*. One mode only.
Alternatives: a secret ballot for the panel — rejected: it is the author's own Tessera, awarded the same week, and the vote is not Pinakion's job. A hidden roster — rejected: the roster is public on purpose. Sealed assignment to concurrent cases (the Athenian acorn) — rejected: a code-of-conduct body rarely has concurrent cases, and without a per-juror secret it is commit-reveal. Ship the open draw with the doors closed — kept as the fallback (T-033 alone) and the kill path.
Consequences: §1 non-goals, §2 claims, §4 INV-13..17 and INV-04′, §9 M1 (T-033..T-042), §11 Q-22..Q-25. The Athenian second lot is cited for *no choosing* (Ath. Pol. 63–65), not for secrecy. Limits stated on page one: single-party phase-2 setup, relayer for seat accounts, anonymity within `k`, recomputation among revealers, deliberation off-chain breaks the seal.

### DEC-016: One mode; `draw_round` leaves `kleroterion` (2026-10-25, M1)
Context: Tyler proposed a sealed mode beside the open one; Nicole and Justin read two modes as two half-products.
Decision: `kleroterion` has one mode. The open Fisher-Yates draw survives only in `beacon-verifier`, as evidence of the open, recomputable draw (its golden vectors stay there).
Alternatives: two modes — rejected; keep `draw_round` as a view — rejected, it is the door that lets a caller pick the round.
Consequences: T-033; INV-13; the kleroterion golden vector 4 is replaced by T-039's.

### DEC-017: The author's 2026-10-25 BN254 verifier and circuit are ported, and said so (2026-10-25, M1)
Context: `../Cautela-geracao-retirada/` holds a Soroban Groth16 verifier on the `Bn254` host functions and `role_membership.circom` (Poseidon Merkle + nullifier). DEC-001 says the withdrawn thesis is not cited; the code is the author's, MIT, and the files were created 2026-10-25 12:54–13:02 UTC (filesystem birth times; the folder has no git history), inside the hacking window. The HackMeridian terms require *original work (or properly licensed)* and forbid *misrepresent your work*; no rule covers earlier code.
Decision: port the verifier (new `vk`) and the circuit (four changes); log both in `docs/SOURCES.md` *Copied code*; a *Prior work* paragraph in the README gives the dates. The withdrawn thesis stays uncited.
Alternatives: rewrite from scratch — rejected as waste; port silently — rejected (rule 6, and the terms).
Consequences: T-036, T-037; SOURCES.md; README.

### DEC-018: CAP-85 considered and rejected for the draw code (2026-10-25, M1)
Context: `verify`/`randomness`/`draw` are copied into two contracts; CAP-85 (Protocol 28, 5 weeks old, no application-level use found) could make them one sealed external executable under an owner contract with no update function.
Decision: not used. A no-update owner makes it an honest commitment, but it is the fleet-upgrade primitive inside a no-upgrade thesis, the tooling is unexercised, and it changes nothing the user can see. The duplication is answered by golden vectors today and by a shared `no_std` crate later.
Alternatives: use it alongside M1 — rejected (Justin: subtracts credibility; Nicole: feature to impress).
Consequences: none in code; this entry is the answer to the question.

### DEC-019: Tessera is named as related, complementary work (2026-10-25, M1)
Context: Tessera (`ThaisFReis/tessera`, the same author, Find Your Way hackathon) is a secret-ballot module with the same sentence shape — *a contract any governance calls*. A judge finds it in two clicks.
Decision: the README names it openly in one line — *Tessera decides how one votes, in secret; Pinakion decides who sits. Pinakion has no ballot on purpose; the panel's seat-account list is Tessera's voter list.* No Tessera machinery (ring signatures, key images, Pedersen, time-lock, tallies) enters this repository; the line is the red line of DEC-015.
Alternatives: not citing — rejected: the omission reads worse than the overlap.
Consequences: T-041, T-042 (export format `[VERIFY]`, Q-25).

### DEC-020: `draw_round` becomes `seal(sig)`, not deleted (2026-10-25, T-033)
Context: T-033 said to delete `draw_round` from `kleroterion`. Ten of its 22 tests go through it, and they are what enforces INV-04, INV-07, INV-08 and INV-12; deleting it would leave those without a test until T-038, and would leave the M1 kill path — *the open draw with T-033 only* — with no draw at all.
Decision (the human's, 2026-10-25): `draw_round(round, sig, k)` is replaced by `seal(sig)`. `lock(k, reveal_after)` fixes `k` and the round (first after the ledger time + `MARGEM_DE_RODADAS` = 10, i.e. 30 s at quicknet's period); `seal` takes neither, verifies the stored round, runs once (`JaSelado`) and stores the panel (`panel()`). `reveal_after` is a ledger number (Q-24), stored and not yet used. `lock` refuses `k = 0` and `k > N` (`AssentosInvalidos`); `impede` is refused after `lock`. Error 1 (`RodadaAnteriorAoRol`) is no longer raised and keeps its number.
Alternatives: delete as written — rejected for the reasons above. Keep `draw_round` with checks — rejected by DEC-016: the round parameter is the door.
Consequences: §2, §3, §4 INV-06/13/14, §5, §8, §9 T-033 amended. T-038 rewrites `seal` from the open draw into the sealed one (seed and `T`, no panel), so this is still one mode at any time (DEC-016). If the M1 kill criterion fires, this is the product. The pre-T-033 testnet deploy in `docs/evidencia-testnet-bilhete.md` keeps the old interface; `scripts/sequencia_bilhete.sh` now calls the new one and has **not** been run against it (T-040). Liveness: if impediments or ticket transfers leave fewer than `k` eligible, `seal` fails with `ElegiveisInsuficientes` and the lock cannot be redone — T-038's `finalize` widening addresses that under Plan B; the open draw does not.

### DEC-021: Two licences — `circuits/` is GPL-3.0, the rest stays MIT (2026-10-25, T-034)
Context: the circuit toolchain (circom, circomlib, circomlibjs, snarkjs) is GPL-3.0 and Pinakion is MIT. `CLAUDE.md` forbids copying copyleft code into a permissive project. The circuit includes circomlib by path, so the circuit and what compiles from it (witness wasm, r1cs, zkey) derive from GPL code. Separately, `stellar/rs-soroban-poseidon`, the sponge over the CAP-75 permutation, exists only for soroban-sdk 28; this workspace is on 29.0.1.
Decision (the human's, 2026-10-25): `circuits/` carries its own `LICENSE`, GPL-3.0 (the canonical text from gnu.org), and holds everything that includes or imports circomlib/circomlibjs — the circuit, its prover script, the input and vector generators. Everything else stays MIT. The npm tools are pinned devDependencies, never copied. Proofs, `vk` and vectors are data and live in `tests/`. The compiled circuit stays out of git. For the sponge, the BN254 t = 2 and t = 3 parts of `rs-soroban-poseidon` v28.0.0 are copied into `contracts/poseidon/` (Apache-2.0, compatible with MIT), with the licence file and a header naming the change.
Alternatives: one licence by reference only — rejected by the human for the compiled prover T-036 will distribute; depend on `soroban-poseidon` 28 — impossible beside soroban-sdk 29; write the sponge and constants by hand — rejected: the upstream is Stellar's own and claims circomlib compatibility, which the 22 vectors then confirm.
Consequences: §3, §6; `docs/SOURCES.md` *Copied code*; README states the split (T-041). Not legal advice; the reading is the common one.

### DEC-022: T-034 says go (2026-10-25, T-034)
Context: the M1 kill criterion — without one proof of our circuit verified on testnet within budget at the end of T-034, M1 stops.
Decision: **go.** (1) Q-22: the contract's sponge returns circomlibjs's value for all 22 vectors (10 Poseidon(1), 12 Poseidon(2), edge values included). (2) A proof of `circuits/pinakion_claim.circom` (depth 6, public `[root, seed, y, claimer]`, 4,053 constraints) verified **on testnet**: `true` at tx `76fc290f…`, and `false` with `y` changed at tx `374632d9…`; 30,837,655 instructions declared, 11,737 stroops charged. (3) Q-23: the testnet ceiling is 400,000,000 instructions per transaction, read from the network config. One `verify` is 7.7 % of it. `docs/evidencia-testnet-spike.md`.
Alternatives: kill — no ground: every check passed with margin.
Consequences: T-035 to T-040 proceed. Three facts for them: a full depth-6 tree costs 109,821,216 instructions on the test host (27.5 % of a transaction) — T-035 should update the path at each `enroll` (≈ 10 M estimated) rather than build the tree inside `lock`; the circuit has 43 constraints of headroom under 2¹², so any addition means a 2¹³ setup (T-036); the spike setup is test-only and is replaced in T-036. No §2 claim is added: Plan B claims wait for the human (§2).

### DEC-023: The commitment tree is the contract's, updated at each `enroll` and `impede` (2026-10-25, T-035)
Context: T-035 asks for a depth-6 Poseidon tree over the authorized holders, built by the contract and frozen at `lock`. T-034 measured a full rebuild at 109,821,216 instructions on the test host, a quarter of a transaction.
Decision: leaf `i` is the commitment `c` of the `i`-th enrolled juror — the circuit's leaf is `Poseidon(s)`, which is `c` — and an impediment sets it to 0; empty leaves are 0, and the empty-subtree roots `Z₀…Z₆` are embedded constants checked against circomlib. `enroll` and `impede` each rewrite one path (six Poseidon(2) with the sibling from storage or `Zₙ`). `lock` reads the root and freezes it in `Fechamento` with `n`, the jurors still authorized, and `k` must satisfy `1 ≤ k ≤ n`. A `c` that is zero or ≥ r is refused; every accepted `c` is recorded forever and refused again from anyone (`CompromissoRepetido`). `impede` now refuses an address not on the roster and a second impediment. Three events — `Inscrito(jurado; indice, c)`, `Impedido(jurado; indice)`, `Fechado(raiz, n, rodada, k, revela_em)` — are enough to rebuild the root, which `scripts/arvore.py` and `web/arvore.mjs` do with no dependencies. Measured on the test host: `enroll` of the 64th juror 10,472,755 instructions, `impede` 8,656,396, `lock` 587,464. The optimised `kleroterion` Wasm grows from 10,918 to 33,677 bytes with the Poseidon constants embedded, against a testnet maximum of 131,072 (`contract_max_size_bytes`, read 2026-10-25).
Alternatives: build the tree at `lock` — rejected: 27.5 % of a transaction for one call, against 2.6 % per enroll; let the clerk post a root — rejected by the criterion; leaf `Poseidon(c)` — rejected: it would not match the circuit.
Consequences: §3, §4 INV-17 (second half), §5 golden data, §8 G2/G5/G6/G7. *Any previous lock*: this contract locks once, so "a `c` seen in a previous lock" is enforced as "a `c` ever seen by this contract"; across contracts nothing is shared — a later roster on a new contract does not see the old `c`s. That is stated, not solved; roster renewal is out of M1. The testnet script now draws each juror's `s`, keeps it in `build/` and sends `c`; it has not been run (T-040).

### DEC-024: The phase-2 setup, and how `claimer` is derived (2026-10-25, T-036)
Context: T-036 asks for a public ptau 2¹², a single-party phase 2 with a published transcript, an exported `vk`, a `prove` CLI and rejected negatives.
Decision: phase 1 is Hermez's `powersOfTau28_hez_final_12.ptau`, blake2b `ded26941…758b8a4a` as snarkjs publishes it. The two URLs in snarkjs 0.7.6's README and in older ones (`storage.googleapis.com/zkevm/ptau/…`, `hermez.s3-eu-west-1.amazonaws.com/…`) returned **403** on 2026-10-25; the current snarkjs README points to `circom.info`, from which the file was fetched and matched the hash. Phase 2 is one contribution with `/dev/urandom` entropy that is never written, then `zkey beacon` with drand quicknet round 33389132, computed from the clock after the contribution hash was printed; `snarkjs zkey verify` says `ZKey Ok!`. Everything is in `circuits/setup/TRANSCRICAO.md`, and r1cs, wasm, zkey and `vk.json` are committed (4.2 MB) so a prover needs neither circom nor the ptau, and anyone can re-run `zkey verify`. `claimer = SHA-256(strkey of the seat account, as ASCII) >> 8`, a field element; T-037 must compute the same on-chain from `Address::to_string`. The CLI is `node circuits/pinakion.mjs prove --eventos … --s … --seed … --assento …`, building the path from the contract's events with `web/arvore.mjs`; the seed's derivation from the drand randomness is T-038's, so the CLI takes the seed as given.
Alternatives: a local ptau (the spike's) — rejected by the criterion; a multi-party phase 2 — out of reach for one person in this window, and the README will say so; claimer from the XDR of the address — rejected: the strkey is what a user can see and recompute by hand.
Consequences: §3, §8 (G8), §9 T-036; `docs/SOURCES.md`. Changing the circuit by one constraint means re-running `circuits/setup.sh`, a new `vk`, and new fixtures. The trust statement for README page one (T-041): *whoever ran the setup could forge seats*; the beacon only shows the last step was not chosen. The acceptance criterion named the CLI `pinakion prove`; it is `node circuits/pinakion.mjs prove`, with no installed binary.

### DEC-025: The verifier embeds the key; `kleroterion` assembles the public inputs (2026-10-25, T-037)
Context: T-037 asks for the ported verifier as a separate contract with T-036's `vk`, and for `kleroterion` to assemble `[root, seed, y, claimer]` itself.
Decision: `groth16-verifier` drops its constructor; the `vk` is constants in `src/vk.rs`, generated from `circuits/setup/vk.json`, readable through `vk()`, and G8 checks the two are equal — so whoever deploys it cannot hand it another key, as INV-10 does for the beacon. `kleroterion`'s constructor takes the verifier's address and keeps it for good (`verifier()`); it calls it through a `#[contractclient]` trait declared on its side, with a `Prova` type of the same fields, so the verifier's code is not linked into it. `prova_de_assento_vale(prova, seed, y, assento)` is internal: it refuses before `lock`, takes the root from `Fechamento`, computes `claimer = SHA-256(assento.to_string()) >> 8` (DEC-024), and passes the four inputs in the circuit's order. It is not exported: until T-038's `claim` gives it the seed from `seal`, an export would let a caller choose the seed. Measured on the test host: the check costs 29,195,235 instructions through `kleroterion` (the verifier alone 29,105,202). Adding the verifier's address to instance storage moves the earlier costs: `enroll` 10,473,619, `impede` 8,676,833, `lock` 599,498.
Alternatives: the verifier with a `vk` in storage, set by its constructor — rejected: a deployer could swap the key; linking the verifier as a library — rejected by the criterion's *separate contract*; computing `claimer` from the address's XDR — rejected in DEC-024.
Consequences: §3, §4 INV-15 (proof half), §8 (G2, G7, G8), §9 T-037. The spike's testnet verifier (`CCICALAV…`) keeps the old interface and the spike key; its evidence stands as history. `scripts/sequencia_bilhete.sh` deploys the verifier first and passes it to `kleroterion`; not run (T-040).

### DEC-026: The sealed flow in `kleroterion` (2026-10-25, T-038)
Context: T-038 asks for `seal`, `claim`, `finalize`, `reveal` and `recuse` under DEC-015, with one mode (DEC-016). Several mechanics were left open.
Decision:
- **Seed and threshold.** `seed = U256(randomness) >> 8`, the drand randomness shifted to be a field element; `T = ⌊r·2k/N⌋`, computed as `(r/N)·2k + ((r mod N)·2k)/N` so it does not overflow 256 bits, and `T = r` when `2k ≥ N`.
- **Claim window.** A constructor parameter `janela`, in ledgers: claims are accepted up to `seal ledger + janela`, `finalize` only after. The constructor changed, and the costs of the other calls moved with it.
- **Claim.** `claim(seat, proof, y)`: the seat account signs; `y < T`; `y` not used before; the proof valid against the stored root and seed (DEC-025). The claims are kept sorted ascending. The event carries `y` only.
- **Recusal.** `recuse(juror, s)`: the main account signs and shows `s`, after `seal` and before `finalize`; its `y` is removed from the list and marked used, so it cannot be claimed afterwards. The event carries the address only.
- **Finalize.** `finalize()`: with at least `k` claims the `k` smallest sit; with fewer, the first call sets `T = r`, reopens the window and returns `None`; the second seats whoever is there and records `completo = false`.
- **Reveal.** `reveal(juror, s)`: after the panel and `revela_em`; checks `Poseidon(s) = c`, computes `y`, records whether it sat and whether the account still holds a ticket (INV-12 as the human rewrote it). Any enrolled juror may reveal, seated or not, so the record can say *drawn and absent* (T-039).
- **The open draw leaves.** Kleroterion's copy of `draw` and the open-draw tests (golden vector 4, `sem_bilhete_nao_senta`, `elegiveis_insuficientes_e_recusado`, …) go; error 7 (`ElegiveisInsuficientes`) is no longer raised and keeps its number. G6 ties only `beacon-verifier` to the Fisher-Yates vectors, as DEC-016 said.
Measured on the test host (N = 8, k = 3): `seal` 26,894,868 instructions, the sixth `claim` 29,514,466, `recuse` 2,832,768, `finalize` 184,731, `reveal` 2,942,518; with the new constructor `enroll` 10,481,144, `impede` 8,684,358, `lock` 606,903, the proof check 29,200,539. The optimised Wasm is 56,487 bytes (33,970 after T-037): `recuse` and `reveal` add the Poseidon(1) constants. The testnet maximum is 131,072.
Alternatives: a fixed claim window — rejected: a constant cannot fit both a test run and a real case; recusal before `seal` — rejected: without the seed there is no `y` to take out, and the juror could still claim from a seat account; keeping a ticket check at claim — impossible without naming the juror (the human's call, INV-12).
Consequences: §3, §4 INV-04, 07, 08, 12, 15, 16, 17, §8 (G2, G6, G7), §9 T-038. Tests: written before the code, and they failed to compile against the old API; unlike T-033 to T-037, no stub was run to see each fail on behaviour. `scripts/sequencia_bilhete.sh` stops at `seal`; claims, `finalize` and reveals on testnet are T-040. README page one still describes the open draw; T-041 rewrites it, and a limitation line says so meanwhile.

### DEC-027: The ledger is enough to check the sealed panel (2026-10-25, T-039)
Context: T-039 asks the verifiers to read only the ledger and drand. Two things were not in the events: the `y` a recusal removed, and the `s` that proves a revealed `y` is that juror's.
Decision: `Recusado` and `Revelado` carry `s` and `y` beside the address. Each is the juror showing themself, and `s` was already public in the transaction's arguments; a recused `y` never sits, so INV-17's purpose — no seated `y` tied to a name before the verdict — holds. With that, `scripts/verificar_painel.py` and `verificarSelado` in `web/sorteio.mjs` recompute everything from the events and the randomness of `Fechado.rodada`: root, seed `randomness >> 8`, `T` (and `T = r` after `Ampliado`), every claim below the threshold in force and once, `Poseidon(s) = c` and `y = Poseidon(seed, s)` for every recusal and reveal, the ascending order without recused `y`, the panel and `completo`, and each revealed juror's `sentado`. Each juror is reported as *sentou*, *revelou, nao sentou*, *sorteado e ausente* (below `T`, never claimed), *nao revelou*, *recusado* or *impedido*, with the count of seats no reveal accounts for. The sealed golden vector is the exact event list of the N = 8, k = 3 flow, whose main accounts are fixed C-strkeys because the test host has no G accounts for the SAC; the contract replays it event by event. The open-draw vectors stay with `beacon-verifier`, and vector 4 stays as the record of the pre-T-038 testnet run; `verificar_painel.py` keeps that mode so `docs/evidencia-testnet-bilhete.md` remains reproducible. Measured: `recuse` 2,832,906 and `reveal` 2,942,915 instructions, the two events now larger.
Alternatives: read `s` from the transactions' arguments — rejected: events are what an RPC lists without decoding invocations, and the verifier should not depend on the contract's argument layout; trust the contract's own check — rejected by the criterion.
Consequences: §3, §4 INV-04′ (now enforced), INV-17, §5, §8 (G2, G5, G6), §9 T-039. Neither verifier checks the drand signature: `verificar_painel.py` fetches or is given the randomness, and the contract checked the signature at `seal` (CAP-59). Fetching the events from a live RPC into the record format is T-040's.

### DEC-028: The sealed sequence on testnet (2026-10-26, T-040)
Context: T-040 asks for the whole sealed flow on testnet with 20 jurors, a cost table from the transactions, and the evidence page rewritten.
Decision: `scripts/sequencia_bilhete.sh` makes its own friendbot identities under a prefix. It covers: issuer flags, then `AUTH_IMMUTABLE`, then master weight 0; 20 `enroll` with `c = Poseidon(s)`; one `impede`; `lock`; `seal` on the live round; every juror under `T` proving with `circuits/pinakion.mjs` from the events `scripts/registro.py` reads back, and claiming from a fresh seat account, with one resent claim refused; one recusal; `finalize`; 18 reveals; and the verifier on the events read back. The run `t040b` (2026-10-26, kleroterion `CCRBKLS6…`) ended with `CONFERE — 77 conferencias, 0 falha(s)`: seated j2, j12, j13; recused j3; impeded j0. Its hashes and event record are committed (`docs/evidencia-testnet-hashes.tsv`, `docs/evidencia-testnet-registro.json`), G6 re-checks the record offline, and G9 reprints the cost table from the network. The recusal runs **before** `finalize`, not after the reveals as the criterion listed: the contract (DEC-026) allows recusal only before the panel. Costs are what each transaction declared and was charged; the network does not report the instructions executed. The largest Soroban call is a `claim`, at about 32 M declared instructions.
Alternatives: build the record from the script's own arguments — rejected: the evidence is that the ledger alone suffices; G-address principals in the golden vector — still impossible on the test host (DEC-027), so the testnet run is where G-accounts are exercised.
Consequences: §8 (G6, G9), §9 T-040, `docs/evidencia-testnet-bilhete.md` (the pre-T-033 run kept below the new one, so its hashes cited in §2 and the README stay valid). Three bugs were found on the way and fixed: `STELLAR_NO_CACHE=1` is not accepted by CLI 28.1.0 (it wants `true`); `stellar tx new` printed no hash for one classic operation, so hashes are now read from Horizon; friendbot needs retries. A 4-juror rehearsal (`t40dry`) went through the widening path and ended incomplete, also `CONFERE`. No §2 claim is added: whether Plan B may now be claimed is the human's call.

### DEC-029: The thesis and the first page say Plan B, with its limits on page one (2026-10-26, T-041)
Context: T-041 rewrites `THESIS.md` and `README.md` for the sealed draw, and the §2 claims about it are the human's to approve.
Decision: the human approved, on 2026-10-26, the four §2 rows above and the three-sentence guarantee — *nobody chooses who judges you; nobody knows who until they have sat and judged; afterwards, anyone recomputes the panel from the ledger, among those who revealed* — with its Portuguese twin in `THESIS.md`. One user, an open-source project with a code of conduct, Rust 2021 first and Node 2017 second; the others in one *it also serves* line; the "tokenized fund or regulated issuer" row is gone. Athens is cited from *Ath. Pol.* 63–65 for the two lots and *no choosing*, and the thesis says the text claims no secrecy. The fourth *why Stellar* is CAP-74/75 with the measured 8 % of the ceiling. README page one carries, before anything else: testnet only and unaudited, single-party setup, seat-account funding and the missing relayer, loss of `s`, recomputation only among revealers, and off-chain deliberation breaking the seal. *Related work* names Tessera as the author's complementary module; *Prior work* dates the ported code. The README's first command is now the sealed check of the testnet run.
Alternatives: a softer second sentence (*the ledger does not tie seat to name until the verdict*) — offered, not chosen; no Plan B claims — offered, not chosen.
Consequences: §2, §9 T-041; `THESIS.md`, `README.md`. G7 green; the §2 forbidden words appear only negated or about others (*Kleros seats anonymous jurors*). `DESIGN.md`, the landing page in `web/` and the pitch still describe the open draw; per §0's order they follow in T-042.

### DEC-030: The ticket page, the Tessera export and the submission text (2026-10-26, T-042)
Context: T-042 asks for a read-only page per person, links to stellar.expert and Horizon, the seat accounts in Tessera's voter format (`[VERIFY]`, Q-25), and the submission text.
Decision: `scripts/registro.py` now also records each event's transaction hash (from the RPC's `getEvents`, matched by event id) and, for each `Reivindicado`, the seat account that sent it (the transaction's source, from Horizon); the committed record was rebuilt with them and still checks out, 77 checks. Public endpoints refused Python's default User-Agent with a 403, so the scripts send their own. `web/bilhetes.html` reads that record and `tests/poseidon-bn254.json` from the repository — as GitHub Pages serves it from `main` (DEC-012) — and recomputes in the browser; `?registro=` and `?emissor=` point it at another run. Tessera's `--aptos` format was read from its source (Q-25 answered); the export lists seat accounts, never main accounts, so a Tessera vote is tied to a seat and not to a name. G6 checks the committed export against the exporter. `docs/SUBMISSAO.md` is the form text, built only from §2 rows and gated numbers; its one statement about the sortition being unmatched is dated to the 2026-10-25 search and says the indexes lag.
Alternatives: a page that queries the RPC live — rejected for now: the committed record is reproducible and the page stays static; exporting main accounts — rejected: it would undo the seal in the next module.
Consequences: §3, §8 (G6), §9 T-042, §11 Q-25; `README.md` (*Try it*), `web/index.html` (a link). The landing page `web/index.html` and `DESIGN.md` still present the open draw as the main story; reworking them, and Q-17 (whether the organiser accepted the Genesis path), remain before the form is sent. Submitting is the human's act.

### DEC-031: The panel-size study is exact arithmetic on a stated model, and gated (2026-10-26, T-044)
Context: the human asked how severity should change the panel size. `docs/limite-inferior-do-rol.md` §4 already printed capture risks, from a computation no gate runs.
Decision: `scripts/tamanho_do_painel.py` computes the hypergeometric tail exactly (`math.comb`, no dependency) for a uniform `k`-subset and a disciplined bloc, and prints every table of the study; G6 checks that each printed table appears verbatim in the study. The re-check of `limite-inferior-do-rol.md` §4 agrees to the rounding there. The proposed levels are Contributor Covenant 2.1's four; the roster floors follow `limite-inferior-do-rol.md` §6, and are a choice, not a result.
Alternatives: Monte Carlo with recusals and absences — rejected for now: exact arithmetic is reproducible to the digit and the model's omissions are stated; a policy table without a model — rejected: §0's numbers rule.
Consequences: §8 (G6), §9 M2, §11 Q-24, Q-26 to Q-28. The study corrects one statement made in conversation: `impede` already acts on one case, because a contract locks once (DEC-023); the gap is that impeding the party is optional.


### DEC-032: Severity table, the party at `lock`, and `suspended` (2026-10-26, T-045)
Context: the human approved INV-18 to INV-20 (Q-28). `lock(k, reveal_after)` let the clerk choose `k` and leave the party on the roll.
Decision:
- **Constructor:** `__constructor(escrivao, bilhete, verificador, janela, niveis, afastamento)`. `minimo` is gone; each level carries its own `rol_minimo`. An empty table or a level with `k = 0` is refused (`TabelaInvalida`). The constructor emits `Constituido { niveis, afastamento }`.
- **`lock(gravidade, revela_em, acusado)`:** `k` comes from `niveis[gravidade-1]` (`GravidadeInvalida` outside it). It refuses below the level's floor, counted on the roll as `minimo` was, so impediments still count (INV-07), and refuses `acusado == escrivao` (`EscrivaoEParte`). It impedes an enrolled, authorized party through the same code path as `impede`, emitting `Impedido` before `Fechado`, and stores `gravidade`, `acusado` and `afastado_ate = ledger + afastamento` in `Fechamento`.
- **Events:** `Fechado` gains `acusado` (a topic), `gravidade` and `afastado_ate`.
- **New calls:** `suspended(a) -> bool` and `levels()`.
- **Golden vector and verifiers:** the sealed vector gains `Constituido` and the new `Fechado` fields, with a party who is not on the roll, so the N = 8 fixture and its proofs stand. Both verifiers check the level, `k`, the floor, the suspension window when the record has the lock's ledger, and the party. They skip those checks on records without `Constituido`, so the 2026-10-26 testnet record still checks out at 77.
- **Public names:** `suspended` and `levels` are in English, like the other public calls; the study's `afastado()` is `suspended()`.

Measured on the test host after the change. Every cost rose, because the instance now holds the table:
- `enroll` 10,589,550 instructions;
- `impede` 8,792,764;
- `lock` 861,209;
- the proof check 29,285,140;
- `seal` 27,015,477;
- the sixth `claim` 29,599,984;
- `recuse` 2,912,530;
- `finalize` 264,892;
- `reveal` 3,028,049.

The optimised Wasm is 60,418 bytes (was 56,487).

Alternatives:
- A `minimo` kept beside the table — rejected: two floors would invite disagreement.
- Impeding the party in a separate call — rejected: that is today's gap.
- Ending the suspension at the verdict — rejected: the contract does not see the verdict (study §2.2).

Consequences:
- **Spec sections:** §3, §4 INV-18..20, §8 (G2, G5, G6), §9 T-045, §11 Q-26, Q-28.
- **Interface:** this changes the constructor, `lock` and `Fechado`. A deploy of this contract is not compatible with `scripts/sequencia_bilhete.sh` as run on 2026-10-26; the script is updated and **has not been run** against it.
- **Tests:** the new tests were written against the new signatures, so they could not be run red against the old code first.


### DEC-033: A floor on the reveal, and TTL that reaches it (2026-10-26, T-046)
Context: INV-21 was approved on 2026-10-26 (Q-28), with `espera_minima` of 30 days. The contract never extended a TTL, so with a late reveal the data `reveal` reads could be archived first. On testnet, `min_persistent_ttl` is 120,960 ledgers and `max_entry_ttl` is 3,110,400 (cited, read 2026-10-26).
Decision:
- **Floor:** the constructor takes `espera_minima`, after `afastamento`, and `Constituido` publishes it. `lock` refuses `revela_em < ledger + espera_minima` (`RevelacaoCedoDemais`).
- **TTL extension:**
  - `lock` extends the instance (with the code) and every enrolled juror's `Compromisso` to `revela_em + janela − now`.
  - `finalize` extends the instance and `Painel` the same way.
  - Both are capped at `Storage::max_ttl()`. Past that cap, someone must extend again. Any account can: `ExtendFootprintTTLOp` has no access control.
  - An archived entry is restored automatically when an invocation touches it (Protocol 23, CAP-66), at normal rent and fees.
- **Verifiers:** they check `revela_em` against `espera_minima` when the record carries the lock's ledger.
- **Ticket page:** shows the reveal ledger, and the level and party when the record has them.

Measured on the test host:
- `lock` with 64 enrolled: 10,781,627 instructions (was 861,209); the cost is the 64 extensions.
- `enroll` 10,600,843; `impede` 8,804,057; the proof check 29,293,686.
- `seal` 27,026,910; the sixth `claim` 29,605,336; `recuse` 2,917,882.
- `finalize` 351,771; `reveal` 2,967,446.

The `lock` cost is 2.7 % of the 400 M per-transaction ceiling (estimated from those two figures). The optimised Wasm is 61,349 bytes.

Alternatives:
- Extending at each `enroll` — rejected: `revela_em` is not known yet.
- Asking the jurors to extend before revealing — rejected: it moves the liveness of the record onto the people it protects.

Consequences:
- **Spec sections:** §4 INV-21, §8 G2, §9 T-046, §11 Q-24.
- **Not extended:** the SAC's own instance, which `reveal` reads through `balance`. The contract cannot extend it; anyone can, through `ExtendFootprintTTLOp`.
- **Not measured:** the network cost of a restore.
- **Not on testnet:** this version has not run there.


### DEC-034: The human waived the stops for M3 (2026-10-26, M3)
Context: on 2026-10-26 the human asked for the whole case cycle end to end, and wrote: *"Não precisa parar pra pedir aprovação, implemente tudo direto"*, *"Implante sozinho. Não precisa me pedir permissão para nada"*, and *"Permito tudo sempre. Pode executar tudo"*.
Decision: for M3 the "ask first" column of `CLAUDE.md` is relaxed by the human. That covers new invariants (INV-22..26), a new contract, deploying to testnet, publishing the page through `main`, and merging the milestone's PR. It does not cover what the human did not say: no mainnet, no production endpoints, no secrets in git, and no claim §2 does not allow. M3 is one branch and one PR rather than one per task, so the PR can be merged once with the run's evidence.
Alternatives: one PR per task, each waiting for review — rejected by the human's instruction.
Consequences: §9 M3. The human still sets tasks to `done`.

### DEC-035: The case is a second contract, `processo`, that reads the roster (2026-10-26, T-048)
Context: the cycle needs a complaint, a record, attendance, a vote and a verdict, and kleroterion's Wasm is already 61,927 bytes.
Decision:
- **A separate contract.** `processo` reads kleroterion through a client with mirrored types (`Fechamento`, `Final`), and Tessera the same way (`Proposta`, `Pergunta`, at commit `356c22f`, MIT, the same author). It never writes to either.
- **One small change in kleroterion.** It stores each seat account's `y` and exposes `seated`, so attendance is checked without naming anyone. The seat account is already public as the claim transaction's source.
- **Records** are SHA-256 hashes in instance storage, extended to the network maximum at each write.
- **Quorum and votes.** The quorum is `max(⌊k/2⌋+1, votos_minimos)`. The votes required are `⌈2k/3⌉` from `dois_tercos_desde`, otherwise `⌊k/2⌋+1`. Both are counted over the whole panel.
- **The privacy floor.** `votos_minimos` is 5 on testnet, because Tessera publishes no score below five confidential votes. That is why a case at level 1 (`k = 3`) is refused.

Alternatives:
- Growing kleroterion — rejected: size, and it would mix the roster with the case.
- Counting votes in `processo` — rejected: that is Tessera's job (DEC-019), and a second ballot would be a weaker one.
- Binding the vote off-chain only — rejected: the contract can recompute Tessera's root for about the cost of a few hashes.

Consequences: §3, §4 INV-22..26, §5, §9 T-047..T-048.

### DEC-036: Tessera at 356c22f, our own testnet deployment, and a one-line CLI fix (2026-10-26, T-049)
Context: the deployed Tessera contract named in Tessera's README (`CDWGDWDP…`, 2026-10-10) is older than its CLI. The CLI at `356c22f` also sends an `abrir` argument, `fim_tempo`, that the contract at the same commit renamed to `rodada_abertura`. Opening a vote failed with *unexpected argument '--fim_tempo'* (observed 2026-10-26).
Decision:
- **The contract.** Tessera's contract at `356c22f`, built unchanged and deployed once on testnet as `CDWBWAHUZ3MC3BJXJIW5O74MNUXH76VCGBIVFQU3XJJFGML3VKFIXQT7`.
- **The CLI.** Tessera's CLI at the same commit, in a worktree outside this repository, with `scripts/tessera-cli.patch`, which sends `rodada_abertura` instead of `fim_tempo`. That is a one-line client fix; nothing in Tessera's contract or cryptography changes.
- **The smoke test.** A five-voter vote opened, voted, tallied 4 to 1, and passed `tessera verificar`, on 2026-10-26.
- **Disclosure.** Everywhere the vote appears — the page, README, THESIS, SUBMISSAO and the video script — it says that the vote is Tessera's, the author's earlier module.

Alternatives:
- The older deployment — rejected: it does not match the CLI.
- Rewriting the vote — rejected (DEC-035).

Consequences: §6, `docs/SOURCES.md`. The fix belongs upstream in Tessera; it is not sent from here.


### DEC-037: Seat accounts are recorded at claim, and costs move (2026-10-26, T-047)
Context: `processo` must check that an attendee sat without learning who they are.
Decision: `claim` stores `YDe(assento) = y` and refuses a second claim from the same seat account (`AssentoRepetido`); `seated(assento)` is true only after the panel, for a seat account whose `y` sat. The seat account was already public as the claim transaction's source, so INV-17 is unchanged.
Measured on the test host after the change:
- the sixth `claim` 29,684,570 instructions;
- `recuse` 2,926,704;
- `finalize` 351,363;
- `reveal` 2,979,856.
The other costs of DEC-033 are unchanged. The kleroterion Wasm is 61,927 bytes.
Alternatives: a list of seated accounts written at `finalize` — rejected: `finalize` does not know the seat accounts without this map anyway.
Consequences: §4 INV-23, §9 T-047.


### DEC-038: The live case runs in the visitor's tab (2026-10-26, T-053)
Context: the human found the recorded case read like a pitch deck, because nobody acts in it, and chose a live case run from the browser.
Decision:
- **`web/vivo/motor.mjs` runs the whole case.** Every key is born in the tab, one friendbot account funds them all, and every step is a real testnet transaction. It is the same sequence as `scripts/sequencia_processo.sh`.
- **The same module runs in Node,** with the dependencies injected, so it was debugged against testnet before the page used it.
- **Libraries:**
  - `@stellar/stellar-sdk` 16.3.1, from jsDelivr. Testnet is on protocol 29, and 15.1.0 cannot read its `SorobanCredentialsType` 2 (*XDR Read Error … value 2*, observed). 16.3.1 is the newest release more than two weeks old.
  - `snarkjs` 0.7.6, from jsDelivr.
  - Tessera's wasm client, built unchanged at `356c22f` into `web/vivo/tessera/` (MIT).
- **The prover** for the browser is `circuits/navegador.mjs`. It sits in `circuits/` so the GPL-3.0 boundary stays where DEC-021 put it.
- **Enrolments and claims go one at a time.** Each rewrites the roster and a tree path, so two in the same ledger are simulated on stale state and fail on apply. The SDK's `signAndSend` returned those failures without throwing (observed: 40 sent, 5 on-chain), so the engine now checks `SUCCESS` on every transaction.
- **The vote runs on Tessera in ring mode with a time lock**, as Tessera's own dapp does: no committee, the tally unlocked by a drand round, and one section.
- **The case page reads the live case** from `localStorage` (`?registro=local`) and re-hashes each document before it shows it.

Alternatives:
- A wallet and a juror role — rejected by the human: it depends on other people and windows, and cannot be shown alone in minutes.
- Live reads of a recorded case — rejected: the visitor still does nothing.

Consequences:
- **Spec sections:** §3, §9 T-053; `DESIGN.md`.
- **Seat accounts:** they are funded by the same account that funds the jurors, so the link the relayer would remove is visible on the ledger. The page says so.
- **Closing the tab** halfway leaves a half-run case on testnet.


### DEC-039: *Why Stellar* leads with the roster, not the proof (2026-10-26)
Context: a Stellar Raven assessment the human ran on 2026-10-26 argued that the project fits *Stellar Unlocked* on two grounds of unequal strength. The strong one is the classic asset as a jury roster: flags from 2015, used for what they were not made for, by an issuer that locks itself out. No such use was visible in three searches. The weak one is the seat proof: the newest primitive and the most contested, because a ZK-focused Stellar hackathon drew many submissions on the same host functions. `docs/SUBMISSAO.md` opened *Why Stellar* with the proof and put the roster third. The human said *"faça isso"*.
Decision:
- **New order** in `THESIS.md` (first, per §0) and in `docs/SUBMISSAO.md`:
  1. the roster and the self-locking issuer;
  2. drand verified on-chain, with prior art stated;
  3. the seat proof, as what Protocol 25 unlocked.
- **The year-ago answer is written down.** A year ago this would have been the open draw, whole, which is the kill path the plan wrote. The seal would not have existed.
Alternatives: keep the proof first, for recency — rejected: it puts the project in a long queue. The tension is not settled; it is Q-34.
Consequences: `THESIS.md` *Why Stellar*, `docs/SUBMISSAO.md` *Why Stellar*, §11 Q-34. Nothing in the code changes.

### DEC-040: The case page shows its own work, and Present ends on the limits (2026-10-26, T-056)
Context: the human asked for a better demo UX. The page already led with the verdict and checked everything in the browser, but a reader could not see the checking happen, had no way in from the hero, read nine phases of identical tick lines, saw no figure of the one idea the project rests on (a seat is a number under a threshold), and had to leave Present to show the limits, which the script asks for at 2:40.
Decision:
- **The browser's progress is a line under the verdict**, written as it happens: the record read with its event count, the documents hashed, the checks with the time they took. The time is measured on the reader's machine and says so; it is never carried into a document.
- **Three ways in from the hero:** the phases, Present, the live case. The register is product (`PRODUCT.md`): the reader is in a task, so the ways in are plain buttons and a link, not a second hero.
- **Each phase's checks sit under one count** (*Conferido no navegador · n de n*), so a judge reads the count and a verifier reads the lines. The lines keep the ASCII names they share with `scripts/verificar_processo.py`, translated by the same table as before.
- **One figure, in the seats phase:** every claimed `y` as a disc on the field `[0, r)`, the threshold as a bronze cut, the seated ones filled. It is drawn from the record alone (`Reivindicado`, `Selado`, `Finalizado`, and `P.R`), so it can show nothing the ledger did not. Shape and word carry the meaning, per `DESIGN.md`; colour only reinforces.
- **Present shows the limits as a tenth screen** after the ninth phase, because the script's 2:40 beat is the limits block and leaving the mode mid-recording was the only way to reach it.
- **The rail compacts below 1100px** to numbers plus the current title, and scrolls the current phase into view, instead of a hidden horizontal scroll with the ninth phase cut off.
- **`PRODUCT.md` is written** from `THESIS.md`, `DESIGN.md` and §1–§2, as the design tooling's strategic file; it adds no fact.
Alternatives:
- A redesign of the look — rejected: `DESIGN.md` follows the thesis (§0 order) and nothing in the thesis changed. The pass stays inside the system.
- Animating the check count upward — rejected: the checks run in one synchronous call, so a count-up would be staged, not measured.
- A fade mask on the rail to signal overflow — rejected: a gradient, and `DESIGN.md` has none; compacting removes the overflow instead.
Consequences: `DESIGN.md` *Layout* (the case page), `README.md`, `docs/DEMO.md`, `PLAN.md`, §9 T-056. No check, no number and no claim changes. No gate covers the page's markup (the page's logic is `processo.mjs`, under G5); the verification is in the browser and in the PR's screenshots.

---

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
| Q-09 | ~~`docs/whitepaper-v0.1.md` states the Soroban CPU ceiling as 100M. A measurement from a previous project of the human's, cited in conversation, gives 400M by bisection on testnet. The two have not been reconciled and no figure in §2 depends on either.~~ **Answered 2026-10-25 for testnet (T-034):** 400,000,000 instructions per transaction, read from the network config (`docs/SOURCES.md`). | — |
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
| Q-21 | ~~**The clerk's `impede` is discretionary.**~~ After `lock()` the clerk can strike any juror; the substitute is the next in the drawn order, so selective striking shapes the panel. Every strike is a transaction with a name and a time, so it is auditable, not prevented. The grounds-and-contestation rules of `docs/desenho-rol-sorteio-recusas.md` are not built. Raised in `docs/avaliacao-genesis-unlocked.md` as the objection a judge will make. **For the human: say it in the video, and decide whether the recusal rules are the next milestone.** | the submission; the next milestone  **Closed by T-033 (2026-10-25):** `impede` is refused after `lock` (`impedir_depois_de_fechar_e_recusado`). Before `lock` the clerk can still strike, but the drand round is not yet fixed, let alone published, so a strike cannot aim at the drawn order. Grounds and contestation remain unbuilt. |
| Q-22 | CAP-75 exposes the Poseidon/Poseidon2 **permutation**, not the full hash (Raven skill `zk-proofs`, 2026-10-25). The sponge and padding assembled in the contract (`stellar/rs-soroban-poseidon`) must match circomlib byte for byte, for t = 2 and t = 3. ~~`[VERIFY: with vectors, T-034]`~~ **Answered 2026-10-25 (T-034, DEC-022):** 22 circomlibjs vectors, t = 2 and t = 3, match byte for byte (`bate_com_a_circomlib`). | — |
| Q-23 | The per-transaction CPU ceiling: Q-09 holds 100 M (old whitepaper) against 400 M (Tessera, bisection on testnet, cited). No fraction of the ceiling is published until it is read from `developers.stellar.org` and logged. **Answered 2026-10-25 (T-034):** 400,000,000 on testnet (`tx_max_instructions`), read with `stellar network settings`; `developers.stellar.org` prints no figure and points to that command. Mainnet not read. | — |
| Q-24 | The default for `reveal_after`. The contract takes a ledger number; the demo uses a small one. With `reveal_after = 0` the seal buys only the claim window, and the spec must say so. **For the human:** is "after the verdict" the documented default? **Answered by the human 2026-10-26: reveal late (option B).** The contract will enforce a floor, `espera_minima`, fixed at construction; proposed default 30 days. Built in T-046: `lock` refuses `revela_em < lock + espera_minima`. | — |
| Q-25 | ~~Tessera's voter-list format for the export (`--aptos` takes CLI identity names or a file with one name per line, per its CLI README — addresses or names? `[VERIFY]`).~~ **Answered 2026-10-26 (T-042):** `tessera abrir --aptos` takes a comma list or a file, one entry per line, `#` for comments; an entry is a 56-character G-address or a local identity; the order is the Merkle root (`ThaisFReis/tessera` `cli/src/comandos.rs` at `39ec925`). `scripts/exportar_tessera.py` writes the seat accounts one per line, in panel order. | — |
| Q-26 | **The person judged can sit on their own case.** Nothing makes the clerk impede the party before `lock`; if they are drawn, their `claim` is sealed and they sit unseen until the reveal. Found 2026-10-26 while studying the human's request. T-045 closes it (INV-18). Until then the README's limits should say it. **Closed in the code by T-045 (2026-10-26):** `lock` impedes the party (INV-18). Not on testnet: the 2026-10-26 run predates it. | — |
| Q-27 | **The reporter's conflict.** The Contributor Covenant requires the reporter's privacy, and no event names them. If the reporter is drawn, impeding them publicly exposes them, and letting them sit puts a party on the panel. Today the best course is not to claim the seat, which shows at reveal as *drawn and absent* with no reason. Not solved. | — |
| Q-28 | ~~**For the human: approve INV-18 to INV-21 and the defaults** of the study (§5 and §1): the four-level table (`k` 3/5/7/9, roster floors 16/40/40/100, 2/3 recommended for level 4), `afastamento`, and `espera_minima` = 30 days. INV-19 takes the choice of `k` away from the clerk; INV-18 changes `lock`'s signature (a new DEC when built).~~ **Answered 2026-10-26: the human approved INV-18 to INV-21 and the defaults** (levels 3/5/7/9, floors 16/40/40/100, 2/3 at level 4, `espera_minima` 30 days). No value was proposed for `afastamento`; it is a constructor argument with no default, and the testnet script uses a short demo value. | — |
| Q-29 | **The Pinakion session is not a meeting.** T-054 read the rite of five families of council: every one allows video and not one gives up the synchronous gathering — the PSF and the Rust council write the hierarchy down (asynchronous only when the synchronous fails). Our quorum is a count inside a window of ledgers. It may be the right translation for a community across time zones, but nothing in this repository defends it. **For the human: record the asynchrony as a DEC (P-2), and decide whether the verifiable summons (P-4) belongs in the next milestone.** | — |
| Q-30 | **Nobody writes the minutes.** In every source read, the record has a designated author — a secretary, a scribe, the author of the winning vote — and is never optional. In `processo`, the clerk's minutes and a juror's minutes are both optional, and nothing requires that **one** record of the session exist. **For the human: approve P-1** (`decidir` refuses without a clerk's minutes filed after the session opened). | — |
| Q-31 | **The Optimism finding is secondhand.** The evidence that voting on conduct with the whole electorate failed — two cases, then a dedicated committee — is Arbitrum recounting Optimism's history on 2024-10-22 (`docs/SOURCES.md`). It is the most direct empirical support this project's thesis has, and **for that reason** it cannot go into the video, the deck or the submission until someone reads Optimism's own source. **For the human: read it, or do not use it.** | the submission |
| Q-32 | **The impediment belongs to the clerk, not to the person.** In the four families with a written rite, whoever has the conflict withdraws, and the CNCF has alternates decide in their place; none gives a third party the power to strike a member from a case. `recuse(juror, s)` is already open and self-declared, but the documentation treats `impede` as the normal path (P-3). This is also the backdrop of Q-27. | — |
| Q-33 | **The Genesis rubric is unverified and the two sources disagree.** `hackmeridian.com/build` (2026-10-25) gives seven weighted criteria, which DEC-013 and §9 are written against; the Scout curated record (2026-10-26) gives four unweighted ones — problem clarity, MVP quality, Stellar integration, continued development potential. The official rules page is behind sign-in. Both agree the review is async over the **demo video, the pitch deck, the repo and the submission answers**, and neither of the first two exists (`PLAN.md`). Logged in `docs/SOURCES.md` → *Conflicting sources*; both scored in `docs/avaliacao-genesis-unlocked.md`. **For the human: which rubric, and is the video or the deck being made before 12:00?** | the submission |
| Q-34 | **Old flags or new primitives?** The judges' spoken framing, recorded in `docs/SOURCES.md` from AI summaries rather than transcripts, leaned to *new primitives* and confidential tokens. If they weigh recency, a roster made of 2015 flags loses for being old. If they weigh *not made for this*, it wins. The written bullets reward both. DEC-039 leads with the flags, and keeps the Protocol 25 seat proof as the third reason. **For the human: does the video lead the same way?** The assessment's figure of more than thirty ZK submissions is Raven's and is not verified here; it does not go into the submission until its source is read. | the video |
