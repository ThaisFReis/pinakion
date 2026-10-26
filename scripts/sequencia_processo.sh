#!/usr/bin/env bash
# The whole case, on testnet, end to end (T-049): from the roster to the
# verdict and the reveal. The evidence behind docs/evidencia-processo.md and
# the case page web/processo.html.
#
#   scripts/sequencia_processo.sh [prefix]
#
# Needs: the Stellar CLI (28.1.0), `stellar contract build` done, `npm ci`
# done, network access, git, and the Tessera repository at
# $TESSERA_REPO (default ../Tessera). It generates and funds its own
# identities through friendbot, named <prefix>-…; nothing of yours is touched.
#
# The vote is Tessera's — the author's earlier secret-ballot module, used as
# it is: its contract at commit 356c22f, deployed once on testnet
# ($TESSERA_ID), and its CLI at the same commit with one client-side fix
# (scripts/tessera-cli.patch: the CLI still sent `fim_tempo`, which the
# contract renamed to `rodada_abertura`). Nothing in Tessera's contract is
# changed.
#
#   1–5   issuer, SAC, verifier, kleroterion (the approved severity table),
#         processo; SAC admin to kleroterion; issuer weight 0
#   6     enroll 40 jurors with c = Poseidon(s)
#   7     lock at level 3 (k = 7) against j0: j0 is impeded in the same call
#   8     the case opens with the complaint; the evidence and the defence
#   9     seal with the live drand round; claims from fresh seat accounts
#   10    finalize: the panel
#   11    six of the seven seated jurors attend; the quorum (5) installs
#         the session; one juror files minutes from a seat account
#   12    the vote opens on Tessera, the attendees as its voter list; the
#         case binds it (the contract recomputes Tessera's voter root)
#   13    the attendees vote; Tessera tallies and verifies
#   14    the case decides from Tessera's result; the clerk files the minutes
#   15    after reveal_after: the jurors reveal
#   16    read everything back and check it (scripts/verificar_processo.py)
set -euo pipefail
cd "$(dirname "$0")/.."
P=${1:-caso1}; N="--network testnet"
JURADOS=${JURADOS:-40}; GRAVIDADE=3; K=7; JANELA=${JANELA:-180}
NIVEIS='[{"k":3,"rol_minimo":16},{"k":5,"rol_minimo":40},{"k":7,"rol_minimo":40},{"k":9,"rol_minimo":100}]'
# Demo values, not recommendations: the approved reveal floor is 30 days.
AFASTAMENTO=${AFASTAMENTO:-3000}; ESPERA_MINIMA=${ESPERA_MINIMA:-300}; ATE_REVELAR=${ATE_REVELAR:-560}
PRAZO_QUORUM=${PRAZO_QUORUM:-200}; VOTOS_MINIMOS=5; DOIS_TERCOS_DESDE=4; PRAZO_VOTO=${PRAZO_VOTO:-45}
TESSERA_REPO=${TESSERA_REPO:-../Tessera}; TESSERA_COMMIT=356c22fbf90ea396203ea411b463ae14a5480d59
TESSERA_ID=${TESSERA_ID:-CDWBWAHUZ3MC3BJXJIW5O74MNUXH76VCGBIVFQU3XJJFGML3VKFIXQT7}
D=build/$P; mkdir -p "$D"; HASHES=$D/hashes.tsv; : > "$HASHES"; SEGREDOS=$D/segredos.json; echo '{}' > "$SEGREDOS"
AUTOS=docs/caso-demo
export STELLAR_NO_CACHE=true
log() { printf '\n== %s\n' "$*"; }
ledger() { curl -s https://horizon-testnet.stellar.org/ | python3 -c "import sys,json;print(json.load(sys.stdin)['history_latest_ledger'])"; }
esperar_ledger() { while [ "$(ledger)" -le "$1" ]; do sleep 5; done; }
conta() { # generate and fund an identity, retrying friendbot; print its address
  local t
  for t in 1 2 3 4 5; do
    stellar keys address "$1" >/dev/null 2>&1 || stellar keys generate "$1" $N >/dev/null 2>&1 || true
    stellar keys fund "$1" $N >/dev/null 2>&1 && { stellar keys address "$1" 2>/dev/null; return 0; }
    sleep $((t * 3))
  done
  echo "could not fund $1" >&2; return 1
}
# invoke <op> <contract> <source> -- args… : sends, records the hash, prints the return value
invocar() {
  local op=$1 id=$2 src=$3; shift 4
  local out t
  for t in 1 2 3; do
    out=$(stellar contract invoke --id "$id" --source "$src" $N --send=yes -- "$@" 2>&1) && break
    case "$out" in *"Error(Contract"*) echo "$out" >&2; return 1 ;; esac
    [ "$t" = 3 ] && { echo "$out" >&2; return 1; }
    sleep 5
  done
  local h; h=$(printf '%s' "$out" | grep -o 'tx/[0-9a-f]\{64\}' | head -1 | cut -d/ -f2 || true)
  [ -n "$h" ] && printf '%s\t%s\n' "$op" "$h" >> "$HASHES"
  printf '%s' "$out" | grep -v '^ℹ️\|^⚠️\|^🌎\|^📅\|^ *Run `stellar\|^ *$\|Signing\|Submitting\|stellar.expert\|Simulat' | tail -1 || true
}
ler() { stellar contract invoke --id "$1" --source "$P-escrivao" $N --send=no -- "${@:2}" 2>/dev/null | tail -1; }
classic() { # op, source identity, then the stellar tx new … arguments
  local op=$1 src=$2; shift 2
  stellar tx new "$@" --source "$src" $N >/dev/null 2>&1
  sleep 6
  local h; h=$(curl -s "https://horizon-testnet.stellar.org/accounts/$(stellar keys address "$src" 2>/dev/null)/transactions?order=desc&limit=1" \
    | python3 -c "import sys,json;print(json.load(sys.stdin)['_embedded']['records'][0]['hash'])")
  printf '%s\t%s\n' "$op" "$h" >> "$HASHES"; echo "tx $h"
}
sha() { sha256sum "$1" | cut -d' ' -f1; }

