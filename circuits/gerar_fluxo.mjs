// tests/fluxo-n8.json: eight jurors, their commitments, seat accounts, y and
// one proof each, for the full sealed flow in contracts/kleroterion (T-038).
// GPL-3.0 (snarkjs; see circuits/LICENSE).
//
//   node circuits/gerar_fluxo.mjs
//
// The seed is the one `seal` derives from the test round the contracts use
// (round 1000000): randomness >> 8. Secrets and seat accounts are derived
// from fixed labels, so the file is reproducible.
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { entrada, provar } from "./pinakion.mjs";
import { criarPoseidon } from "../web/poseidon.mjs";

const RAIZ = join(dirname(fileURLToPath(import.meta.url)), "..");
const P = criarPoseidon(JSON.parse(readFileSync(join(RAIZ, "tests", "poseidon-bn254.json"), "utf8")));
const r = P.R;
const sha = (x) => createHash("sha256").update(x).digest();
const num = (b) => BigInt("0x" + b.toString("hex"));

// strkey of an ed25519 public key (G…), with its CRC16-XModem checksum.
function strkeyG(pk) {
  const dado = Buffer.concat([Buffer.from([6 << 3]), pk]);
  let crc = 0;
  for (const b of dado) {
    crc ^= b << 8;
    for (let i = 0; i < 8; i++) crc = crc & 0x8000 ? ((crc << 1) ^ 0x1021) & 0xffff : (crc << 1) & 0xffff;
  }
  const tudo = Buffer.concat([dado, Buffer.from([crc & 0xff, crc >> 8])]);
  const A = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
  let bits = 0, valor = 0, out = "";
  for (const b of tudo) {
    valor = (valor << 8) | b; bits += 8;
    while (bits >= 5) { out += A[(valor >>> (bits - 5)) & 31]; bits -= 5; }
  }
  if (bits > 0) out += A[(valor << (5 - bits)) & 31];
  return out;
}

const N = 8, K = 3;
const RANDOMNESS_A = BigInt("0xb22aad4794f7451896f7a371aa46106fd84d919f3f569acd5b2fddf1d1440af3");
const seed = RANDOMNESS_A >> 8n;
const limiar = 2n * BigInt(K) >= BigInt(N) ? r : (r * 2n * BigInt(K)) / BigInt(N);

const jurados = Array.from({ length: N }, (_, i) => {
  const s = num(sha(`pinakion:fluxo:s:${i}`)) % r;
  return { i, s, c: P.hash1(s), assento: strkeyG(sha(`pinakion:fluxo:assento:${i}`)) };
});
const eventos = jurados.map((j) => ({ evento: "enroll", indice: j.i, c: j.c.toString() }));
const saida = [];
for (const j of jurados) {
  const prova = await provar(entrada(eventos, j.s, seed, j.assento));
  saida.push({
    indice: j.i, s: j.s.toString(), c: j.c.toString(), assento: j.assento,
    y: prova.decimal.y, abaixo_do_limiar: BigInt(prova.decimal.y) < limiar,
    proof: prova.proof,
  });
}
writeFileSync(join(RAIZ, "tests", "fluxo-n8.json"), JSON.stringify({
  _: "Sealed-flow fixture (T-038), written by node circuits/gerar_fluxo.mjs: eight jurors enrolled in order, no impediment, k = 3, seed = randomness(round 1000000) >> 8, limiar = floor(r * 2k / N). Each juror's y and a proof from their seat account under circuits/setup/.",
  n: N, k: K, seed: seed.toString(), limiar: limiar.toString(),
  raiz: (await import("../web/arvore.mjs")).raiz(eventos, P.hash2).toString(),
  jurados: saida,
}, null, 1) + "\n");
for (const j of saida) console.log(j.indice, j.abaixo_do_limiar ? "< T" : ">= T", j.y.slice(0, 12));
process.exit(0);
