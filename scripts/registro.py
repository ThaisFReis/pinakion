#!/usr/bin/env python3
"""Read a kleroterion's events from the network into the verifier's record (T-040).

    python3 scripts/registro.py <contract-id> <start-ledger> [--network testnet] > registro.json

It pages through `stellar events --id <contract> --output json` (CLI 28.1.0,
which decodes event names and parameters from the contract's own spec),
keeps the contract's events in ledger order as {"evento": name, ...params},
and adds the drand randomness of the round in `Fechado`, fetched from
api.drand.sh. Since T-042 each event also carries "tx", its transaction hash
(from the RPC's own getEvents), and each Reivindicado carries "assento", the
seat account that sent the claim (the transaction's source, from Horizon) —
what the ticket page shows and scripts/exportar_tessera.py exports. The result is what scripts/verificar_painel.py and
web/sorteio.mjs verifySelado read. Nothing here trusts the contract's own
state: only its events and drand.

Needs the Stellar CLI and network access. No Python dependencies.
"""

import json
import subprocess
import sys
import urllib.request

CHAIN = "52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971"
RPC = {"testnet": "https://soroban-testnet.stellar.org"}
HORIZON = {"testnet": "https://horizon-testnet.stellar.org"}
# Some public endpoints refuse urllib's default User-Agent with a 403.
UA = {"User-Agent": "pinakion-registro/1.0"}
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
                saida.append({"evento": e["event_name"], "ledger": e["ledger"], "id": e["id"], **e["params"]})
        cursor = novos[-1]["id"]


def hashes(contrato, inicio, rede):
    """event id -> transaction hash, from the RPC's getEvents."""
    mapa, cursor = {}, None
    while True:
        pag = {"limit": 200, **({"cursor": cursor} if cursor else {})}
        params = {"filters": [{"type": "contract", "contractIds": [contrato]}], "pagination": pag}
        if not cursor:
            params["startLedger"] = inicio
        corpo = json.dumps({"jsonrpc": "2.0", "id": 1, "method": "getEvents", "params": params}).encode()
        req = urllib.request.Request(RPC[rede], corpo, {"Content-Type": "application/json", **UA})
        with urllib.request.urlopen(req, timeout=30) as r:
            res = json.load(r)["result"]
        for e in res["events"]:
            mapa[e["id"]] = e["txHash"]
        if not res["events"]:
            return mapa
        cursor = res["cursor"]


def fonte(h, rede):
    with urllib.request.urlopen(urllib.request.Request(f"{HORIZON[rede]}/transactions/{h}", headers=UA), timeout=30) as r:
        return json.load(r)["source_account"]


def main(argv):
    if len(argv) < 3:
        print(__doc__)
        return 2
    rede = argv[argv.index("--network") + 1] if "--network" in argv else "testnet"
    evs = eventos(argv[1], int(argv[2]), rede)
    txs = hashes(argv[1], int(argv[2]), rede)
    for e in evs:
        e["tx"] = txs.get(e["id"])
        if e["evento"] == "Reivindicado" and e["tx"]:
            e["assento"] = fonte(e["tx"], rede)
    fechado = next((e for e in evs if e["evento"] == "Fechado"), None)
    registro = {"contrato": argv[1], "eventos": evs}
    if fechado:
        url = f"https://api.drand.sh/{CHAIN}/public/{fechado['rodada']}"
        with urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=20) as r:
            registro["aleatoriedade"] = json.load(r)["randomness"]
    print(json.dumps(registro, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
