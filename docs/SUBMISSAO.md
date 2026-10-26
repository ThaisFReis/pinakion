# Submission text — HackMeridian 2026, Genesis · Stellar Unlocked

Draft for the submission form (T-042). It follows the spine *problem → user → what → why Stellar → what is not done*. Every number is one already gated in this repository (G7, G9) and every claim is a row in `docs/SPEC.md` §2. Changing a claim here starts in `THESIS.md`, as §0 says.

---

## One line

**Pinakion seats a jury that nobody chose, and nobody can name until after the verdict.** It is a Soroban module that draws a panel from a roster frozen before the case. Each juror proves their seat in zero knowledge from a fresh account, and anyone recomputes the panel from the ledger once the jurors reveal.

## The problem

When a community has to judge one of its own, the people who decide are appointed by someone with a stake in the outcome.

- **Node.js, 2017.** The Technical Steering Committee voted on removing one of its own members. Four members resigned and the project forked.
- **Rust, 2021.** The whole moderation team resigned. Their proposed fix was a rule about appointment: the replacement moderators should be chosen by people outside the Core Team.

Both reached for a mechanism that did not exist, and both had to settle for a social rule.

## Who it is for

**An open-source project with a code of conduct** that has to judge one of its own. It also serves a grant program revoking a grant, a protocol judging a slashed validator, or a marketplace delisting a seller. What they share: the decision is about a person, and that person never chose the forum.

## What it does

The guarantee has three sentences:

> **Nobody chooses who judges you.** The roster is a classic Stellar asset frozen by a locked issuer. `lock` fixes the drand round from the ledger clock, and `seal` verifies that round on-chain, once.
>
> **Nobody knows who until they have sat and judged.** Each juror committed `c = Poseidon(s)` at enrolment. A juror whose `y = Poseidon(seed, s)` falls under the threshold proves it with Groth16 on BN254, from a fresh seat account. The claim event carries `y` and nothing else.
>
> **Afterwards, anyone recomputes the panel from the ledger, among those who revealed.** Each juror reveals `s`, and a dependency-free script or the browser page recomputes everything from the contract's events and drand.

**On testnet, 2026-10-26.**
- 20 jurors were enrolled and 1 was impeded.
- A live drand round was verified on-chain.
- Six seats were proven from six fresh accounts.
- One juror recused openly.
- The panel came out complete, and 18 jurors revealed.

The independent check, reading only the events and drand, returns **`CONFERE`, 77 checks, 0 failures**. The panel's seat accounts export straight into Tessera, the author's earlier secret-ballot module, as its voter list.

**And then a whole case, on testnet, the same day.** A fictional code-of-conduct case, with real transactions.
- 40 jurors were enrolled. The case was opened at level 3, which seats 7.
- The person judged was on the roster, and the lock itself impeded them.
- The complaint, the evidence, the defence and the minutes were filed by their hashes, each by the role allowed to file it.
- Six of the seven seated jurors attended from their seat accounts, against a quorum of five.
- The vote was secret, and it ran on **Tessera**, the author's earlier secret-ballot module, used as it is (contract at `356c22f`; a one-line CLI fix). The case contract accepted the vote only because Tessera's voter list was exactly the attendees. It recomputes Tessera's root on-chain to check that.
- The vote was 5 to 1. The verdict, *upheld*, is Tessera's score under the level's rule: 4 of 7 required.
- Afterwards the jurors revealed.

`scripts/verificar_processo.py` checks the whole case from the record alone, and so does the page.

- Demo: https://thaisfreis.github.io/pinakion/web/processo.html
- The sealed draw, person by person: https://thaisfreis.github.io/pinakion/web/bilhetes.html
- Code: https://github.com/ThaisFReis/pinakion

## Why Stellar, and why now

- **Seats proven without being named.** Poseidon and BN254 became host functions in Protocol 25, January 2026 (CAP-74, CAP-75). A seat proof is one ordinary transaction: about 32 M instructions declared, 8 % of the 400 M per-transaction ceiling. A year ago this was not possible on Stellar, and it is what makes the second sentence true.
- **Public entropy, verified on-chain.** drand's BLS12-381 signature is verified with CAP-59 host functions, at 26,719,483 instructions on the test host.
- **A roster the ledger itself freezes.** The jurors hold a classic asset with `AUTH_REQUIRED` and `AUTH_IMMUTABLE`, and its issuer sits at master weight 0. The contract creates each trustline (Protocol 26), and any Horizon client lists the roster without trusting us.

## What is not done

- **The setup is single-party.** Whoever ran it could forge seats. The drand beacon at its end only shows that the last step was not chosen.
- **No relayer.** A seat account must be funded with no trail to the juror. In this run every account came from friendbot, and one machine played every juror.
- **A lost secret is a lost seat.** A juror who never reveals stays unaccounted for, and is listed as such on the ledger.
- **The seal ends at the ledger.** Deliberating off-chain breaks it.
- **The demo's windows are short.** The approved reveal floor is 30 days; the run used minutes.
- **The case is built and ran once; the rest of its lifecycle is designed.** Roster renewal, contestation windows, a redraw after an adjournment and confidential records with justified access are not built. The demo's records are public.
- **The vote is Tessera's.** Its privacy floor is five votes, so a case needs five seats or more. The tally's correctness is checked by Tessera's own verifier, not by this project.
- **Testnet only. Not audited.**

Verifying drand on Soroban is not new (`AshFrancis/splicers`, `NibrasD/Stellar-VRF`), and Pinakion does not claim it. What neither does is the sortition: a panel drawn from a roster committed before the case, sealed until the verdict. No Stellar build we could find does it either. That search was run on 2026-10-25 (`docs/SOURCES.md`), and the indexes lag.
