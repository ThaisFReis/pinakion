#!/usr/bin/env bash
# The classic sequence, on testnet, end to end — the evidence behind DEC-014.
#
#   scripts/sequencia_bilhete.sh <issuer-identity> <clerk-identity> <juror-identity>...
#
# Needs: a built contracts/kleroterion (stellar contract build), funded testnet
# identities for the clerk and every juror (stellar keys generate NAME --fund),
# and the issuer identity NOT yet existing — it is generated, funded and locked
# here. Every transaction hash is printed; nothing is hidden.
#
#   1. issuer: AUTH_REQUIRED + AUTH_REVOCABLE            (classic, since 2015)
#   2. SAC for PINAKION:issuer                           (SAC)
#   3. deploy the Groth16 verifier, then kleroterion(clerk, SAC, minimum = #jurors, verifier)
#   4. SAC admin -> kleroterion
#   5. issuer master weight 0: nobody can ever issue or authorize a ticket
#      again except the contract                        (the commitment)
#   6. enroll each juror with c = Poseidon(s): SAC trust (CAP-73) + set_authorized + mint;
#      the contract updates the commitment tree
#   7. impede the first juror of the enrolment order    (art. 463; refused after lock)
#   8. lock(k = 3, reveal_after ledger): ledger, timestamp, the drand round, k, root, N
#   9. wait for that round, then seal with its signature (once; no round is chosen):
#      the seed and the threshold T are fixed
#  10. print what seal fixed; claims, finalize and reveals are T-040
set -euo pipefail
N="--network testnet"
ISSUER=$1; CLERK=$2; shift 2; JURORS=("$@")
WASM=target/wasm32v1-none/release/kleroterion.wasm
log() { printf '\n== %s\n' "$*"; }

log "0. issuer identity $ISSUER (generated and funded)"
stellar keys generate "$ISSUER" $N --fund >/dev/null 2>&1 || true
EMISSOR=$(stellar keys address "$ISSUER"); ESCRIVAO=$(stellar keys address "$CLERK")
echo "issuer   $EMISSOR"; echo "clerk    $ESCRIVAO"; sleep 3

log "1. issuer flags AUTH_REQUIRED + AUTH_REVOCABLE"
stellar tx new set-options --source "$ISSUER" $N --set-required --set-revocable 2>&1 | grep -o '[0-9a-f]\{64\}' | sed 's/^/tx /'

log "2. SAC for PINAKION:$EMISSOR"
SAC=$(stellar contract asset deploy --asset "PINAKION:$EMISSOR" --source "$ISSUER" $N 2>/dev/null | tail -1); echo "sac      $SAC"

log "3. deploy the Groth16 verifier (key embedded), then kleroterion (clerk, SAC, minimum ${#JURORS[@]}, verifier)"
VERIF=$(stellar contract deploy --wasm target/wasm32v1-none/release/groth16_verifier.wasm --source "$CLERK" $N 2>/dev/null | tail -1); echo "verifier $VERIF"
KLERO=$(stellar contract deploy --wasm "$WASM" --source "$CLERK" $N -- --escrivao "$ESCRIVAO" --bilhete "$SAC" --minimo "${#JURORS[@]}" --verificador "$VERIF" --janela "${JANELA:-120}" 2>/dev/null | tail -1); echo "contract $KLERO"

log "4. SAC admin -> kleroterion"
stellar contract invoke --id "$SAC" --source "$ISSUER" $N -- set_admin --new_admin "$KLERO" 2>&1 | grep -o 'tx/[0-9a-f]*' | sed 's#tx/#tx #'

log "5. issuer master weight 0"
stellar tx new set-options --source "$ISSUER" $N --master-weight 0 2>&1 | grep -o '[0-9a-f]\{64\}' | sed 's/^/tx /'
curl -s "https://horizon-testnet.stellar.org/accounts/$EMISSOR" | python3 -c "import sys,json;d=json.load(sys.stdin);print('flags',d['flags']);print('master weight',[s['weight'] for s in d['signers'] if s['key']==d['account_id']])"

