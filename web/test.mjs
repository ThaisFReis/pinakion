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

import { painel, sortear, verificarSelado } from "./sorteio.mjs";
import { criarPoseidon } from "./poseidon.mjs";
import { raiz, zeros } from "./arvore.mjs";

const raiz_ = join(dirname(fileURLToPath(import.meta.url)), "..");
const { vetores } = JSON.parse(
  await readFile(join(raiz_, "tests", "vetores-de-ouro.json"), "utf8"),
);

let falhas = 0;

for (const v of vetores) {
  const obtido = v.rol
    ? await painel(v.semente, v.rol, v.inelegiveis, v.k)
    : await sortear(v.semente, v.n, v.k);
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

// T-035: Poseidon against circomlib's vectors, then the commitment root
// from the events.
const ler = async (nome) => JSON.parse(await readFile(join(raiz_, "tests", nome), "utf8"));
const { hash1, hash2 } = criarPoseidon(await ler("poseidon-bn254.json"));
const poseidon = (await ler("vetores-poseidon.json")).vetores;
let iguais = 0;
for (const v of poseidon) {
  const xs = v.entradas.map(BigInt);
  const h = xs.length === 1 ? hash1(xs[0]) : hash2(xs[0], xs[1]);
  if (h === BigInt(v.saida)) iguais++;
  else {
    falhas++;
    console.log(`  FALHA Poseidon(${v.entradas}) = ${h}, esperado ${BigInt(v.saida)}`);
  }
}
if (iguais === poseidon.length) console.log(`  ok    Poseidon: ${iguais} vetores da circomlib`);

const arvores = await ler("vetores-arvore.json");
if (zeros(hash2).every((z, d) => String(z) === arvores.zeros[d])) {
  console.log("  ok    as raizes vazias");
} else {
  falhas++;
  console.log("  FALHA as raizes vazias");
}
for (const a of arvores.arvores) {
  const obtida = String(raiz(a.eventos, hash2));
  if (obtida === a.raiz) console.log(`  ok    arvore: ${a.nome}`);
  else {
    falhas++;
    console.log(`  FALHA arvore: ${a.nome}`);
    console.log(`        esperado ${a.raiz}`);
    console.log(`        obtido   ${obtida}`);
  }
}

// T-039: the sealed vectors, from the events and the drand randomness alone.
const { selados } = JSON.parse(await readFile(join(raiz_, "tests", "vetores-de-ouro.json"), "utf8"));
for (const v of selados) {
  const r = verificarSelado(v, { R: criarPoseidon(await ler("poseidon-bn254.json")).R, hash1, hash2 }, raiz);
  const ruins = r.conferencias.filter(([, ok]) => !ok).map(([nome]) => nome);
  const estadosOk = Object.entries(v.esperado.jurados).every(([a, e]) => r.estados.get(a) === e);
  const painelOk =
    r.painel.length === v.esperado.painel.length && r.painel.every((y, i) => String(y) === v.esperado.painel[i]);
  const ok = !ruins.length && estadosOk && painelOk && r.completo === v.esperado.completo &&
    r.semRevelacao === v.esperado.assentos_sem_revelacao;
  if (ok) console.log(`  ok    ${v.nome}: ${r.conferencias.length} conferencias`);
  else {
    falhas++;
    console.log(`  FALHA ${v.nome}`);
    for (const n of ruins) console.log(`        nao confere: ${n}`);
    if (!estadosOk) console.log("        os estados dos jurados divergem");
    if (!painelOk) console.log("        o painel diverge");
  }
}

// T-045: a party who sits unimpeded, and a k that is not the level's, fail by name.
for (const v of selados) {
  const fech = v.eventos.find((e) => e.evento === "Fechado");
  const primeiro = v.eventos.find((e) => e.evento === "Inscrito").jurado;
  for (const [campo, valor, nome] of [
    ["acusado", primeiro, "a parte inscrita foi impedida antes do fechamento"],
    ["gravidade", fech.gravidade + 1, "k e o do nivel"],
  ]) {
    const w = structuredClone(v);
    w.eventos.find((e) => e.evento === "Fechado")[campo] = valor;
    const r = verificarSelado(w, { R: criarPoseidon(await ler("poseidon-bn254.json")).R, hash1, hash2 }, raiz);
    const achou = r.conferencias.find(([n]) => n === nome);
    if (achou && !achou[1]) console.log(`  ok    mutacao de ${campo}: '${nome}' nao confere`);
    else {
      falhas++;
      console.log(`  FALHA a mutacao de ${campo} passou sem '${nome}'`);
    }
  }
}

if (falhas) {
  console.log(`\n${falhas} falha(s).`);
  process.exit(1);
}
console.log(
  `\n${vetores.length} vetores conferidos, ${poseidon.length} de Poseidon, ${arvores.arvores.length} arvores, ${selados.length} selado(s).`,
);
