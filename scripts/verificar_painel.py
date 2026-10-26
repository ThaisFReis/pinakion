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
  whether it is complete, against `Finalizado`;
- since T-045, when the record has `Constituido`: the level in `Fechado` is in
  the table, k is that level's, the roll reaches its floor, the suspension
  ends `afastamento` ledgers after the lock, the reveal ledger is at least
  `espera_minima` after it (T-046); and when `Fechado` names the
  party (`acusado`): an enrolled party was impeded before the lock and never
  sits. Records of the contract before T-045 carry neither and are checked as
  they were.

It then says, for each enrolled juror, one of: sat · revealed, did not sit ·
drawn and absent (below T, never claimed) · did not reveal · recused ·
impeded. It also says how many seats nobody has revealed yet. Exit 0 when
everything matches, 1 when anything does not.

`verificar()` returns each state as its key (`sentou`, `revelou, nao sentou`,
`sorteado e ausente`, `nao revelou`, `recusado`, `impedido`): the golden
vectors in tests/vetores-de-ouro.json hold those exact strings. Only the
printed report uses the English labels in `ESTADOS`.

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

# The state keys verificar() returns, and how the report prints them.
ESTADOS = {
    "sentou": "sat",
    "revelou, nao sentou": "revealed, did not sit",
    "sorteado e ausente": "drawn and absent",
    "nao revelou": "did not reveal",
    "recusado": "recused",
    "impedido": "impeded",
}


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
    confere("there was a lock", fechado)
    if not fechado:
        return {}, [], False, 0, conferencias
    confere("the lock root is the root of the commitments", str(raiz(folhas)) == fechado["raiz"])
    n, k = fechado["n"], fechado["k"]
    confere("N is the number of authorized jurors", n == len(ordem) - len(impedidos))

    constituido = um("Constituido")
    if constituido:
        niveis, g = constituido["niveis"], fechado.get("gravidade")
        no_quadro = isinstance(g, int) and 1 <= g <= len(niveis)
        confere("the lock level is in the table", no_quadro)
        if no_quadro:
            confere("k is the k of the level", k == niveis[g - 1]["k"])
            confere("the roster reaches the level minimum", len(ordem) >= niveis[g - 1]["rol_minimo"])
        if "ledger" in fechado:
            confere("the suspension is the one set at construction",
                    fechado["afastado_ate"] == fechado["ledger"] + constituido["afastamento"])
            if "espera_minima" in constituido:
                confere("the reveal respects the minimum wait",
                        fechado["revela_em"] >= fechado["ledger"] + constituido["espera_minima"])
    parte = fechado.get("acusado")
    if parte in jurados:
        antes = eventos[:eventos.index(fechado)]
        confere("the enrolled party was impeded before the lock",
                any(e["evento"] == "Impedido" and e["jurado"] == parte for e in antes))

    aleatoriedade = registro.get("aleatoriedade") or aleatoriedade_da_rodada(fechado["rodada"])
    seed = int(aleatoriedade, 16) >> 8
    limiar = R if 2 * k >= n else R * 2 * k // n

    selado = um("Selado")
    confere("there was a seal", selado)
    if not selado:
        return {}, [], False, 0, conferencias
    confere("the seed is the drand round >> 8", int(selado["seed"]) == seed)
    confere("the threshold is floor(r*2k/N)", int(selado["limiar"]) == limiar)

    reivindicados, recusados, revelados = [], {}, {}
    for e in eventos:
        if e["evento"] == "Ampliado":
            limiar = R
            confere("the widening takes the threshold to the field", int(e["limiar"]) == R)
        elif e["evento"] == "Reivindicado":
            y = int(e["y"])
            confere(f"claim {y} below the threshold", y < limiar)
            confere(f"claim {y} unique", y not in reivindicados)
            reivindicados.append(y)
        elif e["evento"] in ("Recusado", "Revelado"):
            quem, s, y = e["jurado"], int(e["s"]), int(e["y"])
            c = jurados.get(quem, {}).get("c")
            confere(f"{e['evento']} by {quem}: Poseidon(s) = c", c is not None and hash1(s) == c)
            confere(f"{e['evento']} by {quem}: y = Poseidon(seed, s)", hash2(seed, s) == y)
            if e["evento"] == "Recusado":
                recusados[quem] = y
            else:
                revelados[quem] = e

    restantes = sorted(y for y in reivindicados if y not in recusados.values())
    meu_painel = restantes[:k]
    final = um("Finalizado")
    confere("there was a panel", final)
    if final:
        confere("the panel is the k smallest remaining y", [int(v) for v in final["painel"]] == meu_painel)
        confere("complete if and only if it reached k", final["completo"] == (len(meu_painel) == k))
    for quem, e in revelados.items():
        confere(f"Revelado by {quem}: the sat flag matches", e["sentado"] == (int(e["y"]) in meu_painel))
    if parte:
        confere("the party does not sit in their own case", not (parte in revelados and revelados[parte]["sentado"]))

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
        print(f"  {ESTADOS.get(estado, estado):<22} {quem}")
    print(f"\n  panel: {len(meu_painel)} seat(s){'' if completo else ', INCOMPLETE'}; "
          f"{sem_revelacao} without a reveal")
    falhas = [nome for nome, ok in conferencias if not ok]
    for nome in falhas:
        print(f"  DOES NOT MATCH: {nome}")
    print(f"\n{'MATCHES' if not falhas else 'DOES NOT MATCH'} — {len(conferencias)} checks, {len(falhas)} failure(s).")
    return 0 if not falhas else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
