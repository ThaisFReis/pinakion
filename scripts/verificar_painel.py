#!/usr/bin/env python3
"""Recompute a kleroterion panel from the ledger and drand, with no dependencies.

Sealed draw (T-038 onwards, T-039):

    python3 scripts/verificar_painel.py registro.json

`registro.json` holds the contract's events in ledger order, as
{"eventos": [...]}, each event named as the contract names it: Inscrito,
Impedido, Fechado, Selado, Ampliado, Reivindicado, Recusado, Finalizado,
Revelado. It may also hold "aleatoriedade", the drand randomness of the round
in `Fechado`. Without it, the script fetches that round from api.drand.sh.
The script does not verify the drand signature itself; `web/` and the contract
do (CAP-59).

From those alone it recomputes, without trusting the contract:
- the commitment root (Poseidon tree of the Inscrito/Impedido events) against
  `Fechado.raiz`;
- the seed (randomness >> 8) and the threshold T = floor(r*2k/N), the whole
  field if 2k >= N or after Ampliado, against `Selado` and `Ampliado`;
- every claim is below the threshold in force and appears once;
- every recusal and every reveal: Poseidon(s) = c of that juror, and
  y = Poseidon(seed, s);
- the order (ascending y, recused y removed), the panel (its first k) and
  whether it is complete, against `Finalizado`.

It then says, for each enrolled juror, one of: sentou · revelou, nao sentou ·
sorteado e ausente (below T, never claimed) · nao revelou · recusado ·
impedido. It also says how many seats nobody has revealed yet. Exit 0 when
everything matches, 1 when anything does not.

Open draw (the pre-T-038 contract, kept so docs/evidencia-testnet-bilhete.md
stays reproducible):

    python3 scripts/verificar_painel.py <seed-hex> <k> rol.json [--fora ADDR ...]
"""

import json
import pathlib
import sys
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from arvore import raiz  # noqa: E402
from poseidon import R, hash1, hash2  # noqa: E402
from verificar_sorteio import painel  # noqa: E402

CHAIN = "52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971"


def aleatoriedade_da_rodada(rodada):
    url = f"https://api.drand.sh/{CHAIN}/public/{rodada}"
    with urllib.request.urlopen(url, timeout=20) as r:
        return json.load(r)["randomness"]


def verificar(registro):
    """Return (estados, painel, completo, sem_revelacao, conferencias)."""
    eventos = registro["eventos"]
    conferencias = []

    def confere(nome, ok):
        conferencias.append((nome, bool(ok)))

    def um(nome):
        achados = [e for e in eventos if e["evento"] == nome]
        return achados[0] if achados else None

    jurados, ordem, impedidos = {}, [], set()
    folhas = []
    for e in eventos:
        if e["evento"] == "Inscrito":
            jurados[e["jurado"]] = {"indice": e["indice"], "c": int(e["c"])}
            ordem.append(e["jurado"])
            folhas.append({"evento": "enroll", "indice": e["indice"], "c": e["c"]})
        elif e["evento"] == "Impedido":
            impedidos.add(e["jurado"])
            folhas.append({"evento": "impede", "indice": e["indice"]})

    fechado = um("Fechado")
    confere("houve fechamento", fechado)
    if not fechado:
        return {}, [], False, 0, conferencias
    confere("a raiz do fechamento e a dos compromissos", str(raiz(folhas)) == fechado["raiz"])
    n, k = fechado["n"], fechado["k"]
    confere("N e quem estava autorizado", n == len(ordem) - len(impedidos))

    aleatoriedade = registro.get("aleatoriedade") or aleatoriedade_da_rodada(fechado["rodada"])
    seed = int(aleatoriedade, 16) >> 8
    limiar = R if 2 * k >= n else R * 2 * k // n

    selado = um("Selado")
    confere("houve selo", selado)
    if not selado:
        return {}, [], False, 0, conferencias
    confere("a semente e a da rodada drand >> 8", int(selado["seed"]) == seed)
    confere("o limiar e floor(r*2k/N)", int(selado["limiar"]) == limiar)

    reivindicados, recusados, revelados = [], {}, {}
    for e in eventos:
        if e["evento"] == "Ampliado":
            limiar = R
            confere("a ampliacao leva o limiar ao campo", int(e["limiar"]) == R)
        elif e["evento"] == "Reivindicado":
            y = int(e["y"])
            confere(f"reivindicacao {y} abaixo do limiar", y < limiar)
            confere(f"reivindicacao {y} unica", y not in reivindicados)
            reivindicados.append(y)
        elif e["evento"] in ("Recusado", "Revelado"):
            quem, s, y = e["jurado"], int(e["s"]), int(e["y"])
            c = jurados.get(quem, {}).get("c")
            confere(f"{e['evento']} de {quem}: Poseidon(s) = c", c is not None and hash1(s) == c)
            confere(f"{e['evento']} de {quem}: y = Poseidon(seed, s)", hash2(seed, s) == y)
            if e["evento"] == "Recusado":
                recusados[quem] = y
            else:
                revelados[quem] = e

    restantes = sorted(y for y in reivindicados if y not in recusados.values())
    meu_painel = restantes[:k]
    final = um("Finalizado")
    confere("houve painel", final)
    if final:
        confere("o painel sao os k menores y que ficaram", [int(v) for v in final["painel"]] == meu_painel)
        confere("completo se e so se chegou a k", final["completo"] == (len(meu_painel) == k))
    for quem, e in revelados.items():
        confere(f"Revelado de {quem}: sentado confere", e["sentado"] == (int(e["y"]) in meu_painel))

    estados = {}
    for quem in ordem:
        if quem in impedidos:
            estados[quem] = "impedido"
        elif quem in recusados:
            estados[quem] = "recusado"
        elif quem not in revelados:
            estados[quem] = "nao revelou"
        else:
            y = int(revelados[quem]["y"])
            if y in meu_painel:
                estados[quem] = "sentou"
            elif y < limiar and y not in reivindicados:
                estados[quem] = "sorteado e ausente"
            else:
                estados[quem] = "revelou, nao sentou"
    sem_revelacao = len(meu_painel) - sum(1 for e in estados.values() if e == "sentou")
    return estados, meu_painel, len(meu_painel) == k, sem_revelacao, conferencias


def main(argv):
    if len(argv) >= 4 and len(argv[1]) == 64:
        seed, k, rol_path = argv[1], int(argv[2]), argv[3]
        fora = [argv[i + 1] for i, a in enumerate(argv) if a == "--fora" and i + 1 < len(argv)]
        rol = json.loads(pathlib.Path(rol_path).read_text())
        for assento, endereco in enumerate(painel(seed, rol, fora, k)):
            print(f"{assento}  {endereco}")
        return 0
    if len(argv) != 2:
        print(__doc__)
        return 2
    estados, meu_painel, completo, sem_revelacao, conferencias = verificar(
        json.loads(pathlib.Path(argv[1]).read_text())
    )
    for quem, estado in estados.items():
        print(f"  {estado:<22} {quem}")
    print(f"\n  painel: {len(meu_painel)} assento(s){'' if completo else ', INCOMPLETO'}; "
          f"{sem_revelacao} sem revelacao")
    falhas = [nome for nome, ok in conferencias if not ok]
    for nome in falhas:
        print(f"  NAO CONFERE: {nome}")
    print(f"\n{'CONFERE' if not falhas else 'NAO CONFERE'} — {len(conferencias)} conferencias, {len(falhas)} falha(s).")
    return 0 if not falhas else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
