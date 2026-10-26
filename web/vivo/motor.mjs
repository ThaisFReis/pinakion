// A whole case, live on testnet, run from the browser (T-053).
//
// rodarCaso(deps, opcoes, aviso) does every step of scripts/sequencia_processo.sh
// as real transactions, signed in this tab with keys born in this tab:
// accounts, the ticket asset, the contracts, 40 enrolments, the lock against
// an enrolled party, the record, the drand seal, zero-knowledge seat proofs
// made here, the panel, attendance, a Tessera vote bound to the attendees,
// the secret ballots, Tessera's time-locked tally, the verdict, the reveals —
// and then reads it all back from the ledger as a record, which
// web/processo.mjs checks without trusting any contract.
//
// The vote is Tessera's, the author's earlier secret-ballot module, used as it
// is: its contract at commit 356c22f (deployed once, below) and its wasm client
// at the same commit, built unchanged (web/vivo/tessera/).
//
// `deps` are injected so the same module runs in a browser and in Node:
//   sdk      @stellar/stellar-sdk 16.3.1
//   snarkjs  snarkjs 0.7.6
//   tessera  Tessera's wasm client, initialised
//   P        criarPoseidon(...) from ../poseidon.mjs
//   provar   { entrada, provar } from ../../circuits/navegador.mjs
//   circuito { wasm, zkey }: URLs (browser) or paths (Node)
// `opcoes.textos` are the four documents typed before the case opens;
// `opcoes.escolherVotos(presentes)` resolves to one choice per attendee (0 yes, 1 no).
// `aviso(e)` receives the progress: { fase, passo, tx, nota, erro, dado }.

const RPC = "https://soroban-testnet.stellar.org";
const HORIZON = "https://horizon-testnet.stellar.org";
const FRIENDBOT = "https://friendbot.stellar.org";
const DRAND = "https://api.drand.sh";
const CADEIA = "52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971";
const GENESIS = 1692803367;
const PERIODO = 3;

// Installed on testnet by scripts/sequencia_processo.sh (run caso1); the hash is
// the SHA-256 of target/wasm32v1-none/release/<name>.wasm.
export const WASM = {
  kleroterion: "ea8a034a02a86c0a84881d55fcdc87d450b914d3ea37f38c82f5224cf0182bd1",
  processo: "5ac11382ee3f4b5e13c62f631a4b96b010a76b211443af04da4b0e3b760f13b0",
};
export const VERIFICADOR = "CBEX5AEAUDKK4SUZGXQPM26QZHNZZ5X3ELCSIQJ3V6SZKUCQ4B57NI5F";
export const TESSERA = "CDWBWAHUZ3MC3BJXJIW5O74MNUXH76VCGBIVFQU3XJJFGML3VKFIXQT7";

// The approved severity table (docs/desenho-gravidade-afastamento-revelacao.md).
const NIVEIS = [{ k: 3, rol_minimo: 16 }, { k: 5, rol_minimo: 40 }, { k: 7, rol_minimo: 40 }, { k: 9, rol_minimo: 100 }];
export const PLANO = {
  jurados: 40, gravidade: 3, k: 7,
  // Demo windows, in ledgers (~5.5 s each on testnet): short enough to watch.
  // The approved reveal floor is 30 days; here it is minutes, and the page says so.
  janela: 40, afastamento: 400, espera: 60, ateRevelar: 120, prazoQuorum: 40,
  votosMinimos: 5, doisTercosDesde: 4,
  aberturaVoto: 16, duracaoVoto: 12,
};

const R_BN254 = 21888242871839275222246405745257275088548364400416034343698204186575808495617n;
const TOPICOS = { Inscrito: ["jurado"], Impedido: ["jurado"], Recusado: ["jurado"], Revelado: ["jurado"], Fechado: ["acusado"], CasoAberto: ["acusado"] };
const NOMES = {
  kleroterion: ["Constituido", "Inscrito", "Impedido", "Fechado", "Selado", "Ampliado", "Reivindicado", "Recusado", "Finalizado", "Revelado"],
  processo: ["Instituido", "CasoAberto", "Registrado", "Compareceu", "SessaoInstalada", "Adiado", "VotacaoVinculada", "Decidido"],
};

