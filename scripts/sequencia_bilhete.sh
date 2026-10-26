#!/usr/bin/env bash
# The sealed sequence, on testnet, end to end (T-040) — the evidence behind
# docs/evidencia-testnet-bilhete.md.
#
#   scripts/sequencia_bilhete.sh [prefix]
#
# Needs: the Stellar CLI (28.1.0), `stellar contract build` done, `npm ci`
# done (circuits/pinakion.mjs proves with snarkjs), network access. It
# generates and funds its own identities through friendbot, named
# <prefix>-…, so nothing of yours is touched. Every transaction hash goes
# to build/<prefix>/hashes.tsv; every juror's secret s to
# build/<prefix>/segredos.json (git-ignored: losing s is losing the seat).
#
#   1. issuer: AUTH_REQUIRED + AUTH_REVOCABLE, then AUTH_IMMUTABLE
#   2. SAC for PINAKION:issuer
#   3. deploy the Groth16 verifier (key embedded), then kleroterion
#      (clerk, SAC, verifier, claim window, the severity table of
#      docs/desenho-gravidade-afastamento-revelacao.md, suspension window)
#   4. SAC admin -> kleroterion
#   5. issuer master weight 0: nobody but the contract issues or authorizes
#   6. enroll 20 jurors with c = Poseidon(s)
#   7. impede one (before lock; refused after)
#   8. lock(level 1 -> k = 3, reveal_after, the party judged): round, k,
#      root, N; the party is a fresh identity, not on the roll (T-045)
#   9. wait for that drand round; seal: seed and threshold T
#  10. every juror with y < T proves its seat (circuits/pinakion.mjs) and
#      claims it from a fresh friendbot seat account; one claim is resent
#      and refused
#  11. one claimant recuses, openly
#  12. after the window: finalize (widening once if fewer than k)
#  13. after reveal_after: 18 reveals
#  14. read the events back (scripts/registro.py) and recompute everything
#      (scripts/verificar_painel.py): it must say CONFERE
set -euo pipefail
cd "$(dirname "$0")/.."
P=${1:-t040}; N="--network testnet"; JURADOS=${JURADOS:-20}; K=3; JANELA=${JANELA:-60}; MARGEM=${MARGEM:-60}
# T-045: the approved default table (k 3/5/7/9, roster floor 16/40/40/100);
# with 20 jurors only level 1 can lock. The suspension window here is a short
# demo value, not a recommendation.
NIVEIS='[{"k":3,"rol_minimo":16},{"k":5,"rol_minimo":40},{"k":7,"rol_minimo":40},{"k":9,"rol_minimo":100}]'
GRAVIDADE=${GRAVIDADE:-1}; AFASTAMENTO=${AFASTAMENTO:-120}
D=build/$P; mkdir -p "$D"; HASHES=$D/hashes.tsv; : > "$HASHES"; SEGREDOS=$D/segredos.json; echo '{}' > "$SEGREDOS"
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
  local out; out=$(stellar contract invoke --id "$id" --source "$src" $N --send=yes -- "$@" 2>&1) || { echo "$out" >&2; return 1; }
  local h; h=$(printf '%s' "$out" | grep -o 'tx/[0-9a-f]\{64\}' | head -1 | cut -d/ -f2)
  [ -n "$h" ] && printf '%s\t%s\n' "$op" "$h" >> "$HASHES"
  printf '%s' "$out" | grep -v '^ℹ️\|^⚠️\|^🌎\|^📅\|^ *Run `stellar\|^ *$\|Signing\|Submitting\|stellar.expert\|Simulat' | tail -1 || true
}
classic() { # op, source identity, then the stellar tx new … arguments
  local op=$1 src=$2; shift 2
  stellar tx new "$@" --source "$src" $N >/dev/null 2>&1
  sleep 6 # the hash is the source account's latest transaction, read from Horizon
  local h; h=$(curl -s "https://horizon-testnet.stellar.org/accounts/$(stellar keys address "$src" 2>/dev/null)/transactions?order=desc&limit=1" \
    | python3 -c "import sys,json;print(json.load(sys.stdin)['_embedded']['records'][0]['hash'])")
  printf '%s\t%s\n' "$op" "$h" >> "$HASHES"; echo "tx $h"
}

log "0. identities (friendbot)"
EMISSOR=$(conta "$P-emissor"); ESCRIVAO=$(conta "$P-escrivao")
echo "issuer $EMISSOR"; echo "clerk  $ESCRIVAO"

