# Como os conselhos reais se reúnem e geram documentos

**Estudo · 26 de outubro de 2026 · T-054** (registrado primeiro como T-053, número que a dapp ao vivo também recebeu; renumerado no mesmo dia)

> Nada aqui está construído e nada aqui é uma medição. Todos os números e todas as regras deste documento são **citados**: foram lidos nas fontes primárias em 26/10/2026, e cada um tem uma linha em `docs/SOURCES.md` com a data e o endereço. Onde a fonte é secundária — alguém contando o que outra instituição faz — está escrito ao lado.
>
> O estudo não propõe mudar invariante nenhuma. Ele termina em §6 com propostas que **dependem de aprovação humana**, e em §7 com o que ele não cobre.

## 0. Por que este estudo existe

O repositório tinha estudado a fundo um único rito: o júri brasileiro (`desenho-rol-sorteio-recusas.md`, `desenho-sorteio-e-sigilo.md`). Dele vieram as palavras que o contrato `processo` usa — sessão, quórum, ata, escrivão — e dele vieram as regras de sorteio, recusa e suplência. Do Contributor Covenant 2.1 vieram apenas os quatro níveis de consequência.

O que **não** estava registrado em lugar nenhum era como um conselho real conduz um caso: se há reunião, se ela é síncrona, onde ela acontece, quem escreve o que fica nos autos. E o contrato já tinha respondido a essa pergunta sem que a resposta estivesse escrita:

- `comparecer(assento)` é **assíncrono**. Cada jurado manda a sua própria transação quando quiser, dentro de `prazo_quorum` ledgers. A sessão instala sozinha no instante em que a contagem atinge o quórum, e quem chega depois conta (INV-23).
- `registrar(autor, tipo, hash)` guarda **só o sha256**. O documento vive fora da cadeia, e o contrato não tem opinião sobre quem o escreveu nem em que ferramenta.
- A ata de deliberação do jurado é **individual e opcional**: cada um registra a sua, se quiser.

Ou seja: emprestamos o vocabulário de um rito presencial e síncrono e implementamos um processo assíncrono e sem lugar. Este estudo vai ver se isso se sustenta.

## 1. Conselhos profissionais: CFM e CRMs

Fonte primária: **Código de Processo Ético-Profissional (CPEP), Resolução CFM nº 2.306/2022**, lido inteiro em PDF.

| O que a norma manda | Artigo |
|---|---|
| A sindicância e o PEP **tramitam em sigilo processual**, e podem tramitar em formato eletrônico | art. 1º e § 1º |
| A sessão de julgamento **pode ser em ambiente eletrônico, por videoconferência ou outro recurso de transmissão de sons e imagens de forma síncrona** | art. 89 |
| As partes são intimadas da **data** do julgamento com antecedência mínima de 5 dias | art. 88 |
| A sessão começa com a leitura da parte expositiva do relatório, depois sustentação oral de 10 minutos por parte, esclarecimentos, e 5 minutos finais | art. 89, §§ 2º a 6º |
| Encerrados os debates, diligências pedidas são **aprovadas por maioria dos presentes** e a sessão é **suspensa** | art. 90 |
| O voto do relator é **escrito e integral**: preliminares, culpabilidade com artigos imputados, e a sanção com fundamentação da dosimetria | art. 91, § 1º |
| Qualquer conselheiro pode pedir **vistas** e tem 30 dias; o processo volta à pauta e **o rito do art. 89 se repete** | art. 92, § 1º |
| **Não há obrigatoriedade da mesma composição de participantes** da sessão anterior quando o julgamento continua | art. 92, § 2º |
| No julgamento **presencial** sem divergência, o presidente declara o resultado unânime **sem votação nominal** | art. 93, § 1º |
| No julgamento **virtual ou misto**, os votos **deverão ser sempre colhidos individualmente** | art. 93, § 3º |
| Havendo dois ou mais votos divergentes, a votação ganha etapas, nesta ordem: culpabilidade, depois cassação ou não, depois **sanção confidencial ou pública**, depois mais versus menos gravosa | art. 93, § 4º |
| O presidente **vota por último** e desempata | art. 94 |
| O conselheiro presente **não pode abster-se** de votar | art. 95 |
| Proferidos os votos, o autor do voto vencedor é designado para redigir o **acórdão**, que ele assina | arts. 96 e 91, § 3º |
| Na sessão de julgamento só podem estar **as partes, seus defensores, membros do CRM, a assessoria jurídica e os funcionários necessários** | art. 98 |
| O quórum do pleno para interdição cautelar é **mínimo de 11 e máximo de 21 conselheiros**, por maioria simples | art. 29 |
| O interditado presente é notificado **na própria sessão, com registro em ata** | art. 31 |
| A interdição só é publicizada, com o nome do médico, **depois de referendada pelo CFM** | art. 33 |

