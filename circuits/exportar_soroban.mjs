// Copied from the author's withdrawn generation, scripts/export_soroban.mjs
// (2026-10-25, DEC-017). Unchanged except this header and the usage line.
// Converts snarkjs Groth16 artifacts into the byte layout the Soroban BN254 host
// functions expect: G1 = be(x) || be(y) (64 bytes); G2 = be(x.c1) || be(x.c0) ||
// be(y.c1) || be(y.c0) (128 bytes); Fr = be (32 bytes).
// Usage: node circuits/exportar_soroban.mjs <vk.json> <proof.json> <public.json> <out.json>
import { readFileSync, writeFileSync } from "node:fs";

const [vkPath, proofPath, publicPath, outPath] = process.argv.slice(2);
const vk = JSON.parse(readFileSync(vkPath, "utf8"));
const proof = JSON.parse(readFileSync(proofPath, "utf8"));
const pub = JSON.parse(readFileSync(publicPath, "utf8"));

const be32 = (n) => BigInt(n).toString(16).padStart(64, "0");
// snarkjs stores affine points with a trailing "1" (projective z); G2 components are [c0, c1].
const g1 = (p) => be32(p[0]) + be32(p[1]);
const g2 = (p) => be32(p[0][1]) + be32(p[0][0]) + be32(p[1][1]) + be32(p[1][0]);

const out = {
  vk: {
    alpha: g1(vk.vk_alpha_1),
    beta: g2(vk.vk_beta_2),
    gamma: g2(vk.vk_gamma_2),
    delta: g2(vk.vk_delta_2),
    ic: vk.IC.map(g1),
  },
  proof: { a: g1(proof.pi_a), b: g2(proof.pi_b), c: g1(proof.pi_c) },
  public: pub.map(be32),
};
writeFileSync(outPath, JSON.stringify(out, null, 2));
console.log(`wrote ${outPath}: ${out.vk.ic.length} IC points, ${out.public.length} public inputs`);
