# Evidence: T-034, kill-or-go

This page answers one question: does Plan B (DEC-015) fit on Stellar today? It takes two checks:

- **The hash matches.** The Poseidon the contract computes over the CAP-75 host permutation must equal circomlib's.
- **The proof verifies.** A proof of Pinakion's circuit must verify on testnet within the per-transaction budget.

Both checks pass. **The decision is go (DEC-022).** Every hash on this page was produced on **2026-10-25**, and nothing here has run on mainnet.

## 1. Poseidon: the contract's sponge equals circomlib's

`contracts/poseidon/` assembles the sponge circomlib uses. The state starts at zero, the capacity cell is position 0, the inputs follow, one permutation is applied, and the output is read from position 0. The sponge runs over the host permutation with circomlib's BN254 parameters for t = 2 and t = 3. Both the sponge and the parameters are adapted from `stellar/rs-soroban-poseidon` v28.0.0 (Apache-2.0, DEC-021).

`tests/vetores-poseidon.json` holds **22 vectors**, made by `node circuits/vetores_poseidon.mjs` with circomlibjs 0.1.7:

- 10 vectors are Poseidon(1). The inputs include 0, 1, 2⁶⁴, 2¹²⁸, r−2 and r−1.
- 12 vectors are Poseidon(2). The inputs include (0, r−1), (r−1, r−1), 2²⁵³ and hashed seeds.

The contract returns the same 22 values, byte for byte. The test is `bate_com_a_circomlib`, run by G2. Two of the vectors, [1, 2] and [3, 4], also appear in the upstream crate's own tests with the same outputs. An input ≥ r is refused rather than reduced (`entrada_fora_do_campo_e_recusada`).

Costs, **measured on the test host** (`env.cost_estimate().budget()`), not in a transaction:

| | CPU instructions | memory bytes |
|---|---|---|
| Poseidon(1), new sponge | 1,001,368 | 57,296 |
| Poseidon(2), new sponge | 1,454,641 | 81,832 |
| Full tree of depth 6: 64 Poseidon(1) leaves + 63 Poseidon(2) nodes, sponges built once | 109,821,216 | 3,373,240 |

These figures are asserted in `contracts/poseidon/src/test.rs`, and G7 checks that this page still prints them.

## 2. One proof of our circuit, verified on testnet

- **Circuit.** `circuits/pinakion_claim.circom`, ported from the author's `role_membership.circom` of 2026-10-25 (DEC-017).
  - Public inputs: `[root, seed, y, claimer]`.
  - Private inputs: `s` and a Merkle path of depth 6.
  - It enforces `y = Poseidon(seed, s)`.
  - It compiles to 1,936 non-linear and 2,117 linear constraints, 4,053 in total.
- **Proof.** `circuits/provar.sh` ran a **test-only** setup and made one proof, exported to `tests/prova-claim-spike.json`:
  - The setup was local and single-party: powers of tau 2¹² and phase 2 each had one contribution, and the entropy was thrown away.
  - snarkjs 0.7.6 verifies the proof off-chain: `OK!`.
- **Verifier.** `contracts/groth16-verifier/`, ported unchanged in logic from the author's verifier of 2026-10-25 13:02 UTC (DEC-017). The optimised Wasm is 3,141 bytes.

| Step | Result | Transaction |
|---|---|---|
| deploy with this circuit's `vk` | `CCICALAV6OQOOE347USWKQUASXFRXY6VTC2JBWA55SJVSSKVPVXPMCE3` | — |
| `verify`, valid proof | **`true`** | [`76fc290f…`](https://stellar.expert/explorer/testnet/tx/76fc290f9c43ca976a9ccdd11492a409266f026fa295292382b81a45ae1ef2c7) |
| `verify`, same proof, `y` changed to 1 | **`false`** | [`374632d9…`](https://stellar.expert/explorer/testnet/tx/374632d95a8e55dde2e06156c17d3a92bf1eaa3d23d138999610481a8b9d72ed) |

What the network recorded for the valid call. These values were read with `stellar tx fetch envelope` and `stellar tx fetch fee` on 2026-10-25:

- **Instructions declared by the transaction:** 30,837,655. Simulation sets this number, and it is what the fee pays for. It is not a separate measurement of what was executed.
- **Fee charged:** 11,737 stroops (0.0011737 XLM). Of that, 11,637 is the resource fee and 100 the inclusion fee.

**Measured on the test host**, at the spike's commit `dbce1eb`: one valid `verify` cost 29,128,335 CPU instructions and 229,165 memory bytes, with the key read from storage. Since T-037 the verifier embeds the key, and the same verification through the contract costs **29,105,202** instructions and **208,994** memory bytes, asserted in `contracts/groth16-verifier/src/test.rs`. The unit tests also check these failures:

- a changed `y` fails;
- a changed `claimer` fails;
- `root` and `seed` swapped fails;
- one flipped byte in `a` or `c` never verifies;
- an input ≥ r is refused;
- a wrong number of inputs is refused.

To reproduce the run:

```bash
circuits/provar.sh
```

```bash
scripts/spike_testnet.sh <funded-testnet-identity>
```

## 3. The budget

The per-transaction ceiling on testnet is **400,000,000 CPU instructions** (`tx_max_instructions`), and the per-ledger ceiling is 580,000,000. Both were read on 2026-10-25 from the network's own config with `stellar network settings --network testnet`. `developers.stellar.org/docs/networks/resource-limits-fees` gives no figure: it sends the reader to that command and to the Lab's *Network Limits* page. Mainnet's settings were not read.

| Operation | Instructions | Share of 400 M |
|---|---|---|
| `verify` on testnet, declared | 30,837,655 | 7.7 % |
| full depth-6 tree, test host | 109,821,216 | 27.5 % |

Q-09 recorded two ceilings, 100 M in the old whitepaper and 400 M as cited from another project. The 400 M figure is confirmed for testnet.

## Limitations

- **The setup is for testing only.** Whoever ran `circuits/provar.sh` could have kept the toxic waste and forged proofs. T-036 replaces it with a public powers-of-tau file and a published phase-2 transcript. Even then it stays single-party, and the README says so.
- **Two parts are not yet joined.** The proof was made with circomlibjs's tree, and the contract's sponge was checked against circomlibjs. Nothing yet computes the root on-chain and feeds it into a proof; that is T-035 with T-037.
- **The circuit only binds `claimer`.** It squares `claimer` so that it is part of the statement. Deriving it from the sending account is T-037's job.
- **Little room in the constraint count.** 4,053 constraints fit under the 4,096 of a 2¹² setup. Changing the depth or adding a range check means a 2¹³ setup.
- **Not every cost is on-chain yet.** The tree cost is a test-host figure. `kleroterion` does not build the tree yet, so no transaction measures it. If T-035 builds the tree inside `lock`, it uses about a quarter of a transaction. An incremental path update at each `enroll` costs one Poseidon(1) and six Poseidon(2), about 10 M instructions estimated from the table above, and keeps `lock` cheap.
- **Mainnet limits were not read.** Pinakion is testnet only.
