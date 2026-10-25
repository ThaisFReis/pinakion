# Evidence: the ticket as a classic asset, on testnet

Every hash below is real and was produced on **2026-10-25** by one run of
[`scripts/sequencia_bilhete.sh`](../scripts/sequencia_bilhete.sh) — the
draw was re-sent once with `--send=yes`, because the CLI only simulates a
read-only call by default. Links open on stellar.expert; the same hashes
resolve on any Horizon or RPC for testnet. Nothing here has run on mainnet.

| | |
|---|---|
| issuer | `GC3JC24U74YAIY26MEQMV6LZLVBDMR6CVF2AOAN6UA5XEGZ6EZOADRBB` |
| asset | `PINAKION:GC3JC24U…` |
| SAC | `CD4OL6N5PODOHYPWAGDWQIGK7S7FOCEJUEMPZH2AQZEHQOWVDL5OFGZX` |
| kleroterion | `CBUQU5MH2KWBWDHQEZL53U7CYPMKHLI3USOJOHE544IIOG7PWFQRLSFL` |
| clerk (*escrivão*) | `GAEZBHTKFRX5JTCWZXKSX6BPT75WSILEV7OEPWU42IRR6ADMLETEKZQY` |

## What the sequence proves

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

## The transactions, in order

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

## The panel

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

## The roster, as any Horizon client sees it

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

## Limits of this evidence

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
