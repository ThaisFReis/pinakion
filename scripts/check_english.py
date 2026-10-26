#!/usr/bin/env python3
"""Gate G10 (T-058): the repository is written in English.

`CLAUDE.md` says git is written in English (DEC-007). DEC-040 extends that to
the whole tree: file names, identifiers, comments, UI copy and the output of
the checkers. This gate is the test for that decision, so that Portuguese
cannot come back without a gate going red.

    python3 scripts/check_english.py [--list]

It reads the tracked files, so it needs git and nothing else. Exit code 1 and
a report of `path:line` on the first Portuguese it finds.

Three signals, each a line in the report:

  accent    a Portuguese letter (á à â ã é ê í ó ô õ ú ç ñ) outside the
            allowed list below
  word      a word from WORDS, matched whole and case-insensitively
  name      a path whose own name is Portuguese

What is deliberately not checked, and why:

  third party   `web/vivo/tessera/` is Tessera's client, vendored at commit
                356c22f and used unchanged (DEC-019, DEC-036); `web/vivo/icons/`
                is Phosphor; `circuits/setup/` is build output. Translating
                any of them would mean forking it.
  the Tessera   Tessera's own contract is deployed unchanged and its interface
  boundary      is Portuguese. Soroban matches struct field names on the wire,
                so the mirror of its `Proposta` and the `Tessera` trait keep
                its spelling or the cross-contract call stops working. Those
                lines are fenced:

                    // check-english: foreign-interface begin (Tessera 356c22f)
                    ...
                    // check-english: foreign-interface end

                The fence is scoped to the lines between the markers, so our
                own `round`, `threshold` and `bench` are still checked
                everywhere else. An unclosed fence is itself a failure.
  history       commit messages and the names in `docs/SOURCES.md`'s *Copied
                code* rows record what was true when they were written.
  proper nouns  Pinakion, Kleroterion, Tessera, Cautela, Pignus. The first two
                are Greek, the last two name the withdrawn generation (DEC-001)
                and appear only as history.
"""

import re
import subprocess
import sys
import unicodedata

# Paths whose contents are not ours to translate.
SKIP_PREFIXES = (
    "web/vivo/tessera/",
    "web/vivo/icons/",
    "circuits/setup/",
    "contracts/poseidon/LICENSE",
    "circuits/LICENSE",
    "LICENSE",
)

# Binary and generated files the signals cannot read usefully.
SKIP_SUFFIXES = (".bin", ".wasm", ".zkey", ".r1cs", ".svg", ".png", ".lock", ".patch")

# Lines between these markers are Tessera's interface, not ours. The fence is
# per-line, not per-word, so nothing is excused outside it.
FENCE_BEGIN = "check-english: foreign-interface begin"
FENCE_END = "check-english: foreign-interface end"

# Names Tessera owns that appear outside a fence, in a command line or a flag.
# This list only shrinks: a name belongs here when Tessera's deployed contract
# or its CLI at 356c22f requires that exact spelling.
ALLOWED_TESSERA = {
    "aptos",  # its voter-list flag, `--aptos`
    "verificar",  # its own subcommand, `tessera verificar`
    "votacao",  # `tessera votacao`, and its contract's name for a vote
    "proposta",  # `tessera proposta`
}

# Proper nouns and loanwords that stay as they are.
ALLOWED_NAMES = {
    "pinakion",
    "kleroterion",
    "tessera",
    "cautela",
    "pignus",
    "agora",
    "stellar",
    "soroban",
    "lumen",
    "drand",
    "poseidon",  # the hash function, not a Portuguese word
    "groth",
    "circom",
    "circomlib",
}

# Portuguese words, matched whole. Short words that are also English or common
# abbreviations (a, e, o, no, do, da, os, as, se, ir) are deliberately absent:
# they would drown the report. The accent signal catches most of what they miss.
WORDS = """
acusado adiado adiar afastado afastamento ampliado anterior apuracao apurar
arvore assento assinatura ata autos
bilhete bilhetes
caso chave compareceu comparecer compromisso concluido conferencia conferencias
conferido conferir confere constituido contrato contratos
decidido decidir decisao denuncia depositar desenho divergencia divergencias
emissor encerrada entrada erro escolhas escrivao esperando estado etapa
exigidos exportar
falha falhas fase fechado fechamento folha fora
gerar gravidade
impedido improcedente inscricao inscrito instituido
janela jurado jurados juiz
leitura limite limites lista
minimo
navegador nivel niveis nenhum numero numeros
pagina painel papel pergunta placar prazo preencher presentes procedente
processo pronto prova provar
raiz recusa recusado recusou registrado registro registros regra regras
reivindicado revelacao revelar revelou rodada rol
segredo segredos selado selo semente sequencia sessao sorteio
tamanho tipo
urna
veredito verificar vetor vetores vinculada vincular voto votos
""".split()

