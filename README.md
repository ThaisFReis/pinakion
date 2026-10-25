# Pinakion

**Verifiable jury selection for governance on Stellar.**

A deciding panel drawn from a roster committed **before** the case existed, using entropy nobody controlled at the time of the commitment, that **anyone can recompute and check**.

The guarantee, in one sentence: *nobody chooses who judges you.*

> In classical Athens the *pinakion* was a small bronze ticket engraved with a citizen's name. It was slotted into the *kleroterion* — a stone slab with rows of slots and a tube — before any draw began. Black and white balls fell through the tube and decided, row by row, who judged that day. The machine put chance in two places: the order the tickets were placed, and the order the balls came out.

---

## Try it

**In a browser, nothing to install:** [thaisfreis.github.io/pinakion/web/index.html](https://thaisfreis.github.io/pinakion/web/index.html)

Fetch a drand round, let the page recompute the panel, and paste what the chain returned. It says whether they match. The page is this repository's `web/`, served from `main` — what you read here is what runs there.

**On the command line**, with no dependencies at all:

```bash
python3 scripts/verificar_sorteio.py \
  7fe86201ed0c0edcf570b52e55382e4ce981a4ec8dddb9f10fc454e33a2719f1 200 25
```

That prints the panel that contract `CBPHNZLG…` drew on testnet, in [this transaction](https://stellar.expert/explorer/testnet/tx/d077a6a953a4c5c48a75306754e838a89bf00c92bd00a261c02cc6b03f2f1a1d).

---

## How it works

**1. The ticket is an asset, and the roster is frozen in the ledger.** Each juror holds one unit of a classic Stellar asset whose issuer requires authorization. The `kleroterion` contract enrols a juror by creating the trustline itself (SAC `trust`, Protocol 26), authorizing it and minting the ticket; the issuer then sets its master weight to 0, so nobody but the contract can ever issue or authorize a ticket again. `lock(k, reveal_after)` closes the roster — nobody enrols or is impeded after it — and fixes `k` and the exact beacon round from the ledger clock; `seal(sig)` draws once, with that round. Any Horizon client lists the roster: `GET /accounts?asset=PINAKION:issuer`.

**2. The entropy arrives afterwards.** A [drand](https://drand.love) quicknet round — a public beacon that publishes an unpredictable value every three seconds. Its BLS12-381 signature is verified **on-chain**. Nobody, including us, knew the value when the roster was committed.

**3. The draw is reproducible.** The round's randomness reseeds the Soroban PRNG, and the shuffle is Fisher-Yates. The host's own source states that its seed-derivation salt is part of the protocol and must be identical in every implementation — which is why a browser can recompute a panel at all.

---

## What is proven today

| Claim | Evidence |
|---|---|
| A live drand round verifies on Stellar testnet | [tx `c74150ee…`](https://stellar.expert/explorer/testnet/tx/c74150eeb7b73d0a196d20b44ed851a140812b3ac67e579b670b87ee2c3ffeea) → `true` |
| A signature forged under a different keypair is rejected | [tx `30de94fe…`](https://stellar.expert/explorer/testnet/tx/30de94feaf1ac2e6dea42a03edc8fe9be158b10716f03b4be707eb4328559375) → `false` |
| The panel is reproducible off-chain | two independent implementations, Python and JavaScript, written from the host's semantics; locked as a regression test |
| One verification costs 26,719,483 CPU instructions, 161,818 bytes | measured with `env.cost_estimate().budget()` in the test host — **not** a figure taken from a testnet transaction |
| The contract trusts one named beacon, auditable on-chain | `beacon()` returns the quicknet chain hash |
| A draw from a roster of 1,000 costs 2,274,884 instructions | measured the same way |
| The roster is a classic asset, frozen by a locked issuer, with trustlines the contract created | the sequence in [`scripts/sequencia_bilhete.sh`](scripts/sequencia_bilhete.sh), run on testnet: issuer flags [`b104d57e…`](https://stellar.expert/explorer/testnet/tx/b104d57ede1234c87b84c511530862fb81e9bbcdc9fd30e5474d0293e905e7d8), master weight 0 [`77808909…`](https://stellar.expert/explorer/testnet/tx/77808909a7a4a14bbd097a8622c0f354c83505a83eaee5b3c6f5b528e1056f63), lock [`8a98bcf6…`](https://stellar.expert/explorer/testnet/tx/8a98bcf680b55a272e59df393c162a48c83731ee2f160abb039b763dfc7cef9c) |
| An impeded juror is skipped and the next in the drawn order sits; the off-chain recomputation returns the same panel, address for address | draw [`fc4a5f93…`](https://stellar.expert/explorer/testnet/tx/fc4a5f93fa7c01779c81ffc924b29e766496ddbc7ca9556ff0a81babec2b85c2) and `scripts/verificar_painel.py` |

Full record, with every hash: [`docs/evidencia-testnet-sorteio.md`](docs/evidencia-testnet-sorteio.md) for the beacon, [`docs/evidencia-testnet-bilhete.md`](docs/evidencia-testnet-bilhete.md) for the ticket.

Verifying a drand round on Soroban is not new: [`AshFrancis/splicers`](https://github.com/AshFrancis/splicers) and [`NibrasD/Stellar-VRF`](https://github.com/NibrasD/Stellar-VRF) did it before us with the same host functions. What neither does is seat a jury from a roster frozen before the case.

## What is not

- The case lifecycle is designed and **not built**. Of the roster cycle, enrol → impede → lock → seal → claim → finalize → reveal exists in `kleroterion` and has passed unit tests only.
- **Since T-038 `kleroterion` draws sealed** (each juror proves a seat from a fresh account; the panel is public after the verdict). The Fisher-Yates draw described above is `beacon-verifier`'s, and was `kleroterion`'s in the testnet run linked here; this page is rewritten in T-041.
- The evidence is one run with seven jurors on one machine; `AUTH_IMMUTABLE` was not set; the costs of `enroll`, `lock` and `draw_round` were not measured. That run used the pre-T-033 contract (`draw_round` with a caller-given round); the current `lock(k, reveal_after)` → `seal(sig)` has passed its unit tests and has **not** been run on testnet.
- Nothing has been audited. Nothing has run on mainnet.
- Below a roster of about sixteen the draw guarantees nothing, and the contract should refuse rather than pretend — see [`docs/limite-inferior-do-rol.md`](docs/limite-inferior-do-rol.md).

---

## Run it

The toolchain is pinned: `rust-toolchain.toml` gives you rustc 1.97.1 with the `wasm32v1-none` target, and `.nvmrc` gives you Node 24.14.1. There are no npm dependencies. The Stellar CLI (28.1.0) is the one thing you install yourself.

```bash
cargo test --locked --workspace     # 41 tests, including the golden vectors
npm test                            # the same vectors, in the browser's verifier
python3 scripts/conferir_vetores.py # and in the dependency-free one
stellar contract build              # plain `cargo build` to Wasm fails by design
```

The three implementations are held to one file, [`tests/vetores-de-ouro.json`](tests/vetores-de-ouro.json). Change a number in it and all three go red.

Deploy and exercise it on testnet:

```bash
stellar contract deploy --wasm target/wasm32v1-none/release/beacon_verifier.wasm \
  --source <identity> --network testnet
```

Then `verify --round <n> --sig <hex>`, `beacon`, and `draw --seed <hex> --n <size> --k <panel>`.
The beacon key is pinned in the contract — there is no key to pass.

The whole ticket sequence — issue, lock the issuer, enrol, impede, close, wait for the round, draw, recompute — is one script, and it prints every hash. It was last run before T-033 changed `lock` and replaced `draw_round` with `seal`; the current version has not been run (T-040):

```bash
scripts/sequencia_bilhete.sh <issuer-identity> <clerk-identity> <juror-identity>...
```

---

## Layout

```
THESIS.md      the root — the guarantee, who it is for, why Stellar
PLAN.md        where the work stands and what happens next
DESIGN.md      colours, type and the mark
AGENTS.md      how to work here, short
CLAUDE.md      how to work here, long
docs/SPEC.md   goals, invariants, gates, task board, decisions
docs/SOURCES.md every external fact checked, with its date

contracts/beacon-verifier/   the beacon: verify, randomness, draw
contracts/kleroterion/       the roster as an asset: enroll, impede, lock, seal
contracts/poseidon/          Poseidon over BN254 on the host permutation (T-034 spike, not wired in)
contracts/groth16-verifier/  a BN254 Groth16 verifier (T-034 spike, not wired in)
circuits/                    the seat-claim circuit and its test-only prover — GPL-3.0
scripts/verificar_sorteio.py an independent recomputation, no dependencies
scripts/verificar_painel.py  the same, from a roster to a panel of addresses
scripts/sequencia_bilhete.sh the classic sequence on testnet, end to end
web/                         the same recomputation, in a browser
```

---

## Reading

The design is written down before it is built, and the documents say which is which:

- [`docs/desenho-rol-sorteio-recusas.md`](docs/desenho-rol-sorteio-recusas.md) — the roster cycle, the quorum, the recusals, and the arithmetic of over-drawing
- [`docs/desenho-sorteio-e-sigilo.md`](docs/desenho-sorteio-e-sigilo.md) — the draw and confidential case files
- [`docs/limite-inferior-do-rol.md`](docs/limite-inferior-do-rol.md) — how small a community can be before sortition stops meaning anything
- [`docs/avaliacao-genesis-unlocked.md`](docs/avaliacao-genesis-unlocked.md) — the project scored against the hackathon rubric, by its builder, with the objection a judge will raise

Built for HackMeridian 2026.

## Licence

MIT (`LICENSE`), except `circuits/`, which is GPL-3.0 (`circuits/LICENSE`) because it builds on circomlib. `contracts/poseidon/` also carries Apache-2.0 code from Stellar's `rs-soroban-poseidon` (`contracts/poseidon/LICENSE-APACHE`). See DEC-021.
