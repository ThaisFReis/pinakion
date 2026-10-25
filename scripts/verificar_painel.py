#!/usr/bin/env python3
"""Recompute a kleroterion panel from the roster, with no dependencies.

    python3 scripts/verificar_painel.py <seed-hex> <k> rol.json [--fora ADDR ...]

`rol.json` is the ordered roster as `roster()` returns it: a JSON list of
addresses, enrolment order. `--fora` names anyone who does not sit — impeded
(SAC `authorized() == false`) or without a ticket (`balance() == 0`) at the
time of the draw. The seed is `randomness(sig)`, the sha256 of the compressed
drand signature, as the contract and the page compute it.

The order is the same Fisher-Yates the contract runs; the panel is the first
`k` of that order who are not `--fora`. Exit 0 and print the panel, one
address per line, in seat order.
"""

import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from verificar_sorteio import painel  # noqa: E402


def main(argv):
    if len(argv) < 4:
        print(__doc__)
        return 2
    seed, k, rol_path = argv[1], int(argv[2]), argv[3]
    fora = [a for i, a in enumerate(argv[4:]) if argv[4:][i - 1] == "--fora"] if "--fora" in argv else []
    rol = json.loads(pathlib.Path(rol_path).read_text())
    for assento, endereco in enumerate(painel(seed, rol, fora, k)):
        print(f"{assento}  {endereco}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