log "0. Tessera's CLI at $TESSERA_COMMIT, with scripts/tessera-cli.patch"
TW=$HOME/.cache/pinakion/tessera
if [ ! -x "$TW/cli/target/release/tessera" ]; then
  rm -rf "$TW"; mkdir -p "$(dirname "$TW")"
  git -C "$TESSERA_REPO" worktree add --detach "$TW" "$TESSERA_COMMIT" >/dev/null
  git -C "$TW" apply "$PWD/scripts/tessera-cli.patch"
  (cd "$TW/cli" && cargo build --release --locked >/dev/null 2>&1)
fi
TESSERA_BIN=$TW/cli/target/release/tessera; export TESSERA_CONTRATO=$TESSERA_ID
echo "tessera cli $TESSERA_BIN · contract $TESSERA_ID"

log "1. identities and the issuer"
EMISSOR=$(conta "$P-emissor"); ESCRIVAO=$(conta "$P-escrivao")
echo "issuer $EMISSOR"; echo "clerk  $ESCRIVAO"
classic flags "$P-emissor" set-options --set-required --set-revocable
classic immutable "$P-emissor" set-options --set-immutable
SAC=$(stellar contract asset deploy --asset "PINAKION:$EMISSOR" --source "$P-emissor" $N 2>/dev/null | tail -1); echo "sac $SAC"

log "2–3. the verifier, kleroterion and processo"
VERIF=$(stellar contract deploy --wasm target/wasm32v1-none/release/groth16_verifier.wasm --source "$P-escrivao" $N 2>/dev/null | tail -1); echo "verifier $VERIF"
KLERO=$(stellar contract deploy --wasm target/wasm32v1-none/release/kleroterion.wasm --source "$P-escrivao" $N -- \
  --escrivao "$ESCRIVAO" --bilhete "$SAC" --verificador "$VERIF" --janela "$JANELA" \
  --niveis "$NIVEIS" --afastamento "$AFASTAMENTO" --espera_minima "$ESPERA_MINIMA" 2>/dev/null | tail -1)
echo "kleroterion $KLERO"; INICIO=$(( $(ledger) - 2 ))
PROC=$(stellar contract deploy --wasm target/wasm32v1-none/release/processo.wasm --source "$P-escrivao" $N -- \
  --escrivao "$ESCRIVAO" --kleroterion "$KLERO" --tessera "$TESSERA_ID" --prazo_quorum "$PRAZO_QUORUM" \
  --votos_minimos "$VOTOS_MINIMOS" --dois_tercos_desde "$DOIS_TERCOS_DESDE" 2>/dev/null | tail -1)
echo "processo $PROC"
printf 'EMISSOR=%s\nSAC=%s\nVERIF=%s\nKLERO=%s\nPROC=%s\nTESSERA=%s\nINICIO=%s\n' \
  "$EMISSOR" "$SAC" "$VERIF" "$KLERO" "$PROC" "$TESSERA_ID" "$INICIO" > "$D/ids.env"

