#!/usr/bin/env python3
"""Write the sealed golden vector (T-039) into tests/vetores-de-ouro.json.

The scenario is contracts/kleroterion's `fluxo_completo_n8_k3`, on the
jurors of tests/fluxo-n8.json:
- eight enrol (main accounts are fixed C-strkeys) and the roll locks with k = 3
  on round 1000000;
- seal;
- jurors 1, 2, 3, 5, 6, 7 claim, in that order; 0 is below T and does not
  claim; 4 is above T and is refused, which leaves no event;
- juror 6 recuses;
- finalize;
- everyone but one seated juror reveals.

The vector is the exact event list the contract emits — the Rust test
replays it and asserts every event — plus what an independent verifier must
conclude. `scripts/verificar_painel.py` (G6) and `web/sorteio.mjs` (G5) must
reach `esperado` from `eventos` and the drand randomness alone.

    python3 scripts/gerar_vetor_selado.py

No dependencies.
"""

import base64
import hashlib
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from poseidon import R, hash2  # noqa: E402

RAIZ = pathlib.Path(__file__).resolve().parent.parent
FLUXO = RAIZ / "tests" / "fluxo-n8.json"
OURO = RAIZ / "tests" / "vetores-de-ouro.json"

ALEATORIEDADE = "b22aad4794f7451896f7a371aa46106fd84d919f3f569acd5b2fddf1d1440af3"
RODADA, REVELA_EM, LEDGER_DO_SELO, JANELA = 1000000, 17280, 1000, 100


def strkey(versao, pk):
    dado = bytes([versao]) + pk
    crc = 0
    for b in dado:
        crc ^= b << 8
        for _ in range(8):
            crc = ((crc << 1) ^ 0x1021) & 0xFFFF if crc & 0x8000 else (crc << 1) & 0xFFFF
    return base64.b32encode(dado + crc.to_bytes(2, "little")).decode()


def principal(i):
    # A C-address: the test host has no account entries, so the SAC cannot
    # open a trustline for a G-address there. The verifier does not care.
    return strkey(2 << 3, hashlib.sha256(f"pinakion:fluxo:principal:{i}".encode()).digest())


def main():
    f = json.loads(FLUXO.read_text())
    js = f["jurados"]
    n, k = f["n"], f["k"]
    seed, limiar = int(f["seed"]), int(f["limiar"])
    assert seed == int(ALEATORIEDADE, 16) >> 8
    y = {j["indice"]: int(j["y"]) for j in js}

    eventos = [{"evento": "Inscrito", "jurado": principal(j["indice"]), "indice": j["indice"], "c": j["c"]} for j in js]
    eventos.append({"evento": "Fechado", "raiz": f["raiz"], "n": n, "rodada": RODADA, "k": k, "revela_em": REVELA_EM})
    eventos.append({"evento": "Selado", "seed": str(seed), "limiar": str(limiar), "prazo": LEDGER_DO_SELO + JANELA})
    quem, recusa = [1, 2, 3, 5, 6, 7], 6
    eventos += [{"evento": "Reivindicado", "y": str(y[i])} for i in quem]
    eventos.append({"evento": "Recusado", "jurado": principal(recusa), "s": js[recusa]["s"], "y": str(y[recusa])})
    ficaram = [i for i in quem if i != recusa]
    painel = sorted(y[i] for i in ficaram)[:k]
    eventos.append({"evento": "Finalizado", "painel": [str(v) for v in painel], "completo": True})
    silencioso = next(i for i in ficaram if y[i] in painel)
    revelam = [i for i in [0, 1, 2, 3, 4, 5, 7] if i != silencioso]
    for i in revelam:
        eventos.append({"evento": "Revelado", "jurado": principal(i), "s": js[i]["s"], "y": str(y[i]),
                        "sentado": y[i] in painel, "sem_bilhete": False})

    def estado(i):
        if i == recusa:
            return "recusado"
        if i not in revelam:
            return "nao revelou"
        if y[i] in painel:
            return "sentou"
        if y[i] < limiar and i not in quem:
            return "sorteado e ausente"
        return "revelou, nao sentou"

    vetor = {
        "nome": "selado: rol de 8, k = 3, um ausente, um acima do limiar, uma recusa, um silencioso",
        "aleatoriedade": ALEATORIEDADE,
        "eventos": eventos,
        "esperado": {
            "painel": [str(v) for v in painel],
            "completo": True,
            "jurados": {principal(i): estado(i) for i in range(n)},
            "assentos_sem_revelacao": 1,
        },
    }
    assert limiar == (R * 2 * k // n), "limiar"
    assert all(hash2(seed, int(j["s"])) == y[j["indice"]] for j in js)

    ouro = json.loads(OURO.read_text())
    ouro["selados"] = [vetor]
    ouro["_"] = ouro["_"].split(" The sealed vectors")[0] + (
        " The sealed vectors (\"selados\", T-039) are the exact event lists contracts/kleroterion emits"
        " in fluxo_completo_n8_k3 — kleroterion/src/test.rs::bate_com_o_vetor_selado replays them —"
        " and what scripts/verificar_painel.py and web/sorteio.mjs must conclude from them and the drand randomness alone."
    )
    OURO.write_text(json.dumps(ouro, indent=1, ensure_ascii=False) + "\n")
    print(f"wrote {OURO.relative_to(RAIZ)}: {len(eventos)} events")
    for a, e in vetor["esperado"]["jurados"].items():
        print(" ", a[:8], e)


if __name__ == "__main__":
    main()
