# SOURCES

Every external fact this project relies on, and every piece of code copied in. Append-only.

## Facts

| Date | Fact | Value | Version | Source |
|---|---|---|---|---|
| 2026-10-25 | drand quicknet scheme | `bls-unchained-g1-rfc9380` — signature in G1, public key in G2 | — | `https://api.drand.sh/<chain>/info` |
| 2026-10-25 | drand quicknet chain hash | `52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971` | — | same |
| 2026-10-25 | drand quicknet period | 3 s, genesis 1692803367 | — | same |
| 2026-10-25 | drand quicknet public key | `83cf0f28…ece45a` (96 B, compressed G2) | — | same |
| 2026-10-25 | drand signed message, unchained | `sha256(round as u64 big-endian)` | — | **determined empirically**: six candidate combinations tested against round 1000000; exactly one verifies |
| 2026-10-25 | drand hash-to-curve DST | `BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_` | — | same empirical test |
| 2026-10-25 | drand round randomness | `sha256(signature)` | — | verified against the value the API returns |
| 2026-10-25 | Soroban PRNG seed is under validator control, not hard to bias, must not be used where unpredictability matters | — | soroban-sdk | `rs-soroban-sdk/soroban-sdk/src/prng.rs` module docs |
| 2026-10-25 | `Prng::shuffle` is Fisher-Yates via `SliceRandom::shuffle` | — | rand 0.8.5 | `rs-soroban-env/soroban-env-host/src/host/prng.rs` |
| 2026-10-25 | `unbias_prng_seed` is HMAC-SHA256 with a fixed salt, declared to be part of the protocol and identical across implementations | — | soroban-env-host | `soroban-env-host/src/crypto/mod.rs` |
| 2026-10-25 | That salt | `7ac33997…45a979` = `sha256("Public Global Stellar Network ; September 2015")` | — | same; derivation verified locally |
| 2026-10-25 | PRNG crates pinned by the host | `rand` 0.8.5, `rand_chacha` 0.3.1 | — | `soroban-env-host/Cargo.toml` |
| 2026-10-25 | BLS12-381 point encodings on Soroban | G1 uncompressed 96 B `be(X)‖be(Y)`; G2 uncompressed 192 B `be(X_c1)‖be(X_c0)‖be(Y_c1)‖be(Y_c0)` | soroban-sdk 29 | `rs-soroban-sdk/soroban-sdk/src/crypto/bls12_381.rs` |
| 2026-10-25 | Cost of one beacon verification, key pinned | 26,719,483 CPU instructions; 161,818 memory bytes | — | **measured** in the Soroban test host, `cargo test -p beacon-verifier`. The earlier 26,718,596 was measured before T-007, with the key passed as an argument |
| 2026-10-25 | Cost of one draw | 128,952 (roster 50) · 347,784 (200) · 2,274,884 (1000) | — | **measured**, same |
| 2026-10-25 | CAP-85 host functions | `create_executable_tag`, `create_external_ref_contract`, `update_current_contract_executable_ref`; mainnet since Protocol 28 | — | `stellar-protocol/core/cap-0085.md` |
| 2026-10-25 | CAP-85 leaves access control to the owner contract; `del_contract_data` on a tag key panics | — | — | same |
| 2026-10-25 | CAP-71 forwards the same payload and context to the delegate; delegate arrays must be sorted with no duplicates; cycles are not addressed | — | Protocol 27 | `stellar-protocol/core/cap-0071-01.md` |
| 2026-10-25 | CAP-72, weighted delegated signers for G-accounts | status **Draft**, protocol version TBD | — | `stellar-protocol/core/cap-0072.md` |
| 2026-10-25 | Jury draw: 25 drawn (art. 433); session installs with ≥15 present (art. 463); jurors excluded for impediment still count (art. 463 §2); fewer than 15 ⇒ draw suplentes (art. 464) | — | CPP | legjur, Flavio Meirelles Medeiros, Juspodivm — **secondary sources, not Planalto** |
| 2026-10-25 | 3 peremptory challenges per side; the refused juror leaves only that session and the draw continues with the remaining (art. 468); fewer than 7 left ⇒ trials separated (art. 469 §1–2) | — | CPP | Flavio Meirelles Medeiros; STJ REsp 1.540.151/MT for the per-defendant reading |
| 2026-10-25 | General list published by 10 Oct, definitive 10 Nov, anyone may complain (art. 426); whoever served in the last 12 months is excluded (art. 426 §4) | — | CPP | Aurum, Projuris, Dizer o Direito — **secondary sources** |
| 2026-10-25 | Jury service is obligatory; unjustified refusal or absence draws a fine of 1–10 minimum wages (arts. 436 §2, 442); conscientious objection leads to alternative service (art. 438) | — | CPP | Aurum, Cognijus — **secondary sources** |
| 2026-10-25 | Jury no-show rates | Canada ≈ 1/5 · Philadelphia 36–42% · New Zealand ≈ 1/5 · Ireland 25,649 absences vs 20 fines in three years | — | CBC, Philadelphia Magazine, Otago Daily Times, Irish Daily Mail via PressReader |
| 2026-10-25 | Kleros draws jurors randomly, weighted by PNK stake; runs on Arbitrum One | — | — | `docs.kleros.io` |
| 2026-10-25 | Aragon Court drafted from an active pool weighted by stake; one 2023 paper reports under 50 disputes resolved | — | — | Aragon blog; Springer paper |
| 2026-10-25 | drand signature must be decompressed off-chain for `verify` | 48 B compressed → 96 B `be(X)‖be(Y)` | py_ecc | `py_ecc.bls.point_compression.decompress_G1`; **verified** against round 33382274, whose decompressed signature verifies on testnet |
| 2026-10-25 | Live demo path, end to end | round 33383146 verified on-chain (`true`), panel drawn and reproduced off-chain | — | **measured**, tx `bad41b10a5898ad6ace6a45228e42d4033309146f0ff5a6a2f3a4513ee4b48f4` |
| 2026-10-25 | HackMeridian method: THESIS → SPEC → DESIGN, AGENTS.md for the how, public repo day 1, 3-minute demo in four parts | — | — | *Hackathon Speedrun SHIP*, Wlad Mendes, Stellar, 25 Oct 2026 |