log "1. issuer flags: AUTH_REQUIRED + AUTH_REVOCABLE, then AUTH_IMMUTABLE"
classic flags "$P-emissor" set-options --set-required --set-revocable
classic immutable "$P-emissor" set-options --set-immutable

log "2. SAC for PINAKION:$EMISSOR"
SAC=$(stellar contract asset deploy --asset "PINAKION:$EMISSOR" --source "$P-emissor" $N 2>/dev/null | tail -1); echo "sac $SAC"

log "3. deploy the verifier, then kleroterion (window $JANELA ledgers, suspension $AFASTAMENTO)"
VERIF=$(stellar contract deploy --wasm target/wasm32v1-none/release/groth16_verifier.wasm --source "$P-escrivao" $N 2>/dev/null | tail -1); echo "verifier $VERIF"
KLERO=$(stellar contract deploy --wasm target/wasm32v1-none/release/kleroterion.wasm --source "$P-escrivao" $N -- \
  --escrivao "$ESCRIVAO" --bilhete "$SAC" --verificador "$VERIF" --janela "$JANELA" \
  --niveis "$NIVEIS" --afastamento "$AFASTAMENTO" 2>/dev/null | tail -1)
echo "kleroterion $KLERO"; INICIO=$(( $(ledger) - 2 ))
printf 'EMISSOR=%s\nSAC=%s\nVERIF=%s\nKLERO=%s\nINICIO=%s\n' "$EMISSOR" "$SAC" "$VERIF" "$KLERO" "$INICIO" > "$D/ids.env"

log "4. SAC admin -> kleroterion"
invocar set_admin "$SAC" "$P-emissor" -- set_admin --new_admin "$KLERO" >/dev/null; echo ok

log "5. issuer master weight 0"
classic master_weight_0 "$P-emissor" set-options --master-weight 0
curl -s "https://horizon-testnet.stellar.org/accounts/$EMISSOR" | python3 -c "import sys,json;d=json.load(sys.stdin);print('flags',d['flags']);print('master weight',[s['weight'] for s in d['signers'] if s['key']==d['account_id']])"

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

log "7. impede j0 (before lock)"
invocar impede "$KLERO" "$P-escrivao" -- impede --jurado "$(stellar keys address "$P-j0")" >/dev/null; echo ok

log "8. lock, level $GRAVIDADE (k = $K)"
stellar keys generate "$P-parte" --overwrite >/dev/null 2>&1; PARTE=$(stellar keys address "$P-parte")
L=$(ledger); REVELA=$(( L + JANELA + MARGEM ))
invocar lock "$KLERO" "$P-escrivao" -- lock --gravidade "$GRAVIDADE" --revela_em "$REVELA" --acusado "$PARTE"
RODADA=$(stellar contract invoke --id "$KLERO" --source "$P-escrivao" $N --send=no -- closing 2>/dev/null | tail -1 | python3 -c "import sys,json;print(json.load(sys.stdin)['rodada'])")

log "9. wait for drand round $RODADA, then seal"
QUANDO=$(( 1692803367 + (RODADA - 1) * 3 )); ESPERA=$(( QUANDO - $(date +%s) + 3 ))
[ "$ESPERA" -gt 0 ] && sleep "$ESPERA"
SIG=$(python3 scripts/rodada.py "$RODADA" | awk '/^signature/{print $2}')
invocar seal "$KLERO" "$P-escrivao" -- seal --sig "$SIG"

# claim <juror-index>: prove from the events so far, claim from a new seat account
reivindicar() {
  local i=$1 A S ASSENTO
  A=$(stellar keys address "$P-j$i"); S=$(python3 -c "import json,sys;print(json.load(open(sys.argv[1]))[sys.argv[2]])" "$SEGREDOS" "$A")
  ASSENTO=$(conta "$P-assento$i")
  python3 scripts/registro.py "$KLERO" "$INICIO" > "$D/registro.json"
  SEED=$(python3 -c "import json;print(next(e['seed'] for e in json.load(open('$D/registro.json'))['eventos'] if e['evento']=='Selado'))")
  node circuits/pinakion.mjs prove --eventos "$D/registro.json" --s "$S" --seed "$SEED" --assento "$ASSENTO" --saida "$D/prova$i.json" 2>/dev/null
  PROVA=$(python3 -c "import json;print(json.dumps(json.load(open('$D/prova$i.json'))['proof']))"); Y=$(python3 -c "import json;print(json.load(open('$D/prova$i.json'))['decimal']['y'])")
  invocar claim "$KLERO" "$P-assento$i" -- claim --assento "$ASSENTO" --prova "$PROVA" --y "$Y" >/dev/null
  printf '  j%-2s claimed from %s\n' "$i" "$ASSENTO"
}
# below <index>: does this juror's y fall under the threshold in force?
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

