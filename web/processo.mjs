// The whole case, checked from the ledger in the browser (T-050).
//
// verificarProcesso(registro, { P, raiz, docs }) mirrors
// scripts/verificar_processo.py check for check, by the same names, so the
// page and the command line reach the same count. `P` and `raiz` are the
// Poseidon and tree helpers verificarSelado needs (./poseidon.mjs,
// ./arvore.mjs); `docs` maps a document's SHA-256 (hex) to its name, or is
// null to skip the documents.
//
// The vote is Tessera's, the author's earlier secret-ballot module, used as
// it is. What is checked here is the binding: the voter list Tessera stored
// is the list of attendees, under Tessera's own Merkle rule; and the verdict
// is Tessera's published score under the level's rule. That the score is the
// correct tally of the ballots is Tessera's own verifier's job.

import { verificarSelado } from "./sorteio.mjs";

const enc = new TextEncoder();

export async function sha256(bytes) {
  return new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
}

export const hex = (b) => [...b].map((x) => x.toString(16).padStart(2, "0")).join("");

const juntar = (...partes) => {
  const out = new Uint8Array(partes.reduce((n, p) => n + p.length, 0));
  let i = 0;
  for (const p of partes) { out.set(p, i); i += p.length; }
  return out;
};

const u32 = (n) => new Uint8Array([(n >>> 24) & 255, (n >>> 16) & 255, (n >>> 8) & 255, n & 255]);

function base32(s) {
  const A = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
  let bits = 0, valor = 0;
  const out = [];
  for (const c of s) {
    valor = (valor << 5) | A.indexOf(c);
    bits += 5;
    if (bits >= 8) { out.push((valor >>> (bits - 8)) & 255); bits -= 8; }
  }
  return new Uint8Array(out);
}

/** ScVal of an address, as Address::to_xdr writes it (Tessera's leaf). */
export function xdrDoEndereco(s) {
  const dado = base32(s);
  const chave = dado.slice(1, 33);
  return dado[0] === 6 << 3 ? juntar(u32(18), u32(0), u32(0), chave) : juntar(u32(18), u32(1), chave);
}

/** Tessera's voter root (tessera/core/src/merkle.rs at 356c22f), weight 1. */
export async function raizDosAptos(aptos) {
  let nivel = [];
  for (const a of aptos) nivel.push(await sha256(juntar(new Uint8Array([0]), xdrDoEndereco(a), u32(1))));
  const vazio = await sha256(juntar(new Uint8Array([2]), enc.encode("TESSERA-V1-MERKLE-VAZIO")));
  if (!nivel.length) return hex(vazio);
  let alvo = 1;
  while (alvo < nivel.length) alvo *= 2;
  while (nivel.length < alvo) nivel.push(vazio);
  while (nivel.length > 1) {
    const acima = [];
    for (let i = 0; i < nivel.length; i += 2) acima.push(await sha256(juntar(new Uint8Array([1]), nivel[i], nivel[i + 1])));
    nivel = acima;
  }
  return hex(nivel[0]);
}