log "4–5. SAC admin -> kleroterion; issuer master weight 0"
invocar set_admin "$SAC" "$P-emissor" -- set_admin --new_admin "$KLERO" >/dev/null; echo ok
classic master_weight_0 "$P-emissor" set-options --master-weight 0

log "6. enroll $JURADOS jurors with c = Poseidon(s)"
for i in $(seq 0 $((JURADOS - 1))); do
  A=$(conta "$P-j$i")
  C=$(python3 - "$SEGREDOS" "$A" <<'PY'
import json, secrets, sys
sys.path.insert(0, "scripts")
from poseidon import R, hash1
caminho, endereco = sys.argv[1], sys.argv[2]
s = secrets.randbelow(R - 1) + 1
d = json.load(open(caminho)); d[endereco] = str(s); json.dump(d, open(caminho, "w"), indent=1)
print(hash1(s))
PY
)
  invocar enroll "$KLERO" "$P-escrivao" -- enroll --jurado "$A" --c "$C" >/dev/null
  printf '  j%-2s %s\n' "$i" "$A"
done

log "7. lock at level $GRAVIDADE (k = $K) against j0, who is on the roll"
ACUSADO=$(stellar keys address "$P-j0")
L=$(ledger); REVELA=$(( L + ATE_REVELAR ))
invocar lock "$KLERO" "$P-escrivao" -- lock --gravidade "$GRAVIDADE" --revela_em "$REVELA" --acusado "$ACUSADO"
RODADA=$(ler "$KLERO" closing | python3 -c "import sys,json;print(json.load(sys.stdin)['rodada'])")
echo "  suspended(j0) = $(ler "$KLERO" suspended --endereco "$ACUSADO")"

log "8. the case opens with the complaint; the evidence; the defence"
invocar abrir "$PROC" "$P-escrivao" -- abrir --denuncia "$(sha $AUTOS/01-denuncia.md)" >/dev/null; echo "  complaint $(sha $AUTOS/01-denuncia.md | cut -c1-12)"
invocar prova "$PROC" "$P-escrivao" -- registrar --autor "$ESCRIVAO" --tipo 1 --hash "$(sha $AUTOS/02-prova.md)" >/dev/null; echo "  evidence"
invocar defesa "$PROC" "$P-j0" -- registrar --autor "$ACUSADO" --tipo 2 --hash "$(sha $AUTOS/03-defesa.md)" >/dev/null; echo "  defence, from the party's own account"

log "9. wait for drand round $RODADA, seal, claims"
QUANDO=$(( 1692803367 + (RODADA - 1) * 3 )); ESPERA=$(( QUANDO - $(date +%s) + 3 ))
[ "$ESPERA" -gt 0 ] && sleep "$ESPERA"
SIG=$(python3 scripts/rodada.py "$RODADA" | awk '/^signature/{print $2}')
invocar seal "$KLERO" "$P-escrivao" -- seal --sig "$SIG"

reivindicar() {
  local i=$1 A S ASSENTO
  A=$(stellar keys address "$P-j$i"); S=$(python3 -c "import json,sys;print(json.load(open(sys.argv[1]))[sys.argv[2]])" "$SEGREDOS" "$A")
  ASSENTO=$(conta "$P-assento$i")
  python3 scripts/registro.py "$KLERO" "$INICIO" > "$D/registro-klero.json"
  SEED=$(python3 -c "import json;print(next(e['seed'] for e in json.load(open('$D/registro-klero.json'))['eventos'] if e['evento']=='Selado'))")
  node circuits/pinakion.mjs prove --eventos "$D/registro-klero.json" --s "$S" --seed "$SEED" --assento "$ASSENTO" --saida "$D/prova$i.json" 2>/dev/null
  PROVA=$(python3 -c "import json;print(json.dumps(json.load(open('$D/prova$i.json'))['proof']))"); Y=$(python3 -c "import json;print(json.load(open('$D/prova$i.json'))['decimal']['y'])")
  invocar claim "$KLERO" "$P-assento$i" -- claim --assento "$ASSENTO" --prova "$PROVA" --y "$Y" >/dev/null
  printf '  j%-2s claimed from %s\n' "$i" "$ASSENTO"
}
abaixo() {
  python3 - "$SEGREDOS" "$(stellar keys address "$P-j$1")" "$KLERO" "$P-escrivao" <<'PY'
import json, subprocess, sys
sys.path.insert(0, "scripts")
from poseidon import hash2
s = int(json.load(open(sys.argv[1]))[sys.argv[2]])
selo = json.loads(subprocess.run(["stellar", "contract", "invoke", "--id", sys.argv[3], "--source", sys.argv[4],
                                  "--network", "testnet", "--send=no", "--", "seal_state"],
                                 capture_output=True, text=True).stdout.strip().splitlines()[-1])
sys.exit(0 if hash2(int(selo["seed"]), s) < int(selo["limiar"]) else 1)
PY
}
REIVINDICARAM=()
for i in $(seq 1 $((JURADOS - 1))); do
  if abaixo "$i"; then reivindicar "$i"; REIVINDICARAM+=("$i"); fi
