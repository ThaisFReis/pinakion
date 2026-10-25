// Recomputação do sorteio do Soroban, sem dependências.
// Roda no navegador e no Node. A única coisa de fora é o HMAC-SHA256,
// que vem da WebCrypto padrão.
//
// Receita lida do host (rs-soroban-env):
//   1. chave = HMAC-SHA256(SALT, semente)   [crypto/mod.rs: unbias_prng_seed]
//   2. ChaCha20Rng::from_seed(chave)        [rand_chacha 0.3.1]
//   3. Fisher-Yates de rand 0.8.5, índices por amostragem de Lemire
//
// SALT = sha256("Public Global Stellar Network ; September 2015")

export const SALT = hexToBytes(
  "7ac33997544e3175d266bd022439b22cdb16508c01163f26e5cb2a3e1045a979",
);

export function hexToBytes(h) {
  const clean = h.trim().replace(/^0x/, "");
  if (clean.length % 2) throw new Error("hex de tamanho ímpar");
  const out = new Uint8Array(clean.length / 2);
  for (let i = 0; i < out.length; i++) {
    out[i] = parseInt(clean.substr(i * 2, 2), 16);
  }
  return out;
}

const rotl = (v, n) => ((v << n) | (v >>> (32 - n))) >>> 0;

function quarterRound(s, a, b, c, d) {
  s[a] = (s[a] + s[b]) >>> 0; s[d] = rotl(s[d] ^ s[a], 16);
  s[c] = (s[c] + s[d]) >>> 0; s[b] = rotl(s[b] ^ s[c], 12);
  s[a] = (s[a] + s[b]) >>> 0; s[d] = rotl(s[d] ^ s[a], 8);
  s[c] = (s[c] + s[d]) >>> 0; s[b] = rotl(s[b] ^ s[c], 7);
}

function chacha20Block(key, counter) {
  const st = new Uint32Array(16);
  st[0] = 0x61707865; st[1] = 0x3320646e; st[2] = 0x79622d32; st[3] = 0x6b206574;
  const dv = new DataView(key.buffer, key.byteOffset, key.byteLength);
  for (let i = 0; i < 8; i++) st[4 + i] = dv.getUint32(i * 4, true);
  st[12] = counter >>> 0;                      // nonce de 12 bytes fica em zero
  const w = st.slice();
  for (let i = 0; i < 10; i++) {
    quarterRound(w, 0, 4, 8, 12); quarterRound(w, 1, 5, 9, 13);
    quarterRound(w, 2, 6, 10, 14); quarterRound(w, 3, 7, 11, 15);
    quarterRound(w, 0, 5, 10, 15); quarterRound(w, 1, 6, 11, 12);
    quarterRound(w, 2, 7, 8, 13); quarterRound(w, 3, 4, 9, 14);
  }
  const out = new Uint32Array(16);
  for (let i = 0; i < 16; i++) out[i] = (w[i] + st[i]) >>> 0;
  return out;
}

class ChaCha20Rng {
  constructor(key) { this.key = key; this.counter = 0; this.buf = null; this.pos = 16; }
  nextU32() {
    if (this.pos >= 16) {
      this.buf = chacha20Block(this.key, this.counter++);
      this.pos = 0;
    }
    return this.buf[this.pos++];
  }
}

// rand 0.8.5: UniformInt<u32>::sample_single — método de Lemire com rejeição.
function genIndex(rng, ubound) {
  if (ubound === 0) return rng.nextU32();
  const range = BigInt(ubound);
  const lz = Math.clz32(ubound);
  const zone = ((range << BigInt(lz)) & 0xffffffffn) - 1n;
  for (;;) {
    const v = BigInt(rng.nextU32());
    const prod = v * range;
    const hi = prod >> 32n;
    const lo = prod & 0xffffffffn;
    if (lo <= zone) return Number(hi);
  }
}

// rand 0.8.5 SliceRandom::shuffle — Fisher-Yates de trás para a frente.
function shuffle(arr, rng) {
  for (let i = arr.length - 1; i > 0; i--) {
    const j = genIndex(rng, i + 1);
    [arr[i], arr[j]] = [arr[j], arr[i]];
  }
}

async function hmacSha256(keyBytes, msgBytes) {
  const k = await crypto.subtle.importKey(
    "raw", keyBytes, { name: "HMAC", hash: "SHA-256" }, false, ["sign"],
  );
  return new Uint8Array(await crypto.subtle.sign("HMAC", k, msgBytes));
}

/** Recomputa o painel: devolve as `k` primeiras posições de um rol de `n`. */
export async function sortear(sementeHex, n, k) {
  const chave = await hmacSha256(SALT, hexToBytes(sementeHex));
  const rng = new ChaCha20Rng(chave);
  const roll = Array.from({ length: n }, (_, i) => i);
  shuffle(roll, rng);
  return roll.slice(0, k);
}

/**
 * O painel do kleroterion: embaralha a ordem do rol inteiro e senta os `k`
 * primeiros que não estão em `inelegiveis` (impedidos ou sem bilhete). Não há
 * novo sorteio para substituir: o substituto é o próximo da ordem.
 */
export async function painel(sementeHex, rol, inelegiveis, k) {
  const fora = new Set(inelegiveis);
  const ordem = await sortear(sementeHex, rol.length, rol.length);
  const sentados = ordem.map((i) => rol[i]).filter((a) => !fora.has(a));
  if (sentados.length < k) throw new Error(`só ${sentados.length} elegíveis para ${k} assentos`);
  return sentados.slice(0, k);
}

