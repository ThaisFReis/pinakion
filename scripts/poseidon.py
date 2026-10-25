#!/usr/bin/env python3
"""Poseidon over BN254, circomlib-compatible, with no dependencies.

The permutation is the Soroban host's (soroban-env-host 29.0.1,
`crypto/poseidon/poseidon.rs`): per round, add the round constants, apply the
S-box x^5 to every cell in a full round or to cell 0 in a partial one, then
multiply by the MDS matrix. The sponge is circomlib's: state [0, inputs...],
one permutation, output cell 0. The constants are the contract's own, read
from tests/poseidon-bn254.json.

    from poseidon import hash1, hash2
"""

import json
import pathlib

_C = json.loads(
    (pathlib.Path(__file__).resolve().parent.parent / "tests" / "poseidon-bn254.json").read_text()
)
R = int(_C["r"], 16)
_D = _C["d"]
_P = {
    t: {
        "f": _C[k]["rounds_f"],
        "p": _C[k]["rounds_p"],
        "mds": [[int(x, 16) for x in l] for l in _C[k]["mds"]],
        "rc": [[int(x, 16) for x in l] for l in _C[k]["rc"]],
    }
    for t, k in [(2, "t2"), (3, "t3")]
}


def permutacao(estado):
    p = _P[len(estado)]
    meio = p["f"] // 2
    for r, rc in enumerate(p["rc"]):
        estado = [(x + c) % R for x, c in zip(estado, rc)]
        if r < meio or r >= meio + p["p"]:
            estado = [pow(x, _D, R) for x in estado]
        else:
            estado[0] = pow(estado[0], _D, R)
        estado = [sum(m * x for m, x in zip(linha, estado)) % R for linha in p["mds"]]
    return estado


def _hash(*entradas):
    for x in entradas:
        if not 0 <= x < R:
            raise ValueError("Poseidon: entrada fora do campo")
    return permutacao([0, *entradas])[0]


def hash1(a):
    """Poseidon(a), as circomlib's Poseidon(1)."""
    return _hash(a)


def hash2(a, b):
    """Poseidon(a, b), as circomlib's Poseidon(2)."""
    return _hash(a, b)
