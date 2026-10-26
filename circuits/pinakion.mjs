#!/usr/bin/env node
// pinakion prove — the seat claim, from the contract's events and the juror's
// secret (T-036). GPL-3.0 (snarkjs; see circuits/LICENSE).
//
//   node circuits/pinakion.mjs prove --eventos E.json --s S --seed SEED \
//        --assento ADDRESS [--saida prova.json]
//
//   E.json   the contract's events in ledger order: a list, or the record
//            scripts/registro.py writes (Inscrito/Impedido are used)
//   S        the juror's secret, decimal; c = Poseidon(S) must be a leaf
//   SEED     the seed `seal` fixed, decimal field element
//   ADDRESS  the seat account the claim will be sent from (G… or C…)
//
// Writes {proof: {a, b, c}, public: [root, seed, y, claimer]} in the byte
// layout of contracts/groth16-verifier, plus the same values in decimal.
// Proves with circuits/setup/: the circuit's witness generator and the
// final zkey of the phase-2 setup in circuits/setup/TRANSCRICAO.md.
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import * as snarkjs from "snarkjs";

import { criarPoseidon } from "../web/poseidon.mjs";
import { caminho } from "../web/arvore.mjs";

const RAIZ = join(dirname(fileURLToPath(import.meta.url)), "..");
const SETUP = join(RAIZ, "circuits", "setup");
const WASM = join(SETUP, "pinakion_claim.wasm");
const ZKEY = join(SETUP, "claim_final.zkey");
const VK = join(SETUP, "vk.json");

const P = criarPoseidon(JSON.parse(readFileSync(join(RAIZ, "tests", "poseidon-bn254.json"), "utf8")));

// claimer = SHA-256 of the address as its strkey text, shifted right 8 bits
// so it is a field element (DEC-024; T-037 computes the same on-chain).
export const claimer = (endereco) =>
  BigInt("0x" + createHash("sha256").update(endereco, "ascii").digest("hex")) >> 8n;

// The contract names its events Inscrito/Impedido (scripts/registro.py);
// the tree code calls them enroll/impede. Everything else is irrelevant here.
const paraArvore = (eventos) =>
  eventos.flatMap((e) =>
    e.evento === "Inscrito" ? [{ evento: "enroll", indice: e.indice, c: e.c }]
      : e.evento === "Impedido" ? [{ evento: "impede", indice: e.indice }]
      : e.evento === "enroll" || e.evento === "impede" ? [e]
      : [],
  );

// The witness input for the circuit, or an Error saying why there is none.
export function entrada(todos, s, seed, assento) {
  const eventos = paraArvore(todos);
  const c = P.hash1(s);
  const inscricao = eventos.find((e) => e.evento === "enroll" && BigInt(e.c) === c);
  if (!inscricao) throw new Error("Poseidon(s) não é compromisso de ninguém nestes eventos");
  if (eventos.some((e) => e.evento === "impede" && e.indice === inscricao.indice)) {
    throw new Error(`a folha ${inscricao.indice} foi impedida: não há assento a provar`);
  }
  const { pathElements, pathIndices, raiz } = caminho(eventos, inscricao.indice, P.hash2);
  return {
    root: raiz.toString(),
    seed: seed.toString(),
    y: P.hash2(seed, s).toString(),
    claimer: claimer(assento).toString(),
    s: s.toString(),
    pathElements: pathElements.map(String),
    pathIndices,
  };
}

// snarkjs layout → contracts/groth16-verifier (same as circuits/exportar_soroban.mjs).
const be32 = (n) => BigInt(n).toString(16).padStart(64, "0");
const g1 = (p) => be32(p[0]) + be32(p[1]);
const g2 = (p) => be32(p[0][1]) + be32(p[0][0]) + be32(p[1][1]) + be32(p[1][0]);

export async function provar(input) {
  const { proof, publicSignals } = await snarkjs.groth16.fullProve(input, WASM, ZKEY);
  return {
    proof: { a: g1(proof.pi_a), b: g2(proof.pi_b), c: g1(proof.pi_c) },
    public: publicSignals.map(be32),
    decimal: { root: publicSignals[0], seed: publicSignals[1], y: publicSignals[2], claimer: publicSignals[3] },
    snarkjs: { proof, publicSignals },
  };
}

export async function verificar(snark, publicSignals) {
  return snarkjs.groth16.verify(JSON.parse(readFileSync(VK, "utf8")), publicSignals, snark);
}

function argumentos(argv) {
  const a = {};
  for (let i = 0; i < argv.length; i += 2) a[argv[i].replace(/^--/, "")] = argv[i + 1];
  return a;
}

async function main() {
  const [cmd, ...resto] = process.argv.slice(2);
  if (cmd !== "prove") {
    console.error("uso: node circuits/pinakion.mjs prove --eventos E.json --s S --seed SEED --assento ADDRESS [--saida prova.json]");
    process.exit(2);
  }
  const a = argumentos(resto);
  for (const k of ["eventos", "s", "seed", "assento"]) {
    if (!a[k]) { console.error(`falta --${k}`); process.exit(2); }
  }
  const lido = JSON.parse(readFileSync(a.eventos, "utf8"));
  const eventos = Array.isArray(lido) ? lido : lido.eventos;
  const prova = await provar(entrada(eventos, BigInt(a.s), BigInt(a.seed), a.assento));
  if (!(await verificar(prova.snarkjs.proof, prova.snarkjs.publicSignals))) {
    throw new Error("a prova gerada não verifica contra circuits/setup/vk.json");
  }
  const saida = JSON.stringify({ proof: prova.proof, public: prova.public, decimal: prova.decimal }, null, 2) + "\n";
  if (a.saida) writeFileSync(a.saida, saida);
  else process.stdout.write(saida);
  console.error(`y = ${prova.decimal.y}`);
  process.exit(0);
}

if (process.argv[1] && fileURLToPath(import.meta.url) === process.argv[1]) {
  main().catch((e) => { console.error(e.message); process.exit(1); });
}
