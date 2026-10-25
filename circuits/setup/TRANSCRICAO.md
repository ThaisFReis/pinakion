# Phase-2 transcript — `pinakion_claim`

Produced by `circuits/setup.sh` on 2026-10-25T23:25:26Z (UTC). **Single-party**: one contribution, then a public beacon. Test-only in the sense that matters: whoever ran the contribution could have kept the entropy and could forge proofs. Nobody should treat a proof under this key as more than *whoever ran the setup did not cheat*.

| Step | What | Value |
|---|---|---|
| circuit | `circuits/pinakion_claim.circom`, circom 2.2.2 | r1cs sha256 `6fd65763245c664270ada0ca8a41fb709ec2b019cd1d2906a86a1f0bf5cd118f` |
| phase 1 | Hermez `powersOfTau28_hez_final_12.ptau`, from https://circom.info/powersOfTau28_hez_final_12.ptau | blake2b `ded2694169b7b08e898f736d5de95af87c3f1a64594013351b1a796dbee393bd825f88f9468c84505ddd11eb0b1465ac9b43b9064aa8ec97f2b73e04758b8a4a` (as published by snarkjs) |
| phase 2, contribution 1 | *Pinakion phase 2, single contributor*; entropy from `/dev/urandom`, not stored | contribution hash `eaa1114d 5912b716 31768041 23bd28ac 547f770e f9b098d8 49e9c6a8 a842d373 f349a07e 919acd52 16374f7b aa8e73b6 aecd8adb 66b18a35 d2f8cdd0 3e718cd4` |
| phase 2, beacon | drand quicknet round **33389132** (published 1792970760), randomness `0b0106cead205d827c2d078b82acde01bfc4350228d458c2de1f54203c5883f4`, 2^10 iterations. The round was computed from the clock **after** the contribution hash was printed, on the contributor's machine; the hash was not published anywhere before the round, so a third party cannot check that ordering — only that the round is real and that the beacon was applied | — |
| final zkey | `circuits/setup/claim_final.zkey` | sha256 `ed53fd1ba8f5397fa5376b777bd1fa7fffa7332b7d9f9ebc74947971d619f879` |
| vk | `circuits/setup/vk.json` | sha256 `957697d6af4085ba931b3e55b02952a4eb16d84080969343c4d5e81d2076c6b0` |
| witness generator | `circuits/setup/pinakion_claim.wasm` | sha256 `1e3a5545699ec2979841f169979bf46191fb190650ea9501a5a10820bd5e0b20` |

To check it yourself, with the ptau from the link above:

```bash
npx snarkjs zkey verify circuits/setup/pinakion_claim.r1cs powersOfTau28_hez_final_12.ptau circuits/setup/claim_final.zkey
```

`snarkjs zkey verify` output on the day:

```
Reading r1cs
Reading tauG1
Reading tauG2
Reading alphatauG1
Reading betatauG1
Circuit hash: 
		977f7f38 84b58399 11ccc09a bea4cf39
		5a733d58 526e9cc4 531431f7 935438e0
		e723d9fb 1275cca2 d023af46 3b35c02a
		c107253a 9fd67c44 d90b9b90 db7bf0af
Circuit Hash: 
		977f7f38 84b58399 11ccc09a bea4cf39
		5a733d58 526e9cc4 531431f7 935438e0
		e723d9fb 1275cca2 d023af46 3b35c02a
		c107253a 9fd67c44 d90b9b90 db7bf0af
-------------------------
contribution #2 drand quicknet round 33389132:
		2b23c0d7 86fa4add fe82cfa5 d380b7e7
		185ea74d 60baa65c 26e0c3ad f87e168b
		317c4ec6 695619bf 3c81a5eb 38aeb586
		4104c4ce 91185ed4 745db658 486aa231
Beacon generator: 0b0106cead205d827c2d078b82acde01bfc4350228d458c2de1f54203c5883f4
Beacon iterations Exp: 10
-------------------------
contribution #1 Pinakion phase 2, single contributor:
		eaa1114d 5912b716 31768041 23bd28ac
		547f770e f9b098d8 49e9c6a8 a842d373
		f349a07e 919acd52 16374f7b aa8e73b6
		aecd8adb 66b18a35 d2f8cdd0 3e718cd4
-------------------------
ZKey Ok!
```

Check the beacon round at `https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971/public/33389132`.
