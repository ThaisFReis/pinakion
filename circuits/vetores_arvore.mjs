// Vetores da árvore de compromissos (T-035), calculados pela circomlibjs —
// a árvore que o circuito vê. O contrato, scripts/arvore.py e web/arvore.mjs
// têm de chegar à mesma raiz a partir dos mesmos eventos. GPL-3.0 (ver
// circuits/LICENSE); o que sai são números.
//
//   node circuits/vetores_arvore.mjs > tests/vetores-arvore.json
//
// Folha i = c_i = Poseidon(s_i), na ordem de inscrição; um impedido antes do
// fechamento vira folha 0; folhas vazias são 0; profundidade 6.
import { buildPoseidon } from "circomlibjs";
import { createHash } from "node:crypto";

const DEPTH = 6;
const r = BigInt("0x30644e72e131a029b85045b68181585d2833e84879b9709143e1f593f0000001");
const sha = (x) => BigInt("0x" + createHash("sha256").update(x).digest("hex"));
const p = await buildPoseidon();
const h = (xs) => p.F.toObject(p(xs));

function raiz(folhas) {
  let nivel = folhas.slice();
  while (nivel.length < 2 ** DEPTH) nivel.push(0n);
  for (let d = 0; d < DEPTH; d++) {
    const prox = [];
    for (let i = 0; i < nivel.length; i += 2) prox.push(h([nivel[i], nivel[i + 1]]));
    nivel = prox;
  }
  return nivel[0];
}

// [N, índice impedido antes do fechamento, ou null]
const casos = [[1, null], [2, 1], [17, 5], [64, 63]];
const arvores = casos.map(([n, impedido]) => {
  const cs = Array.from({ length: n }, (_, i) => h([sha(`pinakion:arvore:${n}:s:${i}`) % r]));
  const eventos = cs.map((c, i) => ({ evento: "enroll", indice: i, c: c.toString() }));
  const folhas = cs.slice();
  if (impedido !== null) {
    eventos.push({ evento: "impede", indice: impedido });
    folhas[impedido] = 0n;
  }
  return {
    nome: `rol de ${n}` + (impedido === null ? "" : `, impedido o ${impedido}`),
    n,
    autorizados: n - (impedido === null ? 0 : 1),
    eventos,
    raiz: raiz(folhas).toString(),
  };
});
const zeros = [0n];
for (let d = 0; d < DEPTH; d++) zeros.push(h([zeros[d], zeros[d]]));
process.stdout.write(JSON.stringify({
  _: "Commitment-tree vectors (T-035), computed with circomlibjs 0.1.7 by circuits/vetores_arvore.mjs. Leaf i is the commitment c_i in enrolment order; an impediment before lock zeroes its leaf; empty leaves are 0; depth 6. The contract (contracts/kleroterion/src/test.rs), scripts/arvore.py (G6) and web/arvore.mjs (G5) must reach each root from the events.",
  profundidade: DEPTH,
  zeros: zeros.map(String),
  arvores,
}, null, 1) + "\n");
