#!/usr/bin/env python3
"""Export a sealed panel's seat accounts as Tessera's voter list (T-042).

    python3 scripts/exportar_tessera.py registro.json > aptos.txt
    tessera abrir --proposta … --aptos aptos.txt …

Tessera (`ThaisFReis/tessera`, cli/src/comandos.rs at 39ec925) reads
`--aptos` as a comma list or a file with one entry per line, `#` starting a
comment, each entry a 56-character G-address or a local identity name. The
order of the list is its Merkle root. This writes one line per seat, in panel
order (ascending y), with the seat account that claimed it, the address the
claim transaction came from, as scripts/registro.py records it. The juror's
main account never appears: the voter list is the seat accounts, so a vote
in Tessera is tied to a seat, not to a name.

No dependencies.
"""

import json
import pathlib
import sys


def main(argv):
    if len(argv) != 2:
        print(__doc__)
        return 2
    reg = json.loads(pathlib.Path(argv[1]).read_text())
    ev = reg["eventos"]
    final = next((e for e in ev if e["evento"] == "Finalizado"), None)
    if not final:
        print("no Finalizado event: the panel is not out yet", file=sys.stderr)
        return 1
    assento = {e["y"]: e.get("assento") for e in ev if e["evento"] == "Reivindicado"}
    faltam = [y for y in final["painel"] if not assento.get(y)]
    if faltam:
        print("the record has no seat account for some seats; rebuild it with scripts/registro.py", file=sys.stderr)
        return 1
    fechado = next(e for e in ev if e["evento"] == "Fechado")
    print(f"# Pinakion panel, exported for Tessera --aptos (scripts/exportar_tessera.py)")
    print(f"# kleroterion {reg.get('contrato', '?')}, drand round {fechado['rodada']}, root {fechado['raiz']}")
    print(f"# {len(final['painel'])} seat(s), {'complete' if final['completo'] else 'INCOMPLETE'}; one seat account per line, in panel order")
    for y in final["painel"]:
        print(assento[y])
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
