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
from verificar_sorteio import painel, sortear  # noqa: E402
from poseidon import hash1, hash2  # noqa: E402
from arvore import raiz, zeros  # noqa: E402
import exportar_constantes_poseidon as constantes  # noqa: E402

RAIZ = pathlib.Path(__file__).resolve().parent.parent
VETORES = RAIZ / "tests" / "vetores-de-ouro.json"
TESTES_RUST = [
    RAIZ / "contracts" / "beacon-verifier" / "src" / "test.rs",
    RAIZ / "contracts" / "kleroterion" / "src" / "test.rs",
]


def paineis_do_teste_rust(fonte):
    """Every `let esperado_x: [u32; N] = [ ... ];` in a Rust test file, plus
    kleroterion's `const ORDEM_DE_OURO`, which is the small-roster vector."""
    achados = []
    for corpo in re.findall(
        r"(?:let esperado_\w+|const ORDEM_DE_OURO): \[u32; \d+\] = \[(.*?)\];", fonte, re.S
    ):
        achados.append([int(n) for n in re.findall(r"\d+", corpo)])
    return achados


def main():
    dados = json.loads(VETORES.read_text())
    vetores = dados["vetores"]
    falhas = 0

    for v in vetores:
        if "rol" in v:
            obtido = painel(v["semente"], v["rol"], v["inelegiveis"], v["k"])
        else:
            obtido = sortear(v["semente"], v["n"], v["k"])
        if obtido == v["painel"]:
            print(f"  ok    {v['nome']}")
        else:
            falhas += 1
            print(f"  FALHA {v['nome']}")
            print(f"        esperado {v['painel']}")
            print(f"        obtido   {obtido}")

    nossos = [v["painel"] for v in vetores if "rol" not in v]
    enderecos = [v for v in vetores if "rol" in v]
    for teste in TESTES_RUST:
        fonte = teste.read_text()
        esperados_rust = paineis_do_teste_rust(fonte)
        nome = teste.relative_to(RAIZ)
        if sorted(esperados_rust) == sorted(nossos):
            print(f"  ok    os mesmos {len(nossos)} painéis estão em {nome}")
        else:
            falhas += 1
            print(f"  FALHA os vetores divergiram de {nome}")
            print(f"        rust {esperados_rust}")
            print(f"        json {nossos}")
        if "kleroterion" in str(teste):
            for v in enderecos:
                faltam = [a for a in v["rol"] + v["painel"] if a not in fonte]
                if faltam:
                    falhas += 1
                    print(f"  FALHA enderecos do vetor ausentes de {nome}: {faltam}")
                else:
                    print(f"  ok    os enderecos do vetor estão em {nome}")

    # T-035: the constants JSON is still the contract's params.rs ...
    fonte = constantes.PARAMS.read_text()
    json_c = json.loads(constantes.SAIDA.read_text())
    for t, f in [("t2", "2"), ("t3", "3")]:
        for parte in ("mds", "rc"):
            if json_c[t][parte] != constantes.matriz(fonte, f"get_{parte}_bn254_t_{f}"):
                falhas += 1
                print(f"  FALHA tests/poseidon-bn254.json divergiu de params.rs ({t}.{parte})")
    # ... Poseidon equals circomlib's vectors ...
    poseidon = json.loads((RAIZ / "tests" / "vetores-poseidon.json").read_text())["vetores"]
    erradas = 0
    for v in poseidon:
        xs = [int(x, 16) for x in v["entradas"]]
        h = hash1(*xs) if len(xs) == 1 else hash2(*xs)
        if h != int(v["saida"], 16):
            erradas += 1
            print(f"  FALHA Poseidon({v['entradas']}) = {h:#x}, esperado {v['saida']}")
    falhas += erradas
    if not erradas:
        print(f"  ok    Poseidon: {len(poseidon)} vetores da circomlib")
    # ... and the root rebuilt from the events is the one in the vectors.
    arvores = json.loads((RAIZ / "tests" / "vetores-arvore.json").read_text())
    if [str(z) for z in zeros()] == arvores["zeros"]:
        print("  ok    as raizes vazias")
    else:
        falhas += 1
        print("  FALHA as raizes vazias")
    for a in arvores["arvores"]:
        obtida = str(raiz(a["eventos"]))
        if obtida == a["raiz"]:
            print(f"  ok    arvore: {a['nome']}")
        else:
            falhas += 1
            print(f"  FALHA arvore: {a['nome']}\n        esperado {a['raiz']}\n        obtido   {obtida}")

    if falhas:
        print(f"\n{falhas} falha(s).")
        return 1
    print(
        f"\n{len(vetores)} vetores conferidos, {len(poseidon)} de Poseidon, "
        f"{len(arvores['arvores'])} arvores."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
