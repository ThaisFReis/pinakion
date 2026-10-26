#!/usr/bin/env python3
"""The cost table of a testnet run, from the transactions themselves (T-040).

    python3 scripts/custos_testnet.py docs/evidencia-testnet-hashes.tsv

The input lists `operation<TAB>tx-hash`, as scripts/sequencia_bilhete.sh
writes it. For each transaction it reads, with `stellar tx fetch`:
- from the envelope: the Soroban resources the transaction declared
  (instructions, disk read bytes, write bytes), which simulation sets and the
  fee pays for. The network does not report the instructions actually
  executed; these are the budget each transaction bought.
- from the fee record: the fee charged, in stroops.
It prints a Markdown table, one row per operation, with min–max over that
operation's transactions. Needs the Stellar CLI and network access.
"""

import json
import subprocess
import sys
from collections import OrderedDict


def buscar(sub, h):
    out = subprocess.run(["stellar", "tx", "fetch", sub, "--hash", h, "--network", "testnet", "--output", "json"],
                         capture_output=True, text=True, check=True).stdout
    return json.loads(out)


def recursos(envelope):
    try:
        r = envelope["tx"]["tx"]["ext"]["v1"]["resources"]
        return r["instructions"], r["disk_read_bytes"], r["write_bytes"]
    except (KeyError, TypeError):
        return None


def faixa(xs):
    xs = [x for x in xs if x is not None]
    if not xs:
        return "—"
    a, b = min(xs), max(xs)
    return f"{a:,}" if a == b else f"{a:,} – {b:,}"


def main(argv):
    if len(argv) != 2:
        print(__doc__)
        return 2
    por_op = OrderedDict()
    for linha in open(argv[1]):
        if not linha.strip():
            continue
        op, h = linha.split()
        r = recursos(buscar("envelope", h))
        fee = buscar("fee", h)["charged"]["fee"]
        por_op.setdefault(op, []).append((r, fee, h))
    print("| Operation | Txs | Instructions declared | Disk read bytes | Write bytes | Fee charged (stroops) | Example tx |")
    print("|---|---|---|---|---|---|---|")
    for op, linhas in por_op.items():
        rs = [r for r, _, _ in linhas]
        print(f"| `{op}` | {len(linhas)} | {faixa([r[0] if r else None for r in rs])} | "
              f"{faixa([r[1] if r else None for r in rs])} | {faixa([r[2] if r else None for r in rs])} | "
              f"{faixa([f for _, f, _ in linhas])} | [`{linhas[0][2][:8]}…`](https://stellar.expert/explorer/testnet/tx/{linhas[0][2]}) |")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
