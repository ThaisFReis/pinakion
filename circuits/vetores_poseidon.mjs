// Vetores de Poseidon(1) e Poseidon(2) tirados da circomlibjs, a referência
// que o circuito usa (T-034, Q-22). O sponge do contrato tem de devolver os
// mesmos valores, byte a byte: contracts/poseidon/src/test.rs lê este arquivo.
//
//   node circuits/vetores_poseidon.mjs > tests/vetores-poseidon.json
//
// Determinístico: as entradas "aleatórias" são sha256 de um rótulo, reduzidas
// ao campo. circomlibjs é GPL-3.0 e é usada aqui como ferramenta; o que sai
// são números (DEC-021).
import { buildPoseidon } from "circomlibjs";
import { createHash } from "node:crypto";

const r = BigInt("0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");
const de = (rotulo) => BigInt("0x" + createHash("sha256").update(rotulo).digest("hex")) % r;

const p = await buildPoseidon();
const h = (xs) => p.F.toObject(p(xs));

const um = [0n, 1n, 2n, 11n, 2n ** 64n, 2n ** 128n, r - 2n, r - 1n, de("pinakion:s:1"), de("pinakion:s:2")];
const dois = [
  [0n, 0n], [1n, 2n], [3n, 4n], [2n, 1n], [0n, r - 1n], [r - 1n, 0n], [r - 1n, r - 1n],
  [2n ** 253n, 7n], [de("pinakion:seed"), de("pinakion:s:1")], [de("pinakion:seed"), de("pinakion:s:2")],
  [de("pinakion:folha:0"), de("pinakion:folha:1")], [de("pinakion:folha:2"), 0n],
];

const hex = (n) => "0x" + n.toString(16).padStart(64, "0");
const vetores = [
  ...um.map((x) => ({ entradas: [hex(x)], saida: hex(h([x])) })),
  ...dois.map(([a, b]) => ({ entradas: [hex(a), hex(b)], saida: hex(h([a, b])) })),
];
process.stdout.write(JSON.stringify({
  fonte: "circomlibjs 0.1.7 buildPoseidon, BN254",
  vetores,
}, null, 2) + "\n");
