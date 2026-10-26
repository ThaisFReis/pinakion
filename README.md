# Pinakion

**Sealed jury selection for governance on Stellar.** A deciding panel is drawn from a roster committed **before** the case existed, with entropy nobody controlled when the roster closed. Each juror proves their seat without being named, and anyone can recompute the panel once the verdict is in.

**Nobody chooses who judges you. Nobody knows who until they have sat and judged. Afterwards, anyone recomputes the panel from the ledger, among those who revealed.**

> **Before you rely on it.** These are the limits, and the rest of this page is read through them.
> - **Testnet only, unaudited.** Nothing has run on mainnet.
> - **The circuit's setup is single-party.** Whoever ran it could forge seats ([`circuits/setup/TRANSCRICAO.md`](circuits/setup/TRANSCRICAO.md)).
> - **Keeping a seat unnamed depends on the seat account.** It must be funded with no trail back to the juror; a relayer is needed and not built.
> - **A juror who loses `s` loses the seat.**
> - **The panel is recomputable only among the jurors who revealed.** The silent ones are listed by address.
> - **Deliberation off-chain breaks the seal** as soon as the jurors meet.

> In classical Athens the *pinakion* was a bronze ticket with a citizen's name. A first lot decided who judged that day; a second, with lettered acorns, sent each juror to one court, so that nobody chose his court and nobody packed one (*Ath. Pol.* 63–65). Athens is cited for *no choosing*. The sealing is ours.

---

## Try it

