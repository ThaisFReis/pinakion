// Entrada de testemunha para pinakion_claim.circom: árvore de profundidade 6
// com N folhas Poseidon(s_i) e o resto zero, e o caminho de um jurado.
// Spike de T-034: segredos e semente de teste, deterministas. GPL-3.0 (usa a
// circomlibjs; ver circuits/LICENSE).
//
//   node circuits/entrada.mjs <indice> <endereco-do-assento> <saida.json>
import { buildPoseidon } from "circomlibjs";
import { createHash } from "node:crypto";
import { writeFileSync } from "node:fs";

const DEPTH = 6;
const N = 8;
const r = BigInt("0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");
const sha = (x) => BigInt("0x" + createHash("sha256").update(x).digest("hex"));
const [indice = "3", assento = "GBPINAKIONSPIKEXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX", saida = "build/claim/input.json"] =
  process.argv.slice(2);

const p = await buildPoseidon();
const h = (xs) => p.F.toObject(p(xs));

const segredos = Array.from({ length: N }, (_, i) => sha(`pinakion:spike:s:${i}`) % r);
const folhas = segredos.map((s) => h([s]));
while (folhas.length < 2 ** DEPTH) folhas.push(0n);
const niveis = [folhas];
for (let d = 0; d < DEPTH; d++) {
  const ant = niveis[d], prox = [];
  for (let i = 0; i < ant.length; i += 2) prox.push(h([ant[i], ant[i + 1]]));
  niveis.push(prox);
}
const raiz = niveis[DEPTH][0];

// Semente: a aleatoriedade da rodada, reduzida ao campo. claimer: SHA-256 do
// endereço deslocado 8 bits, para caber no campo (T-037 faz o mesmo na cadeia).
const semente = sha("pinakion:spike:seed") % r;
const claimer = sha(assento) >> 8n;

const idx = Number(indice);
const s = segredos[idx];
const pathElements = [], pathIndices = [];
let pos = idx;
for (let d = 0; d < DEPTH; d++) {
  pathElements.push(niveis[d][pos % 2 === 0 ? pos + 1 : pos - 1].toString());
  pathIndices.push(pos % 2);
  pos = Math.floor(pos / 2);
}
const entrada = {
  root: raiz.toString(), seed: semente.toString(), y: h([semente, s]).toString(),
  claimer: claimer.toString(), s: s.toString(), pathElements, pathIndices,
};
writeFileSync(saida, JSON.stringify(entrada, null, 2));
console.log(JSON.stringify({ root: entrada.root, y: entrada.y }));
