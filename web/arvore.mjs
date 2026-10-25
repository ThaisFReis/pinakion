// Recompute kleroterion's commitment root from its events (T-035). No
// dependencies beyond web/poseidon.mjs.
//
// Events in ledger order: {evento: "enroll", indice, c} or
// {evento: "impede", indice}. Leaf i is c_i; an impediment zeroes it; empty
// leaves are 0; depth 6. The result must equal closing().raiz.
//
//   raiz(eventos, hash2) -> BigInt

export const PROFUNDIDADE = 6;

export function zeros(hash2) {
  const z = [0n];
  for (let d = 0; d < PROFUNDIDADE; d++) z.push(hash2(z[d], z[d]));
  return z;
}

export function raiz(eventos, hash2) {
  let nivel = new Map();
  for (const e of eventos) {
    if (e.evento === "enroll") nivel.set(e.indice, BigInt(e.c));
    else if (e.evento === "impede") nivel.set(e.indice, 0n);
    else throw new Error(`evento desconhecido: ${e.evento}`);
  }
  const z = zeros(hash2);
  for (let d = 0; d < PROFUNDIDADE; d++) {
    const acima = new Map();
    for (const i of new Set([...nivel.keys()].map((i) => i >> 1))) {
      const esq = nivel.has(2 * i) ? nivel.get(2 * i) : z[d];
      const dir = nivel.has(2 * i + 1) ? nivel.get(2 * i + 1) : z[d];
      acima.set(i, hash2(esq, dir));
    }
    nivel = acima;
  }
  return nivel.has(0) ? nivel.get(0) : z[PROFUNDIDADE];
}
