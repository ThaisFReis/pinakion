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
#   3. deploy kleroterion(clerk, SAC, minimum = #jurors)
#   4. SAC admin -> kleroterion
#   5. issuer master weight 0: nobody can ever issue or authorize a ticket
#      again except the contract                        (the commitment)
#   6. enroll each juror: SAC trust (CAP-73) + set_authorized + mint
#   7. lock: ledger, timestamp, minimum drand round
#   8. impede the first juror of the enrolment order    (art. 463)
#   9. draw_round with the latest drand round, k = 3
#  10. recompute the panel off-chain, no dependencies
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

log "3. deploy kleroterion (clerk, SAC, minimum ${#JURORS[@]})"
KLERO=$(stellar contract deploy --wasm "$WASM" --source "$CLERK" $N -- --escrivao "$ESCRIVAO" --bilhete "$SAC" --minimo "${#JURORS[@]}" 2>/dev/null | tail -1); echo "contract $KLERO"

log "4. SAC admin -> kleroterion"
stellar contract invoke --id "$SAC" --source "$ISSUER" $N -- set_admin --new_admin "$KLERO" 2>&1 | grep -o 'tx/[0-9a-f]*' | sed 's#tx/#tx #'

log "5. issuer master weight 0"
stellar tx new set-options --source "$ISSUER" $N --master-weight 0 2>&1 | grep -o '[0-9a-f]\{64\}' | sed 's/^/tx /'
curl -s "https://horizon-testnet.stellar.org/accounts/$EMISSOR" | python3 -c "import sys,json;d=json.load(sys.stdin);print('flags',d['flags']);print('master weight',[s['weight'] for s in d['signers'] if s['key']==d['account_id']])"

log "6. enroll ${#JURORS[@]} jurors (clerk signs the envelope; the CLI signs each juror's auth entry with the local identity)"
for J in "${JURORS[@]}"; do
  A=$(stellar keys address "$J")
  H=$(stellar contract invoke --id "$KLERO" --source "$CLERK" $N -- enroll --jurado "$A" 2>&1 | grep -o 'tx/[0-9a-f]*' | cut -d/ -f2)
  printf '%-8s %s  tx %s\n' "$J" "$A" "$H"
done

log "7. lock"
stellar contract invoke --id "$KLERO" --source "$CLERK" $N -- lock 2>&1 | grep -E 'tx/|ledger|tempo|rodada' | sed 's#.*tx/#tx #'
stellar contract invoke --id "$KLERO" --source "$CLERK" $N --send=no -- closing 2>/dev/null | tail -1

log "8. impede the first of the enrolment order"
PRIMEIRO=$(stellar keys address "${JURORS[0]}")
stellar contract invoke --id "$KLERO" --source "$CLERK" $N -- impede --jurado "$PRIMEIRO" 2>&1 | grep -o 'tx/[0-9a-f]*' | sed 's#tx/#tx #'

log "9. draw_round with the latest drand round, k = 3"
RODADA=$(python3 scripts/rodada.py | awk '/^round/{print $2}')
SIG=$(python3 scripts/rodada.py "$RODADA" | awk '/^signature/{print $2}')
echo "round    $RODADA"
stellar contract invoke --id "$KLERO" --source "$CLERK" $N --send=yes -- draw_round --round "$RODADA" --sig "$SIG" --k 3 2>&1 | grep -E 'tx/|^\[' | sed 's#.*tx/#tx #'

log "10. the same panel, recomputed off-chain"
SEMENTE=$(curl -s "https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971/public/$RODADA" | python3 -c "import sys,json;print(json.load(sys.stdin)['randomness'])")
stellar contract invoke --id "$KLERO" --source "$CLERK" $N --send=no -- roster 2>/dev/null | tail -1 > /tmp/rol.json
python3 scripts/verificar_painel.py "$SEMENTE" 3 /tmp/rol.json --fora "$PRIMEIRO"

log "the roster, as any Horizon client sees it"
curl -s "https://horizon-testnet.stellar.org/accounts?asset=PINAKION:$EMISSOR&limit=200" | python3 -c "
import sys,json
for r in json.load(sys.stdin)['_embedded']['records']:
    b=[x for x in r['balances'] if x.get('asset_code')=='PINAKION'][0]
    print(' ', r['account_id'], b['balance'], 'authorized' if b['is_authorized'] else 'IMPEDED')"
echo; echo "ISSUER=$EMISSOR  SAC=$SAC  KLEROTERION=$KLERO"
