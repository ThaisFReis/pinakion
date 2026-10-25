#!/usr/bin/env python3
"""Gate: the Python recomputation still agrees with the golden vectors.

It also checks that the vectors in tests/vetores-de-ouro.json are the same
numbers the Rust test asserts. Without that second check the two could drift
apart quietly, each one green on its own, which is the failure this gate
exists to prevent.

    python3 scripts/conferir_vetores.py

Exit 0 when everything agrees, 1 on the first disagreement. No dependencies.
"""

import json
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from verificar_sorteio import sortear  # noqa: E402

RAIZ = pathlib.Path(__file__).resolve().parent.parent
VETORES = RAIZ / "tests" / "vetores-de-ouro.json"
TESTE_RUST = RAIZ / "contracts" / "beacon-verifier" / "src" / "test.rs"


def paineis_do_teste_rust(fonte):
    """Every `let esperado_x: [u32; N] = [ ... ];` in the Rust test file."""
    achados = []
    for corpo in re.findall(
        r"let esperado_\w+: \[u32; \d+\] = \[(.*?)\];", fonte, re.S
    ):
        achados.append([int(n) for n in re.findall(r"\d+", corpo)])
    return achados


def main():
    dados = json.loads(VETORES.read_text())
    vetores = dados["vetores"]
    falhas = 0

    for v in vetores:
        obtido = sortear(v["semente"], v["n"], v["k"])
        if obtido == v["painel"]:
            print(f"  ok    {v['nome']}")
        else:
            falhas += 1
            print(f"  FALHA {v['nome']}")
            print(f"        esperado {v['painel']}")
            print(f"        obtido   {obtido}")

    esperados_rust = paineis_do_teste_rust(TESTE_RUST.read_text())
    nossos = [v["painel"] for v in vetores]
    if esperados_rust == nossos:
        print(f"  ok    os mesmos {len(nossos)} painéis estão em test.rs")
    else:
        falhas += 1
        print("  FALHA os vetores divergiram do test.rs")
        print(f"        test.rs {esperados_rust}")
        print(f"        json    {nossos}")

    if falhas:
        print(f"\n{falhas} falha(s).")
        return 1
    print(f"\n{len(vetores)} vetores conferidos.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