done
echo "  ${#REIVINDICARAM[@]} claim(s)"

log "10. after the window: finalize"
PRAZO=$(ler "$KLERO" seal_state | python3 -c "import sys,json;print(json.load(sys.stdin)['prazo'])")
esperar_ledger "$PRAZO"; sleep 6
FIM=$(invocar finalize "$KLERO" "$P-escrivao" -- finalize); echo "  $FIM"
if [ "$FIM" = "null" ] || [ -z "$FIM" ]; then
  for i in $(seq 1 $((JURADOS - 1))); do
    case " ${REIVINDICARAM[*]} " in *" $i "*) continue ;; esac
    reivindicar "$i"; REIVINDICARAM+=("$i")
    [ "$(ler "$KLERO" claims | python3 -c "import sys,json;print(len(json.load(sys.stdin)))")" -ge "$K" ] && break
  done
  PRAZO=$(ler "$KLERO" seal_state | python3 -c "import sys,json;print(json.load(sys.stdin)['prazo'])")
  esperar_ledger "$PRAZO"; sleep 6
  invocar finalize "$KLERO" "$P-escrivao" -- finalize
fi
SENTADOS=()
for i in "${REIVINDICARAM[@]}"; do
  [ "$(ler "$KLERO" seated --assento "$(stellar keys address "$P-assento$i")")" = "true" ] && SENTADOS+=("$i")
done
echo "  seated: ${SENTADOS[*]}"

log "11. attendance: all seated but the last; one juror files minutes"
PRESENTES=("${SENTADOS[@]:0:${#SENTADOS[@]}-1}")
for i in "${PRESENTES[@]}"; do
  invocar comparecer "$PROC" "$P-assento$i" -- comparecer --assento "$(stellar keys address "$P-assento$i")" >/dev/null
  printf '  seat of j%-2s attended\n' "$i"
done
echo "  phase $(ler "$PROC" fase)"
J1=${PRESENTES[0]}
invocar ata_jurado "$PROC" "$P-assento$J1" -- registrar --autor "$(stellar keys address "$P-assento$J1")" --tipo 3 --hash "$(sha $AUTOS/04-ata-jurado.md)" >/dev/null; echo "  juror's minutes, from a seat account"

log "12. the vote opens on Tessera; the case binds it"
NOME="pinakion-$P"
APTOS=$(printf "$P-assento%s," "${PRESENTES[@]}"); APTOS=${APTOS%,}
MESA=$(printf "$P-assento%s," "${PRESENTES[@]:0:3}"); MESA=${MESA%,}
mkdir -p "$D/tessera"
(cd "$D/tessera" && "$TESSERA_BIN" abrir --proposta "$NOME" --pergunta "Aplicar a sanção do nível $GRAVIDADE (perda do cargo por seis meses)?" \
   --opcoes sim,nao --aptos "$APTOS" --mesa "$MESA" -k 2 --prazo "$PRAZO_VOTO" --governanca "$P-escrivao" 2>&1 | tail -4)
PID=$(python3 -c "import hashlib;print(hashlib.sha256(b'TESSERA-V1-PROPOSTA'+'$NOME'.encode()).hexdigest())")
invocar vincular "$PROC" "$P-escrivao" -- vincular_votacao --proposta "$PID" >/dev/null; echo "  bound $PID"