log "10. claims: every juror below T, each from a fresh seat account"
REIVINDICARAM=()
for i in $(seq 1 $((JURADOS - 1))); do
  if abaixo "$i"; then reivindicar "$i"; REIVINDICARAM+=("$i"); fi
done
echo "  ${#REIVINDICARAM[@]} claim(s)"
P1=${REIVINDICARAM[0]}
echo "  resending j$P1's claim:"
PROVA=$(python3 -c "import json;print(json.dumps(json.load(open('$D/prova$P1.json'))['proof']))"); Y=$(python3 -c "import json;print(json.load(open('$D/prova$P1.json'))['decimal']['y'])")
stellar contract invoke --id "$KLERO" --source "$P-assento$P1" $N --send=yes -- claim --assento "$(stellar keys address "$P-assento$P1")" --prova "$PROVA" --y "$Y" 2>&1 \
  | grep -o 'Error(Contract, #[0-9]*)' | head -1 | sed 's/^/  refused: /' || true

log "11. one claimant recuses: j${REIVINDICARAM[1]}"
R=${REIVINDICARAM[1]}; AR=$(stellar keys address "$P-j$R")
invocar recuse "$KLERO" "$P-j$R" -- recuse --jurado "$AR" --s "$(python3 -c "import json;print(json.load(open('$SEGREDOS'))['$AR'])")" >/dev/null; echo ok

log "12. after the window: finalize"
PRAZO=$(stellar contract invoke --id "$KLERO" --source "$P-escrivao" $N --send=no -- seal_state 2>/dev/null | tail -1 | python3 -c "import sys,json;print(json.load(sys.stdin)['prazo'])")
esperar_ledger "$PRAZO"; sleep 6
FIM=$(invocar finalize "$KLERO" "$P-escrivao" -- finalize); echo "  $FIM"
if [ "$FIM" = "null" ] || [ -z "$FIM" ]; then
  echo "  fewer than $K: T widened to the field; the next jurors claim"
  for i in $(seq 1 $((JURADOS - 1))); do
    case " ${REIVINDICARAM[*]} " in *" $i "*) continue ;; esac
    reivindicar "$i"; REIVINDICARAM+=("$i")
    [ "$(stellar contract invoke --id "$KLERO" --source "$P-escrivao" $N --send=no -- claims 2>/dev/null | tail -1 | python3 -c "import sys,json;print(len(json.load(sys.stdin)))")" -ge "$K" ] && break
  done
  PRAZO=$(stellar contract invoke --id "$KLERO" --source "$P-escrivao" $N --send=no -- seal_state 2>/dev/null | tail -1 | python3 -c "import sys,json;print(json.load(sys.stdin)['prazo'])")
  esperar_ledger "$PRAZO"; sleep 6
  invocar finalize "$KLERO" "$P-escrivao" -- finalize
fi

log "13. after reveal_after (ledger $REVELA): reveals"
esperar_ledger "$REVELA"
for i in $(seq 1 $((JURADOS - 1))); do
  [ "$i" = "$R" ] && continue
  A=$(stellar keys address "$P-j$i")
  invocar reveal "$KLERO" "$P-j$i" -- reveal --jurado "$A" --s "$(python3 -c "import json;print(json.load(open('$SEGREDOS'))['$A'])")" >/dev/null
done
echo "  $(grep -c '^reveal' "$HASHES") reveal(s)"

log "14. read the events back and recompute everything"
python3 scripts/registro.py "$KLERO" "$INICIO" > "$D/registro.json"
python3 scripts/verificar_painel.py "$D/registro.json"

log "the roster, as any Horizon client sees it"
curl -s "https://horizon-testnet.stellar.org/accounts?asset=PINAKION:$EMISSOR&limit=200" | python3 -c "
import sys,json
for r in json.load(sys.stdin)['_embedded']['records']:
    b=[x for x in r['balances'] if x.get('asset_code')=='PINAKION'][0]
    print(' ', r['account_id'], b['balance'], 'authorized' if b['is_authorized'] else 'IMPEDED')"
echo; cat "$D/ids.env"; echo "hashes: $HASHES"