const dormir = (ms) => new Promise((r) => setTimeout(r, ms));
const hex = (b) => [...b].map((x) => x.toString(16).padStart(2, "0")).join("");
const deHex = (h) => Uint8Array.from(h.match(/../g).map((x) => parseInt(x, 16)));

async function sha256Hex(texto) {
  return hex(new Uint8Array(await crypto.subtle.digest("SHA-256", new TextEncoder().encode(texto))));
}

function segredo() {
  const b = crypto.getRandomValues(new Uint8Array(32));
  const s = BigInt("0x" + hex(b)) % R_BN254;
  return s === 0n ? 1n : s;
}

/** Runs `f` over `itens` with at most `n` at a time, keeping order. */
async function emLotes(itens, n, f) {
  const out = new Array(itens.length);
  let i = 0;
  await Promise.all(Array.from({ length: Math.min(n, itens.length) }, async () => {
    while (i < itens.length) { const j = i++; out[j] = await f(itens[j], j); }
  }));
  return out;
}

export async function rodarCaso(deps, opcoes, aviso = () => {}) {
  const { sdk, tessera: w, P } = deps;
  const { Keypair, Networks, TransactionBuilder, Operation, Asset, Address, contract, rpc, scValToNative, nativeToScVal, Contract, xdr } = sdk;
  const PASSA = Networks.TESTNET;
  const server = new rpc.Server(RPC);
  const B = (h) => (typeof Buffer !== "undefined" ? Buffer.from(h, "hex") : deHex(h));
  const fase = (id, texto) => aviso({ fase: id, texto });
  const nota = (texto) => aviso({ nota: texto });
  const txFeita = (rotulo, hash, extra = {}) => aviso({ tx: hash, rotulo, ...extra });
  const hashes = [];
  const marcar = (rotulo, hash) => { if (hash) { hashes.push([rotulo, hash]); txFeita(rotulo, hash); } return hash; };

  const ledger = async () => (await server.getLatestLedger()).sequence;
  const ateLedger = async (n) => { while ((await ledger()) <= n) await dormir(2500); };
  const ateInstante = async (t) => { while (Date.now() / 1000 < t) await dormir(1000); };

  // ---- sending -------------------------------------------------------------

  async function esperarTx(hash) {
    for (let i = 0; i < 90; i++) {
      const r = await server.getTransaction(hash);
      if (r.status === "SUCCESS") return r;
      if (r.status === "FAILED") throw new Error(`transação ${hash.slice(0, 8)}… falhou`);
      await dormir(1500);
    }
    throw new Error(`transação ${hash.slice(0, 8)}… não entrou num ledger`);
  }

  async function classico(kp, ops) {
    const conta = await server.getAccount(kp.publicKey());
    let b = new TransactionBuilder(conta, { fee: "100000", networkPassphrase: PASSA });
    for (const op of ops) b = b.addOperation(op);
    const tx = b.setTimeout(120).build();
    tx.sign(kp);
    const r = await server.sendTransaction(tx);
    if (r.status === "ERROR") throw new Error(`rede recusou: ${r.errorResult?.result().switch().name}`);
    await esperarTx(r.hash);
    return r.hash;
  }

  async function invocarBruto(kp, op) {
    const conta = await server.getAccount(kp.publicKey());
    const bruta = new TransactionBuilder(conta, { fee: "1000000", networkPassphrase: PASSA }).addOperation(op).setTimeout(120).build();
    const tx = await server.prepareTransaction(bruta);
    tx.sign(kp);
    const r = await server.sendTransaction(tx);
    if (r.status === "ERROR") throw new Error(`rede recusou: ${r.errorResult?.result().switch().name}`);
    await esperarTx(r.hash);
    return r.hash;
  }

  const especs = new Map();
  async function cliente(id, kp) {
    const base = { contractId: id, networkPassphrase: PASSA, rpcUrl: RPC };
    if (!especs.has(id)) especs.set(id, (await contract.Client.from(base)).spec);
    const assinar = contract.basicNodeSigner(kp, PASSA);
    return new contract.Client(especs.get(id), { ...base, publicKey: kp.publicKey(), ...assinar });
  }
  const OPC = { fee: 1000000, timeoutInSeconds: 120 };

  /** Simulates, signs (and the extra auth entries), sends; retries a busy ledger. */
  async function mandar(montar, extras = []) {
    for (let t = 0; ; t++) {
      try {
        const at = await montar();
        for (const kp of extras) {
          await at.signAuthEntries({ address: kp.publicKey(), signAuthEntry: contract.basicNodeSigner(kp, PASSA).signAuthEntry });
        }
        const enviada = await at.signAndSend();
        const hash = enviada.sendTransactionResponse?.hash ?? enviada.getTransactionResponse?.txHash;
        // signAndSend can return a transaction the ledger applied and failed: check.
        const estado = enviada.getTransactionResponse?.status;
        if (estado !== "SUCCESS") throw new Error(`transação ${String(hash).slice(0, 8)}… terminou em ${estado}`);
        // The SDK's `result` getter throws on a function that returns nothing.
        let valor;
        try { valor = enviada.result; } catch { valor = undefined; }
        return { hash, valor };
      } catch (e) {
        // A contract error is an answer, not a busy ledger: it goes up as is.
        if (t >= 2 || /Error\(Contract/.test(String(e?.message ?? e))) throw e;
        await dormir(3000);
      }
    }
  }
  const lerValor = async (montar) => (await montar()).result;

  // ---- 1. people and the ticket ------------------------------------------

  fase("contas", "Criando as contas de todas as pessoas do caso");
  const banco = Keypair.random();
  const fb = await fetch(`${FRIENDBOT}?addr=${banco.publicKey()}`);
  if (!fb.ok) throw new Error(`friendbot: ${fb.status}`);
  nota(`uma conta financiadora, pelo friendbot: ${banco.publicKey()}`);
  const escrivao = Keypair.random(), emissor = Keypair.random(), apurador = Keypair.random();
  const jurados = Array.from({ length: PLANO.jurados }, () => Keypair.random());
  const assentos = Array.from({ length: 26 }, () => Keypair.random());
  const efemeras = Array.from({ length: 10 }, () => Keypair.random());
  const contas = [[escrivao, "600"], [emissor, "60"], [apurador, "60"],
    ...jurados.map((k) => [k, "40"]), ...assentos.map((k) => [k, "40"]), ...efemeras.map((k) => [k, "40"])];
  for (let i = 0; i < contas.length; i += 50) {
    const h = await classico(banco, contas.slice(i, i + 50).map(([k, saldo]) =>
      Operation.createAccount({ destination: k.publicKey(), startingBalance: saldo })));
    marcar("contas", h);
  }

  fase("bilhete", "Emitindo o bilhete PINAKION com o emissor travado");
  const inicio = (await ledger()) - 2;
  marcar("emissor: AUTH_REQUIRED, AUTH_REVOCABLE, AUTH_IMMUTABLE", await classico(emissor, [Operation.setOptions({ setFlags: 1 | 2 | 4 })]));
  const ativo = new Asset("PINAKION", emissor.publicKey());
  const sac = ativo.contractId(PASSA);
  marcar("o contrato do ativo (SAC)", await invocarBruto(emissor, Operation.createStellarAssetContract({ asset: ativo })));

  // ---- 2. the contracts ----------------------------------------------------

  fase("contratos", "Implantando o rol (kleroterion) e o caso (processo)");
  const base = { networkPassphrase: PASSA, rpcUrl: RPC, publicKey: escrivao.publicKey(), ...contract.basicNodeSigner(escrivao, PASSA) };
  const dk = await (await contract.Client.deploy(
    { escrivao: escrivao.publicKey(), bilhete: sac, verificador: VERIFICADOR, janela: PLANO.janela, niveis: NIVEIS,
      afastamento: PLANO.afastamento, espera_minima: PLANO.espera },
    { ...base, wasmHash: WASM.kleroterion, ...OPC })).signAndSend();
  const klero = dk.result.options.contractId;
  marcar("kleroterion implantado", dk.sendTransactionResponse?.hash);
  aviso({ dado: "kleroterion", valor: klero });
  const dp = await (await contract.Client.deploy(
    { escrivao: escrivao.publicKey(), kleroterion: klero, tessera: TESSERA, prazo_quorum: PLANO.prazoQuorum,
      votos_minimos: PLANO.votosMinimos, dois_tercos_desde: PLANO.doisTercosDesde },
    { ...base, wasmHash: WASM.processo, ...OPC })).signAndSend();
  const proc = dp.result.options.contractId;
  marcar("processo implantado", dp.sendTransactionResponse?.hash);
  aviso({ dado: "processo", valor: proc });
  marcar("o rol passa a ser o dono do bilhete", await invocarBruto(emissor,
    new Contract(sac).call("set_admin", new Address(klero).toScVal())));
  marcar("emissor com peso 0: ninguém mais emite", await classico(emissor, [Operation.setOptions({ masterWeight: 0 })]));

  // ---- 3. the roster -------------------------------------------------------

  fase("rol", `Inscrevendo ${PLANO.jurados} pessoas, cada uma com um compromisso`);
  const segredos = jurados.map(() => segredo());
  // One at a time: each enrolment rewrites the roster and a path of the tree,
  // so two in the same ledger would each be simulated on the other's old state.
  for (const [i, kp] of jurados.entries()) {
    const r = await mandar(async () => (await cliente(klero, kp)).enroll({ jurado: kp.publicKey(), c: P.hash1(segredos[i]) }, OPC), [escrivao]);
    marcar(`inscrição ${i + 1} de ${jurados.length}`, r.hash);
  }
  const acusado = jurados[0];

  fase("fechamento", "Fechando o rol para julgar uma pessoa inscrita");
  const kEsc = await cliente(klero, escrivao);
  const revelaEm = (await ledger()) + PLANO.ateRevelar;
  const fech = await mandar(() => kEsc.lock({ gravidade: PLANO.gravidade, revela_em: revelaEm, acusado: acusado.publicKey() }, OPC));
  marcar("fechamento: a parte é impedida na mesma chamada", fech.hash);
  const fechamento = fech.valor;
  aviso({ dado: "fechamento", valor: { n: fechamento.n, k: fechamento.k, rodada: Number(fechamento.rodada), afastado_ate: fechamento.afastado_ate } });

  // ---- 4. the record -------------------------------------------------------

  fase("autos", "Abrindo o caso e registrando os autos");
  const pEsc = await cliente(proc, escrivao);
  const docs = {};
  const guardar = async (nome, texto) => { const h = await sha256Hex(texto); docs[h] = { nome, texto }; return h; };
  const hDen = await guardar("Denúncia", opcoes.textos.denuncia);
  marcar("abertura do caso, com a denúncia", (await mandar(() => pEsc.abrir({ denuncia: B(hDen) }, OPC))).hash);
  const hProva = await guardar("Prova", opcoes.textos.prova);
  marcar("prova, pelo escrivão", (await mandar(() => pEsc.registrar({ autor: escrivao.publicKey(), tipo: 1, hash: B(hProva) }, OPC))).hash);
  const hDef = await guardar("Defesa", opcoes.textos.defesa);
  const pAc = await cliente(proc, acusado);
  marcar("defesa, pela própria pessoa julgada", (await mandar(() => pAc.registrar({ autor: acusado.publicKey(), tipo: 2, hash: B(hDef) }, OPC))).hash);

  // ---- 5. the sealed draw --------------------------------------------------

  const rodada = Number(fechamento.rodada);
  fase("sorteio", `Esperando a rodada ${rodada} do drand, que ainda não existe`);
  await ateInstante(GENESIS + (rodada - 1) * PERIODO + 2);
  const rd = await (await fetch(`${DRAND}/v2/chains/${CADEIA}/rounds/${rodada}`)).json();
  w.conferir_baliza(BigInt(rodada), rd.signature);
  nota(`a assinatura da rodada ${rodada} confere com a chave da quicknet, aqui no navegador`);
  const selo = (await mandar(() => kEsc.seal({ sig: B(w.assinatura_da_baliza(rd.signature)) }, OPC)));
  marcar("selo: o contrato verifica a assinatura do drand", selo.hash);
  const seed = BigInt(selo.valor.seed), limiar = BigInt(selo.valor.limiar);

  fase("assentos", "Cada sorteado prova o assento em conhecimento zero, de uma conta nova");
  const evK = async () => (await lerEventos(klero, inicio)).filter((e) => e.evento === "Inscrito" || e.evento === "Impedido");
  const eventosArvore = await evK();
  const yDe = segredos.map((s) => P.hash2(seed, s));
  const assentoDe = new Map(); // juror index -> seat keypair
  let proximo = 0;
  async function reivindicar(i) {
    const assento = assentos[proximo++];
    assentoDe.set(i, assento);
    const input = await deps.provar.entrada(P, eventosArvore, segredos[i], seed, assento.publicKey());
    const t0 = Date.now();
    const pr = await deps.provar.provar(deps.snarkjs, input, deps.circuito.wasm, deps.circuito.zkey);
    nota(`prova do jurado ${i}: ${((Date.now() - t0) / 1000).toFixed(1)} s no navegador`);
    const prova = { a: B(pr.proof.a), b: B(pr.proof.b), c: B(pr.proof.c) };
    const r = await mandar(async () => (await cliente(klero, assento)).claim({ assento: assento.publicKey(), prova, y: BigInt(pr.y) }, OPC));
    marcar(`assento provado (y = ${pr.y.slice(0, 6)}…)`, r.hash);
  }
  const abaixo = jurados.map((_, i) => i).filter((i) => i !== 0 && yDe[i] < limiar);
  nota(`${abaixo.length} pessoas caíram abaixo do limiar; só cada uma sabe disso`);
  for (const i of abaixo) await reivindicar(i);
  await ateLedger(selo.valor.prazo);
  let fim = (await mandar(() => kEsc.finalize(OPC))).valor;
  if (!fim) {
    nota("menos de k reivindicações: o limiar se amplia uma vez");
    for (const i of jurados.map((_, j) => j).filter((j) => j !== 0 && !assentoDe.has(j))) {
      await reivindicar(i);
      if (assentoDe.size >= PLANO.k) break;
    }
    await ateLedger((await lerValor(() => kEsc.seal_state())).prazo);
    fim = (await mandar(() => kEsc.finalize(OPC))).valor;
  }
  const painelY = fim.painel.map(String);
  const sentados = [...assentoDe.entries()].filter(([i]) => painelY.includes(String(yDe[i])));
  aviso({ dado: "painel", valor: { sentados: sentados.length, reivindicacoes: assentoDe.size } });

  // ---- 6. attendance and the session --------------------------------------

  fase("quorum", "Os jurados sentados comparecem pelas contas de assento");
  const presentes = sentados.slice(0, sentados.length - 1);
  const ausente = sentados[sentados.length - 1];
  for (const [, kp] of presentes) {
    marcar("presença, por uma conta de assento", (await mandar(async () => (await cliente(proc, kp)).comparecer({ assento: kp.publicKey() }, OPC))).hash);
  }
  nota(`um jurado sorteado não compareceu: ${ausente[1].publicKey().slice(0, 6)}…`);
  const hAta = await guardar("Ata de deliberação", opcoes.textos.ataJurado);
  const primeiro = presentes[0][1];
  marcar("ata de deliberação, por uma conta de assento", (await mandar(async () =>
    (await cliente(proc, primeiro)).registrar({ autor: primeiro.publicKey(), tipo: 3, hash: B(hAta) }, OPC))).hash);

  // ---- 7. the vote, on Tessera --------------------------------------------

  fase("votacao", "A votação abre no Tessera, com os presentes como votantes");
  const aptos = presentes.map(([, kp]) => kp.publicKey());
  const xdrs = aptos.map((a) => hex(new Address(a).toScVal().toXDR()));
  const pesos = new Uint32Array(aptos.length).fill(1);
  const idVoto = hex(crypto.getRandomValues(new Uint8Array(32)));
  const raizAptos = w.raiz_de_aptos(xdrs, pesos);
  const agora = await ledger();
  const abreEm = agora + PLANO.aberturaVoto, fechaEm = abreEm + PLANO.duracaoVoto;
  const rodadaVoto = Number(w.rodada_em(BigInt(Math.floor(Date.now() / 1000) + (fechaEm - agora) * 6 + 30)));
  const tEsc = await cliente(TESSERA, escrivao);
  const ab = await mandar(() => tEsc.abrir({ governanca: escrivao.publicKey(), proposta: B(idVoto),
    perguntas: [{ opcoes: 2, confidencial: true }], raiz_aptos: B(raizAptos), mesa: [], limiar: 0,
    abre_em: abreEm, fecha_em: fechaEm, anel: true, secoes: 1, limite_secao: 0,
    rodada: BigInt(rodadaVoto), rodada_abertura: 0n }, OPC));
  marcar("Tessera: votação aberta, cédula sigilosa, fechadura de tempo", ab.hash);
  marcar("o caso vincula a votação: os votantes são exatamente os presentes",
    (await mandar(() => pEsc.vincular_votacao({ proposta: B(idVoto) }, OPC))).hash);
  const chaves = [];
  for (const [j, [, kp]] of presentes.entries()) {
    const chave = w.nova_chave_de_anel();
    const cam = w.caminho_de(xdrs, pesos, j);
    const r = await mandar(async () => (await cliente(TESSERA, kp)).comparecer({ proposta: B(idVoto), votante: kp.publicKey(),
      chave_anel: B(chave.publica), caminho: cam.irmaos.map(B), indice: cam.indice, secao: 0 }, OPC));
    chaves.push(chave);
    marcar("Tessera: comparecimento no anel", r.hash);
  }
  const escolhas = await opcoes.escolherVotos(aptos);
  fase("urna", "Cada presente vota em segredo, de uma chave de uso único");
  await ateLedger(abreEm - 1);
  const tLer = await cliente(TESSERA, apurador);
  const anel = (await lerValor(() => tLer.anel({ proposta: B(idVoto), secao: 0 }))).map((x) => hex(x));
  const hp = hex(await lerValor(() => tLer.hp({ proposta: B(idVoto) })));
  const h = hex(await lerValor(() => tLer.gerador_h()));
  const votos = [];
  for (const [j, chave] of chaves.entries()) {
    const c = w.cedula_anonima(idVoto, hp, h, anel, anel.indexOf(chave.publica), chave.secreta,
      [{ opcoes: 2, confidencial: true }], new Uint32Array([escolhas[j]]), BigInt(rodadaVoto));
    const eph = efemeras[j];
    const r = await mandar(async () => (await cliente(TESSERA, eph)).votar_anonimo({
      proposta: B(idVoto), secao: 0, anel: anel.map(B), imagem: B(c.imagem), c0: BigInt("0x" + c.c0),
      z: c.z.map((x) => BigInt("0x" + x)), compromissos: c.cedula.compromissos.map(B),
      provas: c.cedula.provas.map((p) => ({ a0: B(p.a0), a1: B(p.a1), e0: BigInt("0x" + p.e0), z0: BigInt("0x" + p.z0), e1: BigInt("0x" + p.e1), z1: BigInt("0x" + p.z1) })),
      provas_soma: c.cedula.provas_soma.map((p) => ({ a: B(p.a), z: BigInt("0x" + p.z) })),
      escolhas: Array.from(c.cedula.escolhas), cripto: B(c.cedula.cripto) }, OPC));
    votos.push(r.hash);
    marcar("Tessera: uma cédula, de uma chave de uso único", r.hash);
  }

  fase("apuracao", "A urna fecha; a chave sai do drand; qualquer um apura");
  await ateLedger(fechaEm);
  await ateInstante(Number(w.instante_da_rodada(BigInt(rodadaVoto))) + 3);
  const rv = await (await fetch(`${DRAND}/v2/chains/${CADEIA}/rounds/${rodadaVoto}`)).json();
  w.conferir_baliza(BigInt(rodadaVoto), rv.signature);
  const cedulas = (await lerEventosBrutos(TESSERA, abreEm - 2))
    .filter((e) => scValToNative(e.topic[0]) === "anonimo" && hex(scValToNative(e.topic[1])) === idVoto)
    .map((e) => scValToNative(e.value));
  const compromissos = cedulas.flatMap((v) => v[1].map((x) => hex(x)));
  const aberta = w.abertura_da_secao(compromissos, cedulas.map((v) => hex(v[3])), 2, h, BigInt(rodadaVoto), rv.signature);
  marcar("Tessera: a seção apurada", (await mandar(() => tLer.apurar_secao({ proposta: B(idVoto), quem: apurador.publicKey(),
    secao: 0, compromissos: compromissos.map(B), abertas: Array.from(aberta.abertas) }, OPC))).hash);
  const aberturas = Array.from(aberta.totais).map((_, j) => BigInt("0x" + w.somar_aberturas([aberta.aberturas[j]])));
  const pub = await mandar(() => tLer.publicar_resultado({ proposta: B(idVoto), quem: apurador.publicKey(), secoes: [0],
    totais: Array.from(aberta.totais), aberturas }, OPC));
  marcar(`Tessera: placar publicado, ${Array.from(aberta.totais).join(" a ")}`, pub.hash);

  // ---- 8. the verdict ------------------------------------------------------

  fase("veredito", "O caso lê o placar do Tessera e aplica a regra do nível");
  const dec = await mandar(() => pEsc.decidir(OPC));
  marcar("veredito", dec.hash);
  const d = dec.valor;
  aviso({ dado: "veredito", valor: { sim: d.sim, nao: d.nao, exigidos: d.exigidos, procedente: d.procedente } });
  const hSessao = await guardar("Ata da sessão", `# Ata da sessão\n\nCaso de demonstração, aberto no navegador.\n\n- Presentes: ${presentes.length} de ${sentados.length}; quórum ${PLANO.votosMinimos}.\n- Votação no Tessera: \`${idVoto}\`, com os presentes como votantes.\n`);
  marcar("ata da sessão", (await mandar(() => pEsc.registrar({ autor: escrivao.publicKey(), tipo: 3, hash: B(hSessao) }, OPC))).hash);
  const hDec = await guardar("Ata da decisão", `# Ata da decisão\n\n- Placar do Tessera: ${d.sim} sim, ${d.nao} não; exigidos ${d.exigidos}.\n- Decisão: ${d.procedente ? "procedente" : "improcedente"}.\n`);
  marcar("ata da decisão", (await mandar(() => pEsc.registrar({ autor: escrivao.publicKey(), tipo: 3, hash: B(hDec) }, OPC))).hash);

  // ---- 9. the reveal -------------------------------------------------------

  fase("revelacao", `Esperando o ledger ${revelaEm}; depois, cada jurado mostra quem é`);
  await ateLedger(revelaEm);
  const quemRevela = jurados.map((_, i) => i).filter((i) => i !== 0 && i !== ausente[0]);
  await emLotes(quemRevela, 8, async (i) => {
    const r = await mandar(async () => (await cliente(klero, jurados[i])).reveal({ jurado: jurados[i].publicKey(), s: segredos[i] }, OPC));
    marcar(`revelação ${i}`, r.hash);
  });

  // ---- 10. read it all back ------------------------------------------------

  fase("registro", "Lendo tudo de volta do ledger, sem confiar em nenhum contrato");
  const eventos = [
    ...(await lerEventos(klero, inicio)).map((e) => ({ ...e, contrato: "kleroterion" })),
    ...(await lerEventos(proc, inicio)).map((e) => ({ ...e, contrato: "processo" })),
  ].sort((a, b) => (a.id < b.id ? -1 : 1));
  for (const e of eventos.filter((x) => x.evento === "Reivindicado")) {
    const t = await (await fetch(`${HORIZON}/transactions/${e.tx}`)).json();
    e.assento = t.source_account;
  }
  const proposta = normalizar(await lerValor(() => tLer.proposta({ proposta: B(idVoto) })));
  const resultado = (await lerValor(() => tLer.resultado({ proposta: B(idVoto) }))).map(Number);
  const aleatoriedade = (await (await fetch(`${DRAND}/${CADEIA}/public/${rodada}`)).json()).randomness;
  const registro = {
    contratos: { kleroterion: klero, processo: proc, tessera: TESSERA, bilhete: sac, verificador: VERIFICADOR, emissor: emissor.publicKey() },
    aleatoriedade,
    eventos,
    tessera: { commit: "356c22fbf90ea396203ea411b463ae14a5480d59", id: idVoto, perguntas: [{ texto: "Aplicar a sanção?", opcoes: ["sim", "nao"], confidencial: true }],
      aptos, proposta, resultado, abertura_tx: ab.hash, votos_tx: votos, apuracao: { tx: pub.hash } },
  };
  return { registro, docs, hashes };

  // ---- events, decoded the way scripts/registro.py writes them -----------

  async function lerEventosBrutos(id, desde) {
    const todos = [];
    let cursor;
    for (;;) {
      const r = await server.getEvents(cursor
        ? { cursor, filters: [{ type: "contract", contractIds: [id] }], limit: 200 }
        : { startLedger: desde, filters: [{ type: "contract", contractIds: [id] }], limit: 200 });
      todos.push(...r.events);
      if (r.events.length < 200 || !r.cursor) return todos;
      cursor = r.cursor;
    }
  }

  async function lerEventos(id, desde) {
    const nomes = new Set([...NOMES.kleroterion, ...NOMES.processo]);
    const out = [];
    for (const e of await lerEventosBrutos(id, desde)) {
      const nome = String(scValToNative(e.topic[0])).split("_").map((p) => p[0].toUpperCase() + p.slice(1)).join("");
      if (!nomes.has(nome)) continue;
      const corpo = normalizar(scValToNative(e.value));
      const topo = {};
      (TOPICOS[nome] || []).forEach((campo, j) => { topo[campo] = normalizar(scValToNative(e.topic[j + 1])); });
      out.push({ evento: nome, ledger: e.ledger, id: e.id, tx: e.txHash, ...topo, ...corpo });
    }
    return out;
  }
}

/** Native values in the record's shape: integers above u32 as decimal strings
 *  (u64 rounds and times as numbers), bytes as hex, structs and vectors recursed. */
export function normalizar(v, campo = "") {
  if (typeof v === "bigint") return campo === "rodada" || campo === "tempo" || campo === "rodada_abertura" ? Number(v) : v.toString();
  if (v instanceof Uint8Array) return hex(v);
  if (Array.isArray(v)) return v.map((x) => normalizar(x));
  if (v && typeof v === "object") return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, normalizar(x, k)]));
  return v;
}
