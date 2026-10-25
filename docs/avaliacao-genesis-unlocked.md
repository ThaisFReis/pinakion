# Assessment: Pinakion against Genesis + Stellar Unlocked

Written on **2026-10-25**, after T-029 to T-031, from two Stellar Raven passes
(`docs/SOURCES.md`, same date) and the rubric on `hackmeridian.com/build`.
The scores are **estimates** by the agent that built the thing, not a judge's
figures; the evidence column is what a judge could check.

## The track's own test

*Stellar Unlocked*: "features of Stellar's core protocol … ready for their
breakout project … the breakthrough is in how you use it." The test both
readings share — the written bullets and the judges' spoken framing of
"new primitives" — is **has anyone done this with this primitive yet**.

| What Pinakion uses | CAP · protocol | Mainnet | Used this way before? |
|---|---|---|---|
| `AUTH_REQUIRED` + `AUTH_REVOCABLE`, locked issuer, as a jury roster | classic | 2015 | none visible (two Raven passes, 16 vocabularies) |
| SAC `trust` from inside a contract | CAP-73 · P26 | May 2026 | adjacent only (Authline, an onboarding SDK) |
| SAC `authorized()` / `set_authorized()` as eligibility | classic via SAC | 2024 | none visible |
| BLS12-381 pairing to verify drand | CAP-59 · P22 | Dec 2024 | **yes** — `AshFrancis/splicers`, `NibrasD/Stellar-VRF` |
| host PRNG reseeded from the beacon | P20 | Feb 2024 | splicers uses `prng().gen_range` after drand; the Fisher-Yates over a roster is ours |

Two of the four idea starters answered at once: *Create trustlines through
SACs* and *Old-school assets, new tricks*.

## The rubric, criterion by criterion

| Criterion | Weight | Estimate | Evidence a judge can check | What holds it down |
|---|---|---|---|---|
| Track Alignment | 20% | **4** | the table above; `docs/evidencia-testnet-bilhete.md` | the primitives are the old ones on the track's list; the beacon half has prior art |
| User Value & Problem | 20% | 4 | Node.js 2017 and Rust 2021 in `THESIS.md`, both with sources | no user has run a case through it |
| Technical Execution | 15% | 4 | 38 tests written first; three implementations held to `tests/vetores-de-ouro.json`; seven gates; 17 testnet transactions | costs of `enroll`/`lock`/`draw_round` unmeasured; `AUTH_IMMUTABLE` unset |
| Innovation | 15% | 3–4 | sortition from a roster frozen as a classic asset — unclaimed | drand verification is not new, and the documents say so |
| Hackathon Progress | 10% | 5 | `git log`: the repository is three days old; the ticket is one day old | — |
| Market & Ecosystem Potential | 10% | 3 | nameable callers: Tansu, Soroban Governor, Nouns, Governance Modules Library, Stellot, Reflector DAO, the SCF contracts | none approached |
| Product Design & UX | 10% | 3 | the published page, civic and responsive | the ticket path exists only on the command line |

Weighted, that lands around **3.8 of 5**. The number is an estimate and the
weights are the organiser's; a judge may score any row differently.

## The objection a judge will raise

**The clerk can `impede` anyone, after the lock, at will.** The substitute is
the next in the drawn order, so a clerk who strikes selectively shapes the
panel. Today this is discretionary. The grounds-and-contestation rules exist
in `docs/desenho-rol-sorteio-recusas.md` and are **not built** (Q-21).

The honest answer, in one breath: *the issuer cannot choose, the entropy
cannot choose, the contract cannot choose; the clerk can still exclude, and
every exclusion is a transaction with a name and a time.* That does not
remove the power. It makes it auditable — which is more than the Aquarius
incident had (`docs/SOURCES.md`, proposals #85 and #121), where an issuer
revoked a holder's access and the community's answer was to block the flags.

## How the pitch should say "flags"

The ecosystem reads `AUTH_REQUIRED` / `AUTH_REVOCABLE` as issuer power over
holders. Say the inversion first: **the issuer locks itself.** Then where the
power went: into a contract with no upgrade path, and into a clerk whose
every act is on the ledger. Never say "secure", "trustless" or "first" (§2).

## Limitations of this assessment

- Self-assessment, written by the builder. The scores are not calibrated
  against any past HackMeridian result.
- Both Raven passes are bounded by what Scout and LumenLoop index; absence is
  a whitespace signal, not proof.
- The judges' framing of the track comes from AI summaries of two broadcasts,
  not transcripts; the official notes for 22 Oct are not published.
- The portal still records this participant on Scale, approved (Q-17). If the
  switch to Genesis is not allowed, none of the above applies.
