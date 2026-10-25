# AGENTS.md

`THESIS.md`, `docs/SPEC.md` and `DESIGN.md` say **what**. This file says **how**, in short.
`CLAUDE.md` is the long version: when the two disagree, `CLAUDE.md` wins.

## Mission

**Pinakion** — verifiable jury selection for governance on Stellar.
A panel drawn from a roster committed before the case, by entropy nobody controlled, that anyone can recompute.

Demo on testnet before the submission deadline. Nothing ships that has not run on testnet.

## Ground rules

Read freely. Ask before you push.
Never commit a key. Testnet only; no mainnet without being told, in this session.
**Every Stellar fact needs a source**, logged in `docs/SOURCES.md` with its date.
Read the source, not the summary. A README is a pointer; the code is the evidence.
Say where every number came from: measured, estimated or cited.
Never write that something does not exist without running the positive test: *if it existed, where would it show up?*

## Scope

`docs/SPEC.md` §9 is the scope. Anything else you notice becomes a task or a §11 question — never a drive-by fix.
One task, one branch, one PR. Commit messages start with the task ID.
The guarantee in `THESIS.md` governs: if a design decision does not follow from *nobody chooses who judges you*, the decision is wrong.

## Tools

Raven MCP and the Stellar Skills, always.
`soroban-sdk` 29.0.1 · Stellar CLI 28.x · target `wasm32v1-none`.
`stellar contract build` for Wasm — plain `cargo build` to Wasm fails by design.
Entropy comes from drand quicknet, never from `env.prng()` alone.

## Working style

Blocked? One question plus your best guess. Don't stall on it.
Failed twice? Stop and report. Don't try a third variation.
Prior art is checked **before** a design is written down, not after.
When two sources disagree, name both. When two of our own documents disagree, that is a §11 entry, not something to settle quietly.

## Done when

- ✓ The demo path runs on testnet, with transaction hashes recorded
- ✓ The draw is reproduced by the off-chain verifiers, locked by golden vectors
- ✓ `README.md` opens with the one-liner and says how to run it
- ✓ Gates in `docs/SPEC.md` §8 pass; no new failure against the baseline
- ✓ No keys in git
- ✓ Every published claim is listed in §2 with its evidence, and every limitation is stated in the deliverable itself
