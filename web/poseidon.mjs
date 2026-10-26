// Poseidon over BN254, circomlib-compatible, for browser and Node. No
// dependencies.
//
// The permutation is the Soroban host's (soroban-env-host 29.0.1,
// crypto/poseidon/poseidon.rs): per round, add the round constants, apply the
// S-box x^5 to every cell in a full round or to cell 0 in a partial one, then
// multiply by the MDS matrix. The sponge is circomlib's: state [0, ...inputs],
// one permutation, output cell 0. The constants come from
// tests/poseidon-bn254.json, the contract's own; the caller loads that file.
//
//   const { hash1, hash2 } = criarPoseidon(constantes);

export function criarPoseidon(constantes) {
  const R = BigInt(constantes.r);
  const D = BigInt(constantes.d);
  const prep = (p) => ({
    f: p.rounds_f,
    p: p.rounds_p,
    mds: p.mds.map((l) => l.map(BigInt)),
    rc: p.rc.map((l) => l.map(BigInt)),
  });
  const P = { 2: prep(constantes.t2), 3: prep(constantes.t3) };

  const mod = (x) => ((x % R) + R) % R;
  const pot = (b, e) => {
    let r = 1n;
    b = mod(b);
    while (e > 0n) {
      if (e & 1n) r = (r * b) % R;
      b = (b * b) % R;
      e >>= 1n;
    }
    return r;
  };

  function permutacao(estado) {
    const p = P[estado.length];
    const meio = p.f / 2;
    p.rc.forEach((rc, r) => {
      estado = estado.map((x, i) => mod(x + rc[i]));
      if (r < meio || r >= meio + p.p) estado = estado.map((x) => pot(x, D));
      else estado[0] = pot(estado[0], D);
      estado = p.mds.map((linha) => mod(linha.reduce((s, m, i) => s + m * estado[i], 0n)));
    });
    return estado;
  }

  function hash(...entradas) {
    for (const x of entradas) {
      if (x < 0n || x >= R) throw new Error("Poseidon: input outside the field");
    }
    return permutacao([0n, ...entradas])[0];
  }

  return { R, hash1: (a) => hash(a), hash2: (a, b) => hash(a, b) };
}
