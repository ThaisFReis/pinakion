// Gate G8: the circuit, its setup and `pinakion prove` (T-036). GPL-3.0.
//
//     node circuits/test.mjs
//
// Uses circuits/setup/ as committed: no ptau, no circom needed. Exit 0 when
// every check holds, 1 otherwise.
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import * as snarkjs from "snarkjs";

import { claimer, entrada, provar, verificar } from "./pinakion.mjs";
import { criarPoseidon } from "../web/poseidon.mjs";
import { caminho } from "../web/arvore.mjs";

const RAIZ = join(dirname(fileURLToPath(import.meta.url)), "..");
const ler = (p) => JSON.parse(readFileSync(join(RAIZ, p), "utf8"));
const P = criarPoseidon(ler("tests/poseidon-bn254.json"));
const r = P.R;
const sha = (x) => BigInt("0x" + createHash("sha256").update(x).digest("hex"));

let falhas = 0;
const ok = (cond, msg) => {
  console.log(`  ${cond ? "ok   " : "FALHA"} ${msg}`);
  if (!cond) falhas++;
};
const lanca = async (f) => {
  try { await f(); return false; } catch { return true; }
};

// 1. The path the CLI builds leads to the same roots as circomlib's trees.
const arvores = ler("tests/vetores-arvore.json").arvores;
for (const a of arvores) {
  const ultimo = a.eventos.filter((e) => e.evento === "enroll").length - 1;
  const { raiz } = caminho(a.eventos, ultimo, P.hash2);
  ok(String(raiz) === a.raiz, `caminho da folha ${ultimo} sobe a raiz de "${a.nome}"`);
}

// 2. A real claim in the tree of 17, from the secrets circuits/vetores_arvore.mjs used.
const a17 = arvores.find((a) => a.n === 17);
const s = (i) => sha(`pinakion:arvore:17:s:${i}`) % r;
const SEED = sha("pinakion:t036:seed") >> 8n;
const ASSENTO = "GDPINAKIONT036XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX";
const input = entrada(a17.eventos, s(3), SEED, ASSENTO);
ok(input.y === String(P.hash2(SEED, s(3))), "y = Poseidon(seed, s)");
ok(input.claimer === String(claimer(ASSENTO)), "claimer = SHA-256(endereco) >> 8");
const prova = await provar(input);
const { proof, publicSignals } = prova.snarkjs;
ok(await verificar(proof, publicSignals), "a prova de um jurado autorizado verifica");
ok(publicSignals[0] === a17.raiz, "a raiz publica e a da circomlib");

// 3. Negatives: the same proof against other public inputs fails.
const trocar = (i, v) => publicSignals.map((x, j) => (j === i ? String(v) : x));
ok(!(await verificar(proof, trocar(0, BigInt(a17.raiz) + 1n))), "raiz errada: recusada");
ok(!(await verificar(proof, trocar(1, SEED + 1n))), "seed errada: recusada");
ok(!(await verificar(proof, trocar(3, claimer("GOTHERXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX")))), "claimer de outra conta: recusado");
ok(!(await verificar(proof, trocar(2, P.hash2(SEED, s(4))))), "o y de outro jurado: recusado");

// 4. No witness exists for a lie: the circuit's constraints refuse it.
const wasm = join(RAIZ, "circuits", "setup", "pinakion_claim.wasm");
const zkey = join(RAIZ, "circuits", "setup", "claim_final.zkey");
ok(await lanca(() => snarkjs.groth16.fullProve({ ...input, root: String(BigInt(input.root) + 1n) }, wasm, zkey)), "sem testemunha para raiz falsa");
ok(await lanca(() => snarkjs.groth16.fullProve({ ...input, y: String(BigInt(input.y) + 1n) }, wasm, zkey)), "sem testemunha para y falso");
ok(await lanca(() => entrada(a17.eventos, s(5), SEED, ASSENTO)), "o impedido (folha 5) nao tem assento a provar");
ok(await lanca(() => entrada(a17.eventos, 123456789n, SEED, ASSENTO)), "um s que nao e de ninguem: recusado");

// 5. The key the Rust verifier tests use is this setup's key.
const vk = ler("circuits/setup/vk.json");
const fixture = ler("tests/prova-claim.json");
const be32 = (n) => BigInt(n).toString(16).padStart(64, "0");
ok(fixture.vk.alpha === be32(vk.vk_alpha_1[0]) + be32(vk.vk_alpha_1[1]), "tests/prova-claim.json usa a vk de circuits/setup");
ok(vk.nPublic === 4, "quatro entradas publicas");

if (falhas) {
  console.log(`\n${falhas} falha(s).`);
  process.exit(1);
}
console.log("\ncircuito, setup e pinakion prove conferidos.");
process.exit(0);
