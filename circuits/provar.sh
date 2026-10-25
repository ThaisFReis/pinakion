#!/usr/bin/env bash
# Spike de T-034: compila o circuito, faz um setup de TESTE e uma prova, e a
# exporta no formato do verificador Soroban. GPL-3.0 (ver circuits/LICENSE).
#
# O setup aqui é local e de uma parte só, com entropia descartada: serve para
# medir e provar que a cadeia aceita a prova, e para NADA mais. T-036 troca o
# ptau por um público e publica a transcrição da fase 2.
set -euo pipefail
cd "$(dirname "$0")/.."
B=build/claim; S="npx snarkjs"; mkdir -p $B
circom circuits/pinakion_claim.circom --r1cs --wasm -o $B >/dev/null
if [ ! -f $B/claim_final.zkey ]; then
  $S powersoftau new bn128 12 $B/pot_0.ptau >/dev/null
  $S powersoftau contribute $B/pot_0.ptau $B/pot_1.ptau --name="spike" -e="$(head -c 32 /dev/urandom | base64)" >/dev/null
  $S powersoftau prepare phase2 $B/pot_1.ptau $B/pot_final.ptau >/dev/null
  $S groth16 setup $B/pinakion_claim.r1cs $B/pot_final.ptau $B/claim_0.zkey >/dev/null
  $S zkey contribute $B/claim_0.zkey $B/claim_final.zkey --name="spike" -e="$(head -c 32 /dev/urandom | base64)" >/dev/null
  $S zkey export verificationkey $B/claim_final.zkey $B/vk.json
fi
node circuits/entrada.mjs "${1:-3}" "${2:-GBPINAKIONSPIKEXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX}" $B/input.json
$S wtns calculate $B/pinakion_claim_js/pinakion_claim.wasm $B/input.json $B/witness.wtns
$S groth16 prove $B/claim_final.zkey $B/witness.wtns $B/proof.json $B/public.json
$S groth16 verify $B/vk.json $B/public.json $B/proof.json
node circuits/exportar_soroban.mjs $B/vk.json $B/proof.json $B/public.json tests/prova-claim-spike.json
