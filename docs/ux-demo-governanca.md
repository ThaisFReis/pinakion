# UX da demo ao vivo — referências e aplicação

2026-10-26 · T-057 · Escopo: `web/vivo/`.

Pedido: uma interface mais minimalista, clara e confiável, com fundo claro e sem os clichês visuais de cripto. A comparação abaixo informa decisões de interface; não demonstra que uma cor, fonte ou raio aumente a segurança do sistema. Não houve pesquisa com usuários.

## Referências verificadas

| Referência primária | O que foi observado | Aplicação no Pinakion |
|---|---|---|
| [Agora / Optimism — Security Council Operating Budget](https://vote.optimism.io/proposals/11016722315079092316541525436963784553487811157319342152041523932758691863591) | Página aberta no navegador: texto principal à esquerda; placar, quórum, limiar e estado juntos à direita. Link para a discussão completa. | Antes do início: autos na coluna principal; regras e ação na lateral. Durante a execução: acompanhamento centralizado, conforme o pedido posterior para gravação. Quórum de presença e votos necessários à sanção são rótulos diferentes. |
| [Snapshot / Aave DAO](https://snapshot.box/#/s:aavedao.eth) | Navegador: navegação por seções; propostas em linhas, com título, estado, autor, contagem de votos e tempo. Separadores discretos e hierarquia predominantemente sem serifa. | Títulos curtos, abas de documentos, dados próximos da tarefa e menos superfícies decorativas. O tema escuro observado não é aplicado. |
| [Snapshot — votar em uma proposta](https://docs.snapshot.box/user-guides/voting/vote) | Documentação oficial distingue estratégia de voto e validação de elegibilidade. | Explicitar as regras da sessão antes da ação. Não importar voto ponderado por token nem regras de elegibilidade de outra DAO. |
| [Tally / Cactus — votação](https://docs.tally.xyz/how-to-use-tally/voting-on-proposals/) | Guia oficial: proposta → ação de votar → escolhas → envio. Explica também a espera de indexação. | Ação principal identificável, estado de envio e detalhes de erro acessíveis. A janela e o envio automático da demo permanecem os existentes e são declarados antes do início. |
| [Fórum Optimism / Discourse](https://gov.optimism.io/) | Navegador: tópicos, categorias, respostas e atividade. A estrutura separa discussão e navegação. | Documentos legíveis primeiro; edição explícita; informações complementares em seções expansíveis. Não simular um fórum nem criar discussão sem implementação. |
| [Discourse — criação e gestão de enquetes](https://meta.discourse.org/t/creating-and-managing-polls/77548) | Guia oficial separa tipo de voto, visibilidade dos resultados, grupos elegíveis e encerramento. | Mostrar prazo, opções e estado como informações distintas. Preservar a votação secreta do Tessera. |

**Atualização de nome:** [a documentação oficial informa a mudança de Tally para Cactus em junho de 2026](https://docs.tally.xyz/tally-is-now-cactus/). Resultados de busca antigos sobre encerramento do Tally não foram tratados como descrição atual da interface. A consulta ao Cactus foi documental, não uma execução autenticada de voto.

## Decisões aplicadas

- Fundo branco, superfície cinza muito clara, texto grafite e um único destaque azul-petróleo. Cor reforça rótulos; não certifica segurança.
- Public Sans na interface. Source Serif 4 apenas na assinatura Pinakion. IBM Plex Mono em tempo e identificadores técnicos.
- Cabeçalho de trabalho compacto: tipo do caso e descrição curta. Sem hero promocional, ilustração, gradiente ou estatísticas ornamentais.
- Cantos de 4 px nos controles; moldura da sessão e diálogo com 8 px. Linhas e alinhamento organizam o conteúdo; ícones Phosphor identificam os grupos de etapas.
- Leitura dos autos por padrão; botão explícito para editar. Prévia restrita a texto, títulos, listas e ênfase, construída com nós DOM; não executa HTML nem carrega conteúdo externo dos documentos.
- Rol, painel, quórum e limiar vêm de `PLANO` e da regra de `processo::abrir`. O teste compara as regras exibidas ao evento `CasoAberto` do registro real de testnet.
- A contagem de etapas decorre dos callbacks do motor. Nenhuma porcentagem de tempo ou progresso de rede é inventada.
- O resultado do júri e o resultado da conferência são separados. Uma conferência divergente não recebe mensagem de sucesso.

## Origem dos números

- Rol de 40, painel de 7, quórum de 5 e limiar de 4: configuração de `web/vivo/motor.mjs::PLANO` e regra em `contracts/processo/src/lib.rs::abrir`; comparação com `docs/evidencia-processo-registro.json` em `web/vivo/ui.test.mjs`.
- 15 etapas: enumeração dos callbacks existentes do motor; quatro grupos são organização visual.
- 45 segundos: janela de escolha da interface anterior, preservada. É uma condição da demonstração, não recomendação de acessibilidade para uma votação real.
- 15–20 minutos: estimativa herdada da página anterior, dependente da rede e da geração das provas. Não foi medida novamente nesta revisão e não é uma promessa de duração.
- Medidas tipográficas, espaçamentos e raios são escolhas de design, não resultados empíricos.

## Validação e limites

G1–G8 passaram nesta revisão. G5 inclui testes de conferência divergente, resultado improcedente válido, dados incompletos e regras comparadas ao registro de testnet. A interface foi verificada no navegador com bibliotecas reais para carregamento, e com motor simulado para votação, resultado, divergência e falhas. Os cenários simulados foram servidos separadamente, com aviso visível, sem transações.

Esta alteração não executou novamente o ciclo completo na testnet e não constitui auditoria. Contratos, motor, circuito, formatos e verificador não mudaram. A prova de segurança continua dependente dos limites documentados do projeto, não da aparência da página. A automação de clique do navegador teve timeouts; navegação por teclado permitiu concluir as verificações dos estados. Acessibilidade foi examinada funcionalmente, sem auditoria completa de leitor de tela.

## Refinamento para gravação — critérios de verificação (T-057)

- Ao iniciar, o acompanhamento ocupa o centro: etapa atual, jornada e diário no mesmo enquadramento de desktop.
- Ao abrir a urna, a votação assume o centro e recebe foco; consultar os autos não interrompe o motor. A urna fecha a consulta para que o prazo não passe despercebido.
- Enviar votos uma vez retorna ao acompanhamento. Resultado e erros aparecem no mesmo lugar, com acesso ao diário.
- O diário permite interromper apenas a rolagem automática, sem pausar a execução.
- Movimento respeita a preferência por animação reduzida; títulos e estados também comunicam as mudanças por texto.
- Verificar desktop, viewport estreita, voto manual, envio automático, erro e divergência usando dados claramente simulados. O teste da apresentação não substitui uma execução completa na testnet.

### Implementação e evidência do refinamento

A revisão abre no enquadramento da sessão ao clicar em “Revisar e continuar”. A execução começa somente em “Criar caso na testnet”. A área principal alterna entre operação, votação e resultado; jornada e diário permanecem próximos. O diálogo serve à consulta de documentos, regras e limitações. A urna e os estados finais fecham a consulta e recebem foco. O prazo continua visível no diálogo quando há votação aberta.

Ícones Phosphor regulares locais, sem runtime ou serviço novo. Azul-petróleo para andamento, âmbar para participação e mensagens textuais para cada estado. Animações curtas na mudança de etapa e entrada de registros, com um indicador discreto durante a espera. As animações e transições são condicionadas a `prefers-reduced-motion: no-preference`; o JavaScript também consulta a preferência antes de reativar a entrada.

Verificação manual no navegador, com fixture explicitamente simulada em origem separada: enquadramento 1280 × 720, voto manual com resultado improcedente, envio automático único após 45 s com divergência, falha de execução, consulta interrompida pela abertura da urna, pausa de rolagem, Escape e regras acessíveis no diálogo. Viewport 390 × 844 sem overflow horizontal; no celular há rolagem vertical. Dimensões são condições do teste, não estatística de usuários. G5 (`npm test`) passou novamente. Não foi feito novo ciclo completo na testnet, auditoria de leitor de tela ou medição de confiança com usuários.

O servidor local da página real precisou ser reiniciado ao final. A aba antiga havia virado uma página de erro com protocolo bloqueado pela automação; a reabertura dessa aba não foi validada nesta passagem. Os testes visuais acima foram concluídos antes, no servidor de fixtures. A captura `demo-votacao-central.png` no diretório de visualizações desta conversa contém o aviso de simulação.

### Passagem da preparação para a sessão — critérios (T-057)

A primeira ação abre uma revisão no enquadramento da sessão, sem chamar o motor. Voltar preserva os documentos. Somente “Criar caso na testnet” inicia a execução; ações repetidas não criam outra execução. Alterações feitas na consulta entram no caso e documentos vazios bloqueiam o início. A revisão explica a passagem para autos de consulta, o envio automático dos votos e a manutenção da aba aberta. A transição conserva a identidade do caso e respeita movimento reduzido.

Validação da passagem: a página real carregou e abriu a revisão sem iniciar o motor. Em fixture com motor simulado, a ida e volta foi repetida, as regras mantiveram um único conjunto de IDs, apagar um documento desabilitou criar, a edição sobreviveu ao retorno e criar conduziu à votação. O retorno de foco ao fechar a consulta na revisão é explícito. `npm test` e checagens de sintaxe/diff passaram. Não houve nova execução na testnet. O servidor local temporário recebeu SIGTERM durante a revisão; a prévia foi reiniciada como processo separado do terminal.

Conferência final na página original: revisão e botão de criação visíveis no viewport de 1280 × 720; viewport de 390 × 844 sem overflow horizontal; Escape devolve o foco ao título da revisão. Captura da página original: `demo-revisao-sessao.png` no diretório de visualizações desta conversa. Nenhuma transação foi iniciada nessa conferência.
