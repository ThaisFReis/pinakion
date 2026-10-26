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
  if (!d || !r.conferencias?.length) throw new Error('Resultado incompleto: faltam a decisão ou as conferências.');
  const ruins = r.conferencias.filter(([, ok]) => ok !== true);
  return {
    confere: ruins.length === 0,
    titulo: ruins.length ? 'Registro com divergências' : d.procedente ? 'Procedente' : 'Improcedente',
    placar: `${d.sim} votos a favor da sanção e ${d.nao} contra. Mínimo exigido: ${d.exigidos}.`,
    detalhe: ruins.length
      ? `${ruins.length} de ${r.conferencias.length} conferências falharam: ${ruins.map(([n]) => n).join('; ')}. A decisão não foi validada por esta página.`
      : `${r.conferencias.length} conferências refeitas a partir dos eventos do ledger. Nenhuma divergência encontrada.`,
  };
}

const FASES = [
  ['contas', 'Criar as contas', 0], ['bilhete', 'Fixar o emissor do bilhete', 0],
  ['contratos', 'Implantar os contratos', 0], ['rol', 'Registrar as inscrições', 0],
  ['fechamento', 'Fechar o rol', 0], ['autos', 'Registrar os documentos', 0],
  ['sorteio', 'Selar o sorteio com drand', 1], ['assentos', 'Gerar as provas de assento', 1],
  ['quorum', 'Conferir o quórum', 1], ['votacao', 'Vincular a votação Tessera', 2],
  ['urna', 'Depositar os votos secretos', 2], ['apuracao', 'Apurar a votação', 2],
  ['veredito', 'Registrar o veredito', 2], ['revelacao', 'Revelar os jurados', 3],
  ['registro', 'Conferir o registro', 3],
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
    $('editar').textContent = editando ? 'Concluir edição' : 'Editar texto';
    $('reading-mode').textContent = editando ? 'Edição do documento' : 'Leitura do documento';
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
    ['Preparar o caso', 'files'], ['Constituir o júri', 'users-three'],
    ['Votar e decidir', 'check-square'], ['Conferir o resultado', 'list-checks'],
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
    $('session-caption').textContent = 'Preparação → revisão → acompanhamento';
    $('fechar-autos').lastChild.textContent = 'Voltar à revisão';
    status('Antes do envio', 'neutral');
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
    status('Não iniciado', 'neutral');
    $('comecar').focus({preventScroll: true});
    document.querySelector('.case-heading').scrollIntoView({block: 'start', behavior: 'instant'});
    animar(document.querySelector('.workspace'));
  });
  function contextoConsulta() {
    $('dialog-context').textContent = !$('urna').hidden
      ? `Urna aberta · envio automático em ${$('urna-conta').textContent} s`
      : estado === 'pronto' ? 'Revisão · os documentos ainda podem ser editados'
      : estado === 'executando' ? 'Consulta · a execução continua' : 'Consulta · execução encerrada';
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
    $('seguir-label').textContent = ativo ? 'Pausar rolagem' : 'Seguir registros';
    $('seguir-diario').querySelector('.icon').dataset.icon = ativo ? 'pause' : 'play';
    $('journal-follow-status').textContent = ativo ? 'Novos registros aparecem ao final.' : 'Rolagem pausada · a execução continua.';
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
    const nomes = ['Denúncia', 'Prova', 'Defesa', 'Deliberação'];
    $('review-documents').replaceChildren(...CAMPOS.map((id, i) => {
      const preenchido = Boolean($(`t-${id}`).value.trim());
      const botao = el('button', {type: 'button', class: 'review-document'}, el('span', {class: 'icon', 'data-icon': preenchido ? 'check-circle' : 'files', 'aria-hidden': 'true'}), el('span', {}, nomes[i]), el('small', {}, preenchido ? 'Conferir →' : 'Preencher →'));
      botao.onclick = () => { tabAtivo(id); contextoConsulta(); $('autos-dialog').showModal(); };
      return el('li', {}, botao);
    }));
    $('review-doc-status').textContent = completo ? 'Os quatro documentos estão preenchidos. Você ainda pode editá-los.' : 'Preencha todos os documentos antes de criar o caso.';
    $('pronto').textContent = completo ? 'Documentos preenchidos. Continue para a revisão do caso.' : 'Preencha os quatro documentos para iniciar o caso.';
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
      li.querySelector('.step-state').textContent = done ? 'Concluído' : n === grupo ? erro ? 'Interrompido' : 'Em andamento' : 'Aguardando';
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
    $('phase-label').textContent = `${GRUPOS[fase[2]][0]} · etapa ${FASES.indexOf(fase) + 1} de ${FASES.length}`;
    $('stage-title').textContent = fase[1];
    $('stage-icon').dataset.icon = GRUPOS[fase[2]][1];
    if (mudou) animar($('live-waiting'));
    $('fase-atual').textContent = texto;
    $('phase-help').textContent = id === 'assentos'
      ? 'As provas são calculadas no navegador. Esta etapa pode levar alguns minutos.'
      : id === 'urna' ? 'Os votos serão enviados ao módulo de votação secreta.'
      : 'Aguarde a conclusão desta etapa. O diário registra as transações confirmadas.';
    atualizarJornada(); anotar('fase-d', texto);
  }
  function aviso(e) {
    if (e.fase) entrar(e.fase, e.texto);
    else if (e.tx) {
      transacoes += 1; $('tx-count').textContent = `${transacoes} ${transacoes === 1 ? 'transação' : 'transações'}`;
      anotar('tx', '✓ ', e.rotulo, ' · ', link(`${EXPLORER}/tx/${e.tx}`, curto(e.tx)));
    } else if (e.nota) anotar('nota-d', e.nota);
    else if (e.dado === 'kleroterion' || e.dado === 'processo') fatos.push([e.dado === 'kleroterion' ? 'Contrato do rol' : 'Contrato do caso', e.valor]);
  }
  function escolherVotos(aptos) {
    entrar('urna', 'A urna está aberta para sua participação.');
    const escolhas = aptos.map((_, i) => i === aptos.length - 1 ? 1 : 0);
    $('cedulas').replaceChildren(...aptos.map((a, i) => {
      const sim = el('button', {type: 'button', 'aria-pressed': String(escolhas[i] === 0)}, 'Sim');
      const nao = el('button', {type: 'button', 'aria-pressed': String(escolhas[i] === 1)}, 'Não');
      const marcar = (v) => { escolhas[i] = v; sim.setAttribute('aria-pressed', String(v === 0)); nao.setAttribute('aria-pressed', String(v === 1)); };
      sim.onclick = () => marcar(0); nao.onclick = () => marcar(1);
      return el('li', {}, el('strong', {}, `Jurado ${i + 1}`), el('span', {class: 'conta'}, curto(a)), el('div', {class: 'escolha', role: 'group', 'aria-label': `Voto do jurado ${i + 1}`}, sim, nao));
    }));
    $('urna').hidden = false; $('depositar').disabled = false;
    focarCentro('voting', 'urna-titulo');
    status('Sua participação', 'warning');
    return new Promise((resolver) => {
      // A wall-clock deadline preserves the existing engine's voting window.
      const prazo = Date.now() + 45000;
      let enviado = false;
      function enviar() {
        if (enviado) return;
        enviado = true; clearInterval(conta);
        $('depositar').disabled = true; $('depositar').onclick = null; $('urna').hidden = true;
        anotar('nota-d', 'Escolhas enviadas ao módulo de votação.');
        $('fase-atual').textContent = 'Enviando os votos ao Tessera.';
        $('phase-help').textContent = 'Aguarde a confirmação dos votos e a abertura da apuração.';
        status('Em andamento', 'active');
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
    estado = 'executando'; $('confirmar-inicio').disabled = true; $('comecar').disabled = true; $('comecar').textContent = 'Caso em andamento';
    CAMPOS.forEach((id) => { $(`t-${id}`).readOnly = true; });
    editando = false; CAMPOS.forEach(renderTexto); modoDocumento(); $('editar').hidden = true;
    $('doc-status').textContent = 'Documentos fixados';
    document.querySelector('.section-description').textContent = 'Autos desta execução. Os textos estão disponíveis apenas para consulta.';
    $('pronto').textContent = 'Mantenha esta aba aberta. As chaves desta sessão vivem nela.';
    status('Em andamento', 'active');
    window.addEventListener('beforeunload', antesDeSair);
    t0 = Date.now();
    $('launch-review').hidden = true;
    document.body.classList.remove('reviewing');
    $('session-caption').textContent = 'Sessão em acompanhamento';
    $('fechar-autos').lastChild.textContent = 'Voltar à sessão';
    $('stage-title').textContent = 'Iniciando a criação do caso';
    $('fase-atual').textContent = 'Preparando a primeira operação.';
    focarCentro('waiting', 'stage-title');
    const relogio = setInterval(() => { $('decorrido').textContent = mmss((Date.now() - t0) / 1000); }, 1000);
    try {
      const textos = {denuncia: $('t-denuncia').value, prova: $('t-prova').value, defesa: $('t-defesa').value, ataJurado: $('t-ata').value};
      const {registro, docs} = await executar({textos, escolherVotos}, aviso);
      entrar('registro', 'Refazendo as conferências a partir do registro.');
      const r = await conferir(registro, docs), resumo = resumirConferencia(r, registro);
      if (resumo.confere) concluirAtual();
      else { const li = itens.get('registro'); li.className = 'erro'; li.querySelector('.marca').textContent = '!'; }
      atualizarJornada(!resumo.confere, resumo.confere);
      $('veredito').textContent = resumo.titulo; $('placar').textContent = resumo.placar;
      $('conferencia').textContent = resumo.detalhe;
      $('conferencia-titulo').textContent = resumo.confere ? '✓ Conferência concluída' : '! Divergência na conferência';
      $('resultado').classList.toggle('bad', !resumo.confere);
      $('checks').replaceChildren(...r.conferencias.map(([nome, ok]) => el('li', {class: ok === true ? 'ok' : 'bad'}, el('span', {}, ok === true ? '✓ Confere' : '✗ Falhou'), el('span', {}, nome))));
      if (registro.contratos?.tessera) fatos.push(['Contrato do Tessera', registro.contratos.tessera]);
      for (const [nome, valor] of fatos) $('fatos').append(el('dt', {}, nome), el('dd', {}, link(`${EXPLORER}/contract/${valor}`, curto(valor))));
      try {
        localStorage.setItem('pinakion:registro', JSON.stringify(registro));
        localStorage.setItem('pinakion:docs', JSON.stringify(docs));
      } catch {
        $('ver').hidden = true; $('armazenamento').hidden = false;
        $('armazenamento').textContent = 'O navegador não pôde salvar o caso. Baixe o registro para guardá-lo.';
      }
      $('baixar').onclick = () => {
        const url = URL.createObjectURL(new Blob([JSON.stringify(registro, null, 1)], {type: 'application/json'}));
        const a = el('a', {href: url, download: 'pinakion-caso.json'}); document.body.append(a); a.click(); a.remove();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
      };
      estado = 'concluido'; status(resumo.confere ? 'Concluído' : 'Com divergências', resumo.confere ? 'success' : 'error');
      $('phase-label').textContent = 'Execução encerrada';
      $('fase-atual').textContent = resumo.confere ? 'Caso concluído. O registro confere.' : 'Caso encerrado com divergências no registro.';
      $('phase-help').textContent = 'Consulte o resultado e as conferências nesta sessão.';
      $('comecar').textContent = 'Caso encerrado'; $('pronto').textContent = 'Os documentos desta execução estão disponíveis para consulta.';
      $('resultado').hidden = false; focarCentro('result', 'veredito');
    } catch (err) {
      estado = 'erro'; status('Interrompido', 'error');
      if (atual) { const li = itens.get(atual); li.className = 'erro'; li.querySelector('.marca').textContent = '!'; }
      atualizarJornada(true); $('erro').hidden = false;
      $('fase-atual').textContent = 'Não foi possível concluir esta execução.';
      $('phase-help').textContent = 'O diário contém o erro e as transações já confirmadas.';
      $('phase-label').textContent = 'Execução interrompida';
      $('comecar').textContent = 'Execução interrompida';
      $('pronto').textContent = 'Guarde os detalhes do diário antes de iniciar outra sessão.';
      anotar('erro-d', String(err.message || err));
      $('stage-title').textContent = 'Execução interrompida';
      $('attention-label').textContent = 'Consulte o diário para identificar a falha';
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
    estado = 'erro'; status('Indisponível', 'error');
    $('pronto').textContent = 'Não foi possível carregar a demonstração. Confira sua conexão e recarregue a página.';
    $('pronto').classList.add('error');
    $('fase-atual').textContent = 'As ferramentas não puderam ser carregadas.';
    $('phase-help').textContent = 'Nenhum caso foi iniciado. Os detalhes estão no diário.';
    anotar('erro-d', String(err.message || err));
  });
}
