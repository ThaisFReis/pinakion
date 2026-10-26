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
# Since T-038 kleroterion draws sealed (DEC-015, DEC-016): the open
# Fisher-Yates and its vectors stay with beacon-verifier alone. The address
# vector (the fourth) is still checked by the Python and JS recomputations;
# T-039 replaces it with sealed-flow vectors.
TESTES_RUST = [
    RAIZ / "contracts" / "beacon-verifier" / "src" / "test.rs",
]


def paineis_do_teste_rust(fonte):
    """Every `let expected_x: [u32; N] = [ ... ];` in a Rust test file, plus
    kleroterion's `const ORDEM_DE_OURO`, which is the small-roster vector."""
    achados = []
    for corpo in re.findall(
        r"(?:let expected_\w+|const GOLDEN_ORDER): \[u32; \d+\] = \[(.*?)\];", fonte, re.S
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
    # ... the sealed-flow fixture (T-038) is what Poseidon says it is ...
    fluxo = json.loads((RAIZ / "tests" / "fluxo-n8.json").read_text())
    from poseidon import R  # noqa: E402
    erros = []
    k, n = fluxo["k"], fluxo["n"]
    limiar = R if 2 * k >= n else R * 2 * k // n
    if int(fluxo["limiar"]) != limiar:
        erros.append("limiar")
    seed = int(fluxo["seed"])
    for j in fluxo["jurados"]:
        s_ = int(j["s"])
        if hash1(s_) != int(j["c"]):
            erros.append(f"c do jurado {j['indice']}")
        if hash2(seed, s_) != int(j["y"]) or (int(j["y"]) < limiar) != j["abaixo_do_limiar"]:
            erros.append(f"y do jurado {j['indice']}")
    eventos = [{"evento": "enroll", "indice": j["indice"], "c": j["c"]} for j in fluxo["jurados"]]
    if str(raiz(eventos)) != fluxo["raiz"]:
        erros.append("raiz")
    falhas += len(erros)
    print(f"  {'FALHA' if erros else 'ok   '} fluxo N = {n}: c, y, raiz e limiar" + (f" — {erros}" if erros else ""))
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

    # T-039: the sealed vectors, from the events and the drand randomness alone.
    from verificar_painel import verificar  # noqa: E402
    for v in dados.get("selados", []):
        estados, meu_painel, completo, sem_revelacao, conferencias = verificar(v)
        ruins = [nome for nome, ok in conferencias if not ok]
        esperado = v["esperado"]
        certo = (
            not ruins
            and estados == esperado["jurados"]
            and [str(y) for y in meu_painel] == esperado["painel"]
            and completo == esperado["completo"]
            and sem_revelacao == esperado["assentos_sem_revelacao"]
        )
        if certo:
            print(f"  ok    {v['nome']}: {len(conferencias)} conferencias")
        else:
            falhas += 1
            print(f"  FALHA {v['nome']}")
            for nome in ruins:
                print(f"        nao confere: {nome}")
            if estados != esperado["jurados"]:
                print("        os estados dos jurados divergem")

    # T-045: the verifier catches a party who sits unimpeded, and a k that is
    # not the level's. Each mutation of the sealed vector must fail by name.
    import copy  # noqa: E402
    for v in dados.get("selados", []):
        fech = next(e for e in v["eventos"] if e["evento"] == "Fechado")
        primeiro = next(e["jurado"] for e in v["eventos"] if e["evento"] == "Inscrito")
        for campo, valor, nome in [("acusado", primeiro, "a parte inscrita foi impedida antes do fechamento"),
                                   ("gravidade", fech["gravidade"] + 1, "k e o do nivel")]:
            w = copy.deepcopy(v)
            next(e for e in w["eventos"] if e["evento"] == "Fechado")[campo] = valor
            if dict(verificar(w)[4]).get(nome, True):
                falhas += 1
                print(f"  FALHA a mutacao de {campo} passou sem '{nome}'")
            else:
                print(f"  ok    mutacao de {campo}: '{nome}' nao confere")
    # T-040: the recorded testnet run still checks out under today's verifier.
    registro = RAIZ / "docs" / "evidencia-testnet-registro.json"
    _, painel_tn, completo_tn, _, conf_tn = verificar(json.loads(registro.read_text()))
    ruins = [nome for nome, ok in conf_tn if not ok]
    if ruins or not completo_tn:
        falhas += 1
        print(f"  FALHA a rodada de testnet registrada: {ruins}")
    else:
        print(f"  ok    a rodada de testnet registrada: {len(conf_tn)} conferencias, painel de {len(painel_tn)}")
    # T-042: the Tessera export in docs/ is what the exporter writes today.
    import subprocess  # noqa: E402
    exportado = subprocess.run([sys.executable, str(RAIZ / "scripts" / "exportar_tessera.py"), str(registro)],
                               capture_output=True, text=True).stdout
    if exportado == (RAIZ / "docs" / "evidencia-testnet-aptos.txt").read_text():
        print("  ok    a exportacao para o Tessera e a registrada")
    else:
        falhas += 1
        print("  FALHA docs/evidencia-testnet-aptos.txt divergiu de scripts/exportar_tessera.py")
    # T-050: the recorded case checks out from the ledger alone, and three
    # mutations of it fail by name: a voter list that is not the attendees,
    # a score that is not Tessera's, and the party attending as a juror.
    import verificar_processo  # noqa: E402
    caso = json.loads((RAIZ / "docs" / "evidencia-processo-registro.json").read_text())
    autos = RAIZ / "docs" / "caso-demo"
    r = verificar_processo.verificar(caso, autos)
    ruins = [n for n, ok in r[1] if not ok]
    if ruins or not all(r[0].values()):
        falhas += 1
        print(f"  FALHA o processo registrado: {ruins or r[0]}")
    else:
        print(f"  ok    o processo registrado: {len(r[1])} conferencias, {len(r[0])} fases")
    def mutado(f):
        w = copy.deepcopy(caso)
        f(w)
        return dict(verificar_processo.verificar(w, autos)[1])
    def aptos(w):
        w["tessera"]["aptos"] = w["tessera"]["aptos"][:-1]
    def placar(w):
        d = next(e for e in w["eventos"] if e["evento"] == "Decidido")
        d["sim"] += 1
    def parte(w):
        c = next(e for e in w["eventos"] if e["evento"] == "Compareceu")
        c["assento"] = next(e for e in w["eventos"] if e["evento"] == "CasoAberto")["acusado"]
    for nome, f, conferencia in [("aptos", aptos, "a lista de votantes do Tessera e a dos presentes, na ordem"),
                                 ("placar", placar, "o placar e o que o Tessera publicou"),
                                 ("parte", parte, "a parte nunca compareceu como jurada")]:
        if mutado(f).get(conferencia, True):
            falhas += 1
            print(f"  FALHA a mutacao '{nome}' passou sem '{conferencia}'")
        else:
            print(f"  ok    mutacao do processo '{nome}': '{conferencia}' nao confere")
    # T-049: docs/evidencia-processo.md is what its generator prints from the record.
    impresso = subprocess.run([sys.executable, str(RAIZ / "scripts" / "evidencia_processo.py"),
                               str(RAIZ / "docs" / "evidencia-processo-registro.json")],
                              capture_output=True, text=True, cwd=RAIZ).stdout
    if impresso == (RAIZ / "docs" / "evidencia-processo.md").read_text():
        print("  ok    docs/evidencia-processo.md e o que o registro imprime")
    else:
        falhas += 1
        print("  FALHA docs/evidencia-processo.md divergiu de scripts/evidencia_processo.py")
    # T-044: every table of the panel-size study is what the model prints today.
    saida = subprocess.run([sys.executable, str(RAIZ / "scripts" / "tamanho_do_painel.py")],
                           capture_output=True, text=True).stdout
    estudo = (RAIZ / "docs" / "desenho-gravidade-afastamento-revelacao.md").read_text()
    blocos = [b.strip() for b in saida.split("\n\n") if b.strip()]
    fora = [b.splitlines()[0] for b in blocos if b not in estudo]
    if fora or not blocos:
        falhas += 1
        print(f"  FALHA o estudo de gravidade divergiu de scripts/tamanho_do_painel.py: {fora}")
    else:
        print(f"  ok    o estudo de gravidade: {len(blocos)} tabelas iguais ao modelo")

    if falhas:
        print(f"\n{falhas} falha(s).")
        return 1
    print(
        f"\n{len(vetores)} vetores conferidos, {len(poseidon)} de Poseidon, "
        f"{len(arvores['arvores'])} arvores, {len(dados.get('selados', []))} selado(s)."
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
