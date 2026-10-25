# Pinakion

**Verifiable jury selection for governance on Stellar.**

> In classical Athens the *pinakion* was a small bronze ticket, engraved with a citizen's name and his tribe. It was slotted into the *kleroterion* — a stone slab with rows of slots and a tube — before any draw began. Black and white balls fell through the tube and decided, row by row, who judged that day.
>
> The machine put chance in two places at once: in the order the tickets were placed, and in the order the balls came out. Neither could be arranged by whoever wanted a verdict.
>
> This project is that, on Stellar. The roster is committed before the case exists; the entropy arrives afterwards from a public beacon; and anyone can recompute the panel and check it.

---

## The guarantee

**Nobody chooses who judges you.**

Everything below is derived from that one sentence. Where a design decision does not follow from it, the decision is wrong.

---

## Who it is for

A governance that has to decide **about a person**, not about a parameter.

| Who | The decision they have to make |
|---|---|
| An open-source project or DAO with a code of conduct | whether a member violated it |
| A grant program | whether to revoke a grant already given |
| A tokenized fund or regulated issuer | whether to contain a holder's position, and who gets to say so |
| A protocol with slashing | whether a validator was malicious or merely unlucky |
| A marketplace | whether to delist a seller who disputes it |

What they share: the decision is about a specific party, it requires human judgement, the decider could plausibly be chosen to produce an outcome, and **the affected person never consented to the forum**.

Not for commercial arbitration, where choosing your arbitrator is a right rather than a defect.

---

## The problem

In every case above, the body that decides is appointed by someone with an interest in the outcome — or, worse, is the accused's own body. Two documented cases:

**Node.js, 2017.** A member of the Technical Steering Committee was accused of violating the code of conduct, including interfering with moderation. **The TSC voted on removing one of its own.** The accused abstained; of the ten remaining votes, six were against removal. Four members who had pushed for it resigned, one writing that the decision undermined the committee's ability to govern. The project forked.

**Rust, 2021.** The entire moderation team resigned, stating that the Core Team had placed itself answerable to no one but itself, and that this left them unable to enforce the code of conduct. Their own proposed fix was a rule about appointment: **the replacement moderators should be chosen by people who are not on the Core Team.**

Both reached for the same missing mechanism. Both had to settle for a social rule, because no mechanism existed. In both, the only exit available was resignation or a fork.

The guarantee that was missing is old and has a name: the deciding body is determined by rules fixed **before** the dispute, and its members are drawn **by lot** — so that nobody, including the people who run the institution, picks who hears your case.

---

## The ticket

The *pinakion* is literally a ticket here: **one unit of a classic Stellar asset.** The issuer sets `AUTH_REQUIRED` and `AUTH_REVOCABLE`, hands the asset's contract admin to the `kleroterion` contract, and then sets its own master weight to 0. From that ledger on, nobody can issue or authorize a ticket except the contract — and the contract refuses enrolment after `lock()`.

| In Athens | Here |
|---|---|
| the bronze ticket with your name | one PINAKION on your trustline, created by the contract itself (SAC `trust`, Protocol 26) |
| the ticket slotted before the draw | `enroll` before `lock`; the enrolment order is the order the balls fall on |
| the slab closed | the issuer locked, the roster frozen, the minimum beacon round fixed from the ledger clock |
| a citizen struck off | `set_authorized(false)` — the ticket stays, the seat does not (art. 463: the next in the drawn order takes it) |
| anyone counting the tickets | any Horizon: `GET /accounts?asset=PINAKION:issuer` |

None of this needed a new primitive. Asset authorization flags date from 2015 and were built for regulated issuers; a contract creating a trustline is Protocol 26. They were never used to seat a jury — which is the point.

---

## Why Stellar

Not "because blockchain". Three specific things, each measured or read from source:

**The beacon verification is affordable on-chain.** A drand round is a BLS12-381 signature. Stellar's host functions verify it for **26,719,483 CPU instructions** — measured in the Soroban test host, and executed on testnet in [transaction `c74150ee…`](https://stellar.expert/explorer/testnet/tx/c74150eeb7b73d0a196d20b44ed851a140812b3ac67e579b670b87ee2c3ffeea). The draw itself costs 2,274,884 for a roster of a thousand.

**The draw is exactly reproducible off-chain.** The Soroban host derives its PRNG seed with `HMAC-SHA256` under a fixed salt, and the host's own source states that this salt is part of the protocol and must be identical in every implementation. The shuffle is Fisher-Yates from pinned crate versions. That declaration is the reason a browser can recompute a panel at all — and we did: an implementation written from scratch, with no dependencies, reproduces the on-chain panel exactly.

**The roster can live in the classic ledger, frozen by the ledger's own rules.** A locked issuer cannot sign; an `AUTH_REQUIRED` asset cannot be held without the issuer's — now the contract's — say-so. Those are protocol rules, not contract code, and any Horizon client can read the result without trusting us. Proven on testnet in `docs/evidencia-testnet-bilhete.md`.

**The randomness the chain offers is not good enough, and says so.** The `soroban-sdk` documents that its PRNG seed is under validator control and is not hard to bias. So the entropy comes from outside, from drand, and the chain's job is to *verify* it rather than produce it. Building this on a chain whose own documentation is honest about that limit is easier than building it on one that is not.

---

## What it is, and is not

**It is** a module an existing governance calls to run a draw. It keeps a roster, fixes a beacon round, draws a panel, and emits a record anyone can recheck.

**It is not** a DAO, a court, or a dispute marketplace. It does not decide who may be on the roster, what the panel rules, or what follows from the ruling. Those belong to the governance that calls it.

**It is not Kleros.** Kleros answers *who will judge?* with a market of anonymous jurors selected by stake. Pinakion answers it with **the institution's own roster, fixed before the case**. For a commercial dispute a market works. For a conduct process inside a community, a paid anonymous stakeholder judging a harassment claim is what no ethics council in the world accepts — which is why real ones are composed of elected peers.

---

## What is proven today

- A live drand round verifies on Stellar testnet; a tampered one is rejected. Transaction hashes in `docs/evidencia-testnet-sorteio.md`. This part is not new on Stellar: `AshFrancis/splicers` and `NibrasD/Stellar-VRF` verified drand with the same host functions before us, and we say so.
- The roster is a classic asset: seven jurors enrolled with trustlines the contract created, the issuer locked at master weight 0, the roster closed with its minimum round fixed from the ledger clock, one juror impeded, a panel of three drawn from a live round and reproduced off-chain, address for address. Every hash in `docs/evidencia-testnet-bilhete.md`.
- The panel drawn on testnet is reproduced, digit for digit, by two independent implementations — one in Python, one in JavaScript — written from the host's semantics rather than its code.
- The agreement is locked as a regression test, so a change to the host's algorithm turns the suite red.

## What is not

- The case lifecycle is designed (`docs/desenho-rol-sorteio-recusas.md`) and **not built**. Of the roster cycle, only enrol → lock → impede exists; renewal, contestation windows and the quorum arithmetic do not.
- **The clerk can still exclude.** `impede` is discretionary after the lock; the substitute comes from the drawn order, but who is struck is the clerk's call. Each strike is a transaction with a name and a time — auditable, not prevented (`docs/avaliacao-genesis-unlocked.md`).
- Nothing has been audited. Nothing has run on mainnet.
- Below a roster of about sixteen the draw guarantees nothing, and the contract should refuse to operate rather than pretend (`docs/limite-inferior-do-rol.md`).

---

*This file is the root. When the thesis changes, it changes here first — then `SPEC.md`, then `DESIGN.md`, then the landing page and the pitch.*
