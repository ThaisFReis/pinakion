#!/usr/bin/env python3
"""Recompute kleroterion's commitment root from its events, with no dependencies.

    python3 scripts/arvore.py eventos.json

`eventos.json` is the list of the contract's events in ledger order, each one
{"evento": "enroll", "indice": i, "c": "<decimal>"} or
{"evento": "impede", "indice": i}. Leaf i is c_i; an impediment zeroes it;
empty leaves are 0; depth 6. Prints the root, which must equal
`closing().raiz` (T-035).
"""

import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from poseidon import hash2  # noqa: E402

PROFUNDIDADE = 6


def zeros():
    z = [0]
    for _ in range(PROFUNDIDADE):
        z.append(hash2(z[-1], z[-1]))
    return z


def raiz(eventos):
    folhas = {}
    for e in eventos:
        if e["evento"] == "enroll":
            folhas[e["indice"]] = int(e["c"])
        elif e["evento"] == "impede":
            folhas[e["indice"]] = 0
        else:
            raise ValueError(f"evento desconhecido: {e['evento']}")
    z = zeros()
    nivel = folhas
    for d in range(PROFUNDIDADE):
        acima = {}
        for i in {i >> 1 for i in nivel}:
            esq = nivel.get(2 * i, z[d])
            dir_ = nivel.get(2 * i + 1, z[d])
            acima[i] = hash2(esq, dir_)
        nivel = acima
    return nivel.get(0, z[PROFUNDIDADE])


def main(argv):
    if len(argv) != 2:
        print(__doc__)
        return 2
    print(raiz(json.loads(pathlib.Path(argv[1]).read_text())))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