## Checked and not found

| Date | What we looked for | Where we looked | Result |
|---|---|---|---|
| 2026-10-25 | Prior art for verifiable sortition of a committed roster on Soroban | Stellar Light repo search (591 rows inspected to depth 8), project directory | none visible as of this date. `Stellar-VRF` and `NebulaVRF/vrf-contract` are adjacent and **have not been read** |
| 2026-10-25 | Prior art for CAP-85 used for governance | GitHub code search, Stellar Light directory, LumenLoop corpus | none visible as of this date |
| 2026-10-25 | Prior art for due process over clawback | Stellar Light, LumenLoop, keyword search | **found, late**: `Prashant-Mishra-12569/habeas`, MIT, 3rd place in a hackathon that closed the same week. The earlier "not found" was wrong; the search vocabulary did not overlap with the author's |
| 2026-10-25 | Prior art for CAP-71 delegation | GitHub code search for `delegate_account_auth`, `get_delegated_signers` | **found**: `stellar-registry/perch`, `accensa/accensa-contracts`, `debuuuuuu/PAY3`, `ShippedLabs/soroban-upgrade-safeguard`, `pedro-pelicioni/tusst` |

## Copied code

| Date | Our path | Origin (URL + commit) | License | Task |
|---|---|---|---|---|
| 2026-10-25 | `CLAUDE.md` | provided by the human as a file to adapt; origin not stated | **unknown — see §11 Q-01** | T-000 |

No third-party source code has been copied into this repository. `contracts/`, `scripts/` and `web/` were written here.
