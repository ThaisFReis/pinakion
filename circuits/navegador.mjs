// The seat proof in a browser (or Node), from the contract's events and the
// juror's secret (T-053). GPL-3.0, like the rest of circuits/ (snarkjs; see
// circuits/LICENSE).
//
// Same arithmetic as circuits/pinakion.mjs, with no Node imports: `snarkjs`
// is passed in (the browser loads snarkjs.min.js from a CDN), the circuit's
// witness generator and zkey are URLs (or paths, in Node), and SHA-256 is
// Web Crypto.

import { caminho } from "../web/arvore.mjs";

// claimer = SHA-256 of the address as its strkey text, shifted right 8 bits
// (DEC-024; kleroterion computes the same on-chain).
export async function claimer(endereco) {
  const d = new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(endereco)));
  return BigInt("0x" + [...d].map((b) => b.toString(16).padStart(2, "0")).join("")) >> 8n;
}

const paraArvore = (eventos) =>
  eventos.flatMap((e) =>
    e.evento === "Inscrito" ? [{ evento: "enroll", indice: e.indice, c: e.c }]
      : e.evento === "Impedido" ? [{ evento: "impede", indice: e.indice }]
      : [],
  );

/** The witness input, or an Error saying why there is none. */
export async function entrada(P, eventos, s, seed, assento) {
  const arvore = paraArvore(eventos);
  const c = P.hash1(s);
  const inscricao = arvore.find((e) => e.evento === "enroll" && BigInt(e.c) === c);
  if (!inscricao) throw new Error("Poseidon(s) não é compromisso de ninguém nestes eventos");
  if (arvore.some((e) => e.evento === "impede" && e.indice === inscricao.indice)) {
    throw new Error(`a folha ${inscricao.indice} foi impedida: não há assento a provar`);
  }
  const { pathElements, pathIndices, raiz } = caminho(arvore, inscricao.indice, P.hash2);
  return {
    root: raiz.toString(),
    seed: seed.toString(),
    y: P.hash2(seed, s).toString(),
    claimer: (await claimer(assento)).toString(),
    s: s.toString(),
    pathElements: pathElements.map(String),
    pathIndices,
  };
}

const be32 = (n) => BigInt(n).toString(16).padStart(64, "0");
const g1 = (p) => be32(p[0]) + be32(p[1]);
const g2 = (p) => be32(p[0][1]) + be32(p[0][0]) + be32(p[1][1]) + be32(p[1][0]);

/** Proves, and returns the proof in contracts/groth16-verifier's byte layout. */
export async function provar(snarkjs, input, wasm, zkey) {
  const { proof, publicSignals } = await snarkjs.groth16.fullProve(input, wasm, zkey);
  return {
    proof: { a: g1(proof.pi_a), b: g2(proof.pi_b), c: g1(proof.pi_c) },
    y: publicSignals[2],
  };
}