/** A aleatoriedade de uma rodada é o sha256 da assinatura. */
export async function aleatoriedadeDaAssinatura(sigHex) {
  const d = await crypto.subtle.digest("SHA-256", hexToBytes(sigHex));
  return [...new Uint8Array(d)].map((b) => b.toString(16).padStart(2, "0")).join("");
}

// ---- the sealed draw (T-038 onwards, T-039) --------------------------------
//
// verificarSelado(registro, P, raiz): recomputes a kleroterion panel from the
// contract's events and the drand randomness alone, without trusting the
// contract — the same checks as scripts/verificar_painel.py. `P` is
// criarPoseidon(constantes) from ./poseidon.mjs and `raiz` is raiz() from
// ./arvore.mjs, passed in so this file keeps no imports of its own.
//
// registro = { aleatoriedade: hex, eventos: [...] }, events named as the
// contract names them. Returns { estados, painel, completo, semRevelacao,
// conferencias: [[name, ok], ...] }.
export function verificarSelado(registro, P, raiz) {
  const { eventos } = registro;
  const conferencias = [];
  const confere = (nome, ok) => conferencias.push([nome, Boolean(ok)]);
  const um = (nome) => eventos.find((e) => e.evento === nome);

  const jurados = new Map();
  const ordem = [];
  const impedidos = new Set();
  const folhas = [];
  for (const e of eventos) {
    if (e.evento === "Inscrito") {
      jurados.set(e.jurado, { indice: e.indice, c: BigInt(e.c) });
      ordem.push(e.jurado);
      folhas.push({ evento: "enroll", indice: e.indice, c: e.c });
    } else if (e.evento === "Impedido") {
      impedidos.add(e.jurado);
      folhas.push({ evento: "impede", indice: e.indice });
    }
  }
  const vazio = { estados: new Map(), painel: [], completo: false, semRevelacao: 0, conferencias };

  const fechado = um("Fechado");
  confere("houve fechamento", fechado);
  if (!fechado) return vazio;
  confere("a raiz do fechamento e a dos compromissos", String(raiz(folhas, P.hash2)) === fechado.raiz);
  const n = BigInt(fechado.n);
  const k = fechado.k;
  confere("N e quem estava autorizado", fechado.n === ordem.length - impedidos.size);

  const seed = BigInt("0x" + registro.aleatoriedade) >> 8n;
  let limiar = 2n * BigInt(k) >= n ? P.R : (P.R * 2n * BigInt(k)) / n;
  const selado = um("Selado");
  confere("houve selo", selado);
  if (!selado) return vazio;
  confere("a semente e a da rodada drand >> 8", BigInt(selado.seed) === seed);
  confere("o limiar e floor(r*2k/N)", BigInt(selado.limiar) === limiar);

  const reivindicados = [];
  const recusados = new Map();
  const revelados = new Map();
  for (const e of eventos) {
    if (e.evento === "Ampliado") {
      limiar = P.R;
      confere("a ampliacao leva o limiar ao campo", BigInt(e.limiar) === P.R);
    } else if (e.evento === "Reivindicado") {
      const y = BigInt(e.y);
      confere(`reivindicacao ${y} abaixo do limiar`, y < limiar);
      confere(`reivindicacao ${y} unica`, !reivindicados.includes(y));
      reivindicados.push(y);
    } else if (e.evento === "Recusado" || e.evento === "Revelado") {
      const s = BigInt(e.s);
      const y = BigInt(e.y);
      const c = jurados.get(e.jurado)?.c;
      confere(`${e.evento} de ${e.jurado}: Poseidon(s) = c`, c !== undefined && P.hash1(s) === c);
      confere(`${e.evento} de ${e.jurado}: y = Poseidon(seed, s)`, P.hash2(seed, s) === y);
      if (e.evento === "Recusado") recusados.set(e.jurado, y);
      else revelados.set(e.jurado, e);
    }
  }

  const fora = new Set(recusados.values());
  const restantes = reivindicados.filter((y) => !fora.has(y)).sort((a, b) => (a < b ? -1 : a > b ? 1 : 0));
  const painel = restantes.slice(0, k);
  const final = um("Finalizado");
  confere("houve painel", final);
  if (final) {
    confere(
      "o painel sao os k menores y que ficaram",
      final.painel.length === painel.length && final.painel.every((v, i) => BigInt(v) === painel[i]),
    );
    confere("completo se e so se chegou a k", final.completo === (painel.length === k));
  }
  for (const [quem, e] of revelados) {
    confere(`Revelado de ${quem}: sentado confere`, e.sentado === painel.includes(BigInt(e.y)));
  }

  const estados = new Map();
  for (const quem of ordem) {
    if (impedidos.has(quem)) estados.set(quem, "impedido");
    else if (recusados.has(quem)) estados.set(quem, "recusado");
    else if (!revelados.has(quem)) estados.set(quem, "nao revelou");
    else {
      const y = BigInt(revelados.get(quem).y);
      if (painel.includes(y)) estados.set(quem, "sentou");
      else if (y < limiar && !reivindicados.includes(y)) estados.set(quem, "sorteado e ausente");
      else estados.set(quem, "revelou, nao sentou");
    }
  }
  const sentou = [...estados.values()].filter((e) => e === "sentou").length;
  return { estados, painel, completo: painel.length === k, semRevelacao: painel.length - sentou, conferencias };
}
