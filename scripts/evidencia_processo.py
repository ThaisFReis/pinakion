#!/usr/bin/env python3
"""Print docs/evidencia-processo.md from the recorded case (T-049).

    python3 scripts/evidencia_processo.py docs/evidencia-processo-registro.json > docs/evidencia-processo.md

Every figure and hash in that document comes from the record, through this
script, and G6 checks that the document is still what it prints. No
dependencies.
"""

import json
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import verificar_processo  # noqa: E402

EXPLORER = "https://stellar.expert/explorer/testnet"
TIPO = ["denúncia", "prova", "defesa", "ata"]
PAPEL = ["escrivão", "pessoa julgada", "jurado (conta de assento)"]


def main(argv):
    if len(argv) != 2:
        print(__doc__)
        return 2
    reg = json.loads(pathlib.Path(argv[1]).read_text())
    fases, conf, estados, presentes, docs = verificar_processo.verificar(reg, "docs/caso-demo")
    falhas = [n for n, ok in conf if not ok]
    ev = reg["eventos"]
    k = lambda n: [e for e in ev if e["contrato"] == "kleroterion" and e["evento"] == n]
    p = lambda n: [e for e in ev if e["contrato"] == "processo" and e["evento"] == n]
    fechado, final = k("Fechado")[0], (k("Finalizado") or [None])[0]
    aberto, decid = p("CasoAberto")[0], (p("Decidido") or [None])[0]
    inst, vinc = p("Instituido")[0], (p("VotacaoVinculada") or [None])[0]
    c = reg["contratos"]
    t = reg["tessera"]
    tx = lambda h: f"[`{h[:12]}…`]({EXPLORER}/tx/{h})" if h else "—"
    conta = lambda a: f"[`{a[:6]}…{a[-4:]}`]({EXPLORER}/{'contract' if a.startswith('C') else 'account'}/{a})"
    silencio = sum(1 for e in estados.values() if e == "nao revelou")
    out = []
    w = out.append
    w("# Evidência: um caso inteiro na testnet")
    w("")
    w("**Caso fictício, transações reais.** Gerado por `python3 scripts/evidencia_processo.py docs/evidencia-processo-registro.json`; o G6 confere que este arquivo é o que o comando imprime. Sequência: `scripts/sequencia_processo.sh`.")
    w("")
    w("> A votação roda no **Tessera**, o módulo de voto secreto anterior da mesma autora: o contrato no commit `356c22f`, sem mudança, e a CLI no mesmo commit com uma correção de uma linha (`scripts/tessera-cli.patch`). O que é deste projeto é a ligação: o contrato `processo` só aceitou a votação porque a lista de votantes do Tessera é exatamente a dos jurados presentes.")
    w("")
    w("## O que a conferência diz")
    w("")
    w(f"`python3 scripts/verificar_processo.py docs/evidencia-processo-registro.json`: **{'CONFERE' if not falhas else 'NÃO CONFERE'}, {len(conf)} conferências, {len(falhas)} falha(s)**, sem confiar em nenhum dos contratos. A página [`web/processo.html`](../web/processo.html) faz as mesmas conferências no navegador.")
    w("")
    w("## Os contratos")
    w("")
    w("| Papel | Endereço |")
    w("|---|---|")
    for nome, papel in [("kleroterion", "o rol e o sorteio selado"), ("processo", "o caso"), ("tessera", "a votação (Tessera, 356c22f)"),
                        ("verificador", "o verificador Groth16"), ("bilhete", "o SAC do bilhete PINAKION")]:
        w(f"| {papel} | {conta(c[nome])} |")
    w(f"| o emissor, com peso 0 | {conta(c['emissor'])} |")
    w("")
    w("## O caso, em números")
    w("")
    w("| | |")
    w("|---|---|")
    w(f"| Inscritos no rol | {len(k('Inscrito'))} |")
    w(f"| Nível de gravidade · assentos | {fechado['gravidade']} · {fechado['k']} |")
    w(f"| A pessoa julgada estava no rol e foi impedida no fechamento | {'sim' if any(e['jurado'] == fechado['acusado'] for e in k('Impedido')) else 'não'} |")
    w(f"| Rodada drand | {fechado['rodada']} |")
    w(f"| Reivindicações (provas em conhecimento zero) | {len(k('Reivindicado'))} |")
    w(f"| Painel | {len(final['painel']) if final else 0}, {'completo' if final and final['completo'] else 'incompleto'} |")
    w(f"| Quórum · presentes | {aberto['quorum']} · {len(presentes)} |")
    w(f"| Documentos nos autos | {len(p('Registrado'))} |")
    if decid:
        w(f"| Placar do Tessera (sim · não) · exigidos | {decid['sim']} · {decid['nao']} · {decid['exigidos']} |")
        w(f"| Veredito | {'procedente' if decid['procedente'] else 'improcedente'} |")
    w(f"| Revelaram · em silêncio | {len(k('Revelado'))} · {silencio} |")
    w("")
    w("## Os autos")
    w("")
    w("| # | Tipo | Por | Documento | Transação |")
    w("|---|---|---|---|---|")
    for i, r in enumerate(p("Registrado")):
        doc = docs[i] if i < len(docs) and docs[i] else "—"
        w(f"| {i} | {TIPO[r['tipo']]} | {PAPEL[r['papel']]} | [`{doc}`](caso-demo/{doc}) | {tx(r.get('tx'))} |")
    w("")
    w("## Cada passo, com a transação")
    w("")
    w("| Passo | Transação |")
    w("|---|---|")
    passos = [("fechamento do rol (e o impedimento da parte)", fechado.get("tx")),
              ("abertura do caso", aberto.get("tx")),
              ("selo (assinatura drand verificada na cadeia)", k("Selado")[0].get("tx")),
              ("painel", final.get("tx") if final else None),
              ("sessão instalada", (p("SessaoInstalada") or [{}])[0].get("tx")),
              ("abertura da votação no Tessera", t.get("abertura_tx")),
              ("votação vinculada ao caso", vinc.get("tx") if vinc else None),
              ("apuração no Tessera", (t.get("apuracao") or {}).get("tx")),
              ("veredito", decid.get("tx") if decid else None)]
    for nome, h in passos:
        w(f"| {nome} | {tx(h)} |")
    w("")
    w("## O que esta rodada não mostra")
    w("")
    w("- **Uma máquina fez o papel de todos.** Todas as contas vieram do friendbot; o financiamento da conta de assento sem rastro precisa de um relayer, que não existe.")
    espera = k("Constituido")[0]["espera_minima"]
    w(f"- **Os prazos são de demonstração.** A espera mínima da revelação foi de {espera} ledgers, "
      f"cerca de {espera * 5 // 60} minutos a 5 s por ledger (estimativa); o padrão aprovado é 30 dias.")
    w("- **Os autos são públicos aqui.** Um caso real precisa de autos sigilosos com acesso justificado (desenhados, não construídos).")
    w("- **A correção do placar é do Tessera.** Este projeto confere a ligação e a regra; a apuração é conferida por `tessera verificar`, cuja saída está em `docs/evidencia-processo-tessera.txt`.")
    w("- **É só testnet, e nada foi auditado.**")
    print("\n".join(out))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