**The demo: a whole case, from the roster to the verdict:** [thaisfreis.github.io/pinakion/web/processo.html](https://thaisfreis.github.io/pinakion/web/processo.html). It is a fictional code-of-conduct case with real testnet transactions:
- 40 people on the roster, and a level-3 case with 7 seats;
- the person judged was on the roster and was impeded by the lock itself;
- the complaint, the evidence, the defence and the minutes were filed by their hashes;
- 6 of the 7 seated jurors attended from their seat accounts, against a quorum of 5;
- the vote was secret, on Tessera, and came out 5 to 1, upheld with 4 required;
- the jurors revealed afterwards.

The page checks every phase in your browser from the ledger's events. **Present** shows one phase per screen, for a projector or a recording.

> **The vote runs on Tessera.** [Tessera](https://github.com/ThaisFReis/tessera) is the author's earlier secret-ballot module, from the Find Your Way hackathon. It is used as it is: its contract at commit `356c22f`, deployed unchanged on testnet, and its CLI at the same commit with a one-line client fix ([`scripts/tessera-cli.patch`](scripts/tessera-cli.patch)). What this project adds is the binding: the `processo` contract accepts a Tessera vote only if its voter list is exactly the jurors who attended. It recomputes Tessera's Merkle root on-chain to check that. On top of Tessera's score it applies the level's verdict rule.

```bash
python3 scripts/verificar_processo.py docs/evidencia-processo-registro.json
```

The same checks from the command line, with no dependencies: the draw, the opening, the record against the documents in [`docs/caso-demo/`](docs/caso-demo), attendance against the panel, Tessera's voter root against the attendees, and the verdict against Tessera's score. Every transaction is in [`docs/evidencia-processo.md`](docs/evidencia-processo.md). To run a case yourself: `scripts/sequencia_processo.sh`.

**The first sealed draw (2026-10-26), on its own:**

**Check the testnet panel yourself, from the ledger and drand, with no dependencies:**

```bash
python3 scripts/verificar_painel.py docs/evidencia-testnet-registro.json
```

It recomputes, from the 49 events of the 2026-10-26 run:
- the commitment root, the seed and the threshold;
- every claim, the recusal and all 18 reveals;
- the panel itself.

It prints each juror's state and **`CONFERE — 77 conferencias, 0 falha(s)`**. To rebuild that record from the network instead of the file:

```bash
python3 scripts/registro.py CCRBKLS64WDUSKQSQF4ON4GJF4Q5KPEAF54TQD4RIWDAJ5TOERKU2NVL 5390695 > registro.json
```

**The same check in a browser, person by person:** [thaisfreis.github.io/pinakion/web/bilhetes.html](https://thaisfreis.github.io/pinakion/web/bilhetes.html). It is read-only. It shows each roster account as *enrolled · impeded · recused · seated · revealed · silent*, with the transaction links, the root, the drand round, the seed and `T`, and runs the same recomputation in your browser.

**On the ledger itself:**
- the asset, on [stellar.expert](https://stellar.expert/explorer/testnet/asset/PINAKION-GC5Z55ME4WQPSLT7GIFOOJEGZVKWDUVVGGACXQ4LDVWOLTOJK7KGZXEL);
- the roster, as any Horizon sees it: [`/accounts?asset=PINAKION:GC5Z55ME…`](https://horizon-testnet.stellar.org/accounts?asset=PINAKION:GC5Z55ME4WQPSLT7GIFOOJEGZVKWDUVVGGACXQ4LDVWOLTOJK7KGZXEL);
- the contract, [`CCRBKLS6…`](https://stellar.expert/explorer/testnet/contract/CCRBKLS64WDUSKQSQF4ON4GJF4Q5KPEAF54TQD4RIWDAJ5TOERKU2NVL).

**Hand the panel to Tessera:** `python3 scripts/exportar_tessera.py docs/evidencia-testnet-registro.json` writes the seat accounts in panel order, in the format Tessera's `--aptos` reads. The output for this run is [`docs/evidencia-testnet-aptos.txt`](docs/evidencia-testnet-aptos.txt).

**The open draw, in a browser:** [thaisfreis.github.io/pinakion/web/index.html](https://thaisfreis.github.io/pinakion/web/index.html) recomputes a Fisher-Yates panel from a drand round. That is `beacon-verifier`'s draw, the one the project began with.

---

## How it works

**1. The roster is a frozen classic asset, and each ticket carries a commitment.**
- Each juror holds one unit of PINAKION, a classic Stellar asset with `AUTH_REQUIRED`, `AUTH_REVOCABLE` and `AUTH_IMMUTABLE`.
- `kleroterion` enrols a juror by creating the trustline itself (SAC `trust`, Protocol 26) and records the juror's commitment `c = Poseidon(s)`. Only the juror knows `s`.
- The contract keeps the Poseidon Merkle tree of the commitments.
- The issuer's master weight is 0, so nobody else can ever issue or authorize a ticket.
- `lock(gravidade, reveal_after, acusado)` freezes the roster, the tree's root, `k` and the drand round, all from the ledger clock. Since T-045, `k` comes from a severity table fixed at construction, the person judged is impeded in the same call if enrolled, and `suspended(acusado)` is true for a window also fixed at construction. That version passes its tests and has not run on testnet; the 2026-10-26 run used `lock(k, reveal_after)`.

**2. The entropy arrives afterwards, and it is verified on-chain.** `seal` takes the signature of that [drand](https://drand.love) quicknet round, verifies it on-chain (BLS12-381, CAP-59) and fixes the seed and a threshold `T`. Nobody, including us, knew the value when the roster closed.

**3. A seat is proven, not listed.**
- A juror whose `y = Poseidon(seed, s)` falls under `T` makes a Groth16 proof on BN254: *my commitment is a leaf of this root, and this is my `y`*. The tool is [`circuits/pinakion.mjs`](circuits/pinakion.mjs), which reads the contract's own events.
- The juror sends the proof from a fresh account. The contract assembles the public inputs itself, so nobody hands it a ready-made claim.
- The claim event carries `y` and nothing else.
- `finalize` seats the `k` smallest `y`, widening `T` once if fewer than `k` claimed; the panel is never redrawn.

**4. After the verdict, the panel is public.** Each juror reveals `s`. Anyone checks `Poseidon(s) = c` and `y = Poseidon(seed, s)`, and recomputes the panel from the events and drand alone.

---

## What is proven today

| Claim | Evidence |
|---|---|
| A seat is proven by a Groth16 proof (BN254, Poseidon) verified on testnet, and the claim event carries no address | run of 2026-10-26: six claims from six fresh accounts, e.g. [`e9c3d285…`](https://stellar.expert/explorer/testnet/tx/e9c3d285a10198f02e8fcf7a634acb34fa23fb86195e9940517a0b78d44c000e); `nenhum_evento_liga_y_a_endereco` |
| The sealed panel is recomputable from the ledger and drand alone, among those who revealed | `scripts/verificar_painel.py` on that run: `CONFERE`, 77 checks; re-checked offline by G6 |
| Before reveal, no seat is tied to a roster address on the ledger; seats are **indistinguishable within the panel of k**, as far as the seat account's funding leaves no trail | the events of that run (`docs/evidencia-testnet-registro.json`); INV-17 |
| A seat claim costs about 32 M instructions declared on testnet, **8 %** of the 400 M per-transaction ceiling | [`docs/evidencia-testnet-bilhete.md`](docs/evidencia-testnet-bilhete.md), from the 53 transactions (G9) |
| The contract's Poseidon and commitment tree are circomlib's | 22 Poseidon vectors and 4 trees, in the contract, Python and JS (G2, G5, G6) |
| A live drand round verifies on Stellar testnet; a signature forged under another keypair does not | [`c74150ee…`](https://stellar.expert/explorer/testnet/tx/c74150eeb7b73d0a196d20b44ed851a140812b3ac67e579b670b87ee2c3ffeea) → `true`, [`30de94fe…`](https://stellar.expert/explorer/testnet/tx/30de94feaf1ac2e6dea42a03edc8fe9be158b10716f03b4be707eb4328559375) → `false` |
| One beacon verification costs 26,719,483 CPU instructions, 161,818 bytes | `env.cost_estimate().budget()` in the test host, **not** a figure from a testnet transaction |
| The open draw (`beacon-verifier`) from a roster of 1,000 costs 2,274,884 instructions, and is reproduced off-chain | measured the same way; two independent implementations |
| The roster is a classic asset frozen by a locked issuer, with trustlines the contract created | issuer flags, `AUTH_IMMUTABLE` and master weight 0 in [`docs/evidencia-testnet-bilhete.md`](docs/evidencia-testnet-bilhete.md); the earlier open-draw run, kept on the same page, has [`b104d57e…`](https://stellar.expert/explorer/testnet/tx/b104d57ede1234c87b84c511530862fb81e9bbcdc9fd30e5474d0293e905e7d8), [`77808909…`](https://stellar.expert/explorer/testnet/tx/77808909a7a4a14bbd097a8622c0f354c83505a83eaee5b3c6f5b528e1056f63), [`8a98bcf6…`](https://stellar.expert/explorer/testnet/tx/8a98bcf680b55a272e59df393c162a48c83731ee2f160abb039b763dfc7cef9c), [`fc4a5f93…`](https://stellar.expert/explorer/testnet/tx/fc4a5f93fa7c01779c81ffc924b29e766496ddbc7ca9556ff0a81babec2b85c2) |

Full records: [`docs/evidencia-testnet-bilhete.md`](docs/evidencia-testnet-bilhete.md) for the sealed run and the ticket, and [`docs/evidencia-testnet-sorteio.md`](docs/evidencia-testnet-sorteio.md) for the beacon.

## What is not

- **Part of the case lifecycle is built and part is only designed.** Built and run on testnet: the opening, the record, attendance and quorum, the bound vote and the verdict, all in `contracts/processo`. Designed and not built: roster renewal, contestation windows, a redraw after an adjournment, and confidential records with justified access. In the demo the case documents are public. The roster cycle — enrol → impede → lock → seal → claim → recuse → finalize → reveal — is built, passes its unit tests and ran once on testnet.
- **The vote depends on Tessera.** Its privacy floor is five confidential votes, so a case needs at least five seats. Below that, the case is refused at `abrir`. The correctness of the score is Tessera's verifier's job (`tessera verificar`), not this repository's. The demo's windows are short: the approved reveal floor is 30 days.
- **The testnet run was one machine playing all 20 jurors and every seat account,** with short windows: 60 ledgers to claim, and a reveal ledger minutes away rather than "after the verdict" (§11 Q-24).
- **Nothing has been audited. Nothing has run on mainnet.**
- **The drand signature is verified on-chain at `seal`,** not again by the off-chain checker, which takes the randomness from drand's API.
- **Below a roster of about sixteen the draw guarantees little.** See [`docs/limite-inferior-do-rol.md`](docs/limite-inferior-do-rol.md).

## Related work

- **[Tessera](https://github.com/ThaisFReis/tessera)** is the author's earlier module, awarded the same week. *Tessera decides how one votes, in secret; Pinakion decides who sits.* In the case cycle, the vote **is** a Tessera vote, bound to the attendees (see *Try it*).
- **Kleros** seats anonymous jurors selected by stake, from a market. Pinakion seats the institution's own roster, fixed before the case.
- **[`AshFrancis/splicers`](https://github.com/AshFrancis/splicers) and [`NibrasD/Stellar-VRF`](https://github.com/NibrasD/Stellar-VRF)** verified drand on Soroban before us, with the same host functions. Neither seats a jury.

## Prior work, dated

The Groth16 verifier and the claim circuit are ported from the author's own earlier code. That code was written on **2026-10-25** inside the hacking window, in a withdrawn generation of this project: `groth16-verifier` at 13:02 UTC and `role_membership.circom` at 12:54 UTC (DEC-017, `docs/SOURCES.md` *Copied code*). The Poseidon sponge is adapted from Stellar's `rs-soroban-poseidon` v28.0.0, Apache-2.0 (DEC-021).

---

## Run it

The toolchain is pinned. `rust-toolchain.toml` gives you rustc 1.97.1 with the `wasm32v1-none` target, and `.nvmrc` gives you Node 24.14.1. `npm ci` installs the circuit tools, pinned and GPL-3.0, used by `circuits/` only. The Stellar CLI (28.1.0) is the one thing you install yourself.

```bash
cargo test --locked --workspace     # 85 tests, including the golden vectors
npm test                            # the same vectors, in the browser's verifier
node circuits/test.mjs              # the circuit, its setup and pinakion prove
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

The whole sealed sequence — issue, lock the issuer, enrol, impede, close, seal, prove and claim from fresh seat accounts, recuse, finalize, reveal, read the events back and recompute — is one script, and it records every hash. Its last run (2026-10-26) is `docs/evidencia-testnet-bilhete.md`:

```bash
scripts/sequencia_bilhete.sh <prefix>
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
contracts/kleroterion/       the roster as an asset, drawn sealed: enroll … seal, claim, recuse, finalize, reveal
contracts/processo/          the case: abrir, comparecer, registrar, vincular_votacao (Tessera), decidir
contracts/poseidon/          Poseidon over BN254 on the host permutation, circomlib's
contracts/groth16-verifier/  the BN254 Groth16 verifier, with the claim circuit's key embedded
circuits/                    the seat-claim circuit, its setup and `pinakion prove` — GPL-3.0
scripts/verificar_sorteio.py an independent recomputation, no dependencies
scripts/verificar_painel.py  the sealed panel, checked from the contract's events and drand alone
scripts/sequencia_bilhete.sh the sealed sequence on testnet, end to end
scripts/registro.py          the contract's events, read from the network, with tx hashes
scripts/exportar_tessera.py  the panel's seat accounts, as Tessera's --aptos list
scripts/sequencia_processo.sh the whole case on testnet, the vote on Tessera
scripts/registro_processo.py the case's events and Tessera's vote, read from the network
scripts/verificar_processo.py the whole case, checked from the record alone
scripts/tessera-cli.patch    the one-line fix Tessera's CLI needs to talk to its own contract
web/                         the recomputations in a browser; processo.html is the case, bilhetes.html the sealed panel
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
