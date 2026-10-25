# DEMO

One path, run live on testnet, with a recording as backup. Everything below was executed on **2026-10-25**; the transaction hashes are real.

The deck's rule: *judges score demos, not ideas.* So the demo shows the guarantee working, not the architecture.

---

## What the demo has to prove

**Nobody chooses who judges you** — and you do not have to take our word for it.

Three claims, in this order, because each one is useless without the one before it:

1. The entropy came from outside, and the chain checked it.
2. The panel came from that entropy, and from a roster fixed beforehand.
3. Anyone can recompute the panel and get the same answer.

---

## Timing, measured

The command path took **11 seconds end to end** on 2026-10-25 — 2 s for the round, 7 s for the on-chain `verify`, 1 s each for `beacon` and `draw`, under 1 s for the recomputation. The 90-second demo window is not the constraint; narrating it is.

---

## Before recording

```bash
export PINAKION=CBPHNZLGQIUEC4BEC53ZP7P65LLIBT5JTWTHMZVAMCGZCEN3PSUHTPQN
export ID=<your funded testnet identity>
pip install py_ecc          # only for scripts/rodada.py
```

Two terminals and a browser window. Terminal font large enough to read in a 1080p recording. The browser on `web/index.html`, served over HTTP — it is an ES module, so `file://` will not load it:

```bash
python3 -m http.server 8731 --directory web
```

Check the clock: a drand round is only *live* if it is minutes old, and that is the whole point of step 1.

---

## The path

### 1 · Take a round nobody could have known

```bash
python3 scripts/rodada.py
```

Prints the latest round, its randomness, and its signature **uncompressed** — `BytesN<96>`, `be(X)‖be(Y)`, which is what the contract takes. drand publishes 48 compressed bytes; the decompression happens off-chain, and it is safe to do there because a wrong point simply fails the pairing.

Say out loud: *this round did not exist two minutes ago.*

### 2 · Let the chain check it

```bash
stellar contract invoke --id $PINAKION --source $ID --network testnet --send=yes \
  -- verify --round <round> --sig <signature>
```

→ **`true`**, and a transaction hash.

Run on 2026-10-25 with round `33383146`: [`bad41b10…`](https://stellar.expert/explorer/testnet/tx/bad41b10a5898ad6ace6a45228e42d4033309146f0ff5a6a2f3a4513ee4b48f4)

Then the sentence that matters:

```bash
stellar contract invoke --id $PINAKION --source $ID --network testnet -- beacon
```

→ the quicknet chain hash. **The key is not a parameter.** It is a constant in the contract, and `beacon()` publishes which one, so nobody — including us — can point this contract at a beacon they control. That was [issue #1](https://github.com/ThaisFReis/pinakion/issues/1), and a signature forged under a different keypair is rejected on-chain: [`30de94fe…`](https://stellar.expert/explorer/testnet/tx/30de94feaf1ac2e6dea42a03edc8fe9be158b10716f03b4be707eb4328559375).

### 3 · Draw the panel

```bash
stellar contract invoke --id $PINAKION --source $ID --network testnet \
  -- draw --seed <randomness> --n 200 --k 25
```

→ 25 positions, in the order they were drawn. A read-only call, so it needs no transaction.

### 4 · Recompute it without us

```bash
python3 scripts/verificar_sorteio.py <randomness> 200 25
```

Zero dependencies — ChaCha20, Lemire rejection sampling and Fisher-Yates written by hand. Same panel, digit for digit.

Run on 2026-10-25, round `33383146`, both sides:

```
[196, 83, 84, 139, 118, 22, 175, 145, 171, 70, 177, 109, 169, 68, 164, 181,
 3, 142, 74, 88, 66, 178, 60, 92, 160]
```

### 5 · And again in a browser, so it is not a programmer's trick

Serve the page and open it:

```bash
python3 -m http.server 8731 --directory web
```

In this order — it matters:

1. open **Assinatura e dados de conferência**
2. click **Buscar última rodada**. The page fetches the round, computes the seed itself, and says whether its seed matches the one drand published. It also clears the comparison field, because the panel there belonged to the previous round.
3. paste the panel step 3 printed into **Painel informado para comparação**
4. → **Os painéis coincidem**, and the numbers take the accent colour

Then change one digit and it says they diverge. That is the whole guarantee, done by someone who installed nothing.

Rehearsed on 2026-10-25 with round 33383606: the page recomputed the same panel the contract drew.

---

## The three-minute script

The deck's four parts. Timings are the target, not a stopwatch.

### 0:00 — The problem *(30 s)*

> In 2017 a Node.js committee voted on removing one of its own members. The accused abstained; six of the ten remaining votes were against removal, and four of the people who had pushed for it resigned. In 2021 the entire Rust moderation team resigned, saying the Core Team answered to nobody but itself. Their own proposed fix was a rule about appointment: the next moderators should be chosen by people who are not on the Core Team.
>
> Both reached for the same missing mechanism. Neither had one.

No slides of logos. One sentence of each case, then the guarantee: **nobody chooses who judges you.**

### 0:30 — The demo *(90 s)*

Steps 1 to 4 above, live. Do not narrate the commands; narrate what they mean.

- the round nobody could predict → `verify` → **`true`**
- `beacon()` → *the key is not a parameter*
- `draw` → the panel
- `verificar_sorteio.py` → **the same panel**
- the browser → *and anyone can do this*

If a command fails, cut to the recording. Do not debug on camera.

### 2:00 — Why Stellar *(30 s)*

Three facts, each measured or read from source:

- one beacon verification costs **26,719,483 CPU instructions** — measured in the Soroban test host, and executed on testnet
- the host's own source says its PRNG seed-derivation salt is part of the protocol and must be identical in every implementation. **That declaration is why a browser can recompute a panel at all.**
- the SDK documents that its PRNG seed is under validator control and is not hard to bias. So the entropy comes from outside, and the chain's job is to check it. Building on a chain whose documentation is honest about its own limit is easier than building on one that is not.

### 2:30 — What is next *(30 s)*

Say the limits out loud, because a judge who finds them later stops believing the rest:

- the case lifecycle, the roster cycle and the recusal rules are **designed and not built** — three documents, 587 lines
- nothing audited, nothing on mainnet
- below a roster of about sixteen the draw guarantees nothing, and the contract should refuse rather than pretend

Then what the draw already solves, which is the part people assume is impossible: a juror who refuses is replaced by **the next already-drawn juror**, so refusing advances the queue and never selects. That is Brazilian jury procedure, art. 468, and it is why the recusal problem is not a hole.

Close on the guarantee, the repository link, and nothing else.

---

## What this demo does not show

Part of the deliverable, not an appendix:

- The page recomputes and compares. It does **not** read the chain and does **not** verify the beacon signature in the browser.
- `verify` proves the round is authentic. It does **not** prove the roster was committed before the round — that is [issue #6](https://github.com/ThaisFReis/pinakion/issues/6), INV-06, unbuilt.
- There is no roster contract. `draw` takes a size, not a list of people.
- The costs are measured in the test host, not extracted from these transactions.

---

## Recording the backup

Record the whole path once, unedited, before recording the final take. If the live run fails on the day, that file is the demo and the failure costs nothing.