Três coisas chamam a atenção. A primeira é que **remoto é permitido, assíncrono não**: o art. 89 autoriza a videoconferência e no mesmo período exige "forma síncrona". A segunda é que **a modalidade muda a regra do voto**: no presencial a unanimidade pode ser declarada em bloco, no virtual cada voto é colhido um a um — exatamente a desconfiança de que, à distância, "ninguém se opôs" não prova que todos concordaram. A terceira é que **o colegiado pode mudar de uma sessão para a outra** (art. 92, § 2º), o que é o oposto do nosso painel, fixado no `seal`.

## 2. Conselhos profissionais: OAB

Fonte primária: **Regimento Interno do TED da OAB/BA, 2019**, lido inteiro em PDF.

| O que o regimento manda | Artigo |
|---|---|
| As Turmas **se instalam com a maioria absoluta** dos membros e **deliberam por maioria simples**; só os titulares contam para o quórum | art. 4º, § 1º |
| O calendário de sessões é definido pela Diretoria do Tribunal; janeiro e julho são recesso | art. 4º e § 2º |
| **Na ausência do Secretário**, o presidente da Turma designa um dos membros **para lavrar a ata da sessão** | art. 11 |
| As atas das sessões de julgamento devem **consignar os nomes dos presentes e dos patronos**, e registrar o uso da palavra e as questões preliminares | art. 32, § 2º |
| Processo não julgado na sessão pautada **permanece na pauta das próximas**, sem nova notificação, e a ocorrência é **registrada na ata** | art. 5º, § 8º |
| O julgamento do processo disciplinar se dá em **sessão secreta**, admitida a presença das partes, procuradores e integrantes do Conselho | art. 72 |
| As fases do julgamento: leitura do relatório, voto do relator com proposta de ementa, sustentação oral, discussão com manifestação de cada membro, votação, proclamação com leitura da súmula | art. 73 |
| Do julgamento **lavra-se acórdão** | art. 75 |
| A Secretaria mantém **livros de registro de atas** das sessões das Turmas Julgadoras e do Órgão Consultivo, e providencia **absoluto sigilo** nos trabalhos | art. 30, V e § 1º |

O essencial aqui é institucional e não procedimental: **existe um cartório**. A ata não é um documento que alguém resolve escrever; é uma entrada obrigatória num livro de registro mantido por uma secretaria, e quando o secretário falta, o presidente **designa** quem a lavra. O registro não é opcional em nenhum ponto da cadeia.

Sobre sessão virtual na OAB, a Resolução TED nº 11/2020 da OAB/SP regulamenta julgamento virtual e telepresencial, e um Diário Eletrônico de 23/09/2026 convoca sessão extraordinária **híbrida** do TED-BA para 22/10/2026 — ambos apareceram em busca e **não foram lidos na fonte**. O regimento de 2019 que eu li não trata de sessão remota.

## 3. Comitês de ética em pesquisa

Fonte primária: **Regimento Interno do CEP da UFMG**, lido na página oficial.

| O que o regimento manda | Artigo |
|---|---|
| Reuniões ordinárias **quinzenais ou mensais**, com **no mínimo 12 por ano** | art. 20, § 2º |
| Quórum de instalação e de deliberação: **50% + 1** dos membros; consultores *ad hoc* não contam | art. 20, § 1º; art. 10, § 4º |
| As reuniões **podem ocorrer na modalidade virtual, total ou parcial**; quem participa por vídeo fica em sala privativa | art. 20, § 3º; art. 21, parágrafo único |
| As reuniões são **fechadas ao público**, preservando sigilo e confidencialidade | art. 20 |
| A **secretária administrativa** secretaria as reuniões e **elabora as atas** | art. 19 |
| A ata registra as **decisões do plenário, hora de início e de fim, nomes dos presentes e motivos das ausências**; é assinada pelos presentes em até 30 dias; fica visível **para todos os membros do CEP** | art. 21-A e §§ |
| Cada protocolo é relatado por **ao menos um membro**, com prazo máximo de **15 dias**; o plenário delibera o parecer final antes de a coordenação assiná-lo | arts. 29, 31 e parágrafo único |
| O CEP tem **30 dias** para liberar o parecer, emitido pela **Plataforma Brasil**, em caráter confidencial | arts. 40, 3º e 28 |
| Membro com interesse direto ou indireto **abstém-se** da deliberação; o sigilo dura além do mandato | art. 14-A, § 2º; arts. 8 e 11 |

