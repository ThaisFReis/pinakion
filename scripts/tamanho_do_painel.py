#!/usr/bin/env python3
"""How the panel size changes the risk of a captured panel (T-044).

    python3 scripts/tamanho_do_painel.py

Prints the tables of docs/desenho-gravidade-afastamento-revelacao.md, which
G6 compares with this output so the published numbers cannot drift.

The model, and where it stops being valid:

- The sealed draw seats the k smallest y = Poseidon(seed, s). The model takes
  that as a uniform k-subset of the N eligible jurors, drawn without
  replacement, so the count of a bloc's members on the panel is
  hypergeometric. Recusals, absences and the one widening of the threshold in
  `finalize` are not modelled.
- A bloc is `a` jurors of the N who vote together and know each other: the
  worst case, as in docs/limite-inferior-do-rol.md §7. Partial sympathy is not
  modelled.
- A bloc *imposes* a ruling when it holds at least the m votes the ruling
  needs, and *vetoes* one when it holds more than k - m. With a simple majority
  on an odd k the two thresholds coincide.

Every figure is exact arithmetic on that model (math.comb), not a simulation:
an estimate of the real risk, a calculation of the model. No dependencies.
"""

import math
import sys


def ao_menos(n, a, k, m):
    """P(at least m of a k-subset of n fall in a fixed set of a)."""
    if m <= 0:
        return 1.0
    total = math.comb(n, k)
    return sum(math.comb(a, j) * math.comb(n - a, k - j) for j in range(m, min(a, k) + 1)) / total


def maioria(k):
    return k // 2 + 1


def dois_tercos(k):
    return -(-2 * k // 3)


def pct(p):
    return f"{100 * p:.1f} %"


def tabela(cabecalho, linhas):
    out = ["| " + " | ".join(cabecalho) + " |", "|" + "---|" * len(cabecalho)]
    out += ["| " + " | ".join(l) + " |" for l in linhas]
    return "\n".join(out)


def tabelas():
    t = {}

    # 1. Re-check of docs/limite-inferior-do-rol.md §4: roster 100, simple majority.
    ks = [3, 5, 7, 9, 12]
    t["reconferencia"] = tabela(
        ["Bloco"] + [f"k = {k}" for k in ks],
        [[f"{b} %"] + [pct(ao_menos(100, b, k, maioria(k))) for k in ks] for b in (20, 30, 40, 45)],
    )

    # 2. Simple majority, bloc of 30 % of the roster, by roster and panel size.
    ks = [3, 5, 7, 9, 11, 15]
    linhas = []
    for n in (20, 40, 100, 200):
        a = round(0.3 * n)
        linhas.append([f"N = {n} (bloco {a})"] + [pct(ao_menos(n, a, k, maioria(k))) if k <= n else "—" for k in ks])
    t["captura"] = tabela(["Rol"] + [f"k = {k}" for k in ks], linhas)

    # 3. The smallest odd k that keeps the bloc's chance of a majority under a target, N = 100.
    linhas = []
    for b in (10, 20, 30):
        linha = [f"{b} %"]
        for alvo in (0.10, 0.05, 0.01):
            k = next((k for k in range(1, 101, 2) if ao_menos(100, b, k, maioria(k)) <= alvo), None)
            linha.append(str(k) if k else "—")
        linhas.append(linha)
    t["k_minimo"] = tabela(["Bloco (N = 100)", "≤ 10 %", "≤ 5 %", "≤ 1 %"], linhas)

    # 4. Two-thirds against simple majority, bloc of 30 of 100.
    linhas = []
    for k in (3, 5, 7, 9, 11, 15):
        m = dois_tercos(k)
        linhas.append([
            str(k),
            pct(ao_menos(100, 30, k, maioria(k))),
            f"{m} de {k}",
            pct(ao_menos(100, 30, k, m)),
            pct(ao_menos(100, 30, k, k - m + 1)),
        ])
    t["dois_tercos"] = tabela(
        ["k", "Maioria simples: impor ou vetar", "2/3 exige", "2/3: impor", "2/3: vetar"], linhas
    )

    # 5. The seal is k/N: the chance that a guess about one enrolled juror is right.
    ks = [3, 5, 7, 9]
    t["selo"] = tabela(
        ["Rol"] + [f"k = {k}" for k in ks],
        [[f"N = {n}"] + [pct(k / n) for k in ks] for n in (19, 40, 100, 200)],
    )

    # 6. The proposed default table: k, roster floor and vote rule per level.
    linhas = []
    for nivel, exemplos, k, piso, regra in NIVEIS:
        m = dois_tercos(k) if regra == "2/3" else maioria(k)
        nome_regra = f"2/3 ({m} de {k})" if regra == "2/3" else "maioria simples"
        impor = ao_menos(100, 30, k, m)
        vetar = ao_menos(100, 30, k, k - m + 1)
        linhas.append([str(nivel), exemplos, str(k), str(piso), nome_regra, f"{pct(impor)} / {pct(vetar)}", pct(k / piso)])
    t["proposta"] = tabela(
        ["Nível", "Exemplos", "`k`", "Rol mínimo", "Regra de votação recomendada",
         "Bloco de 30 % impõe / veta (N = 100)", "`k/N` no rol mínimo"],
        linhas,
    )
    return t


# The proposed defaults (T-044): Contributor Covenant 2.1's four levels; the
# roster floors follow docs/limite-inferior-do-rol.md §6. A choice, not a result.
NIVEIS = [
    (1, "correção", 3, 16, "maioria"),
    (2, "advertência", 5, 40, "maioria"),
    (3, "banimento temporário, perda de cargo ou de commit", 7, 40, "maioria"),
    (4, "banimento permanente, expulsão", 9, 100, "2/3"),
]


def main():
    for nome, texto in tabelas().items():
        print(f"<!-- tamanho_do_painel: {nome} -->")
        print(texto)
        print()
    return 0


if __name__ == "__main__":
    sys.exit(main())
