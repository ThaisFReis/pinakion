#!/usr/bin/env python3
"""Check a whole case from the ledger, with no dependencies (T-050).

    python3 scripts/verificar_processo.py registro.json [--autos docs/caso-demo]

`registro.json` is what scripts/registro_processo.py writes: the events of
the case's kleroterion and processo, in ledger order, the drand randomness,
and the Tessera vote as Tessera's contract stores it. Without trusting either
contract, it checks:

- the draw, with scripts/verificar_painel.py: root, seed, threshold, every
  claim, recusal and reveal, the panel; the party impeded before the lock;
- the case: it opened after the lock, for the lock's party, level and k, with
  the quorum and the votes required that the rule gives;
- the record: one complaint first, from the clerk; every entry's role fits
  what that role may file; every hash is the SHA-256 of a document in
  `--autos` (when the folder is given);
- attendance: each attendee is a seat account whose claim sat on the panel,
  once; the session opens exactly when the count reaches the quorum;
- the vote: the vote the case bound is Tessera's proposal with this id; its
  voter root is the root of the attendees, in order, under Tessera's own
  rule, recomputed here; its committee are attendees; it opened after the
  session;
- the verdict: the score is the one Tessera published; the case is upheld
  exactly when "yes" reaches the votes required.

What it does not check: that Tessera's published score is the correct tally
of its ballots. That is Tessera's own verifier (`tessera verificar`), and its
output is kept beside the record.

Exit 0 when everything matches, 1 when anything does not.
"""

import base64
import hashlib
import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from verificar_painel import verificar as verificar_sorteio  # noqa: E402


# The phase keys verificar() returns, and how the report prints them.
FASES = {"sorteio": "draw", "abertura": "opening", "quorum": "quorum", "autos": "case record",
         "votacao": "vote", "veredito": "verdict", "revelacao": "reveal"}


def xdr_do_endereco(s):
    """ScVal of an address, as Address::to_xdr writes it (Tessera's leaf)."""
    dado = base64.b32decode(s)
    versao, chave = dado[0], dado[1:33]
    v = (18).to_bytes(4, "big")
    if versao == 6 << 3:  # G…: account, ed25519
        v += (0).to_bytes(4, "big") + (0).to_bytes(4, "big")
    else:  # C…: contract
        v += (1).to_bytes(4, "big")
    return v + chave


def raiz_dos_aptos(aptos):
    """Tessera's voter root (tessera/core/src/merkle.rs at 356c22f), weight 1."""
    sha = lambda b: hashlib.sha256(b).digest()
    nivel = [sha(b"\x00" + xdr_do_endereco(a) + (1).to_bytes(4, "big")) for a in aptos]
    vazio = sha(b"\x02" + b"TESSERA-V1-MERKLE-VAZIO")
    if not nivel:
        return vazio.hex()
    alvo = 1
    while alvo < len(nivel):
        alvo *= 2
    nivel += [vazio] * (alvo - len(nivel))
    while len(nivel) > 1:
        nivel = [sha(b"\x01" + nivel[i] + nivel[i + 1]) for i in range(0, len(nivel), 2)]
    return nivel[0].hex()


def hashes_dos_autos(pasta):
    if not pasta:
        return None
    return {hashlib.sha256(p.read_bytes()).hexdigest(): p.name
            for p in sorted(pathlib.Path(pasta).glob("*.md"))}