Este é o modelo mais próximo de um conselho de comunidade: colegiado permanente, pauta recorrente, relator por caso, parecer escrito num sistema externo. E é aqui que a divisão do trabalho documental aparece mais nítida: **o relator escreve o parecer, a secretária escreve a ata.** São dois documentos de autoria diferente, com prazos diferentes, e nenhum dos dois é escrito pelo colegiado como corpo.

## 4. Times de código de conduta em projetos open source

Quatro fontes, todas lidas.

**Python Software Foundation** — *Code of Conduct Working Group Enforcement Procedures*, em `policies.python.org`:
- o grupo **se reúne duas vezes por mês**; ao receber um relato, avalia conflitos de interesse e **convoca uma reunião dos membros sem conflito**;
- o assíncrono é o **plano B**: se não houver fórum de discussão em tempo real, os membros sem conflito discutem o caso numa thread de e-mail separada — e o documento diz que **usar lista de e-mail não é recomendado**;
- a decisão sobre plano de modificação de comportamento e consequências sai por **voto de dois terços** do grupo, excluídos os conflitados;
- o grupo mantém **dois tipos de documento compartilhado**: uma planilha de status e **um documento por relato**, com acesso apenas para membros sem conflito; toda a documentação de discussões e decisões vai para documentação compartilhada online;
- quem tem conflito **se retira da discussão e do voto**, e **não precisa explicar o conflito** — só declarar que existe;
- quem denunciou recebe confirmação de recebimento **em até uma semana**, e o retorno sai em até uma semana depois da reunião em que o caso foi discutido;
- o grupo **não diz quem relatou o incidente**; publicar um relato do incidente é uma das respostas possíveis.

**CNCF** — *Code of Conduct Transparency Report*:
- em 2026, **13 queixas: 5 violações, 0 não-violações, 7 fora de escopo**;
- membros conflitados **se recusaram da decisão e suplentes decidiram no lugar deles**;
- a determinação precisa ser baseada em prova suficiente;
- os documentos do processo que o relatório menciona: um **relatório investigativo** (redigido e **redatado** antes de circular entre os outros membros), uma **notificação formal escrita** à pessoa acusada, advertências formais com condições, e o próprio relatório de transparência;
- em um caso de 2024, a identidade de quem relatou era **conhecida apenas pelo ombudsperson** e pelo membro que recebeu o relato presencialmente; o relatório **não dá informação que identifique vítimas ou denunciantes** e publica **só sumários**.

**rOpenSci** — *template de incidente*:
- um **único documento por relato**, com permissões só do comitê, cobrindo intake (id, data, meio, dois prazos: um para a confirmação pública, outro para a decisão), investigação (checklist, pessoas, testemunhas, conflitos, prova), decisão (tipo de comportamento, resposta, **e a razão da decisão**) e seguimento (ações, avisos às duas partes, declaração pública, prevenção);
- o uso do template é **ação privada, confidencial e interna**; o relatório anual de transparência traz **apenas descrições anonimizadas**;
- a página **não diz** se a deliberação é síncrona.

**Rust, Leadership Council** — `procedures/synchronous-meetings.md` (não é o time de moderação, é o conselho de liderança; é o que tem procedimento público):
- o conselho **se reúne a cada duas semanas**, por vídeo;
- **se o quórum falha, a reunião síncrona é cancelada e a pauta é percorrida assincronamente no chat** — a mesma hierarquia da PSF: síncrono primeiro, assíncrono como contingência;
- os *scribes* tomam as notas; **a ata tem que ser pública antes da reunião seguinte**, ou o atraso é anunciado publicamente;
- o que não pode ser público é **retirado da ata publicada, com um marcador indicando que houve uma seção privada**, e os *librarians* devem **preencher depois** o que se tornar seguro publicar.

## 5. DAOs

