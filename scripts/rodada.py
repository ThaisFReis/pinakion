#!/usr/bin/env python3
"""Fetch a drand quicknet round and print it ready for the contract.

The contract's `verify` takes the signature as BytesN<96>: the G1 point
UNCOMPRESSED, be(X)||be(Y). drand publishes it compressed, in 48 bytes.
Decompression happens off the chain, which is safe — a wrong point simply
fails the pairing — but it has to happen somewhere, and this is where.

Unlike `verificar_sorteio.py`, this script needs py_ecc:

    pip install py_ecc

    python3 scripts/rodada.py            # the latest round
    python3 scripts/rodada.py 33382274   # a specific one
"""

import json
import sys
import urllib.request

from py_ecc.bls.point_compression import decompress_G1
from py_ecc.optimized_bls12_381 import normalize
from py_ecc.bls.typing import G1Compressed

CHAIN = "52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971"


def buscar(rodada):
    url = f"https://api.drand.sh/{CHAIN}/public/{rodada}"
    with urllib.request.urlopen(url, timeout=20) as r:
        return json.load(r)


def descomprimir(sig_hex):
    """48 bytes compressed -> 96 bytes be(X)||be(Y), what the contract wants."""
    ponto = decompress_G1(G1Compressed(int(sig_hex, 16)))
    x, y = normalize(ponto)
    return int(x).to_bytes(48, "big") + int(y).to_bytes(48, "big")


def main():
    rodada = sys.argv[1] if len(sys.argv) > 1 else "latest"
    d = buscar(rodada)
    sig = descomprimir(d["signature"]).hex()

    print(f"round       {d['round']}")
    print(f"randomness  {d['randomness']}")
    print(f"signature   {sig}")
    print()
    print("stellar contract invoke --id $PINAKION --source $ID --network testnet \\")
    print(f"  -- verify --round {d['round']} --sig {sig}")


if __name__ == "__main__":
    main()