log "13. the attendees vote (secret, on Tessera); tally; verify"
ULTIMO=${PRESENTES[${#PRESENTES[@]}-1]}
for i in "${PRESENTES[@]}"; do
  OP=sim; [ "$i" = "$ULTIMO" ] && OP=nao
  (cd "$D/tessera" && "$TESSERA_BIN" votar --proposta "$NOME" --opcao "$OP" --identidade "$P-assento$i" >/dev/null 2>&1)
  (cd "$D/tessera" && "$TESSERA_BIN" queimar --identidade "$P-assento$i" >/dev/null 2>&1 || true)
  printf '  seat of j%-2s voted\n' "$i"
done
FECHA=$(python3 -c "import json;print(json.load(open('$D/tessera/estado/$NOME.json'))['prazo_ledger'])")
esperar_ledger "$FECHA"; sleep 6
(cd "$D/tessera" && "$TESSERA_BIN" apurar --proposta "$NOME" 2>&1 | tail -6)
(cd "$D/tessera" && "$TESSERA_BIN" verificar --proposta "$NOME" > ../tessera-verificar.txt 2>&1; tail -6 ../tessera-verificar.txt)
python3 - "$D/tessera/estado/$NOME.json" "$HASHES" <<'PY'
import json, sys
e = json.load(open(sys.argv[1]))
with open(sys.argv[2], "a") as f:
    f.write(f"tessera_abrir\t{e['abertura_tx']}\n")
    for v in e.get("votos", []):
        f.write(f"tessera_votar\t{v['tx']}\n")
    f.write(f"tessera_apurar\t{e['apuracao']['tx']}\n")
PY

log "14. the case decides from Tessera's result; the clerk files the minutes"
DECISAO=$(invocar decidir "$PROC" "$P-escrivao" -- decidir); echo "  $DECISAO"
python3 - "$D" "$AUTOS" "$DECISAO" "$PID" "${#SENTADOS[@]}" "${#PRESENTES[@]}" <<'PY'
import json, pathlib, sys
d, autos, decisao, pid, sentados, presentes = sys.argv[1:]
dec = json.loads(decisao)
pathlib.Path(autos, "05-ata-sessao.md").write_text(f"""# Ata da sessão

**Caso fictício, para demonstração do Pinakion.**

- O painel tem {sentados} assentos. Compareceram {presentes}, pelas contas de assento; o quórum era 5.
- A sessão leu a denúncia, a prova, a defesa e a ata de deliberação de uma pessoa jurada.
- A votação foi aberta no Tessera, o módulo de voto secreto anterior da mesma autora, com os {presentes} presentes como lista de votantes.
- Identificador da votação no Tessera: `{pid}`.
- O processo conferiu, na cadeia, que a lista de votantes do Tessera é exatamente a dos presentes.
""")
resultado = "procedente: a sanção do nível 3 se aplica" if dec["procedente"] else "improcedente: a sanção não se aplica"
pathlib.Path(autos, "06-ata-decisao.md").write_text(f"""# Ata da decisão

**Caso fictício, para demonstração do Pinakion.**

- Placar publicado pelo Tessera: {dec['sim']} sim, {dec['nao']} não.
- A regra do nível 3 exige a maioria do painel inteiro: {dec['exigidos']} votos sim. Ausência não conta a favor.
- Decisão: **{resultado}**.
- O afastamento cautelar termina com esta decisão. A sanção é aplicada pela governança do projeto, fora deste contrato.
- Os jurados revelam quem são depois do prazo de revelação; até lá, os assentos continuam selados.
""")
PY
invocar ata_sessao "$PROC" "$P-escrivao" -- registrar --autor "$ESCRIVAO" --tipo 3 --hash "$(sha $AUTOS/05-ata-sessao.md)" >/dev/null; echo "  session minutes"
invocar ata_decisao "$PROC" "$P-escrivao" -- registrar --autor "$ESCRIVAO" --tipo 3 --hash "$(sha $AUTOS/06-ata-decisao.md)" >/dev/null; echo "  decision minutes"
echo "  suspended(j0) per processo = $(ler "$PROC" suspended --endereco "$ACUSADO")"

log "15. after reveal_after (ledger $REVELA): the jurors reveal; the absent juror stays silent"
esperar_ledger "$REVELA"
AUSENTE=${SENTADOS[${#SENTADOS[@]}-1]}
for i in $(seq 1 $((JURADOS - 1))); do
  [ "$i" = "$AUSENTE" ] && continue
  A=$(stellar keys address "$P-j$i")
  invocar reveal "$KLERO" "$P-j$i" -- reveal --jurado "$A" --s "$(python3 -c "import json;print(json.load(open('$SEGREDOS'))['$A'])")" >/dev/null
done
echo "  revealed"

log "16. read back and check"
PINAKION_FONTE="$P-escrivao" python3 scripts/registro_processo.py "$D/ids.env" "$D/tessera/estado/$NOME.json" > "$D/registro-processo.json"
python3 scripts/verificar_processo.py "$D/registro-processo.json"