Fontes: o fórum da Arbitrum DAO, lido em duas threads.

- **Quem decide.** A aplicação do código de conduta acontece "através de qualquer programa que compense financeiramente a atividade de governança", e o gestor, conselho ou facilitador do programa é "o responsável por determinar violações do Código de Conduta" (Entropy, 09/07/2025). O responsável tem de **acusar o recebimento da queixa e investigar**, mas "a determinação final e o curso de ação resultante, que pode incluir arquivar a queixa, dependerá da estrutura do programa subjacente" (17/07/2025).
- **Privado primeiro.** "Recomenda-se que os contribuidores busquem primeiro resolver conflitos de boa fé e em privado"; se isso falhar, ou se a conduta for ameaçadora, o caso vai à Arbitrum Foundation, que atua como mediadora neutra e "terá a palavra final".
- **Nada é obrigatoriamente publicado.** A Foundation "reserva-se o direito de determinar se a questão deve ser levada ao conhecimento da comunidade como um todo". A única exigência de publicação é para substituição de membros.
- **O precedente da Optimism — e é o achado mais importante deste estudo.** Contando a história da Optimism, a Entropy escreveu em 22/10/2024 que o código foi inicialmente aplicado pela Optimism Foundation, que depois "o processo foi mudado para que as violações fossem trazidas e **votadas diretamente pelos delegados**", e que "**depois de apenas dois casos isso foi considerado uma abordagem sub-ótima**". A conclusão: "os delegados não queriam ser responsáveis por votar em casos de conduta". Dali saiu um **comitê dedicado de código de conduta**, que hoje "só trata de casos severos de quebra de conduta".

Esse último parágrafo é testemunho de segunda mão: é a Arbitrum contando o que a Optimism fez. A fonte primária da Optimism **não foi lida**. Mas, se se confirmar, é a evidência empírica mais direta de que a tese deste projeto resolve um problema que alguém realmente teve: **votar conduta no colégio inteiro não funcionou, e a saída foi um corpo pequeno** — que é exatamente o que o sorteio compõe, com a diferença de que ninguém escolhe quem está nele.

## 6. O que atravessa as quatro famílias

| | CFM/CRM | OAB/BA | CEP UFMG | PSF / CNCF / rOpenSci / Rust | Arbitrum | **Pinakion hoje** |
|---|---|---|---|---|---|---|
| Há reunião? | sim, sessão pautada | sim, calendário | sim, 12+/ano | sim, 2×/mês (PSF, Rust) | não há órgão fixo | **não há reunião** |
| Síncrona? | **exigido**, mesmo remoto | presencial no texto lido | sim, virtual total ou parcial | síncrona primeiro, assíncrona se falhar | assíncrona, fórum | **assíncrona, janela de ledgers** |
| Remoto permitido? | sim (art. 89) | não no texto de 2019 | sim (art. 20, § 3º) | sim, é o padrão | sim | sim, é o único modo |
| Quem lavra o registro? | o autor do voto vencedor (acórdão) | o Secretário, ou quem o presidente designar | a secretária (ata) e o relator (parecer) | o comitê, num doc por caso | ninguém obrigatoriamente | **cada autor registra o seu hash** |
| Quórum | 11 a 21 conselheiros | maioria absoluta | 50% + 1 | 2/3 para decidir (PSF) | — | maioria do painel, nunca < piso de sigilo |
| Voto individual? | só no virtual (art. 93, § 3º) | votação após discussão | deliberação em plenário | voto de 2/3 (PSF) | — | **sempre, e secreto (Tessera)** |
| O que é público | a sanção, depois do referendo | nada: sessão secreta | nada: parecer confidencial | sumários anonimizados | nada obrigatório | **os hashes, os eventos e o veredito** |
| Conflito de interesse | impedimento e suspeição | impedimento | abstenção do membro | **autodeclarado**, sem explicar | — | `impede` pelo escrivão |
| Substituto | suplência; composição pode mudar entre sessões | lista de antiguidade | consultor *ad hoc* | **suplentes decidem no lugar (CNCF)** | — | o próximo da ordem sorteada |

Seis padrões:

