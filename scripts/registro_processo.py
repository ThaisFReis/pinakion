#!/usr/bin/env python3
"""Read a whole case back from testnet into one record (T-049).

    PINAKION_FONTE=<identity> python3 scripts/registro_processo.py build/<p>/ids.env build/<p>/tessera/estado/<vote>.json > registro.json

It reads the events of the case's two contracts — kleroterion (the roster
and the sealed draw) and processo (the case around it) — with
scripts/registro.py, tags each with the contract it came from, and keeps them
in ledger order. It adds the drand randomness of the round in `Fechado`, and
from Tessera, the secret-ballot module the vote ran on: the proposal as
Tessera's contract stores it (voter root, committee, window), the published
result, and the transaction of each step, from the Tessera CLI's own state
file. The ballots' content is not in it, because it is nowhere: Tessera only
ever publishes commitments.

What scripts/verificar_processo.py and web/processo.mjs read. Needs the
Stellar CLI and network access. No Python dependencies.
"""

import json
import os
import pathlib
import subprocess
import sys
import urllib.request

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import registro  # noqa: E402

NOMES_PROCESSO = {"Instituido", "CasoAberto", "Registrado", "Compareceu", "SessaoInstalada",
                  "Adiado", "VotacaoVinculada", "Decidido"}


def ler(contrato, metodo, *args):
    # A read: simulated, never sent. Any identity of the Stellar CLI will do.
    fonte = os.environ.get("PINAKION_FONTE", "default")
    cmd = ["stellar", "contract", "invoke", "--id", contrato, "--source", fonte, "--network", "testnet",
           "--send=no", "--", metodo, *args]
    s = subprocess.run(cmd, capture_output=True, text=True)
    linhas = [l for l in s.stdout.strip().splitlines() if l.strip()]
    return json.loads(linhas[-1]) if linhas else None


def main(argv):
    if len(argv) != 3:
        print(__doc__)
        return 2
    ids = dict(l.split("=", 1) for l in pathlib.Path(argv[1]).read_text().split() if "=" in l)
    estado = json.loads(pathlib.Path(argv[2]).read_text())
    inicio = int(ids["INICIO"])
    evs = []
    for nome, contrato, nomes in [("kleroterion", ids["KLERO"], registro.NOMES | {"Constituido"}),
                                  ("processo", ids["PROC"], NOMES_PROCESSO)]:
        es = registro.eventos(contrato, inicio, "testnet", nomes)
        txs = registro.hashes(contrato, inicio, "testnet")
        for e in es:
            e["contrato"] = nome
            e["tx"] = txs.get(e["id"])
            if e["evento"] == "Reivindicado" and e["tx"]:
                e["assento"] = registro.fonte(e["tx"], "testnet")
        evs += es
    evs.sort(key=lambda e: e["id"])
    pid = next(e["proposta"] for e in evs if e["evento"] == "VotacaoVinculada")
    proposta = ler(ids["TESSERA"], "proposta", "--proposta", pid)
    resultado = ler(ids["TESSERA"], "resultado", "--proposta", pid)
    fechado = next(e for e in evs if e["evento"] == "Fechado")
    url = f"https://api.drand.sh/{registro.CHAIN}/public/{fechado['rodada']}"
    with urllib.request.urlopen(urllib.request.Request(url, headers=registro.UA), timeout=20) as r:
        aleatoriedade = json.load(r)["randomness"]
    saida = {
        "contratos": {"kleroterion": ids["KLERO"], "processo": ids["PROC"], "tessera": ids["TESSERA"],
                      "bilhete": ids["SAC"], "verificador": ids["VERIF"], "emissor": ids["EMISSOR"]},
        "aleatoriedade": aleatoriedade,
        "eventos": evs,
        "tessera": {
            "commit": "356c22fbf90ea396203ea411b463ae14a5480d59",
            "nome": estado["proposta"],
            "id": pid,
            "perguntas": estado["perguntas"],
            "aptos": estado["aptos"],
            "proposta": proposta,
            "resultado": resultado,
            "abertura_tx": estado["abertura_tx"],
            "votos_tx": [v["tx"] for v in estado.get("votos", [])],
            "apuracao": estado.get("apuracao"),
        },
    }
    print(json.dumps(saida, indent=1, ensure_ascii=False))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
