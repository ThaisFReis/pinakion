// Gate: the browser recomputation still agrees with the golden vectors.
//
// The same file scripts/conferir_vetores.py reads, so the contract, the
// Python verifier and this one are held to one set of numbers.
//
//     npm test
//
// Exits 0 when everything agrees, 1 on the first disagreement. No dependencies.

import { readFile } from "node:fs/promises";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

import { sortear } from "./sorteio.mjs";

const raiz = join(dirname(fileURLToPath(import.meta.url)), "..");
const { vetores } = JSON.parse(
  await readFile(join(raiz, "tests", "vetores-de-ouro.json"), "utf8"),
);

let falhas = 0;

for (const v of vetores) {
  const obtido = await sortear(v.semente, v.n, v.k);
  const bate =
    obtido.length === v.painel.length &&
    obtido.every((x, i) => x === v.painel[i]);

  if (bate) {
    console.log(`  ok    ${v.nome}`);
  } else {
    falhas++;
    console.log(`  FALHA ${v.nome}`);
    console.log(`        esperado [${v.painel}]`);
    console.log(`        obtido   [${obtido}]`);
  }
}

if (falhas) {
  console.log(`\n${falhas} falha(s).`);
  process.exit(1);
}
console.log(`\n${vetores.length} vetores conferidos.`);