1. **Remoto, sim; assíncrono, não.** Todas as quatro famílias com rito escrito aceitam vídeo e nenhuma delas abre mão do encontro. A PSF e o conselho da Rust chegam a escrever a hierarquia: o assíncrono é o que se faz quando o síncrono falha. O Pinakion não tem encontro nenhum.
2. **A modalidade remota endurece o voto, não o afrouxa.** O CFM é explícito: no presencial a unanimidade pode ser declarada em bloco; no virtual cada voto é colhido individualmente (art. 93, § 3º). O nosso voto individual e secreto está, nesse eixo, na frente do rito que imitamos.
3. **O registro tem autor designado, e não é o colegiado.** Secretária no CEP, Secretário ou membro designado na OAB, autor do voto vencedor no CFM, *scribes* na Rust, um documento por caso na PSF e na rOpenSci. Em nenhuma delas a ata é opcional, e em nenhuma delas ela é escrita por quem quiser.
4. **Dois níveis de publicidade, sempre.** Sigilo durante, sumário depois: o CFM publica a sanção só depois do referendo e distingue em votação **sanção confidencial ou pública** (art. 93, § 4º, III); a OAB julga em sessão secreta e lavra acórdão; a CNCF e a rOpenSci publicam só sumários anonimizados; a Rust risca a parte privada da ata com um marcador e preenche depois.
5. **O conflito de interesse é autodeclarado, e o suplente é a resposta.** A PSF deixa o membro se retirar sem explicar o motivo; a CNCF põe suplentes a decidir no lugar dos recusados. Nenhuma das fontes dá a terceiros o poder de riscar um membro do caso.
6. **Os prazos contam dias e partem de atos humanos.** Cinco dias de antecedência da sessão, uma semana para confirmar o recebimento, quinze dias para o parecer, trinta para as vistas, trinta para assinar a ata. Nenhum deles é um relógio que corre sozinho.

## 7. O que isso diz do Pinakion

**O que bate.**

- **Voto individual e secreto.** Padrão 2. O Tessera dá o que o CFM só exige no virtual, e dá mais: secreto.
- **Dois níveis de publicidade.** Padrão 4. Hash na cadeia, documento fora, veredito público: é a forma on-chain de "sigilo durante, sumário depois".
- **Suplência pela ordem.** Padrão 5, metade. O próximo da ordem sorteada é o suplente da CNCF.
- **Impedimento antes do fechamento.** O CFM veda a interdição cautelar na própria sessão de julgamento (art. 29, § 2º); nós vedamos o `impede` depois do `lock` (Q-21). Mesma intuição: a composição não se mexe durante o julgamento.
- **Afastamento durante o caso.** A interdição cautelar do art. 29 é exatamente o `afastado` da INV-20, com a diferença de que a nossa não precisa de referendo.

**O que não bate, e cada um é uma escolha que ninguém registrou.**

1. **Não existe reunião.** Padrão 1. O nosso quórum é uma contagem numa janela de ledgers, não a instalação de um encontro. Nenhuma das cinco fontes funciona assim. Isso pode ser a tradução correta para uma comunidade distribuída em vários fusos — mas é uma divergência do rito que o vocabulário do contrato promete, e hoje nada no repositório a defende.
2. **Ninguém é convocado para uma data.** O CFM intima as partes com 5 dias de antecedência (art. 88). No Pinakion a pessoa julgada não é intimada de nada: ela registra a defesa quando descobre que o caso existe, e o painel descobre que foi sorteado ao tentar reivindicar. Não há notificação, e o `prazo_quorum` corre sem que ninguém tenha sido avisado.
3. **Ninguém lavra a ata.** Padrão 3. O `registrar` aceita ata do escrivão e ata de jurado, as duas opcionais. Nada obriga a existir **uma** ata da sessão, e nada diz quem a escreve. No caso demo o escrivão escreveu uma; o contrato não pediu.
4. **O impedimento não é autodeclarado.** Padrão 5. O `impede` é do escrivão, o que inverte a prática de todas as quatro famílias: lá, quem tem o conflito se retira; aqui, um terceiro risca. E a Q-27 (o conflito de quem denunciou) continua sem saída justamente porque não existe uma retirada silenciosa — hoje a pessoa simplesmente não reivindica o assento, e aparece como "sorteada e ausente".
5. **Não há vistas, nem diligências, nem suspensão.** O CFM permite a qualquer conselheiro parar o julgamento por 30 dias para ler os autos (art. 92), e permite ao plenário suspender a sessão para diligências (art. 90). O nosso único freio é o `adiar`, e ele serve para a falta de quórum. Um jurado que queira mais prova não tem o que fazer além de votar.
6. **O painel é fixo; os deles não são.** O art. 92, § 2º do CPEP diz que a composição pode mudar quando o julgamento continua. O nosso painel é selado no `seal` e é a nossa garantia central — então aqui a divergência é deliberada e deve continuar. Vale escrevê-la como escolha, não deixá-la como coincidência.
7. **Não há desempate nem dever de votar.** O presidente do CFM vota por último e desempata (art. 94), e o conselheiro presente não pode se abster (art. 95). No Pinakion, empate é improcedência por não atingir os votos exigidos, e o silêncio de quem sentou é apenas ausência. É defensável — não existe presidente, e um voto obrigatório num voto secreto não é verificável — mas é diferente, e não está escrito.
8. **Ninguém registra documento por hash.** Procurei e não achei (§8 e `SOURCES.md`): nenhuma das famílias compromete o conteúdo dos autos publicamente. O registro delas é um livro de cartório ou um documento com permissão de comitê. Isso quer dizer duas coisas: o `registrar` não tem prática externa para copiar, e também não tem prática externa que o contradiga.

