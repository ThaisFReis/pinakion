#!/usr/bin/env bash
# Phase-2 setup for circuits/pinakion_claim.circom (T-036). GPL-3.0.
#
#   circuits/setup.sh
#
# Phase 1 is public: Hermez's powersOfTau28_hez_final_12.ptau, checked against
# the blake2b hash snarkjs publishes. Phase 2 is SINGLE-PARTY: one
# contribution from whoever runs this, with entropy from /dev/urandom that is
# never written down, then a public random beacon — a drand quicknet round
# chosen after the contribution's hash is printed — so the last step is one
# nobody chose. Single-party means: if the contributor kept their entropy,
# they could forge proofs. The README says so on page one.
#
# Writes circuits/setup/{pinakion_claim.r1cs, pinakion_claim.wasm,
# claim_final.zkey, vk.json, TRANSCRICAO.md}. Anyone can re-check the result
# with the `snarkjs zkey verify` line in TRANSCRICAO.md.
set -euo pipefail
cd "$(dirname "$0")/.."
B=build/setup; O=circuits/setup; S="npx snarkjs"; mkdir -p $B $O
PTAU=build/ptau/powersOfTau28_hez_final_12.ptau
PTAU_URL=https://circom.info/powersOfTau28_hez_final_12.ptau
PTAU_B2=ded2694169b7b08e898f736d5de95af87c3f1a64594013351b1a796dbee393bd825f88f9468c84505ddd11eb0b1465ac9b43b9064aa8ec97f2b73e04758b8a4a
CHAIN=52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971

[ -f $PTAU ] || { mkdir -p build/ptau; curl -sfL -o $PTAU $PTAU_URL; }
[ "$(b2sum $PTAU | cut -c1-128)" = "$PTAU_B2" ] || { echo "ptau hash mismatch"; exit 1; }

circom circuits/pinakion_claim.circom --r1cs --wasm -o $B >/dev/null
$S groth16 setup $B/pinakion_claim.r1cs $PTAU $B/claim_0000.zkey >/dev/null
INICIO=$(date -u +%Y-%m-%dT%H:%M:%SZ)
CONTRIB=$($S zkey contribute $B/claim_0000.zkey $B/claim_0001.zkey \
  --name="Pinakion phase 2, single contributor" -e="$(head -c 64 /dev/urandom | base64)" 2>&1 \
  | sed -n '/Contribution Hash/,/^$/p' | grep -v "Contribution Hash" | tr -s ' \t\n' ' ' | sed 's/\x1b\[[0-9;]*m//g; s/\[INFO\]  snarkJS: //g; s/^ *//; s/ *$//')
echo "contribution hash: $CONTRIB"
# The beacon: first quicknet round published strictly after now, plus 10.
AGORA=$(date +%s); RODADA=$(( (AGORA - 1692803367) / 3 + 2 + 10 )); QUANDO=$(( 1692803367 + (RODADA - 1) * 3 ))
echo "beacon: drand quicknet round $RODADA, published at $QUANDO; waiting"
sleep $(( QUANDO - $(date +%s) + 3 ))
BEACON=$(curl -sf "https://api.drand.sh/$CHAIN/public/$RODADA" | python3 -c "import sys,json;print(json.load(sys.stdin)['randomness'])")
$S zkey beacon $B/claim_0001.zkey $B/claim_final.zkey "$BEACON" 10 -n="drand quicknet round $RODADA" >/dev/null
VERIFICA=$($S zkey verify $B/pinakion_claim.r1cs $PTAU $B/claim_final.zkey 2>&1 | sed 's/\x1b\[[0-9;]*m//g; s/\[INFO\]  snarkJS: //g')
echo "$VERIFICA" | grep -q "ZKey Ok!" || { echo "$VERIFICA"; exit 1; }
$S zkey export verificationkey $B/claim_final.zkey $O/vk.json >/dev/null
cp $B/pinakion_claim.r1cs $B/claim_final.zkey $O/
cp $B/pinakion_claim_js/pinakion_claim.wasm $O/

h() { sha256sum "$1" | cut -c1-64; }
cat > $O/TRANSCRICAO.md <<MD
# Phase-2 transcript — \`pinakion_claim\`

Produced by \`circuits/setup.sh\` on $INICIO (UTC). **Single-party**: one contribution, then a public beacon. Test-only in the sense that matters: whoever ran the contribution could have kept the entropy and could forge proofs. Nobody should treat a proof under this key as more than *whoever ran the setup did not cheat*.

| Step | What | Value |
|---|---|---|
| circuit | \`circuits/pinakion_claim.circom\`, circom $(circom --version | awk '{print $3}') | r1cs sha256 \`$(h $O/pinakion_claim.r1cs)\` |
| phase 1 | Hermez \`powersOfTau28_hez_final_12.ptau\`, from $PTAU_URL | blake2b \`$PTAU_B2\` (as published by snarkjs) |
| phase 2, contribution 1 | *Pinakion phase 2, single contributor*; entropy from \`/dev/urandom\`, not stored | contribution hash \`$CONTRIB\` |
| phase 2, beacon | drand quicknet round **$RODADA** (published $QUANDO), randomness \`$BEACON\`, 2^10 iterations. The round was computed from the clock **after** the contribution hash was printed, on the contributor's machine; the hash was not published anywhere before the round, so a third party cannot check that ordering — only that the round is real and that the beacon was applied | — |
| final zkey | \`circuits/setup/claim_final.zkey\` | sha256 \`$(h $O/claim_final.zkey)\` |
| vk | \`circuits/setup/vk.json\` | sha256 \`$(h $O/vk.json)\` |
| witness generator | \`circuits/setup/pinakion_claim.wasm\` | sha256 \`$(h $O/pinakion_claim.wasm)\` |

To check it yourself, with the ptau from the link above:

\`\`\`bash
npx snarkjs zkey verify circuits/setup/pinakion_claim.r1cs powersOfTau28_hez_final_12.ptau circuits/setup/claim_final.zkey
\`\`\`

\`snarkjs zkey verify\` output on the day:

\`\`\`
$VERIFICA
\`\`\`

Check the beacon round at \`https://api.drand.sh/$CHAIN/public/$RODADA\`.
MD
echo "wrote $O"