def verificar(registro, autos=None):
    """Return (fases, conferencias, estados, presentes, documentos)."""
    evs = registro["eventos"]
    klero = [e for e in evs if e.get("contrato") == "kleroterion"]
    proc = [e for e in evs if e.get("contrato") == "processo"]
    estados, painel, completo, _, conferencias = verificar_sorteio(
        {"eventos": klero, "aleatoriedade": registro["aleatoriedade"]})
    conferencias = [("draw: " + n, ok) for n, ok in conferencias]

    def confere(nome, ok):
        conferencias.append((nome, bool(ok)))

    def um(nome, lista=proc):
        return next((e for e in lista if e["evento"] == nome), None)

    fases = {"sorteio": completo and all(ok for _, ok in conferencias)}
    inst, aberto = um("Instituido"), um("CasoAberto")
    fechado = um("Fechado", klero)
    confere("there was a rule (Instituido)", inst)
    confere("the case opened", aberto)
    if not (inst and aberto and fechado):
        return fases, conferencias, estados, [], []
    contratos = registro.get("contratos", {})
    if contratos:
        confere("the rule points to this kleroterion", inst["kleroterion"] == contratos["kleroterion"])
        confere("the rule points to this Tessera", inst["tessera"] == contratos["tessera"])
    confere("the case opened after the lock", evs.index(aberto) > evs.index(fechado))
    confere("the party is the one in the lock", aberto["acusado"] == fechado["acusado"])
    confere("the level is the one in the lock", aberto["gravidade"] == fechado["gravidade"])
    k = fechado["k"]
    confere("k is the one in the lock", aberto["k"] == k)
    confere("the quorum is a majority of the panel, never below the vote floor",
            aberto["quorum"] == max(k // 2 + 1, inst["votos_minimos"]))
    exigidos = -(-2 * k // 3) if fechado["gravidade"] >= inst["dois_tercos_desde"] else k // 2 + 1
    confere("the required votes follow the level rule", aberto["exigidos"] == exigidos)
    fases["abertura"] = True

    # Seats that sat: the seat account of each claim whose y is on the panel.
    sentados = {e["assento"] for e in klero if e["evento"] == "Reivindicado" and int(e["y"]) in painel}
    presentes, sessao = [], None
    for e in proc:
        if e["evento"] == "Compareceu":
            confere(f"attendance of {e['assento'][:8]}: a seat that sat", e["assento"] in sentados)
            confere(f"attendance of {e['assento'][:8]}: once only", e["assento"] not in presentes)
            presentes.append(e["assento"])
            confere(f"attendance of {e['assento'][:8]}: the count matches", e["presentes"] == len(presentes))
        elif e["evento"] == "SessaoInstalada":
            sessao = e
            confere("the session opens exactly at the quorum",
                    len(presentes) == aberto["quorum"] == e["quorum"] == e["presentes"])
    confere("the party never attended as a juror", aberto["acusado"] not in presentes)
    fases["quorum"] = sessao is not None

    docs = hashes_dos_autos(autos)
    regs = [e for e in proc if e["evento"] == "Registrado"]
    confere("the first filing is the complaint, from the clerk",
            regs and regs[0]["tipo"] == 0 and regs[0]["papel"] == 0)
    escrivao = regs[0]["autor"] if regs else None
    vistos = []
    for i, r in enumerate(regs):
        confere(f"filing {i}: in sequence", r["seq"] == i)
        antes = [e["assento"] for e in proc[:proc.index(r)] if e["evento"] == "Compareceu"]
        papel_ok = {0: r["autor"] == escrivao and r["tipo"] in (0, 1, 3),
                    1: r["autor"] == aberto["acusado"] and r["tipo"] == 2,
                    2: r["autor"] in antes and r["tipo"] == 3}.get(r["papel"], False)
        confere(f"filing {i}: the role may file this type", papel_ok)
        if docs is not None:
            confere(f"filing {i}: the document matches the hash", r["hash"] in docs)
            vistos.append(docs.get(r["hash"]))
    fases["autos"] = bool(regs)

    vinc, decid = um("VotacaoVinculada"), um("Decidido")
    t = registro.get("tessera") or {}
    fases["votacao"] = vinc is not None
    if vinc:
        p = t.get("proposta") or {}
        raiz = raiz_dos_aptos(presentes)
        confere("the bound vote is the recorded Tessera vote", vinc["proposta"] == t.get("id"))
        if t.get("nome"):  # Tessera's CLI derives the id from a name; its dapp draws it at random
            confere("the identifier comes from the vote name in Tessera",
                    hashlib.sha256(b"TESSERA-V1-PROPOSTA" + t["nome"].encode()).hexdigest() == vinc["proposta"])
        confere("the voter root is the root of the attendees, under the Tessera rule", vinc["raiz_aptos"] == raiz)
        confere("Tessera stores that same root", p.get("raiz_aptos") == raiz)
        confere("the Tessera voter list is the attendees, in order", t.get("aptos") == presentes)
        confere("the Tessera committee is made of attendees", all(m in presentes for m in p.get("mesa", [])))
        confere("the ballot is one confidential question with two options",
                p.get("perguntas") == [{"opcoes": 2, "confidencial": True}])
        if sessao and "ledger" in sessao:
            confere("the vote opened after the session", p.get("abre_em", 0) >= sessao["ledger"])
    fases["veredito"] = decid is not None
    if decid:
        res = t.get("resultado") or []
        confere("the score is the one Tessera published", [decid["sim"], decid["nao"]] == res)
        confere("the required votes are the ones fixed at opening", decid["exigidos"] == aberto["exigidos"])
        confere("upheld if and only if yes reaches the required votes",
                decid["procedente"] == (decid["sim"] >= decid["exigidos"]))
        confere("the verdict came after the vote", proc.index(decid) > proc.index(vinc))
    fases["revelacao"] = any(e["evento"] == "Revelado" for e in klero)
    return fases, conferencias, estados, presentes, vistos


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    autos = argv[argv.index("--autos") + 1] if "--autos" in argv else "docs/caso-demo"
    r = verificar(json.loads(pathlib.Path(argv[1]).read_text()), autos)
    fases, conferencias = r[0], r[1]
    for f, ok in fases.items():
        print(f"  {'ok ' if ok else '-- '} {FASES.get(f, f)}")
    falhas = [n for n, ok in conferencias if not ok]
    for n in falhas:
        print(f"  DOES NOT MATCH: {n}")
    print(f"\n{'MATCHES' if not falhas else 'DOES NOT MATCH'} — {len(conferencias)} checks, {len(falhas)} failure(s).")
    return 0 if not falhas else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
