# DEMO

The demo is one link and one recording of it:

- **Link:** https://thaisfreis.github.io/pinakion/web/processo.html
- **Recording:** the same page, in **Apresentar** mode (`?apresentar=1`): one phase per screen, with arrow and space keys, or the auto-advance at 9 s per phase.

Everything on the page is a fictional code-of-conduct case with **real testnet transactions**, run by `scripts/sequencia_processo.sh` on 2026-10-26 and checked from the record in your browser. The vote is **Tessera's**, the author's earlier secret-ballot module, and the video says so before it shows the vote (DEC-036).

The deck's rule: *judges score demos, not ideas.* So the video shows the case working and being checked, not the architecture.

---

## Before recording

```bash
python3 -m http.server 8778
```

Open `http://localhost:8778/web/processo.html?apresentar=1`, full screen, 1080p, the browser zoom at 125 %. Or record the published link above; it is the same page.

Have a second tab ready on `docs/evidencia-processo.md` on GitHub, for one shot of the transaction table. Have a terminal ready, font large enough to read:

```bash
python3 scripts/verificar_processo.py docs/evidencia-processo-registro.json
```

---

## The script, 3 minutes

| Time | On screen | Say |
|---|---|---|
| 0:00 | the landing page hero | *When a community judges one of its own, someone with a stake usually picks the judges. Rust in 2021 and Node.js in 2017 both ended in resignations. Pinakion takes that choice away from everyone.* |
| 0:15 | **Apresentar**, phase 01 · O rol | *Forty people volunteer to judge. Each holds a ticket, a Stellar asset whose issuer is locked, and a commitment to a secret only they know.* |
| 0:30 | 02 · O fechamento | *The case opens at level 3, so seven seats. The table fixed that before any case; nobody chooses it now. The person judged was on the roster, and the lock itself removed them. They are suspended, but not silenced.* |
| 0:45 | 03 · A denúncia e os autos — open the **Defesa** | *The record is hashes on the ledger. Each role files only what it may. This page downloaded each document and matched its hash. Here is the defence, filed by the person judged.* |
| 1:00 | 04 · O sorteio selado | *The entropy is a drand round that did not exist when the roster closed. The contract verified its signature on-chain.* (One verification costs 26,719,483 instructions, measured in the test host.) |
| 1:10 | 05 · Os assentos | *Each juror checks privately whether they were drawn, and proves it in zero knowledge, from a fresh account. On the ledger, a seat is a number, never a name.* |
| 1:25 | 06 · O quórum | *Six of the seven attend, from their seat accounts. The contract only asks the roster whether that account sat. Five was the quorum, and the session opens.* |
| 1:40 | 07 · A votação — the bronze box | *The vote runs on Tessera, my earlier secret-ballot module, used as it is. What is new is the binding: the case contract accepted this vote only because Tessera's voter list is exactly the six who attended. It recomputed Tessera's Merkle root on-chain to check.* |
| 2:00 | 08 · O veredito | *Five to one. Level 3 needs a majority of the whole panel, four of seven; an absent juror never counts in favour. Upheld. The suspension ends here; the sanction is the project's.* |
| 2:15 | 09 · A revelação | *Only now do the jurors reveal who they were, on purpose, after the verdict. With the reveals, anyone recomputes the panel.* |
| 2:25 | the terminal: `verificar_processo.py` → `CONFERE` | *All of this was checked from the ledger alone, in the browser and here, trusting neither contract.* |
| 2:40 | the page's limits block | *What it is not yet: one machine played everyone, there is no relayer for seat accounts, the setup is single-party, the records are public, testnet only, unaudited.* |
| 2:55 | the landing page | *Nobody chooses who judges you. Nobody knows who, until the verdict.* |

---

## If something goes wrong while recording

- **The page says *Não confere*.** It names the failing check. Record nothing until `python3 scripts/verificar_processo.py docs/evidencia-processo-registro.json` says `CONFERE` too, then reload with the cache bypassed.
- **A transaction link opens empty.** stellar.expert can lag a few minutes behind testnet; the hash is still in `docs/evidencia-processo-hashes.tsv`.
- **Testnet was reset.** The record and the page still check out, because the check reads the committed record, not a live RPC. The explorer links will not open. Say so if it comes up.

---

## What the demo does not show

- **One machine played every person.** Every account came from friendbot.
- **There is no relayer**, so the seat accounts are not funded anonymously.
- **The setup is single-party.**
- **The records are public.**
- **The windows are minutes, not the approved 30 days.**
- **The tally's correctness is Tessera's verifier's**, shown in `docs/evidencia-processo-tessera.txt`, not this project's.
- **Testnet only; nothing is audited.**