log "6. enroll ${#JURORS[@]} jurors with c = Poseidon(s) (clerk signs the envelope; the CLI signs each juror's auth entry)"
# Each juror's secret s is drawn here and kept in build/ (git-ignored): losing
# it means losing the seat (T-036/T-038 need it). Only c goes on-chain.
mkdir -p build; SEGREDOS=build/segredos-$KLERO.json; echo '{}' > "$SEGREDOS"
for J in "${JURORS[@]}"; do
  A=$(stellar keys address "$J")
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
  H=$(stellar contract invoke --id "$KLERO" --source "$CLERK" $N -- enroll --jurado "$A" --c "$C" 2>&1 | grep -o 'tx/[0-9a-f]*' | cut -d/ -f2)
  printf '%-8s %s  tx %s\n' "$J" "$A" "$H"
done

log "7. impede the first of the enrolment order (before lock: after it, impede is refused)"
PRIMEIRO=$(stellar keys address "${JURORS[0]}")
stellar contract invoke --id "$KLERO" --source "$CLERK" $N -- impede --jurado "$PRIMEIRO" 2>&1 | grep -o 'tx/[0-9a-f]*' | sed 's#tx/#tx #'

log "8. lock, k = 3; reveal_after = the ledger about one day ahead (unused until T-038)"
LEDGER=$(curl -s https://horizon-testnet.stellar.org/ | python3 -c "import sys,json;print(json.load(sys.stdin)['history_latest_ledger'])")
stellar contract invoke --id "$KLERO" --source "$CLERK" $N -- lock --k 3 --revela_em $(( LEDGER + 17280 )) 2>&1 | grep -E 'tx/|ledger|tempo|rodada' | sed 's#.*tx/#tx #'
FECHAMENTO=$(stellar contract invoke --id "$KLERO" --source "$CLERK" $N --send=no -- closing 2>/dev/null | tail -1); echo "$FECHAMENTO"
RODADA=$(printf '%s' "$FECHAMENTO" | python3 -c "import sys,json;print(json.load(sys.stdin)['rodada'])")

log "9. wait for round $RODADA, then seal: the seed and the threshold T"
QUANDO=$(( 1692803367 + (RODADA - 1) * 3 )); ESPERA=$(( QUANDO - $(date +%s) + 3 ))
[ "$ESPERA" -gt 0 ] && { echo "round $RODADA is published at $QUANDO; waiting ${ESPERA}s"; sleep "$ESPERA"; }
SIG=$(python3 scripts/rodada.py "$RODADA" | awk '/^signature/{print $2}')
echo "round    $RODADA"
stellar contract invoke --id "$KLERO" --source "$CLERK" $N --send=yes -- seal --sig "$SIG" 2>&1 | grep -E 'tx/|^\[' | sed 's#.*tx/#tx #'

log "10. what seal fixed"
stellar contract invoke --id "$KLERO" --source "$CLERK" $N --send=no -- seal_state 2>/dev/null | tail -1
# Claims from fresh seat accounts (circuits/pinakion.mjs prove), finalize,
# reveals and recusal are T-040; the open-draw recomputation that used to
# follow here no longer applies — kleroterion draws sealed since T-038.

log "the roster, as any Horizon client sees it"
curl -s "https://horizon-testnet.stellar.org/accounts?asset=PINAKION:$EMISSOR&limit=200" | python3 -c "
import sys,json
for r in json.load(sys.stdin)['_embedded']['records']:
    b=[x for x in r['balances'] if x.get('asset_code')=='PINAKION'][0]
    print(' ', r['account_id'], b['balance'], 'authorized' if b['is_authorized'] else 'IMPEDED')"
echo; echo "ISSUER=$EMISSOR  SAC=$SAC  KLEROTERION=$KLERO"
