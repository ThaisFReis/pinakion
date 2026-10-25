# Sorteio verificável e sigilo com acesso justificado

**Documento de desenho · 25 de outubro de 2026**

> Toda afirmação aqui tem fonte. O que eu não consegui verificar está marcado como **não verificado**. Nada foi afirmado de memória.

---

## 0. O fato que decide a arquitetura

A documentação do próprio `soroban-sdk`, no módulo `prng`, diz que o PRNG é um CSPRNG forte (ChaCha20) e reseeding pelo contrato, **mas**:

- a semente vem de dado **público assim que o ledger é nomeado**;
- a semente está **sob controle dos validadores**;
- a semente deriva do hash do conjunto de transações, e **não é secreta nem criptograficamente difícil de enviesar** por um validador corrompido;
- o módulo deve ser usado apenas onde **o risco de influência de validador é aceitável**;
- e a própria doc diz que isso é mais fraco do que um contrato obteria com **commit/reveal sobre uma fonte externa de entropia confiável**.

Fonte: [`soroban-sdk/src/prng.rs`](https://github.com/stellar/rs-soroban-sdk/blob/main/soroban-sdk/src/prng.rs).

**Consequência direta:** para sortear quem vai julgar a conduta de alguém, risco de influência de validador **não é aceitável**. `env.prng()` sozinho não serve.

**O que serve, e já está no nosso repertório:** commit/reveal com beacon público — a mesma construção de time-lock que o projeto anterior usou. A semente vem de uma rodada do beacon que **ainda não existia** quando o caso e o rol foram comprometidos.

**O que aproveitamos do PRNG mesmo assim:** `Prng::shuffle` implementa **Fisher-Yates**, que é exatamente o algoritmo de sorteio. Reseedamos o PRNG local com a semente do beacon (`Prng::seed`) e usamos `shuffle`. A aleatoriedade vem de fora; o embaralhamento, de dentro — determinístico e recomputável por qualquer um.

---

## 1. Como o júri é montado

O modelo é o **Tribunal do Júri brasileiro**, que resolve isto em três estágios separados — e a separação é o que dá a garantia.

| Estágio | No CPP | On-chain |
|---|---|---|
| **Alistamento** | lista geral anual publicada até 10/10, contestável por qualquer cidadão até 10/11; quem serviu nos 12 meses anteriores fica de fora (art. 426) | raiz do rol comprometida, com janela de contestação pública e **exclusão de quem serviu no último ciclo** |
| **Sorteio do corpo** | o juiz retira cédulas a portas abertas até completar **25**, entre o 15º e o 10º dia útil antes da reunião; MP, OAB e Defensoria são intimados a acompanhar (arts. 432 e 433) | sorteio por beacon, **antes** de o caso ser instruído, verificável por qualquer um |
| **Conselho de sentença** | novo sorteio em plenário forma **7**, com **3 recusas imotivadas por parte** | painel final, com recusas limitadas e substituto vindo do **próximo sorteado**, nunca de nova seleção |

Fontes: [art. 433](https://www.legjur.com/legislacao/art/dcl_00036891941-433), [composição e recusas](https://jusdocs.com/fluxogramas/tribunal-do-juri-2a-fase-conselho-de-sentenca-e-requisitos-para-jurados), [lista geral e exclusão dos que serviram](https://www.conjur.com.br/2023-jun-03/tribunal-juri-paridade-generos-conselho-sentenca-tribunal-juri/).

### Três propriedades que vêm de graça com esse desenho

**O rol precede o caso.** É a definição de juiz natural. O commitment do rol carrega a rodada do beacon, então a anterioridade é verificável, não declarada.

**Quem serviu, descansa.** A regra dos 12 meses não é cortesia: impede que um subgrupo se torne o tribunal permanente. É rotação forçada, e é anticaptura.

**Recusa não é escolha.** Cada parte recusa um número fixo e pequeno, e o substituto é o **próximo da ordem já sorteada**. Nenhuma recusa produz um julgador novo — só avança na fila que o beacon já fixou. É assim que se recusa sem selecionar, e é o problema que eu não sabia resolver antes de ler o CPP.

---

## 2. E se os sorteados não tiverem interesse?

Essa é a pergunta certa, e os números dizem que **é o modo de falha dominante do sorteio no mundo real**:

| Jurisdição | Não comparecimento | Sanção |
|---|---|---|
| Canadá | cerca de **1/5** dos convocados | diária de US$ 20 a 100 |
| Filadélfia | **36% a 42%** não respondem à intimação | — |
| Nova Zelândia | cerca de **1 em 5** | multa de NZ$ 1.000, aplicada uma vez, em 2007 |
| Irlanda | **25.649** faltas em três anos | multa de €500 prevista — **20 multas aplicadas no período** |

Fontes: [CBC](https://www.cbc.ca/1.1412886), [Philadelphia Magazine](https://www.phillymag.com/news/2018/05/10/philadelphians-skip-jury-duty/), [Otago Daily Times](https://www.odt.co.nz/node/255393), [Irish Daily Mail via PressReader](https://www.pressreader.com/ireland/irish-daily-mail/20190218/281809990167393).

O caso irlandês é o mais eloquente: **a sanção existe no papel e quase nunca é aplicada.** 20 multas contra 25 mil faltas.

### As três respostas, e a primeira é a boa

**Sobre-sorteio.** É o que o CPP faz: sorteia **25 para sentar 7**. Fator de 3,5. A resposta institucional ao desinteresse não é punir o faltante até ele comparecer — é **sortear gente suficiente para que a falta não quebre o quórum**. Barato, não exige coerção, e é o primeiro mecanismo a implementar.

**Dispensa com motivo, falta sem motivo.** Instituições reais distinguem as duas. Quem declara impedimento, conflito ou impossibilidade sai sem ônus e o próximo da fila entra. Quem simplesmente silencia é outra coisa.

**Consequência automática — e aqui a cadeia é melhor que o original.** A multa irlandesa falha porque alguém precisa decidir aplicá-la. On-chain, a consequência do silêncio é executada pelo contrato: perda da diária, do peso de reputação, ou exclusão do próximo ciclo de alistamento. Não é mais severa que a lei — é **efetiva**, que é precisamente o que a lei não consegue ser.

**Cuidado que precisa estar escrito:** punir silêncio pune também quem perdeu a chave, adoeceu ou não foi notificado. Por isso a dispensa tem que ser barata e a consequência tem que ser graduada, nunca em um único passo.

---

## 3. Revogabilidade do poder

O poder do painel é, por construção, o mais curto possível:

- **Nasce com o caso e morre com a decisão.** Não há painel permanente.
- **Morre também sem decisão.** Se o prazo vence sem julgamento, o painel se dissolve e o caso vai a novo sorteio — o que remove o incentivo de travar o processo por inação.
- **O rol tem mandato.** Ciclo anual, com contestação pública antes de valer.
- **Quem serviu sai do rol pelo ciclo seguinte.**
- **Impedimento é obrigatório, não opcional.** Quem tem relação declarada com as partes não entra no sorteio daquele caso.

E a regra que fecha o buraco que o Rust e o Node.js não tinham: **o painel não pode ser composto pelo corpo a que a pessoa julgada pertence**, e a instância de recurso não pode conter quem decidiu na primeira.

---

## 4. Documentos sigilosos com acesso justificado

Aqui eu preciso separar o que a criptografia faz do que ela não faz, porque a sua formulação mistura duas coisas.

### 4.1 O que vai on-chain

Nada do documento. Vai **commitment** — hash, ou raiz de Merkle do conjunto de peças. O documento em si fica fora da cadeia, cifrado. Isso não é escolha de desenho: é a única opção compatível com registro permanente e dado pessoal.

### 4.2 Onde o ZK serve de verdade

ZK **não** é o mecanismo de acesso. ZK é o mecanismo que torna a trilha pública **real sem expor conteúdo** — provar propriedades do documento selado sem abri-lo:

- que a defesa foi protocolada **dentro do prazo**;
- que a acusação invoca **uma das hipóteses do regimento**, sem dizer qual nem contra quem;
- que o parecer do relator **cita a peça de defesa**;
- que o documento aberto hoje é **o mesmo** que foi selado no dia do protocolo.

Sem isso, a auditoria pública de um processo sigiloso é só a palavra do conselho. Com isso, o processo é verificável e o conteúdo continua fechado. **É aqui que o ZK é estrutural**, e é diferente do que eu descrevi antes.

### 4.3 O acesso, e por que decisão e chave devem ser o mesmo ato

O modelo real é o **segredo de justiça**: o processo corre selado, e quem quer acesso **peticiona demonstrando interesse legítimo**, com o juiz deferindo em escopo definido.

A tradução técnica que eu recomendo: **a chave de decifração é partida entre os membros do conselho por compartilhamento de segredo, e o deferimento do acesso é a liberação das partes.**

A consequência é a propriedade que vale o desenho: **não existe deferir o acesso e reter a chave, nem entregar a chave sem deferimento.** A decisão e a capacidade técnica são o mesmo ato. Um conselho que quer esconder tem que negar em público; um membro que quer vazar sozinho não consegue, porque sozinho não reconstrói.

O repertório para isso já é conhecido: compartilhamento de segredo com limiar k-de-n, que é a mesma primitiva de Shamir usada no projeto anterior.

### 4.4 Permissão temporária: como a instituição realmente faz

O art. 189 do CPC não concede "acesso temporário a tudo mediante bom motivo". Ele faz algo mais fino, e melhor:

| Quem | O que obtém | Como |
|---|---|---|
| **Partes e seus procuradores** | os autos inteiros | por legitimidade, de forma contínua enquanto o processo corre (§ 1º) |
| **Terceiro com interesse jurídico** | **apenas certidão do dispositivo da sentença** — a decisão, não as provas | a pedido, caso a caso (§ 2º) |
| **Advogado ainda não constituído** | nada, até se habilitar | peticiona habilitação juntando procuração |
| **Qualquer pessoa** | o resultado e as propriedades verificáveis | público |

Fontes: [art. 189 do CPC](https://www.legjur.com/legislacao/art/lei_00131052015-189), [análise do art. 189](https://www.cognijus.com/blog/art-189-cpc), [especial do STJ pela OAB-PE](https://www.oabpe.org.br/noticias/especial-stj-os-limites-legais-para-o-segredo-de-justica-kZDcbH).

**A lição de desenho: escopo antes de prazo.** A regra brasileira limita principalmente *o quê*, não *por quanto tempo*. O terceiro não recebe os autos por uma semana — recebe **o dispositivo, e só**. Isso é muito mais forte que uma janela temporal, e é mais fácil de implementar: as peças são cifradas por fragmento, e o deferimento cobre exatamente um fragmento.

### 4.5 O mecanismo: a chave nunca sai

Permissão temporária aos documentos é implementável, e a construção é a mesma do § 4.3: **o conselho decifra sob demanda, enquanto a permissão vale; quem recebeu acesso nunca detém a chave.**

- Cada consulta é uma operação de limiar nova, contra o deferimento vigente on-chain.
- Vencido o prazo ou encerrado o escopo, os detentores de parte simplesmente deixam de cooperar, e não há mais decifração.
- Cada liberação é evento público, então **o número e o momento de cada acesso são auditáveis** mesmo com o conteúdo fechado.

**Prior art, no Ethereum: o Lit Protocol.** É uma rede descentralizada de gestão de chaves com criptografia de limiar e *access control conditions*: ao menos dois terços dos nós precisam concordar, e o usuário precisa satisfazer as condições, antes que os nós forneçam a parte da chave para decifrar. Tem recurso de *Time Lock*, combinável com outras condições por AND e OR, e o fluxo típico guarda o cifrado em IPFS ou Arweave e pede a decifração aos nós. ([primer](https://spark.litprotocol.com/lit-protocol-a-primer/), [access control](https://spark.litprotocol.com/working-with-decentralized-access-control/), [anúncio do time lock](https://twitter.com/LitProtocol/status/1521516009701711873))

As fontes encontradas sobre o Lit têm de três a quatro anos. **Não verificado:** a sintaxe atual, e se existe equivalente na Stellar.

**O resíduo, e ele é o mesmo da vista física.** Quem consultou durante a janela pode copiar o que leu. Isso vale igualmente no cartório: o advogado com carga dos autos tira cópia. A resposta institucional nunca foi impedir — é **atribuir**: fica registrado quem pediu, com que fundamento, que fragmento recebeu e quando. Se o conteúdo aparecer fora, o conjunto de suspeitos é exatamente a lista registrada.

Ou seja: não é limitação do desenho on-chain. É a mesma propriedade do instituto que ele traduz.

---

## 5. Prior art, com o que encontrei

**Kleros** sorteia jurados, no Ethereum, desde 2018 — hoje na Arbitrum One. Jurados se autosselecionam fazendo stake de PNK, e o sorteio é **proporcional ao stake**, para resistência a Sybil. A documentação deles afirma que o procedimento torna extremamente difícil ser selecionado de propósito para um caso em que você está envolvido. Tipicamente três jurados por disputa. ([docs](https://docs.kleros.io/welcome/faq), [paper abreviado](https://medium.com/kleros/kleros-a-decentralized-court-system-for-the-internet-abridged-1e415c04604a))

**Aragon Court** sorteava de um pool ativo, também ponderado por stake, exigindo 10.000 ANJ ativados. Um paper acadêmico de 2023 afirma que ele resolveu **menos de 50 disputas**. ([guia pré-lançamento](https://blog.aragon.org/juror-pre-activation-guide/), [paper](https://link.springer.com/article/10.1365/s43439-023-00086-x))

**A distinção que precisa sobreviver:** os dois respondem *"quem vai julgar?"* com um **mercado de jurados anônimos selecionados por stake**. O sorteio aqui responde com **o colegiado da própria instituição, fixado antes do caso**. Para disputa comercial, mercado funciona. Para conduta dentro de uma comunidade, jurado anônimo remunerado por stake julgando uma acusação de assédio é o que nenhum conselho de ética do mundo real aceita — por isso eles são compostos por pares eleitos.

**Não verificado:** se alguém já fez sorteio de colegiado, com rol anterior ao caso, na Soroban. A busca em 591 repositórios não trouxe nada, mas busca por palavra-chave não prova ausência. Existem `Stellar-VRF` e `NebulaVRF/vrf-contract` fazendo aleatoriedade verificável na Stellar, o que é vizinho e precisa ser lido.

**E o achado que vale o primeiro slide:** o sorteio de relator do **STF** é feito por um **algoritmo de código fechado** que, segundo o resumo do artigo, alegadamente distribui os casos aleatoriamente conforme a carga de cada gabinete. Acadêmicos tiveram que **construir um modelo estatístico para testar se aquilo era mesmo aleatório**. ([Marcondes, Peixoto e Stern, 2018](https://arxiv.org/abs/1805.08141))

**Não verificado:** qual foi a conclusão do artigo. O resumo não a traz e eu não li o PDF. **Não afirmar que encontraram viés.**

Mas o fato que não depende da conclusão já é a tese inteira: **quando o sorteio não é verificável, a única forma de auditá-lo é teste estatístico depois do fato.** Sorteio verificável torna o teste desnecessário — qualquer pessoa recomputa o embaralhamento a partir do beacon e confere o painel.

---

## 6. Ordem de verificação

**Portão 0 — a semente.** Confirmar empiricamente na testnet: reseedar o PRNG local com um valor de beacon via `Prng::seed` e conferir que `shuffle` é determinístico e recomputável fora da cadeia. Medir o custo para rol de 50, 200 e 1.000. Sem isso, não há sorteio.

**Portão 0b — o beacon.** Decidir a fonte de entropia externa e verificar que a rodada usada é posterior ao commitment do rol e do caso. É o mesmo problema que o projeto anterior resolveu, então há caminho conhecido.

**Portão 0c — prior art.** Ler `Stellar-VRF` e `NebulaVRF/vrf-contract`. Busca de código por sorteio de colegiado na Soroban.

Depois: rol com janela de contestação → sorteio do corpo → recusas com substituto pela ordem → sobre-sorteio e dispensa → consequência graduada do silêncio → commitment das peças → provas ZK de propriedades do selado → limiar de decifração acoplado ao deferimento → registro de acesso.
