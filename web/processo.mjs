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
  const conferencias = sorteio.conferencias.map(([n, ok]) => ["sorteio: " + n, ok]);
  const confere = (nome, ok) => conferencias.push([nome, Boolean(ok)]);
  const um = (nome, lista = proc) => lista.find((e) => e.evento === nome);
  const painel = sorteio.painel.map(String);

  const fases = { sorteio: sorteio.completo && conferencias.every(([, ok]) => ok) };
  const resposta = (extra = {}) => ({ fases, conferencias, estados: sorteio.estados, sorteio, presentes: [], documentos: [], ...extra });
  const inst = um("Instituido"), aberto = um("CasoAberto"), fechado = um("Fechado", klero);
  confere("houve a regra (Instituido)", inst);
  confere("o caso abriu", aberto);
  if (!(inst && aberto && fechado)) return resposta();
  const contratos = registro.contratos || {};
  if (Object.keys(contratos).length) {
    confere("a regra aponta para este kleroterion", inst.kleroterion === contratos.kleroterion);
    confere("a regra aponta para este Tessera", inst.tessera === contratos.tessera);
  }
  confere("o caso abriu depois do fechamento", evs.indexOf(aberto) > evs.indexOf(fechado));
  confere("a parte e a do fechamento", aberto.acusado === fechado.acusado);
  confere("o nivel e o do fechamento", aberto.gravidade === fechado.gravidade);
  const k = fechado.k;
  confere("k e o do fechamento", aberto.k === k);
  confere("o quorum e a maioria do painel, nunca abaixo do piso de votos",
    aberto.quorum === Math.max(Math.floor(k / 2) + 1, inst.votos_minimos));
  const exigidos = fechado.gravidade >= inst.dois_tercos_desde ? Math.ceil((2 * k) / 3) : Math.floor(k / 2) + 1;
  confere("os votos exigidos seguem a regra do nivel", aberto.exigidos === exigidos);
  fases.abertura = true;

  const sentados = new Set(klero.filter((e) => e.evento === "Reivindicado" && painel.includes(String(e.y))).map((e) => e.assento));
  const presentes = [];
  let sessao = null;
  for (const e of proc) {
    if (e.evento === "Compareceu") {
      const quem = e.assento.slice(0, 8);
      confere(`presenca de ${quem}: e um assento que sentou`, sentados.has(e.assento));
      confere(`presenca de ${quem}: uma vez so`, !presentes.includes(e.assento));
      presentes.push(e.assento);
      confere(`presenca de ${quem}: a contagem confere`, e.presentes === presentes.length);
    } else if (e.evento === "SessaoInstalada") {
      sessao = e;
      confere("a sessao se instala exatamente no quorum",
        presentes.length === aberto.quorum && aberto.quorum === e.quorum && e.quorum === e.presentes);
    }
  }
  confere("a parte nunca compareceu como jurada", !presentes.includes(aberto.acusado));
  fases.quorum = sessao !== null;

  const regs = proc.filter((e) => e.evento === "Registrado");
  confere("o primeiro registro e a denuncia, do escrivao", regs.length && regs[0].tipo === 0 && regs[0].papel === 0);
  const escrivao = regs.length ? regs[0].autor : null;
  const documentos = [];
  regs.forEach((r, i) => {
    confere(`registro ${i}: em sequencia`, r.seq === i);
    const antes = proc.slice(0, proc.indexOf(r)).filter((e) => e.evento === "Compareceu").map((e) => e.assento);
    const papelOk = { 0: r.autor === escrivao && [0, 1, 3].includes(r.tipo),
      1: r.autor === aberto.acusado && r.tipo === 2,
      2: antes.includes(r.autor) && r.tipo === 3 }[r.papel] ?? false;
    confere(`registro ${i}: o papel pode registrar este tipo`, papelOk);
    if (docs) {
      confere(`registro ${i}: o documento confere com o hash`, r.hash in docs);
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
    confere("a votacao vinculada e a do Tessera registrada", vinc.proposta === t.id);
    // Tessera's CLI derives the id from a name; its dapp draws it at random.
    if (t.nome) {
      const idEsperado = hex(await sha256(juntar(enc.encode("TESSERA-V1-PROPOSTA"), enc.encode(t.nome))));
      confere("o identificador e o do nome da votacao no Tessera", idEsperado === vinc.proposta);
    }
    confere("a raiz dos aptos e a dos presentes, pela regra do Tessera", vinc.raiz_aptos === raizPresentes);
    confere("o Tessera guarda essa mesma raiz", p.raiz_aptos === raizPresentes);
    confere("a lista de votantes do Tessera e a dos presentes, na ordem",
      JSON.stringify(t.aptos) === JSON.stringify(presentes));
    confere("a mesa do Tessera e de presentes", (p.mesa || []).every((m) => presentes.includes(m)));
    const qs = p.perguntas || [];
    confere("a cedula e uma pergunta sigilosa de duas opcoes",
      qs.length === 1 && qs[0].opcoes === 2 && qs[0].confidencial === true);
    if (sessao && "ledger" in sessao) confere("a votacao abriu depois da sessao", (p.abre_em || 0) >= sessao.ledger);
  }
  fases.veredito = Boolean(decid);
  if (decid) {
    const res = t.resultado || [];
    confere("o placar e o que o Tessera publicou", decid.sim === res[0] && decid.nao === res[1] && res.length === 2);
    confere("os votos exigidos sao os da abertura", decid.exigidos === aberto.exigidos);
    confere("procedente se e so se sim alcanca os exigidos", decid.procedente === decid.sim >= decid.exigidos);
    confere("o veredito veio depois da votacao", proc.indexOf(decid) > proc.indexOf(vinc));
  }
  fases.revelacao = klero.some((e) => e.evento === "Revelado");
  return resposta({ presentes, documentos, raizPresentes });
}
