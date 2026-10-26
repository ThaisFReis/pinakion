# Evidence: the sealed draw, on testnet

Every hash on this page is real. They were produced on **2026-10-26** (UTC) by one run of [`scripts/sequencia_bilhete.sh t040b`](../scripts/sequencia_bilhete.sh) (T-040). It used the contracts on `main` plus T-037 to T-039. Links open on stellar.expert, and the same hashes resolve on any testnet Horizon or RPC. Nothing here has run on mainnet.

| | |
|---|---|
| issuer | `GC5Z55ME4WQPSLT7GIFOOJEGZVKWDUVVGGACXQ4LDVWOLTOJK7KGZXEL` |
| asset | `PINAKION:GC5Z55ME…` |
| SAC | `CDHYWEJZC3FJHF2WP77CTFXLZ5GRXOORALMMVHQYKSGOEFD2P65CJPJC` |
| Groth16 verifier (key embedded) | `CCWD4XWB6L5LF3RVL2JHPKPIRIFJ47PCNRCBRJLPEMYGQSPNAZXKZCSH` |
| kleroterion | `CCRBKLS64WDUSKQSQF4ON4GJF4Q5KPEAF54TQD4RIWDAJ5TOERKU2NVL` |
| clerk (*escrivão*) | `GDHJSZACGX5JDBX622S3TSWAF3F6IV7XUB5KCAE6CJROA4YVD23O6XBZ` |
| drand quicknet round | 33391343, randomness `fce63082…c84b69` |

## What the run shows

