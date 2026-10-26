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
# T-034: (cpu, mem) tuples, `const NOME: (u64, u64) = (cpu, mem);`.
TUPLAS = {
    "groth16": (RAIZ / "contracts" / "groth16-verifier" / "src" / "test.rs", ["VERIFY_COST"]),
    "poseidon": (
        RAIZ / "contracts" / "poseidon" / "src" / "test.rs",
        ["HASH1_COST", "HASH2_COST", "TREE_6_COST"],
    ),
    "kleroterion": (
        RAIZ / "contracts" / "kleroterion" / "src" / "test.rs",
        [
            "ENROLL_COST", "IMPEDE_COST", "LOCK_COST", "PROOF_COST",
            "SEAL_COST", "CLAIM_COST", "RECUSE_COST", "FINALIZE_COST", "REVEAL_COST",
        ],
    ),
}

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
    "docs/SPEC.md": [
        "verify_cpu", "verify_mem",
        "kleroterion_enroll_cost_cpu", "kleroterion_impede_cost_cpu", "kleroterion_lock_cost_cpu",
        "kleroterion_proof_cost_cpu", "groth16_verify_cost_cpu",
        "kleroterion_seal_cost_cpu", "kleroterion_claim_cost_cpu", "kleroterion_recuse_cost_cpu",
        "kleroterion_finalize_cost_cpu", "kleroterion_reveal_cost_cpu",
    ],
    "docs/SOURCES.md": ["verify_cpu", "verify_mem"],
    "docs/DEMO.md": ["verify_cpu"],
    "docs/evidencia-testnet-spike.md": [
        "groth16_verify_cost_cpu", "groth16_verify_cost_mem",
        "poseidon_hash1_cost_cpu", "poseidon_hash2_cost_cpu",
        "poseidon_tree_6_cost_cpu", "poseidon_tree_6_cost_mem",
    ],
}


def numeros_do_teste_rust(fonte):
    """The cost constants test.rs asserts against."""
    def u64(nome):
        m = re.search(rf"const {nome}: u64 = ([\d_]+);", fonte)
        if not m:
            raise SystemExit(f"constante {nome} não achada em test.rs")
        return int(m.group(1).replace("_", ""))

    vals = {"verify_cpu": u64("VERIFY_CPU_COST"), "verify_mem": u64("VERIFY_MEM_COST")}

    bloco = re.search(r"const DRAW_COST: \[\(u32, u64, u64\); \d+\] = \[(.*?)\];", fonte, re.S)
    if not bloco:
        raise SystemExit("constante DRAW_COST não achada em test.rs")
    for n, cpu, mem in re.findall(r"\(([\d_]+), ([\d_]+), ([\d_]+)\)", bloco.group(1)):
        n = n.replace("_", "")
        vals[f"draw_{n}_cpu"] = int(cpu.replace("_", ""))
        vals[f"draw_{n}_mem"] = int(mem.replace("_", ""))
    return vals


def numeros_das_tuplas():
    """The (cpu, mem) constants of the T-034 crates, as groth16_custo_verify_cpu etc."""
    vals = {}
    for prefixo, (caminho, nomes) in TUPLAS.items():
        fonte = caminho.read_text()
        for nome in nomes:
            m = re.search(rf"const {nome}: \(u64, u64\) = \(([\d_]+), ([\d_]+)\);", fonte)
            if not m:
                raise SystemExit(f"constante {nome} não achada em {caminho.relative_to(RAIZ)}")
            base = f"{prefixo}_{nome.lower()}"
            vals[base + "_cpu"] = int(m.group(1).replace("_", ""))
            vals[base + "_mem"] = int(m.group(2).replace("_", ""))
    return vals


def sem_separadores(texto):
    """26.719.483 and 26,719,483 both become 26719483."""
    return re.sub(r"(?<=\d)[.,](?=\d{3}\b)", "", texto)


def main():
    vals = numeros_do_teste_rust(TESTE_RUST.read_text())
    vals.update(numeros_das_tuplas())
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