## 8. Propostas

Nenhuma destas é uma invariante nova até a pessoa responsável aprovar. Três são baratas e duas são milestone.

| # | Proposta | O que muda | Custo |
|---|---|---|---|
| P-1 | **Uma ata da sessão obrigatória.** `decidir` recusa se não houver pelo menos um registro de tipo `Ata` do escrivão depois da instalação da sessão | uma condição em `decidir`, um teste | baixo |
| P-2 | **Escrever a assincronia como escolha.** Um DEC dizendo que o Pinakion troca a sessão síncrona por uma janela de comparecimento, com o motivo (fusos, custo de coordenação, verificabilidade) e o que se perde (deliberação conjunta, esclarecimentos ao relator) | documentação | baixo |
| P-3 | **Recusa autodeclarada.** O `recuse(juror, s)` já existe e é aberto, antes do `finalize`. Falta dizer na documentação que ele é a forma preferida, e que o `impede` do escrivão é a exceção, não a regra | documentação, mais um item nos limites do README | baixo |
| P-4 | **Intimação verificável.** Um evento `Intimado(parte, prazo)` que o escrivão emite e sem o qual o `prazo_quorum` não começa a correr. Dá à pessoa julgada o equivalente ao art. 88 | muda o início do prazo; precisa de DEC | milestone |
| P-5 | **Vistas.** Um jurado presente pode, uma vez, estender o prazo de votação em `espera_vistas` ledgers, registrando uma ata com o motivo | novo estado, novo prazo, DEC | milestone |

## 9. Limites deste estudo

- **Tudo é citado, nada é medido.** Nenhuma destas instituições foi observada funcionando; eu li as normas que elas publicam. Norma escrita não é prática: o CPEP manda a sessão ser síncrona, e isso não diz quantos conselheiros estavam de fato olhando a tela.
- **O recorte é brasileiro de um lado e anglófono do outro.** Conselhos profissionais e CEPs são do Brasil; os times de código de conduta são de projetos de língua inglesa. Não há nenhum conselho profissional de fora do Brasil aqui, e nenhum conselho de ética de comunidade não-anglófona.
- **Um regimento não representa a categoria.** Li o TED da OAB/BA de 2019 e o CEP da UFMG. Cada seccional e cada instituição tem o seu; onde a busca indicou variação (a OAB/SP regulamentou sessão virtual em 2020; o CEP/UNIT usa dois terços em primeira convocação), eu não li a fonte.
- **A fonte da Optimism é de segunda mão.** O achado da §5 — delegados votando conduta deu errado em dois casos — é a Arbitrum contando a história da Optimism em 22/10/2024. A fonte primária da Optimism não foi lida, e o achado não deve entrar em nenhuma peça pública do projeto antes disso.
- **Não procurei o time de moderação da Rust, só o conselho de liderança.** O procedimento que eu li é do Leadership Council. Se o time de moderação publica procedimento próprio, eu não achei.
- **Django e Kubernetes ficaram de fora.** Apareceram em busca, não foram lidos. A CNCF cobre o Kubernetes por cima, não o comitê do projeto.
- **Nada aqui foi construído.** Nenhuma linha de contrato mudou, nenhum gate novo existe, nenhuma invariante foi tocada.
