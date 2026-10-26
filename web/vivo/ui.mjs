// Presentation only: the case engine and the verifier remain the sources of truth.
export function regrasDoPlano(plano) {
  // Same rule displayed by CasoAberto; checked against the recorded testnet case.
  return {
    quorum: Math.max(Math.floor(plano.k / 2) + 1, plano.votosMinimos),
    exigidos: plano.gravidade >= plano.doisTercosDesde ? Math.ceil(2 * plano.k / 3) : Math.floor(plano.k / 2) + 1,
  };
}

export function resumirConferencia(r, registro) {
  const d = registro.eventos.find((e) => e.evento === 'Decidido');
  if (!d || !r.conferencias?.length) throw new Error('Incomplete result: the decision or the checks are missing.');
  const ruins = r.conferencias.filter(([, ok]) => ok !== true);
  return {
    confere: ruins.length === 0,
    titulo: ruins.length ? 'Record with divergences' : d.procedente ? 'Upheld' : 'Not upheld',
    placar: `${d.sim} votes for the sanction and ${d.nao} against. Minimum required: ${d.exigidos}.`,
    detalhe: ruins.length
      ? `${ruins.length} of ${r.conferencias.length} checks failed: ${ruins.map(([n]) => n).join('; ')}. The decision was not validated by this page.`
      : `${r.conferencias.length} checks redone from the ledger’s events. No divergence found.`,
  };
}

const FASES = [
  ['contas', 'Create the accounts', 0], ['bilhete', 'Lock the ticket issuer', 0],
  ['contratos', 'Deploy the contracts', 0], ['rol', 'File the enrolments', 0],
  ['fechamento', 'Lock the roster', 0], ['autos', 'File the documents', 0],
  ['sorteio', 'Seal the draw with drand', 1], ['assentos', 'Make the seat proofs', 1],
  ['quorum', 'Check the quorum', 1], ['votacao', 'Bind the Tessera vote', 2],
  ['urna', 'Cast the secret votes', 2], ['apuracao', 'Tally the vote', 2],
  ['veredito', 'File the verdict', 2], ['revelacao', 'Reveal the jurors', 3],
  ['registro', 'Check the record', 3],
];
const CAMPOS = ['denuncia', 'prova', 'defesa', 'ata'];
const EXPLORER = 'https://stellar.expert/explorer/testnet';
const mmss = (s) => `${String(Math.floor(s / 60)).padStart(2, '0')}:${String(Math.floor(s % 60)).padStart(2, '0')}`;
const curto = (s) => s.length > 16 ? `${s.slice(0, 7)}…${s.slice(-5)}` : s;
const el = (tag, attrs = {}, ...filhos) => {
  const e = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) e.setAttribute(k, v);
  e.append(...filhos.flat());
  return e;
};
const link = (href, texto) => el('a', {href, target: '_blank', rel: 'noopener'}, texto);

