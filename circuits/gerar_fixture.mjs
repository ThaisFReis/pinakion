// Writes tests/prova-claim.json: this setup's vk plus one proof by
// `pinakion prove`, for juror 3 of the tree of 17 in tests/vetores-arvore.json
// (T-036). contracts/groth16-verifier's tests read it. GPL-3.0.
//
//   node circuits/gerar_fixture.mjs
import { createHash } from "node:crypto";
import { readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import { entrada, provar } from "./pinakion.mjs";

const RAIZ = join(dirname(fileURLToPath(import.meta.url)), "..");
const ler = (p) => JSON.parse(readFileSync(join(RAIZ, p), "utf8"));
const r = BigInt("0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");
const sha = (x) => BigInt("0x" + createHash("sha256").update(x).digest("hex"));

const a17 = ler("tests/vetores-arvore.json").arvores.find((a) => a.n === 17);
const s3 = sha("pinakion:arvore:17:s:3") % r;
const SEED = sha("pinakion:t036:seed") >> 8n;
const ASSENTO = "GD7ZNPRHTOTUPWH7NZF772OV6HNGQ3M6AVYPFY2MXRGPW4BSG5YXEFHH";
const prova = await provar(entrada(a17.eventos, s3, SEED, ASSENTO));

const vk = ler("circuits/setup/vk.json");
const be32 = (n) => BigInt(n).toString(16).padStart(64, "0");
const g1 = (p) => be32(p[0]) + be32(p[1]);
const g2 = (p) => be32(p[0][1]) + be32(p[0][0]) + be32(p[1][1]) + be32(p[1][0]);
const out = {
  _: "One proof of circuits/pinakion_claim.circom under the key in circuits/setup/ (T-036): juror 3 of the 17-juror tree in tests/vetores-arvore.json, seed sha256('pinakion:t036:seed') >> 8, seat account " + ASSENTO + " (a valid strkey, public key sha256('pinakion:t037:assento')). Written by node circuits/gerar_fixture.mjs.",
  vk: {
    alpha: g1(vk.vk_alpha_1), beta: g2(vk.vk_beta_2), gamma: g2(vk.vk_gamma_2),
    delta: g2(vk.vk_delta_2), ic: vk.IC.map(g1),
  },
  proof: prova.proof,
  public: prova.public,
  decimal: prova.decimal,
};
writeFileSync(join(RAIZ, "tests", "prova-claim.json"), JSON.stringify(out, null, 2) + "\n");
console.log("wrote tests/prova-claim.json");
process.exit(0);