export async function verificarProcesso(registro, { P, raiz, docs = null }) {
  const evs = registro.eventos;
  const klero = evs.filter((e) => e.contrato === "kleroterion");
  const proc = evs.filter((e) => e.contrato === "processo");
  const sorteio = verificarSelado({ eventos: klero, aleatoriedade: registro.aleatoriedade }, P, raiz);
  const conferencias = sorteio.conferencias.map(([n, ok]) => ["draw: " + n, ok]);
  const confere = (nome, ok) => conferencias.push([nome, Boolean(ok)]);
  const um = (nome, lista = proc) => lista.find((e) => e.evento === nome);
  const painel = sorteio.painel.map(String);

  const fases = { sorteio: sorteio.completo && conferencias.every(([, ok]) => ok) };
  const resposta = (extra = {}) => ({ fases, conferencias, estados: sorteio.estados, sorteio, presentes: [], documentos: [], ...extra });
  const inst = um("Instituido"), aberto = um("CasoAberto"), fechado = um("Fechado", klero);
  confere("there was a rule (Instituido)", inst);
  confere("the case opened", aberto);
  if (!(inst && aberto && fechado)) return resposta();
  const contratos = registro.contratos || {};
  if (Object.keys(contratos).length) {
    confere("the rule points to this kleroterion", inst.kleroterion === contratos.kleroterion);
    confere("the rule points to this Tessera", inst.tessera === contratos.tessera);
  }
  confere("the case opened after the lock", evs.indexOf(aberto) > evs.indexOf(fechado));
  confere("the party is the one in the lock", aberto.acusado === fechado.acusado);
  confere("the level is the one in the lock", aberto.gravidade === fechado.gravidade);
  const k = fechado.k;
  confere("k is the one in the lock", aberto.k === k);
  confere("the quorum is a majority of the panel, never below the vote floor",
    aberto.quorum === Math.max(Math.floor(k / 2) + 1, inst.votos_minimos));
  const exigidos = fechado.gravidade >= inst.dois_tercos_desde ? Math.ceil((2 * k) / 3) : Math.floor(k / 2) + 1;
  confere("the required votes follow the level rule", aberto.exigidos === exigidos);
  fases.abertura = true;

  const sentados = new Set(klero.filter((e) => e.evento === "Reivindicado" && painel.includes(String(e.y))).map((e) => e.assento));
  const presentes = [];
  let sessao = null;
  for (const e of proc) {
    if (e.evento === "Compareceu") {
      const quem = e.assento.slice(0, 8);
      confere(`attendance of ${quem}: a seat that sat`, sentados.has(e.assento));
      confere(`attendance of ${quem}: once only`, !presentes.includes(e.assento));
      presentes.push(e.assento);
      confere(`attendance of ${quem}: the count matches`, e.presentes === presentes.length);
    } else if (e.evento === "SessaoInstalada") {
      sessao = e;
      confere("the session opens exactly at the quorum",
        presentes.length === aberto.quorum && aberto.quorum === e.quorum && e.quorum === e.presentes);
    }
  }
  confere("the party never attended as a juror", !presentes.includes(aberto.acusado));
  fases.quorum = sessao !== null;

  const regs = proc.filter((e) => e.evento === "Registrado");
  confere("the first filing is the complaint, from the clerk", regs.length && regs[0].tipo === 0 && regs[0].papel === 0);
  const escrivao = regs.length ? regs[0].autor : null;
  const documentos = [];
  regs.forEach((r, i) => {
    confere(`filing ${i}: in sequence`, r.seq === i);
    const antes = proc.slice(0, proc.indexOf(r)).filter((e) => e.evento === "Compareceu").map((e) => e.assento);
    const papelOk = { 0: r.autor === escrivao && [0, 1, 3].includes(r.tipo),
      1: r.autor === aberto.acusado && r.tipo === 2,
      2: antes.includes(r.autor) && r.tipo === 3 }[r.papel] ?? false;
    confere(`filing ${i}: the role may file this type`, papelOk);
    if (docs) {
      confere(`filing ${i}: the document matches the hash`, r.hash in docs);
      documentos.push(docs[r.hash] || null);
    }
  });
  fases.autos = regs.length > 0;

  const vinc = um("VotacaoVinculada"), decid = um("Decidido");
  const t = registro.tessera || {};
  fases.votacao = Boolean(vinc);
  let raizPresentes = null;
  if (vinc) {
    const p = t.proposta || {};
    raizPresentes = await raizDosAptos(presentes);
    confere("the bound vote is the recorded Tessera vote", vinc.proposta === t.id);
    // Tessera's CLI derives the id from a name; its dapp draws it at random.
    if (t.nome) {
      const idEsperado = hex(await sha256(juntar(enc.encode("TESSERA-V1-PROPOSTA"), enc.encode(t.nome))));
      confere("the identifier comes from the vote name in Tessera", idEsperado === vinc.proposta);
    }
    confere("the voter root is the root of the attendees, under the Tessera rule", vinc.raiz_aptos === raizPresentes);
    confere("Tessera stores that same root", p.raiz_aptos === raizPresentes);
    confere("the Tessera voter list is the attendees, in order",
      JSON.stringify(t.aptos) === JSON.stringify(presentes));
    confere("the Tessera committee is made of attendees", (p.mesa || []).every((m) => presentes.includes(m)));
    const qs = p.perguntas || [];
    confere("the ballot is one confidential question with two options",
      qs.length === 1 && qs[0].opcoes === 2 && qs[0].confidencial === true);
    if (sessao && "ledger" in sessao) confere("the vote opened after the session", (p.abre_em || 0) >= sessao.ledger);
  }
  fases.veredito = Boolean(decid);
  if (decid) {
    const res = t.resultado || [];
    confere("the score is the one Tessera published", decid.sim === res[0] && decid.nao === res[1] && res.length === 2);
    confere("the required votes are the ones fixed at opening", decid.exigidos === aberto.exigidos);
    confere("upheld if and only if yes reaches the required votes", decid.procedente === decid.sim >= decid.exigidos);
    confere("the verdict came after the vote", proc.indexOf(decid) > proc.indexOf(vinc));
  }
  fases.revelacao = klero.some((e) => e.evento === "Revelado");
  return resposta({ presentes, documentos, raizPresentes });
}