export function montarDemo({ carregar, executar, conferir, plano }) {
  const $ = (id) => document.getElementById(id);
  let estado = 'carregando', atual = null, inicioFase = 0, t0 = 0, transacoes = 0;
  const feitas = new Set(), itens = new Map(), fatos = [];
  const antesDeSair = (e) => { e.preventDefault(); e.returnValue = ''; };
  $('rol-total').textContent = plano.jurados;
  $('juri-total').textContent = plano.k;
  const regras = regrasDoPlano(plano);
  $('quorum-total').textContent = regras.quorum;
  $('exigidos-total').textContent = regras.exigidos;
  let documentoAtivo = 'denuncia', editando = false;
  function renderTexto(campo) {
    const preview = $(`preview-${campo}`);
    const inline = (texto) => texto.split(/(\*\*[^*]+\*\*|`[^`]+`)/g).map((parte) =>
      parte.startsWith('**') ? el('strong', {}, parte.slice(2, -2)) : parte.startsWith('`') ? el('code', {}, parte.slice(1, -1)) : parte);
    // Restricted, text-only Markdown preview. No HTML, images or remote embeds.
    preview.replaceChildren(...$(`t-${campo}`).value.split(/\n\s*\n/).filter(Boolean).map((bloco) => {
      const heading = bloco.match(/^#{1,6}\s+(.+)$/);
      if (heading) return el('h3', {}, ...inline(heading[1]));
      if (/^\d+\. |^[-*] /.test(bloco)) {
        const linhas = bloco.split('\n');
        return el(/^\d/.test(bloco) ? 'ol' : 'ul', {}, ...linhas.map((l) => el('li', {}, ...inline(l.replace(/^(\d+\.|[-*])\s+/, '')))));
      }
      return el('p', {}, ...inline(bloco));
    }));
  }
  function modoDocumento() {
    CAMPOS.forEach((campo) => { $(`t-${campo}`).hidden = !editando; $(`preview-${campo}`).hidden = editando; });
    $('editar').textContent = editando ? 'Finish editing' : 'Edit the text';
    $('reading-mode').textContent = editando ? 'Editing the document' : 'Reading the document';
  }
  $('editar').addEventListener('click', () => {
    if (estado !== 'pronto') return;
    if (editando) CAMPOS.forEach(renderTexto);
    editando = !editando; modoDocumento();
    if (editando) $(`t-${documentoAtivo}`).focus();
  });
  document.querySelector('a[href="#limites"]').addEventListener('click', () => { $('limites').open = true; });
  const diario = $('diario');
  const GRUPOS = [
    ['Prepare the case', 'files'], ['Constitute the jury', 'users-three'],
    ['Vote and decide', 'check-square'], ['Check the result', 'list-checks'],
  ];
  let acompanhando = false, seguirDiario = true;
  const origens = new Map();
  function mover(no, destino) {
    const ancora = document.createComment('session-origin');
    no.before(ancora); origens.set(no, ancora); destino.append(no);
  }
  function animar(elemento) {
    if (window.matchMedia('(prefers-reduced-motion: reduce)').matches) return;
    elemento.classList.remove('stage-enter');
    void elemento.offsetWidth;
    elemento.classList.add('stage-enter');
  }
  function focarCentro(modo, alvo) {
    if ($('autos-dialog').open) $('autos-dialog').close();
    $('live-canvas').dataset.mode = modo;
    $('live-waiting').hidden = ['review', 'voting', 'result'].includes(modo);
    animar($('live-canvas'));
    if (alvo) {
      $(alvo).focus({preventScroll: true});
      const pos = $(alvo).getBoundingClientRect();
      if (pos.top < 0 || pos.bottom > window.innerHeight) $(alvo).scrollIntoView({block: 'nearest', behavior: 'instant'});
    }
  }
  function abrirAcompanhamento() {
    if (acompanhando) return;
    acompanhando = true;
    mover($('session-status'), $('live-status'));
    mover($('jornada'), $('live-journey'));
    mover(document.querySelector('.current-stage'), $('live-stage'));
    mover(document.querySelector('.metrics'), $('live-metrics'));
    ['urna', 'resultado', 'erro'].forEach(id => mover($(id), $('live-canvas')));
    mover($('journal'), $('live-journal'));
    mover(document.querySelector('.technical'), $('live-technical'));
    mover(document.querySelector('.documents'), $('autos-dialog-content'));
    mover(document.querySelector('.progress-panel .case-rules'), $('session-context'));
    mover(document.querySelector('.disclosures'), $('session-context'));
    $('journal').open = true;
    $('journal').querySelector('summary').setAttribute('aria-disabled', 'true');
    $('live-room').hidden = false;
    document.body.classList.add('session-started');
    $('decorrido').textContent = '00:00';
    window.scrollTo({top: 0, behavior: 'instant'});
  }
  function revisarCaso() {
    if (estado !== 'pronto') return;
    abrirAcompanhamento();
    document.body.classList.add('reviewing');
    $('launch-review').hidden = false;
    $('session-caption').textContent = 'Preparation → review → following along';
    $('fechar-autos').lastChild.textContent = 'Back to the review';
    status('Before sending', 'neutral');
    const regras = document.querySelector('#session-context .case-rules').cloneNode(true);
    regras.querySelectorAll('[id]').forEach(no => no.removeAttribute('id'));
    $('review-rules').replaceChildren(regras);
    validar();
    focarCentro('review', 'review-title');
    animar($('live-room'));
  }
  $('voltar-preparacao').addEventListener('click', () => {
    if (estado !== 'pronto') return;
    if ($('autos-dialog').open) $('autos-dialog').close();
    for (const [no, ancora] of origens) ancora.replaceWith(no);
    origens.clear(); acompanhando = false;
    $('live-room').hidden = true; $('launch-review').hidden = true;
    $('live-waiting').hidden = false;
    document.body.classList.remove('session-started', 'reviewing');
    $('journal').open = false;
    $('journal').querySelector('summary').removeAttribute('aria-disabled');
    status('Not started', 'neutral');
    $('comecar').focus({preventScroll: true});
    document.querySelector('.case-heading').scrollIntoView({block: 'start', behavior: 'instant'});
    animar(document.querySelector('.workspace'));
  });
  function contextoConsulta() {
    $('dialog-context').textContent = !$('urna').hidden
      ? `Ballot open · sent automatically in ${$('urna-conta').textContent} s`
      : estado === 'pronto' ? 'Review · the documents can still be edited'
      : estado === 'executando' ? 'Reading · the run continues' : 'Reading · run finished';
  }
  $('consultar-autos').addEventListener('click', () => { contextoConsulta(); $('autos-dialog').showModal(); });
  $('fechar-autos').addEventListener('click', () => $('autos-dialog').close());
  $('autos-dialog').addEventListener('close', () => {
    if (estado === 'pronto' && acompanhando) $('review-title').focus({preventScroll: true});
  });
  $('journal').querySelector('summary').addEventListener('click', (e) => {
    if (acompanhando) e.preventDefault();
  });
  function seguir(ativo) {
    seguirDiario = ativo;
    $('seguir-diario').setAttribute('aria-pressed', String(ativo));
    $('seguir-label').textContent = ativo ? 'Pause scrolling' : 'Follow the entries';
    $('seguir-diario').querySelector('.icon').dataset.icon = ativo ? 'pause' : 'play';
    $('journal-follow-status').textContent = ativo ? 'New entries appear at the end.' : 'Scrolling paused · the run continues.';
    if (ativo) diario.scrollTop = diario.scrollHeight;
  }
  $('seguir-diario').addEventListener('click', () => seguir(!seguirDiario));
  diario.addEventListener('wheel', (e) => { if (e.deltaY < 0) seguir(false); }, {passive: true});
  diario.addEventListener('keydown', (e) => { if (['ArrowUp', 'PageUp', 'Home'].includes(e.key)) seguir(false); });
  function anotar(cls, ...conteudo) {
    $('journal-empty')?.remove();
    diario.append(el('div', {class: `log-entry ${cls}`}, el('time', {class: 'log-time'}, t0 ? mmss((Date.now() - t0) / 1000) : '—'), el('span', {}, ...conteudo)));
    if (seguirDiario) diario.scrollTop = diario.scrollHeight;
  }
  function status(texto, tipo) {
    $('session-status').textContent = texto;
    $('session-status').className = `status ${tipo}`;
  }
  for (const [id, texto] of FASES) {
    const li = el('li', {'data-fase': id}, el('span', {class: 'marca', 'aria-hidden': 'true'}, '·'), el('span', {}, texto), el('span', {class: 'tempo'}));
    itens.set(id, li); $('fases').append(li);
  }
  function tabAtivo(id, focar = false) {
    documentoAtivo = id;
    for (const campo of CAMPOS) {
      const tab = $(`tab-${campo}`), selecionado = campo === id;
      tab.setAttribute('aria-selected', String(selecionado));
      tab.tabIndex = selecionado ? 0 : -1;
      $(`panel-${campo}`).hidden = !selecionado;
    }
    if (focar) $(`tab-${id}`).focus();
  }
  CAMPOS.forEach((id, i) => {
    const tab = $(`tab-${id}`);
    tab.addEventListener('click', () => tabAtivo(id));
    tab.addEventListener('keydown', (e) => {
      const n = e.key === 'ArrowRight' ? (i + 1) % 4 : e.key === 'ArrowLeft' ? (i + 3) % 4 : e.key === 'Home' ? 0 : e.key === 'End' ? 3 : null;
      if (n !== null) { e.preventDefault(); tabAtivo(CAMPOS[n], true); }
    });
    $(`t-${id}`).addEventListener('input', validar);
  });
  function validar() {
    if (estado !== 'pronto') return;
    const completo = CAMPOS.every((id) => $(`t-${id}`).value.trim());
    $('comecar').disabled = !completo;
    $('confirmar-inicio').disabled = !completo;
    const nomes = ['Complaint', 'Evidence', 'Defence', 'Deliberation minutes'];
    $('review-documents').replaceChildren(...CAMPOS.map((id, i) => {
      const preenchido = Boolean($(`t-${id}`).value.trim());
      const botao = el('button', {type: 'button', class: 'review-document'}, el('span', {class: 'icon', 'data-icon': preenchido ? 'check-circle' : 'files', 'aria-hidden': 'true'}), el('span', {}, nomes[i]), el('small', {}, preenchido ? 'Check →' : 'Fill in →'));
      botao.onclick = () => { tabAtivo(id); contextoConsulta(); $('autos-dialog').showModal(); };
      return el('li', {}, botao);
    }));
    $('review-doc-status').textContent = completo ? 'All four documents are filled in. You can still edit them.' : 'Fill in every document before creating the case.';
    $('pronto').textContent = completo ? 'Documents filled in. Continue to the case review.' : 'Fill in all four documents to start the case.';
  }
  function atualizarJornada(erro = false, concluida = false) {
    const grupo = FASES.find(([id]) => id === atual)?.[2] ?? 0;
    for (const li of $('jornada').children) {
      const n = Number(li.dataset.group), done = n < grupo || concluida;
      li.className = done ? 'done' : n === grupo ? erro ? 'failed' : 'current' : '';
      li.querySelector('.step-number').textContent = '';
      li.querySelector('.step-number').classList.add('icon');
      li.querySelector('.step-number').dataset.icon = done ? 'check-circle' : GRUPOS[n][1];
      li.querySelector('.step-number').setAttribute('aria-hidden', 'true');
      if (n === grupo && !concluida) li.setAttribute('aria-current', 'step');
      else li.removeAttribute('aria-current');
      li.querySelector('.step-state').textContent = done ? 'Done' : n === grupo ? erro ? 'Stopped' : 'In progress' : 'Waiting';
    }
    $('etapas-concluidas').textContent = feitas.size;
  }
  function concluirAtual() {
    if (!atual) return;
    feitas.add(atual);
    const li = itens.get(atual);
    li.className = 'feito'; li.querySelector('.marca').textContent = '✓';
    li.querySelector('.tempo').textContent = mmss((Date.now() - inicioFase) / 1000);
  }
  function entrar(id, texto) {
    if (!itens.has(id)) { anotar('nota-d', texto); return; }
    const mudou = atual !== id;
    if (mudou) {
      concluirAtual(); atual = id; inicioFase = Date.now();
    }
    const li = itens.get(id); li.className = 'atual'; li.querySelector('.marca').textContent = '→';
    const fase = FASES.find(([f]) => f === id);
    $('phase-label').textContent = `${GRUPOS[fase[2]][0]} · step ${FASES.indexOf(fase) + 1} of ${FASES.length}`;
    $('stage-title').textContent = fase[1];
    $('stage-icon').dataset.icon = GRUPOS[fase[2]][1];
    if (mudou) animar($('live-waiting'));
    $('fase-atual').textContent = texto;
    $('phase-help').textContent = id === 'assentos'
      ? 'The proofs are computed in your browser. This step can take a few minutes.'
      : id === 'urna' ? 'The votes go to the secret-ballot module.'
      : 'Wait for this step to finish. The journal records each confirmed transaction.';
    atualizarJornada(); anotar('fase-d', texto);
  }
  function aviso(e) {
    if (e.fase) entrar(e.fase, e.texto);
    else if (e.tx) {
      transacoes += 1; $('tx-count').textContent = `${transacoes} ${transacoes === 1 ? 'transaction' : 'transactions'}`;
      anotar('tx', '✓ ', e.rotulo, ' · ', link(`${EXPLORER}/tx/${e.tx}`, curto(e.tx)));
    } else if (e.nota) anotar('nota-d', e.nota);
    else if (e.dado === 'kleroterion' || e.dado === 'processo') fatos.push([e.dado === 'kleroterion' ? 'The roster contract' : 'The case contract', e.valor]);
  }
  function escolherVotos(aptos) {
    entrar('urna', 'The ballot is open for your part.');
    const escolhas = aptos.map((_, i) => i === aptos.length - 1 ? 1 : 0);
    $('cedulas').replaceChildren(...aptos.map((a, i) => {
      const sim = el('button', {type: 'button', 'aria-pressed': String(escolhas[i] === 0)}, 'Yes');
      const nao = el('button', {type: 'button', 'aria-pressed': String(escolhas[i] === 1)}, 'No');
      const marcar = (v) => { escolhas[i] = v; sim.setAttribute('aria-pressed', String(v === 0)); nao.setAttribute('aria-pressed', String(v === 1)); };
      sim.onclick = () => marcar(0); nao.onclick = () => marcar(1);
      return el('li', {}, el('strong', {}, `Juror ${i + 1}`), el('span', {class: 'conta'}, curto(a)), el('div', {class: 'escolha', role: 'group', 'aria-label': `Juror ${i + 1}’s vote`}, sim, nao));
    }));
    $('urna').hidden = false; $('depositar').disabled = false;
    focarCentro('voting', 'urna-titulo');
    status('Your part', 'warning');
    return new Promise((resolver) => {
      // A wall-clock deadline preserves the existing engine's voting window.
      const prazo = Date.now() + 45000;
      let enviado = false;
      function enviar() {
        if (enviado) return;
        enviado = true; clearInterval(conta);
        $('depositar').disabled = true; $('depositar').onclick = null; $('urna').hidden = true;
        anotar('nota-d', 'Choices sent to the voting module.');
        $('fase-atual').textContent = 'Sending the votes to Tessera.';
        $('phase-help').textContent = 'Wait for the votes to confirm and the tally to open.';
        status('In progress', 'active');
        focarCentro('waiting', 'stage-title');
        resolver(escolhas.slice());
      }
      const tick = () => { const resta = Math.max(0, Math.ceil((prazo - Date.now()) / 1000)); $('urna-conta').textContent = resta; if ($('autos-dialog').open) contextoConsulta(); if (resta <= 0) enviar(); };
      const conta = setInterval(tick, 1000); tick();
      $('depositar').onclick = enviar;
    });
  }
  $('form').addEventListener('submit', async (e) => {
    e.preventDefault();
    if (estado !== 'pronto' || CAMPOS.some((id) => !$(`t-${id}`).value.trim())) return;
    if (e.submitter !== $('confirmar-inicio') || !acompanhando) { revisarCaso(); return; }
    estado = 'executando'; $('confirmar-inicio').disabled = true; $('comecar').disabled = true; $('comecar').textContent = 'Case in progress';
    CAMPOS.forEach((id) => { $(`t-${id}`).readOnly = true; });
    editando = false; CAMPOS.forEach(renderTexto); modoDocumento(); $('editar').hidden = true;
    $('doc-status').textContent = 'Documents fixed';
    document.querySelector('.section-description').textContent = 'This run’s record. The texts are read-only.';
    $('pronto').textContent = 'Keep this tab open. This session’s keys live in it.';
    status('In progress', 'active');
    window.addEventListener('beforeunload', antesDeSair);
    t0 = Date.now();
    $('launch-review').hidden = true;
    document.body.classList.remove('reviewing');
    $('session-caption').textContent = 'Session in progress';
    $('fechar-autos').lastChild.textContent = 'Back to the session';
    $('stage-title').textContent = 'Starting to create the case';
    $('fase-atual').textContent = 'Preparing the first operation.';
    focarCentro('waiting', 'stage-title');
    const relogio = setInterval(() => { $('decorrido').textContent = mmss((Date.now() - t0) / 1000); }, 1000);
    try {
      const textos = {denuncia: $('t-denuncia').value, prova: $('t-prova').value, defesa: $('t-defesa').value, ataJurado: $('t-ata').value};
      const {registro, docs} = await executar({textos, escolherVotos}, aviso);
      entrar('registro', 'Redoing the checks from the record.');
      const r = await conferir(registro, docs), resumo = resumirConferencia(r, registro);
      if (resumo.confere) concluirAtual();
      else { const li = itens.get('registro'); li.className = 'erro'; li.querySelector('.marca').textContent = '!'; }
      atualizarJornada(!resumo.confere, resumo.confere);
      $('veredito').textContent = resumo.titulo; $('placar').textContent = resumo.placar;
      $('conferencia').textContent = resumo.detalhe;
      $('conferencia-titulo').textContent = resumo.confere ? '✓ Check complete' : '! The check diverged';
      $('resultado').classList.toggle('bad', !resumo.confere);
      $('checks').replaceChildren(...r.conferencias.map(([nome, ok]) => el('li', {class: ok === true ? 'ok' : 'bad'}, el('span', {}, ok === true ? '✓ Matches' : '✗ Failed'), el('span', {}, nome))));
      if (registro.contratos?.tessera) fatos.push(['Tessera’s contract', registro.contratos.tessera]);
      for (const [nome, valor] of fatos) $('fatos').append(el('dt', {}, nome), el('dd', {}, link(`${EXPLORER}/contract/${valor}`, curto(valor))));
      try {
        localStorage.setItem('pinakion:registro', JSON.stringify(registro));
        localStorage.setItem('pinakion:docs', JSON.stringify(docs));
      } catch {
        $('ver').hidden = true; $('armazenamento').hidden = false;
        $('armazenamento').textContent = 'The browser could not save the case. Download the record to keep it.';
      }
      $('baixar').onclick = () => {
        const url = URL.createObjectURL(new Blob([JSON.stringify(registro, null, 1)], {type: 'application/json'}));
        const a = el('a', {href: url, download: 'pinakion-case.json'}); document.body.append(a); a.click(); a.remove();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
      };
      estado = 'concluido'; status(resumo.confere ? 'Done' : 'With divergences', resumo.confere ? 'success' : 'error');
      $('phase-label').textContent = 'Run finished';
      $('fase-atual').textContent = resumo.confere ? 'Case finished. The record checks out.' : 'Case closed with divergences in the record.';
      $('phase-help').textContent = 'See the result and the checks in this session.';
      $('comecar').textContent = 'Case closed'; $('pronto').textContent = 'This run’s documents are available to read.';
      $('resultado').hidden = false; focarCentro('result', 'veredito');
    } catch (err) {
      estado = 'erro'; status('Stopped', 'error');
      if (atual) { const li = itens.get(atual); li.className = 'erro'; li.querySelector('.marca').textContent = '!'; }
      atualizarJornada(true); $('erro').hidden = false;
      $('fase-atual').textContent = 'This run could not be completed.';
      $('phase-help').textContent = 'The journal holds the error and the transactions already confirmed.';
      $('phase-label').textContent = 'Run stopped';
      $('comecar').textContent = 'Run stopped';
      $('pronto').textContent = 'Keep the journal’s detail before starting another session.';
      anotar('erro-d', String(err.message || err));
      $('stage-title').textContent = 'Run stopped';
      $('attention-label').textContent = 'Check the journal to find the failure';
      $('erro').setAttribute('tabindex', '-1'); focarCentro('error', 'erro');
    } finally {
      clearInterval(relogio); window.removeEventListener('beforeunload', antesDeSair);
      $('decorrido').textContent = mmss((Date.now() - t0) / 1000);
    }
  });
  return carregar().then((textos) => {
    CAMPOS.forEach((id, i) => { $(`t-${id}`).value = textos[i]; $(`t-${id}`).disabled = false; });
    CAMPOS.forEach(renderTexto); $('editar').disabled = false;
    estado = 'pronto'; validar();
  }).catch((err) => {
    estado = 'erro'; status('Unavailable', 'error');
    $('pronto').textContent = 'Could not load the demonstration. Check your connection and reload the page.';
    $('pronto').classList.add('error');
    $('fase-atual').textContent = 'The tools could not be loaded.';
    $('phase-help').textContent = 'No case was started. The detail is in the journal.';
    anotar('erro-d', String(err.message || err));
  });
}
