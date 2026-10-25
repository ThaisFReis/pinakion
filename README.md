# Pinakion

**Verifiable jury selection for governance on Stellar.**

A deciding panel drawn from a roster committed **before** the case existed, using entropy nobody controlled at the time of the commitment, that **anyone can recompute and check**.

The guarantee, in one sentence: *nobody chooses who judges you.*

> In classical Athens the *pinakion* was a small bronze ticket engraved with a citizen's name. It was slotted into the *kleroterion* — a stone slab with rows of slots and a tube — before any draw began. Black and white balls fell through the tube and decided, row by row, who judged that day. The machine put chance in two places: the order the tickets were placed, and the order the balls came out.

---

## Try it

**In a browser, nothing to install.** Paste a drand round and a panel, and the page recomputes it:
[claude.ai/artifact/E8fZMeWK3bcXo8BH1MqiaM](https://claude.ai/artifact/E8fZMeWK3bcXo8BH1MqiaM)

**On the command line**, with no dependencies at all:

```bash
python3 scripts/verificar_sorteio.py \
  104870a5f58e570f7bb7d3f63331a752e42245543c17d1628c75bf775ff268d1 200 25
```

That prints the panel that contract `CCLWP7WO…` drew on testnet, in [this transaction](https://stellar.expert/explorer/testnet/tx/dda87cc05c16beed6e2720f579990d15c79e46dd9dc9f920b24917e8e1d87d65).

---

## How it works

**1. The roster is committed first.** A Merkle root of the eligible members, published with a window in which anyone can contest an entry, and a minimum beacon round fixed when it becomes definitive. No draw may use a round older than that.

**2. The entropy arrives afterwards.** A [drand](https://drand.love) quicknet round — a public beacon that publishes an unpredictable value every three seconds. Its BLS12-381 signature is verified **on-chain**. Nobody, including us, knew the value when the roster was committed.

**3. The draw is reproducible.** The round's randomness reseeds the Soroban PRNG, and the shuffle is Fisher-Yates. The host's own source states that its seed-derivation salt is part of the protocol and must be identical in every implementation — which is why a browser can recompute a panel at all.

---

## What is proven today

| Claim | Evidence |
|---|---|
| A live drand round verifies on Stellar testnet | [tx `9de19747…`](https://stellar.expert/explorer/testnet/tx/9de197478cd68e93a1f2d7a16c9b3f997580584239c0e5e83fddc0c322a2238d) → `true` |
| A tampered round is rejected | [tx `2ebda22b…`](https://stellar.expert/explorer/testnet/tx/2ebda22bc89b0f8f72133381e9b30d94047d6fecc028c42a68a59b08053ec5ea) → `false` |
| The panel is reproducible off-chain | two independent implementations, Python and JavaScript, written from the host's semantics; locked as a regression test |
| One verification costs 26,718,596 CPU instructions, 161,546 bytes | measured with `env.cost_estimate().budget()` in the test host — **not** a figure taken from a testnet transaction |
| A draw from a roster of 1,000 costs 2,274,884 instructions | measured the same way |

Full record, with every hash: [`docs/evidencia-testnet-sorteio.md`](docs/evidencia-testnet-sorteio.md).

## What is not

- **The beacon public key is still a call parameter, not a pinned constant.** Anyone can pass their own key and have any round verify. Nothing should depend on this contract until that changes.
- The case lifecycle, the roster cycle and the recusal rules are designed and **not built**.
- Nothing has been audited. Nothing has run on mainnet.
- Below a roster of about sixteen the draw guarantees nothing, and the contract should refuse rather than pretend — see [`docs/limite-inferior-do-rol.md`](docs/limite-inferior-do-rol.md).

---

## Run it

Requires Rust with the `wasm32v1-none` target, Stellar CLI 28.x and Node 20+.

```bash
cargo test --locked --workspace     # 13 tests, including the golden vectors
stellar contract build              # plain `cargo build` to Wasm fails by design
```

Deploy and exercise it on testnet:

```bash
stellar contract deploy --wasm target/wasm32v1-none/release/beacon_verifier.wasm \
  --source <identity> --network testnet
```

Then `verify --round <n> --sig <hex> --pk <hex>` and `draw --seed <hex> --n <size> --k <panel>`.

---

## Layout

```
THESIS.md      the root — the guarantee, who it is for, why Stellar
DESIGN.md      colours, type and the mark
AGENTS.md      how to work here, short
CLAUDE.md      how to work here, long
docs/SPEC.md   goals, invariants, gates, task board, decisions
docs/SOURCES.md every external fact checked, with its date

contracts/beacon-verifier/   the Soroban contract: verify, randomness, draw
scripts/verificar_sorteio.py an independent recomputation, no dependencies
web/                         the same recomputation, in a browser
```

---

## Reading

The design is written down before it is built, and the documents say which is which:

- [`docs/desenho-rol-sorteio-recusas.md`](docs/desenho-rol-sorteio-recusas.md) — the roster cycle, the quorum, the recusals, and the arithmetic of over-drawing
- [`docs/desenho-sorteio-e-sigilo.md`](docs/desenho-sorteio-e-sigilo.md) — the draw and confidential case files
- [`docs/limite-inferior-do-rol.md`](docs/limite-inferior-do-rol.md) — how small a community can be before sortition stops meaning anything

Built for HackMeridian 2026.
