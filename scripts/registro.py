#!/usr/bin/env python3
"""Read a kleroterion's events from the network into the verifier's record (T-040).

    python3 scripts/registro.py <contract-id> <start-ledger> [--network testnet] > registro.json

It pages through `stellar events --id <contract> --output json` (CLI 28.1.0,
which decodes event names and parameters from the contract's own spec),
keeps the contract's events in ledger order as {"evento": name, ...params},
and adds the drand randomness of the round in `Fechado`, fetched from
api.drand.sh. The result is what scripts/verificar_painel.py and
web/sorteio.mjs verifySelado read. Nothing here trusts the contract's own
state: only its events and drand.

Needs the Stellar CLI and network access. No Python dependencies.
"""

import json
import subprocess
import sys
import urllib.request

CHAIN = "52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971"
NOMES = {"Inscrito", "Impedido", "Fechado", "Selado", "Ampliado", "Reivindicado",
         "Recusado", "Finalizado", "Revelado"}


def eventos(contrato, inicio, rede):
    vistos, cursor, saida = set(), None, []
    while True:
        cmd = ["stellar", "events", "--id", contrato, "--output", "json", "--count", "200", "--network", rede]
        cmd += ["--cursor", cursor] if cursor else ["--start-ledger", str(inicio)]
        linhas = subprocess.run(cmd, capture_output=True, text=True, check=True).stdout.splitlines()
        novos = [json.loads(l) for l in linhas if l.startswith("{")]
        novos = [e for e in novos if e["id"] not in vistos]
        if not novos:
            return saida
        for e in novos:
            vistos.add(e["id"])
            if e.get("event_name") in NOMES:
                saida.append({"evento": e["event_name"], "ledger": e["ledger"], **e["params"]})
        cursor = novos[-1]["id"]


def main(argv):
    if len(argv) < 3:
        print(__doc__)
        return 2
    rede = argv[argv.index("--network") + 1] if "--network" in argv else "testnet"
    evs = eventos(argv[1], int(argv[2]), rede)
    fechado = next((e for e in evs if e["evento"] == "Fechado"), None)
    registro = {"contrato": argv[1], "eventos": evs}
    if fechado:
        url = f"https://api.drand.sh/{CHAIN}/public/{fechado['rodada']}"
        with urllib.request.urlopen(url, timeout=20) as r:
            registro["aleatoriedade"] = json.load(r)["randomness"]
    print(json.dumps(registro, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