# File and directory names that are Portuguese.
NAME_WORDS = set(WORDS) | {"vivo", "ouro", "custos", "evidencia"}

ACCENTED = "áàâãéêíóôõúüçñÁÀÂÃÉÊÍÓÔÕÚÜÇÑ"

WORD_RE = re.compile(r"[A-Za-z%s]+" % ACCENTED)
WORDS_SET = set(WORDS)


def tracked_files():
    out = subprocess.run(
        ["git", "ls-files", "-co", "--exclude-standard"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    for path in out.splitlines():
        if path.startswith(SKIP_PREFIXES) or path.endswith(SKIP_SUFFIXES):
            continue
        yield path


def split_identifier(token):
    """snake_case, camelCase and kebab-case all come apart into words."""
    for part in re.split(r"[_\-]+", token):
        for piece in re.findall(r"[A-Z]?[a-z%s]+|[A-Z]+(?![a-z])" % ACCENTED, part):
            yield piece.lower()


def unaccented(word):
    return "".join(
        c for c in unicodedata.normalize("NFD", word) if not unicodedata.combining(c)
    )


def check_name(path):
    """A Portuguese word in the path's own name."""
    hits = set()
    for segment in path.split("/"):
        for word in split_identifier(segment):
            if word in ALLOWED_NAMES or word in ALLOWED_TESSERA:
                continue
            if word in NAME_WORDS or unaccented(word) in NAME_WORDS:
                hits.add(word)
    return sorted(hits)


def check_text(path):
    """Accented letters and Portuguese words, line by line."""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except (UnicodeDecodeError, FileNotFoundError, IsADirectoryError):
        return []
    found = []
    fenced = False
    for n, line in enumerate(lines, 1):
        if FENCE_BEGIN in line:
            if fenced:
                found.append((n, "fence", "nested", "a fence is already open"))
            fenced = True
            continue
        if FENCE_END in line:
            if not fenced:
                found.append((n, "fence", "unopened", "fence closed but not open"))
            fenced = False
            continue
        if fenced:
            continue
        accents = sorted({c for c in line if c in ACCENTED})
        if accents:
            found.append((n, "accent", "".join(accents), line.strip()[:90]))
        hits = set()
        for token in WORD_RE.findall(line):
            for word in split_identifier(token):
                if word in ALLOWED_NAMES or word in ALLOWED_TESSERA:
                    continue
                if word in WORDS_SET or unaccented(word) in WORDS_SET:
                    hits.add(word)
        if hits:
            found.append((n, "word", " ".join(sorted(hits)), line.strip()[:90]))
    if fenced:
        found.append((len(lines), "fence", "unclosed", "fence never closed"))
    return found


def main():
    verbose = "--list" in sys.argv
    files = 0
    bad_names = []
    bad_lines = []
    for path in sorted(tracked_files()):
        files += 1
        hits = check_name(path)
        if hits:
            bad_names.append((path, hits))
        for n, kind, what, line in check_text(path):
            bad_lines.append((path, n, kind, what, line))

    for path, hits in bad_names:
        print("name    %s  (%s)" % (path, ", ".join(hits)))
    shown = bad_lines if verbose else bad_lines[:40]
    for path, n, kind, what, line in shown:
        print("%-7s %s:%d  [%s]  %s" % (kind, path, n, what, line))
    if len(shown) < len(bad_lines):
        print("... and %d more lines (--list for all)" % (len(bad_lines) - len(shown)))

    per_file = len({p for p, *_ in bad_lines} | {p for p, _ in bad_names})
    print()
    print("%d files read." % files)
    if not bad_names and not bad_lines:
        print("ENGLISH — no Portuguese found.")
        return 0
    print(
        "PORTUGUESE — %d path(s) and %d line(s), in %d file(s)."
        % (len(bad_names), len(bad_lines), per_file)
    )
    return 1


if __name__ == "__main__":
    sys.exit(main())