**The roster is a classic ledger fact, and it is frozen.**
- The issuer set `AUTH_REQUIRED` and `AUTH_REVOCABLE` ([`ddc31989…`](https://stellar.expert/explorer/testnet/tx/ddc319897e0482046a000c3135b26c137c1be5f7ed3747dd90b52ced8222662d)).
- Then it set **`AUTH_IMMUTABLE`** ([`c5688011…`](https://stellar.expert/explorer/testnet/tx/c5688011dc5473d00468f03ec16b233a6412dd124e8468b2a8fc9454582b2172)), so those flags can never change.
- It handed the SAC admin to `kleroterion` ([`f839e164…`](https://stellar.expert/explorer/testnet/tx/f839e16474dcba7b7fefe39e69b97dfcc52cd0cb7db1f4933134b4839c3100cc)).
- Then it set its master weight to 0 ([`7e1fe46f…`](https://stellar.expert/explorer/testnet/tx/7e1fe46f4d1cc9d37587f32698022ee1a6bb2051041b8f2564bba25dc9f2b9f1)).
- Horizon shows `auth_immutable: true` and master weight `[0]`.

**Twenty jurors enrolled, each with a commitment `c = Poseidon(s)`.** Every `s` was drawn locally and kept in `build/`, outside git. The contract built the Poseidon tree from those commitments, one path per `enroll`. The clerk impeded j0 before the lock ([`d5f915cc…`](https://stellar.expert/explorer/testnet/tx/d5f915cc5c3a6b4bd2da744e309a41cfd16f751b361fcba47d16333858e063c6)).

**`lock(k = 3)`** ([`10ba3324…`](https://stellar.expert/explorer/testnet/tx/10ba332494fb1021faa2d1bae3d26020ca4aad525b66b7615d37d15898b2ece0)) fixed the following:
- the root;
- N = 19;
- drand round 33391343, the first round after the ledger time plus 10;
- `reveal_after`.

**`seal`** ([`a556ebfb…`](https://stellar.expert/explorer/testnet/tx/a556ebfb3377ee8814a163f642172d9eb1547487e7dbf939d6dddb59ca7f7ec5)) verified that round on-chain (CAP-59). It then fixed the seed (the round's randomness >> 8) and `T = ⌊r·6/19⌋`.

**Six jurors were under `T` and claimed a seat**, each from a fresh friendbot seat account with a Groth16 proof made by `circuits/pinakion.mjs prove` from the contract's own events: [`e9c3d285…`](https://stellar.expert/explorer/testnet/tx/e9c3d285a10198f02e8fcf7a634acb34fa23fb86195e9940517a0b78d44c000e), [`6be696dc…`](https://stellar.expert/explorer/testnet/tx/6be696dced46dc13ebc24a1326447da88a87807b78f6b9a48584c4793d0b20d2), [`80519b89…`](https://stellar.expert/explorer/testnet/tx/80519b898704562a9afa325e760509217cb2f696f5a3012b57d19240d10ba711), [`c7ca8039…`](https://stellar.expert/explorer/testnet/tx/c7ca8039e7c2ba658b2a9647cee54e7746f758590250eeb3d50393af7d79781f), [`b67cd1fd…`](https://stellar.expert/explorer/testnet/tx/b67cd1fdecb37c64ad414bbc4c5103ccf4ca8229c5b84d4f1d00c918c00ac042), [`b3e2390c…`](https://stellar.expert/explorer/testnet/tx/b3e2390c50c3e7c5075cb8937aad806adcdc2e1e861116373ebf4189fc8b1b32).
- No event names a juror beside a `y`.
- j2's claim was resent and **refused** with `Error(Contract, #17)`, `YRepetido`. The refusal happened at simulation, so there is no transaction to link.

**j3, a claimant, recused openly** ([`9942327b…`](https://stellar.expert/explorer/testnet/tx/9942327beb2d70df7976e777b7ab22994e099fb7767b67809796e0859e4d4972)). Its `y` left the list.

**`finalize`** ([`e139cf51…`](https://stellar.expert/explorer/testnet/tx/e139cf519239b9f5f9e9f3f71ff8eadf4f3e852c14993f63d605981688e6d4a4)) seated the three smallest `y` that remained, and the panel is complete. **Then all 18 jurors who could reveal did so**; the transactions are in [`evidencia-testnet-hashes.tsv`](evidencia-testnet-hashes.tsv).

**The panel was checked from the ledger and drand alone.**
- `scripts/registro.py` read the contract's events back from the RPC, into [`evidencia-testnet-registro.json`](evidencia-testnet-registro.json).
- `scripts/verificar_painel.py` then recomputed, without the contract:
  - the root;
  - the seed and `T`;
  - every claim;
  - `Poseidon(s) = c` and `y = Poseidon(seed, s)` for the recusal and for all 18 reveals;
  - the order and the panel.
- It found: **seated j2, j12, j13 · recused j3 · impeded j0 · 15 revealed without a seat · 0 seats without a reveal**.
- Its verdict is **`CONFERE — 77 conferencias, 0 falha(s)`**.

Anyone can re-run the check:

```bash
python3 scripts/verificar_painel.py docs/evidencia-testnet-registro.json
```

```bash
python3 scripts/registro.py CCRBKLS64WDUSKQSQF4ON4GJF4Q5KPEAF54TQD4RIWDAJ5TOERKU2NVL 5390695 > registro.json
```

The second command rebuilds the record from the network, with no file from this repository.

## Costs, from the transactions

Produced by `python3 scripts/custos_testnet.py docs/evidencia-testnet-hashes.tsv` (§8 G9), which reads each transaction with `stellar tx fetch`.

- **Instructions declared.** This is the budget each transaction declared, which simulation sets and the fee pays for. The network does not report the instructions actually executed.
- **Fee charged.** This is what the ledger debited, in stroops.

These are testnet figures from a single run. They are not the test-host figures in `docs/SPEC.md`.

| Operation | Txs | Instructions declared | Disk read bytes | Write bytes | Fee charged (stroops) | Example tx |
|---|---|---|---|---|---|---|
| `flags` | 1 | — | — | — | 100 | [`ddc31989…`](https://stellar.expert/explorer/testnet/tx/ddc319897e0482046a000c3135b26c137c1be5f7ed3747dd90b52ced8222662d) |
| `immutable` | 1 | — | — | — | 100 | [`c5688011…`](https://stellar.expert/explorer/testnet/tx/c5688011dc5473d00468f03ec16b233a6412dd124e8468b2a8fc9454582b2172) |
| `set_admin` | 1 | 210,084 | 0 | 488 | 3,764 | [`f839e164…`](https://stellar.expert/explorer/testnet/tx/f839e16474dcba7b7fefe39e69b97dfcc52cd0cb7db1f4933134b4839c3100cc) |
| `master_weight_0` | 1 | — | — | — | 100 | [`7e1fe46f…`](https://stellar.expert/explorer/testnet/tx/7e1fe46f4d1cc9d37587f32698022ee1a6bb2051041b8f2564bba25dc9f2b9f1) |
| `enroll` | 20 | 10,555,671 – 10,735,576 | 288 | 2,268 – 3,104 | 99,136 – 183,717 | [`38073d3c…`](https://stellar.expert/explorer/testnet/tx/38073d3c3ec1e700fb9dd5c9b73e62a3eed65819fbb02db30c6e26d1e1037357) |
| `impede` | 1 | 9,709,535 | 268 | 1,480 | 22,053 | [`d5f915cc…`](https://stellar.expert/explorer/testnet/tx/d5f915cc5c3a6b4bd2da744e309a41cfd16f751b361fcba47d16333858e063c6) |
| `lock` | 1 | 1,044,826 | 0 | 672 | 28,162 | [`10ba3324…`](https://stellar.expert/explorer/testnet/tx/10ba332494fb1021faa2d1bae3d26020ca4aad525b66b7615d37d15898b2ece0) |
| `seal` | 1 | 28,740,922 | 0 | 856 | 30,636 | [`a556ebfb…`](https://stellar.expert/explorer/testnet/tx/a556ebfb3377ee8814a163f642172d9eb1547487e7dbf939d6dddb59ca7f7ec5) |
| `claim` | 6 | 31,828,209 – 31,874,350 | 0 | 268 – 448 | 33,812 – 45,173 | [`e9c3d285…`](https://stellar.expert/explorer/testnet/tx/e9c3d285a10198f02e8fcf7a634acb34fa23fb86195e9940517a0b78d44c000e) |
| `recuse` | 1 | 4,866,801 | 0 | 548 | 23,465 | [`9942327b…`](https://stellar.expert/explorer/testnet/tx/9942327beb2d70df7976e777b7ab22994e099fb7767b67809796e0859e4d4972) |
| `finalize` | 1 | 965,530 | 0 | 280 | 32,971 | [`e139cf51…`](https://stellar.expert/explorer/testnet/tx/e139cf519239b9f5f9e9f3f71ff8eadf4f3e852c14993f63d605981688e6d4a4) |
| `reveal` | 18 | 4,886,193 – 4,886,792 | 124 | 240 | 32,046 – 32,047 | [`4025bb44…`](https://stellar.expert/explorer/testnet/tx/4025bb44cd564bbb370afc05244b58cd2ff5012084db2cde56628ad0fdde89ee) |

Every Soroban call fits well inside the 400,000,000-instruction ceiling. The largest is a `claim`, at about 32 M instructions, or 8 %.

## Limits of this evidence

- **One run, on one machine.** Every juror, and the relayer for every seat account, was this script. In a real roster each juror runs `prove` and funds their own seat account. Friendbot is a public faucet, so it is not a trail to a juror here. In production the seat account's funding is the weak point, and the README names it.
- **The setup is single-party** (`circuits/setup/TRANSCRICAO.md`). Whoever ran it could forge seats.
- **The windows are short.** The claim window was 60 ledgers. `reveal_after` was the lock ledger + 120, minutes and not "after the verdict" (Q-24).
- **Nobody claimed late or was absent.** Every juror under `T` claimed and everyone revealed, so the run shows no *drawn and absent* juror and no silent seat. The unit tests and the golden vector cover both. The 4-juror rehearsal before this run (`t40dry`, same day) went through the widening path and ended with an incomplete panel, also `CONFERE`.
- **The recusal ran before `finalize`.** The contract allows recusal only before the panel is out, while the criterion listed it after the reveals. DEC-028 records this.
- **The drand signature was not re-checked off-chain.** The contract verified it on-chain at `seal`. The Python check takes the randomness from drand's HTTP API.

---

## Previous run: the open draw (2026-10-25, before T-033)

Every hash in this section is real and was produced on **2026-10-25** by one run of
[`scripts/sequencia_bilhete.sh`](../scripts/sequencia_bilhete.sh) — the
draw was re-sent once with `--send=yes`, because the CLI only simulates a
read-only call by default. Links open on stellar.expert; the same hashes
resolve on any Horizon or RPC for testnet. Nothing here has run on mainnet.

**This run predates T-033.** The contract deployed here has `lock()` with no
arguments, a *minimum* round, `impede` open after `lock`, and
`draw_round(round, sig, k)`. The contract in the repository today has
`lock(k, reveal_after)`, an exact round, `impede` refused after `lock`, and
`seal(sig)` once (DEC-020). The run above, at the top of this page, is the current one.

| | |
|---|---|
| issuer | `GC3JC24U74YAIY26MEQMV6LZLVBDMR6CVF2AOAN6UA5XEGZ6EZOADRBB` |
| asset | `PINAKION:GC3JC24U…` |
| SAC | `CD4OL6N5PODOHYPWAGDWQIGK7S7FOCEJUEMPZH2AQZEHQOWVDL5OFGZX` |
| kleroterion | `CBUQU5MH2KWBWDHQEZL53U7CYPMKHLI3USOJOHE544IIOG7PWFQRLSFL` |
| clerk (*escrivão*) | `GAEZBHTKFRX5JTCWZXKSX6BPT75WSILEV7OEPWU42IRR6ADMLETEKZQY` |

### What the sequence proves

**The roster is a classic ledger fact, and it is frozen.** The issuer set
`AUTH_REQUIRED` and `AUTH_REVOCABLE`, handed the SAC admin to the contract,
and set its own master weight to 0. After ledger 5387444 the issuer cannot
sign anything, so no ticket can be issued or authorized by anyone except
the contract — and the contract refuses enrolment after `lock()`.

**The trustline is created by the contract.** Each juror's PINAKION trustline
was opened by SAC `trust` (CAP-73) from inside `enroll`, with the juror's own
authorization; the CLI signed each auth entry with the local identity.

**The minimum round is not a parameter.** `lock()` ran at ledger 5387455,
timestamp 1792960862, and fixed the minimum drand round at **33385833** —
the first round published strictly after that second. The draw used round
33385836, published at 1792960872, ten seconds after the lock.

**The off-chain recomputation seats the same three people in the same
order.** `scripts/verificar_painel.py`, with no dependencies, given the
roster, the impeded juror and the round's randomness.

### The transactions, in order

| Step | Transaction | Ledger | Time (UTC) |
|---|---|---|---|
| issuer funded | [`b2c6423c…`](https://stellar.expert/explorer/testnet/tx/b2c6423c23db6d3adefe581d5d2a0fdeab8b81b6b8accc85c510156863896a87) | 5387435 | 20:39:22 |
| 1 · `AUTH_REQUIRED` + `AUTH_REVOCABLE` | [`b104d57e…`](https://stellar.expert/explorer/testnet/tx/b104d57ede1234c87b84c511530862fb81e9bbcdc9fd30e5474d0293e905e7d8) | 5387437 | 20:39:32 |
| 2 · SAC deployed | [`58a18f49…`](https://stellar.expert/explorer/testnet/tx/58a18f499cd8440db7abcfd37b2dbe542a658ec74abae972f2366fed554a4f80) | 5387438 | 20:39:37 |
| 3 · kleroterion Wasm uploaded | [`3304f458…`](https://stellar.expert/explorer/testnet/tx/3304f4584eb77fdbbf8f66ceefed866dd0ef88b388f8a1ac0afec814e5e9908b) | 5387440 | 20:39:47 |
| 3 · kleroterion deployed (clerk, SAC, minimum 7) | [`1b0c61a6…`](https://stellar.expert/explorer/testnet/tx/1b0c61a6badd7499502d38b6912fe392dfcfc67360ca7df49be0174ccd39fc05) | 5387442 | 20:39:57 |
| 4 · SAC admin → kleroterion | [`7f9279a7…`](https://stellar.expert/explorer/testnet/tx/7f9279a79cf733a03580de85af001b5490595543e2ccded2d53a8351fa483cae) | 5387443 | 20:40:02 |
| 5 · **issuer master weight 0** | [`77808909…`](https://stellar.expert/explorer/testnet/tx/77808909a7a4a14bbd097a8622c0f354c83505a83eaee5b3c6f5b528e1056f63) | 5387444 | 20:40:07 |
| 6 · enroll `GD24…` (ana) | [`2d0320a7…`](https://stellar.expert/explorer/testnet/tx/2d0320a736f313e0ef62475e39ef53d3390c108548f62b91db61a54ec45e2109) | 5387446 | 20:40:17 |
| 6 · enroll `GDK4…` | [`ed4d2849…`](https://stellar.expert/explorer/testnet/tx/ed4d2849dbe38f2408ab93a083edd7c0859995766ea5f5792fbfed99c9535925) | 5387447 | 20:40:22 |
| 6 · enroll `GAO2…` | [`af59bbc1…`](https://stellar.expert/explorer/testnet/tx/af59bbc17cfe9fdcdcda744a5a8706527de11260f980cb83aa1f631a714d5503) | 5387448 | 20:40:27 |
| 6 · enroll `GDTB…` | [`8e9febd1…`](https://stellar.expert/explorer/testnet/tx/8e9febd11b7577e0b28a4b8364f872b061756f558d1fa1897fc66e6075f9d333) | 5387449 | 20:40:32 |
| 6 · enroll `GB4V…` | [`412def96…`](https://stellar.expert/explorer/testnet/tx/412def96e0e53edf831a66a2f38e299073908a9a2f89f3bd2c32a2babecd3ec2) | 5387450 | 20:40:37 |
| 6 · enroll `GBRV…` | [`4898c20e…`](https://stellar.expert/explorer/testnet/tx/4898c20ef3b13e7c1b2893108f3dc8d840bdd451e4f322878a1994bdda3dc256) | 5387451 | 20:40:42 |
| 6 · enroll `GAQD…` | [`de9ff13f…`](https://stellar.expert/explorer/testnet/tx/de9ff13f9285a8f1b980d3baa2ecdee082ccbcfeee3705921ff427e3f60d8aa6) | 5387453 | 20:40:52 |
| 7 · **lock** → `{ledger: 5387455, tempo: 1792960862, rodada_minima: 33385833}` | [`8a98bcf6…`](https://stellar.expert/explorer/testnet/tx/8a98bcf680b55a272e59df393c162a48c83731ee2f160abb039b763dfc7cef9c) | 5387455 | 20:41:02 |
| 8 · impede `GD24…` (the first of the enrolment order) | [`c1a2f592…`](https://stellar.expert/explorer/testnet/tx/c1a2f5925d638cb5ee0a01b485a5065eb4c32c8144b736f95e6195b440db9896) | 5387457 | 20:41:12 |
| 9 · **draw_round** round 33385836, k = 3 | [`fc4a5f93…`](https://stellar.expert/explorer/testnet/tx/fc4a5f93fa7c01779c81ffc924b29e766496ddbc7ca9556ff0a81babec2b85c2) | — | — |

### The panel

On-chain, from `draw_round`:

```
GDK4BAODKA75J4TXRZXJTBH33YCVCGC72VI2YFDG7PLMMLORU4IXIMG3
GAO2Y4Z4O4OC2O6BDDLCPLTQNM2KF4SJ4C44PIBY3GBDI6HOILPHV7YL
GAQDEW5UWVQG24CBCF5M2CNB2VBLB4IS3Z376FIRFUGNJJDOCEO7GMHU
```

Off-chain, with the roster `roster()` returned, the impeded `GD24…` and the
round's published randomness
`652d0c8fef318081c86bf1cff2522773a93c363d36156865cd78c790330d725b`:

```bash
python3 scripts/verificar_painel.py \
  652d0c8fef318081c86bf1cff2522773a93c363d36156865cd78c790330d725b 3 rol.json \
  --fora GD24NV3SBYMZWEZKUEKPKGSFOO756NEERITZNKMDJKDOC25IMODC6SSX
```

```
0  GDK4BAODKA75J4TXRZXJTBH33YCVCGC72VI2YFDG7PLMMLORU4IXIMG3
1  GAO2Y4Z4O4OC2O6BDDLCPLTQNM2KF4SJ4C44PIBY3GBDI6HOILPHV7YL
2  GAQDEW5UWVQG24CBCF5M2CNB2VBLB4IS3Z376FIRFUGNJJDOCEO7GMHU
```

### The roster, as any Horizon client sees it

`GET https://horizon-testnet.stellar.org/accounts?asset=PINAKION:GC3JC24U74YAIY26MEQMV6LZLVBDMR6CVF2AOAN6UA5XEGZ6EZOADRBB`

```
GAO2Y4Z4…  1.0000000  authorized
GAQDEW5U…  1.0000000  authorized
GB4VIN5V…  1.0000000  authorized
GBRVLTI7…  1.0000000  authorized
GD24NV3S…  1.0000000  IMPEDED      (is_authorized: false)
GDK4BAOD…  1.0000000  authorized
GDTB46PU…  1.0000000  authorized
```

Horizon returned them in ascending account id. That is an observation from
one page, not a documented guarantee — which is why the order the draw
shuffles is the contract's enrolment list, not Horizon's.

### Limits of this evidence

- One run, seven jurors, testnet. The roster minimum the design argues for
  is about sixteen (`docs/limite-inferior-do-rol.md`); seven is a demo size.
- The jurors are identities on one machine. Nothing here shows a juror
  refusing to sign.
- The clerk impeded one juror at will. Nothing stopped it, and nothing
  would have stopped a second or a sixth: `impede` is discretionary, and
  only the ledger record of each strike restrains it (Q-21).
- `AUTH_IMMUTABLE` was not set, so the issuer's flags could in principle be
  changed — by an account that can no longer sign. Setting it would close
  even that door, and it is a one-line addition to step 1.
- The costs of `enroll`, `lock` and `draw_round` were not measured; nothing
  in the documents states them.
- The draw was first run as a simulation (the CLI's default for a read-only
  call) and re-sent as a transaction; the two results were identical.
