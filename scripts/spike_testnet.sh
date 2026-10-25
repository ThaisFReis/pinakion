#!/usr/bin/env bash
# T-034, kill-or-go: one proof of circuits/pinakion_claim.circom verified on
# testnet by the ported verifier, with what the network charged.
#
#   scripts/spike_testnet.sh [source-identity]
#
# Needs: circuits/provar.sh already run (tests/prova-claim-spike.json) and a
# funded testnet identity. Prints the contract id, each tx hash, and for each
# call the CPU instructions from simulation (--cost) plus the resources the
# sent transaction declared. Nothing touches mainnet.
set -euo pipefail
cd "$(dirname "$0")/.."
SOURCE=${1:-cautela}; N="--network testnet"
stellar keys address "$SOURCE" >/dev/null 2>&1 || stellar keys generate "$SOURCE" $N --fund
stellar contract build --package groth16-verifier >/dev/null 2>&1
WASM=target/wasm32v1-none/release/groth16_verifier.wasm
J=tests/prova-claim-spike.json
VK=$(node -e 'const j=JSON.parse(require("fs").readFileSync(process.argv[1]));const v=j.vk;console.log(JSON.stringify({alpha:v.alpha,beta:v.beta,gamma:v.gamma,delta:v.delta,ic:v.ic}))' $J)
PROOF=$(node -e 'const j=JSON.parse(require("fs").readFileSync(process.argv[1]));console.log(JSON.stringify(j.proof))' $J)
PUBLIC=$(node -e 'const j=JSON.parse(require("fs").readFileSync(process.argv[1]));console.log(JSON.stringify(j.public.map(h=>BigInt("0x"+h).toString())))' $J)
TAMPERED=$(node -e 'const p=JSON.parse(process.argv[1]);p[2]="1";console.log(JSON.stringify(p))' "$PUBLIC")

ID=$(stellar contract deploy --wasm "$WASM" --source "$SOURCE" $N -- --vk "$VK" 2>/dev/null | tail -1)
echo "verifier $ID"

chamar() { # label public
  echo "== $1"
  stellar contract invoke --id "$ID" --source "$SOURCE" $N --send=yes --cost -- \
    verify --proof "$PROOF" --public "$2" 2>&1 | grep -E 'tx/|^(true|false)$|[Cc]pu|instructions|[Mm]em' | sed 's#.*tx/#tx #'
}
chamar "valid proof" "$PUBLIC"
chamar "tampered y" "$TAMPERED"
