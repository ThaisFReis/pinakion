#!/usr/bin/env python3
"""Gate: the published cost figures still match what the tests measure.

`CLAUDE.md` says every number in the project's documents must be reproducible
by a gate. The contract's cost is measured by the Rust tests, which assert it
against constants in test.rs — so a change in cost turns G2 red. This closes
the other half: a document that drifts away from those constants turns G7 red.

The numbers appear in two spellings, 26.719.483 in Portuguese and 26,719,483
in English, so digit separators are stripped before comparing.

    python3 scripts/conferir_numeros.py

Exit 0 when every document agrees, 1 otherwise. No dependencies.
"""

import pathlib
import re
import sys

RAIZ = pathlib.Path(__file__).resolve().parent.parent
TESTE_RUST = RAIZ / "contracts" / "beacon-verifier" / "src" / "test.rs"

# Which figures each document publishes, by the name used below.
ONDE = {
    "docs/evidencia-testnet-sorteio.md": [
        "verify_cpu", "verify_mem",
        "draw_50_cpu", "draw_50_mem",
        "draw_200_cpu", "draw_200_mem",
        "draw_1000_cpu", "draw_1000_mem",
    ],
    "README.md": ["verify_cpu", "verify_mem", "draw_1000_cpu"],
    "THESIS.md": ["verify_cpu", "draw_1000_cpu"],
    "docs/SPEC.md": ["verify_cpu", "verify_mem"],
    "docs/SOURCES.md": ["verify_cpu", "verify_mem"],
    "docs/DEMO.md": ["verify_cpu"],
}


def numeros_do_teste_rust(fonte):
    """The cost constants test.rs asserts against."""
    def u64(nome):
        m = re.search(rf"const {nome}: u64 = ([\d_]+);", fonte)
        if not m:
            raise SystemExit(f"constante {nome} não achada em test.rs")
        return int(m.group(1).replace("_", ""))

    vals = {"verify_cpu": u64("CUSTO_VERIFY_CPU"), "verify_mem": u64("CUSTO_VERIFY_MEM")}

    bloco = re.search(r"const CUSTO_DRAW: \[\(u32, u64, u64\); \d+\] = \[(.*?)\];", fonte, re.S)
    if not bloco:
        raise SystemExit("constante CUSTO_DRAW não achada em test.rs")
    for n, cpu, mem in re.findall(r"\(([\d_]+), ([\d_]+), ([\d_]+)\)", bloco.group(1)):
        n = n.replace("_", "")
        vals[f"draw_{n}_cpu"] = int(cpu.replace("_", ""))
        vals[f"draw_{n}_mem"] = int(mem.replace("_", ""))
    return vals


def sem_separadores(texto):
    """26.719.483 and 26,719,483 both become 26719483."""
    return re.sub(r"(?<=\d)[.,](?=\d{3}\b)", "", texto)


def main():
    vals = numeros_do_teste_rust(TESTE_RUST.read_text())
    falhas = 0

    for doc, nomes in ONDE.items():
        caminho = RAIZ / doc
        if not caminho.exists():
            print(f"  FALHA {doc} não existe")
            falhas += 1
            continue

        texto = sem_separadores(caminho.read_text())
        faltando = [n for n in nomes if str(vals[n]) not in texto]
        if faltando:
            falhas += 1
            print(f"  FALHA {doc}")
            for n in faltando:
                print(f"        não diz {vals[n]} ({n})")
        else:
            print(f"  ok    {doc} — {len(nomes)} número(s)")

    if falhas:
        print(f"\n{falhas} documento(s) divergindo do que os testes medem.")
        return 1
    print(f"\n{sum(len(v) for v in ONDE.values())} números conferidos em {len(ONDE)} documentos.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
