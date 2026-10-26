# Gravidade, afastamento e revelação tardia

**Estudo e proposta · 26 de outubro de 2026 · T-044**

> Nada aqui está construído. Este documento estuda três pedidos feitos em 26/10 e propõe o que mudar no contrato, em duas tarefas (T-045, T-046) que só começam depois de a pessoa responsável aprovar as invariantes da §5.
>
> Os números da §1 são **calculados** num modelo (`python3 scripts/tamanho_do_painel.py`, conferido pelo G6). O modelo é uma estimativa do risco real, não uma medição. Onde ele deixa de valer está escrito ao lado. Os parâmetros da rede são **citados**: foram lidos no testnet hoje com `stellar network settings`.

Os três pedidos:

1. Um conselho não decide só expulsão. Ele também decide rebaixamento e perda de poder. A gravidade deveria mudar o tamanho do painel.
2. Durante o caso, a pessoa julgada deveria perder os poderes com que poderia interferir.
3. Os membros do conselho deveriam ficar ocultos por mais tempo, para não sofrerem pressão nem retaliação. A escolha foi revelar tarde (opção B).

---

## 1. Gravidade e tamanho do painel

### A escala

O Contributor Covenant 2.1 tem quatro níveis de consequência (citado, conferido em 26/10 em contributor-covenant.org):

1. **Correção**: advertência privada e escrita.
2. **Advertência**: com prazo sem interação com as pessoas envolvidas.
3. **Banimento temporário**.
4. **Banimento permanente**.

Projetos reais acrescentam degraus intermediários: perda do direito de commit, saída de um comitê, rebaixamento de mantenedor. O Pinakion não decide qual é a sanção (THESIS, *What it is, and is not*). Quem chama o módulo encaixa cada sanção sua num dos quatro níveis.

### O modelo

O sorteio selado senta os `k` menores `y`. O modelo trata isso como um subconjunto uniforme de `k` entre os `N` elegíveis, e trata um **bloco** como `a` jurados que votam juntos e se conhecem. Esse é o pior caso, o mesmo de `docs/limite-inferior-do-rol.md` §7.

- O bloco **impõe** uma decisão quando tem os `m` votos que ela exige.
- O bloco **veta** uma decisão quando tem mais de `k − m` votos.

Recusas, ausências e a ampliação única do limiar em `finalize` ficam de fora do modelo. A conta é exata (hipergeométrica), não uma simulação.

A primeira tabela refaz a §4 de `limite-inferior-do-rol.md` para N = 100. Os números batem com os de lá, arredondados.

<!-- tamanho_do_painel: reconferencia -->
| Bloco | k = 3 | k = 5 | k = 7 | k = 9 | k = 12 |
|---|---|---|---|---|---|
| 20 % | 10.1 % | 5.3 % | 2.8 % | 1.5 % | 0.2 % |
| 30 % | 21.3 % | 15.8 % | 11.8 % | 8.9 % | 2.9 % |
| 40 % | 35.1 % | 31.4 % | 28.4 % | 25.8 % | 14.3 % |
| 45 % | 42.4 % | 40.5 % | 38.8 % | 37.4 % | 24.7 % |

### O que aumentar o painel compra

Esta é a chance de um bloco de 30 % do rol ter maioria simples no painel:

<!-- tamanho_do_painel: captura -->
| Rol | k = 3 | k = 5 | k = 7 | k = 9 | k = 11 | k = 15 |
|---|---|---|---|---|---|---|
| N = 20 (bloco 6) | 20.2 % | 13.1 % | 7.8 % | 3.8 % | 1.2 % | 0.0 % |
| N = 40 (bloco 12) | 20.9 % | 14.9 % | 10.4 % | 7.2 % | 4.7 % | 1.7 % |
| N = 100 (bloco 30) | 21.3 % | 15.8 % | 11.8 % | 8.9 % | 6.6 % | 3.7 % |
| N = 200 (bloco 60) | 21.5 % | 16.0 % | 12.2 % | 9.4 % | 7.2 % | 4.3 % |

**O painel compra pouca segurança, e devagar.** Ir de 3 para 9 jurados reduz o risco de cerca de 1 em 5 para cerca de 1 em 11.

O tamanho do rol quase não muda essa chance. Ele só ajuda quando `k` chega perto de `N`, como em N = 20 com k = 15. Mas nesse ponto o painel quase é o rol, e o selo acaba (veja abaixo).

Este é o menor `k` ímpar que mantém a chance de maioria do bloco abaixo de um alvo, com N = 100:

<!-- tamanho_do_painel: k_minimo -->
| Bloco (N = 100) | ≤ 10 % | ≤ 5 % | ≤ 1 % |
|---|---|---|---|
| 10 % | 1 | 3 | 5 |
| 20 % | 5 | 7 | 11 |
| 30 % | 9 | 13 | 23 |

Um painel de 23 contra um bloco de 30 % não serve a um projeto de código aberto. Contra um bloco grande, o que protege não é o tamanho do painel. São três outras coisas:

- **Impedimentos declarados antes do `lock`.** Eles tiram o bloco do sorteio, em vez de diluí-lo.
- **O preço de montar o bloco antes de saber o caso.** É o argumento econômico de `limite-inferior-do-rol.md` §2.
- **A regra de votação**, que é a próxima tabela.

### Maioria qualificada: protege um lado e arma o outro

Bloco de 30 em 100:

<!-- tamanho_do_painel: dois_tercos -->
| k | Maioria simples: impor ou vetar | 2/3 exige | 2/3: impor | 2/3: vetar |
|---|---|---|---|---|
| 3 | 21.3 % | 2 de 3 | 21.3 % | 21.3 % |
| 5 | 15.8 % | 4 de 5 | 2.7 % | 47.4 % |
| 7 | 11.8 % | 5 de 7 | 2.4 % | 35.0 % |
| 9 | 8.9 % | 6 de 9 | 2.0 % | 26.3 % |
| 11 | 6.6 % | 8 de 11 | 0.3 % | 43.0 % |
| 15 | 3.7 % | 10 de 15 | 0.2 % | 26.5 % |

Com 9 jurados e exigência de 2/3:

- um bloco de 30 % quase nunca **impõe** a pena (2.0 %);
- o mesmo bloco **veta** a pena em 1 de cada 4 painéis (26.3 %).

Exigir 2/3 é uma escolha explícita: na dúvida, não se expulsa. Para o nível 4 ela parece certa, mas quem a adota precisa dizer que é isso que está comprando.

O `k` par e alguns ímpares (5, 11) ficam mal com 2/3, porque o arredondamento sobe a exigência. Por isso a proposta usa 3, 5, 7 e 9.

A regra de votação não é do Pinakion, que não conta votos. Ela é de quem chama o módulo, ou do Tessera. O Pinakion só fixa `k` e publica a gravidade.

### O outro lado: painel grande, selo fraco

Antes da revelação, quem quer pressionar não sabe quem sentou. Mas sabe que qualquer inscrito sentou com chance `k/N`:

<!-- tamanho_do_painel: selo -->
| Rol | k = 3 | k = 5 | k = 7 | k = 9 |
|---|---|---|---|---|
| N = 19 | 15.8 % | 26.3 % | 36.8 % | 47.4 % |
| N = 40 | 7.5 % | 12.5 % | 17.5 % | 22.5 % |
| N = 100 | 3.0 % | 5.0 % | 7.0 % | 9.0 % |
| N = 200 | 1.5 % | 2.5 % | 3.5 % | 4.5 % |

No rol da rodada de testnet (N = 19), um painel de 9 deixa cada palpite certo quase metade das vezes. **Contra a captura, a gravidade pede um painel maior. Contra a pressão, o selo pede `k/N` pequeno. Logo, gravidade alta exige rol grande.**

### Proposta

A tabela abaixo é padrão, para a governança que usa o módulo trocar. Ela é fixada no construtor, antes de qualquer caso.

<!-- tamanho_do_painel: proposta -->
| Nível | Exemplos | `k` | Rol mínimo | Regra de votação recomendada | Bloco de 30 % impõe / veta (N = 100) | `k/N` no rol mínimo |
|---|---|---|---|---|---|---|
| 1 | correção | 3 | 16 | maioria simples | 21.3 % / 21.3 % | 18.8 % |
| 2 | advertência | 5 | 40 | maioria simples | 15.8 % / 15.8 % | 12.5 % |
| 3 | banimento temporário, perda de cargo ou de commit | 7 | 40 | maioria simples | 11.8 % / 11.8 % | 17.5 % |
| 4 | banimento permanente, expulsão | 9 | 100 | 2/3 (6 de 9) | 2.0 % / 26.3 % | 9.0 % |

- **De onde vêm os pisos do rol.** Seguem as faixas de `limite-inferior-do-rol.md` §6: abaixo de 16 recusar; entre 16 e 40, só k = 3; entre 40 e 100, k de 5 a 7; com 100 ou mais, o desenho completo.
- **Como ler os riscos.** A coluna de risco usa N = 100. Na prática, o rol mínimo de cada nível muda pouco esses números (tabela de captura).
- **Os pisos são uma escolha.** O modelo não prova que estão certos.

### Quem classifica a gravidade

O escrivão escolhe o nível no `lock`, e isso é uma escolha humana. O desenho torna essa escolha pouco perigosa de duas formas:

- **A gravidade é um teto.** O painel aberto como "advertência" só pode decidir até advertência. Rebaixar o nível só limita a pena, o que protege a pessoa julgada; nunca a expõe a um painel menor do que a pena exigiria. Fazer valer esse teto é tarefa de quem chama o módulo. O Pinakion publica o nível no evento `Fechado`, antes de a rodada do drand existir.
- **A escolha é pública.** O nível aparece com nome e hora, e pode ser contestado. É auditável, não impedido.

---

## 2. Afastamento cautelar

### Uma correção ao que eu disse antes

Na conversa, eu disse que o `impede` "tira a pessoa do rol inteiro, não só daquele caso". Isso está errado na prática.

Este contrato fecha o rol uma vez só (DEC-023), então cada contrato é um caso. O `impede` já vale para um caso só. O buraco é outro: impedir é **opcional e depende do escrivão**. Se ele não impede a pessoa julgada e ela é sorteada, o `claim` dela é selado: ela senta em silêncio no próprio caso, e isso só aparece na revelação.

### 2.1 Dentro do Pinakion: a parte nunca senta no próprio caso

O `lock` passa a receber a pessoa julgada: `lock(gravidade, revela_em, acusado)`.

- **Se ela está inscrita e autorizada,** o contrato a impede na mesma chamada. Emite `Impedido`, a folha vira zero, e `N` desce. Isso acontece antes de a rodada do drand ser fixada, então não mira a ordem sorteada (o mesmo argumento do Q-21).
- **O escrivão não pode ser a pessoa julgada.** O `lock` com `acusado == escrivão` é recusado. Quem fecha o rol não fecha o próprio caso.
- **Os verificadores (Python e JS) passam a conferir** que a pessoa nomeada no `Fechado` tem um `Impedido` antes dele, se estava inscrita, e que nenhum assento é dela na revelação.

O bilhete da pessoa julgada fica desautorizado naquele contrato. É, literalmente, o *pinakion* dela recolhido durante o caso.

### 2.2 Fora do Pinakion: o que a governança suspende

O contrato não tira credenciais do GitHub nem de nenhum outro sistema. O que ele pode fazer é **publicar uma regra que outros leem**:

- **Uma consulta nova:** `afastado(endereco) -> bool` (no código, desde T-045, `suspended(endereco)`, em inglês como as outras chamadas públicas). Ela é verdadeira só para a pessoa julgada, do `lock` até `lock + afastamento`. O `afastamento` é um prazo em ledgers fixado no construtor, antes do caso.
- **O prazo vence sozinho.** Ninguém prorroga.
- **Quem lê:** um contrato da governança (um multisig, um contrato de papéis) pode consultar `afastado` e suspender o que é dele. Um bot do projeto pode ler o evento e tirar permissões fora da cadeia.
- **O Pinakion não vê o veredito,** então o afastamento termina no prazo. Encerrá-lo antes, depois do veredito, é poder de quem chama o módulo.

O que a governança deveria suspender, e só isso:

- poder sobre o rol e sobre o papel de escrivão;
- moderação dos canais onde o caso corre;
- contato com quem denunciou e com as pessoas envolvidas. É o mesmo "sem interação com as pessoas envolvidas" do nível 2 do Covenant.

O que **não** deveria suspender: o direito de se defender, de ler o que pesa contra si e de responder. O afastamento cautelar não é pena e não deve ter cara de pena.

### O risco que fica: a denúncia como arma

Quem abre um caso afasta alguém por `afastamento` ledgers. Uma denúncia vazia tira um rival do caminho por esse prazo. O desenho não impede isso; ele limita o estrago:

- o afastamento é estreito e curto;
- é fixado antes;
- cada caso aberto é público, com nome e hora.

É auditável, não impedido.

### Quem denunciou

Nenhum evento leva o nome de quem denunciou. O Covenant exige a privacidade dessa pessoa. Isso deixa um problema aberto (Q-27): se ela for sorteada, impedi-la publicamente a expõe, e deixá-la sentar põe a parte no painel. Hoje a melhor saída é ela simplesmente não reivindicar o assento. Ela aparece então como "sorteada e ausente" na revelação, sem motivo dado. Não está resolvido.

---

## 3. Revelação tardia (opção B)

### A regra

O construtor recebe `espera_minima`, em ledgers. O `lock` recusa `revela_em < ledger do lock + espera_minima`.

- **Quem fixa a regra:** quem chama o módulo, antes de qualquer caso. O escrivão ainda escolhe `revela_em`, mas não abaixo desse piso.
- **O Q-24 fica respondido:** o padrão documentado é "depois do veredito e do recurso", com um piso que o contrato faz valer.
- **Um prazo proposto, para trocar:** 30 dias, que dão 518.400 ledgers. A conta usa o fechamento-alvo de 5 s por ledger e é uma estimativa; o tempo real de fechamento varia.

### O que a rede impõe: TTL

Parâmetros citados, lidos no testnet em 26/10 com `stellar network settings`:

- `min_persistent_ttl` = 120.960 ledgers;
- `max_entry_ttl` = 3.110.400 ledgers;
- fechamento-alvo de 5.000 ms.

Isso dá cerca de **7 dias** e **180 dias**. É uma estimativa: ledgers multiplicados por 5 s.

**Hoje o contrato nunca estende o TTL** (nenhum `extend_ttl` em `contracts/kleroterion/src/lib.rs`). Na rodada de 26/10 a revelação veio minutos depois, e isso não importou. Com uma revelação em 30 dias, os dados que `reveal` lê podem ser arquivados antes de serem lidos:

- o compromisso de cada jurado;
- os `y` reivindicados;
- o painel;
- a instância.

O que T-046 precisa fazer:

- **Estender os TTLs.** O `lock` estende o TTL da instância e do código, e o `finalize` o dos `y` e do painel, até `revela_em + janela`. O teto é `env.storage().max_ttl()`, que existe no soroban-sdk 29.0.1 (`src/storage.rs:153`, conferido hoje).
- **Revelação além de ~180 dias** não cabe numa extensão só. Alguém precisa estender de novo no caminho. A extensão de TTL é uma operação que qualquer conta pode pagar, sem permissão do contrato. `[VERIFY: ExtendFootprintTTLOp não exige autorização do dono da entrada]`.
- **Entradas arquivadas podem ser restauradas.** `[VERIFY: restauração automática no protocolo atual, e o custo]`. Restaurar não apaga nada, mas custa, e a página de bilhetes precisa saber lidar com isso.

### O que a revelação tardia compra, e o que não compra

- **Compra:** os nomes ficam selados durante o caso, durante o recurso, e por um tempo depois. A pressão sobre o voto acaba com o voto.
- **Não compra:** proteção contra retaliação depois da revelação. Ela só adia. Quem quer se vingar em seis meses ainda pode.
- **Custa:** a terceira frase da garantia ("depois, qualquer um recalcula o painel") continua verdadeira, mas o "depois" fica mais longe. Até lá, cada assento vale pela prova no `claim`, e só por ela. Isso põe mais peso no setup feito por uma só parte durante mais tempo.

---

## 4. Ordem de construção

- **T-045 — gravidade, parte e afastamento.**
  - Contrato: construtor com a tabela de níveis e `afastamento`; `lock(gravidade, revela_em, acusado)`; `afastado()`.
  - Eventos e verificadores: o `Fechado` ganha `gravidade`, `acusado` e `afastado_ate`; um evento `Constituido` no construtor publica a tabela, para os verificadores conferirem `k` a partir dos eventos.
  - Testes e números: verificadores Python e JS, vetor selado regenerado, custos refixados.
- **T-046 — revelação tardia.**
  - Contrato: `espera_minima` no construtor e a recusa no `lock`; extensão de TTL no `lock` e no `finalize`.
  - Testes: um teste que avança o ledger além de `min_persistent_ttl` e revela.
  - Página de bilhetes: mostra "selado até o ledger X".
- **Testnet:** nenhuma das duas roda no testnet sem um pedido novo. A evidência de 26/10 continua valendo para o contrato daquele dia, e os documentos dizem isso.

---

## 5. Invariantes propostas (precisam de aprovação)

| ID | Enunciado | Teste que a faria valer |
|---|---|---|
| INV-18 | A pessoa julgada nunca senta no próprio caso: o `lock` a nomeia e a impede na mesma chamada, e o escrivão não pode ser ela | `o_acusado_inscrito_e_impedido_no_fechamento`, `escrivao_acusado_e_recusado`, verificadores |
| INV-19 | `k` vem da tabela de gravidade fixada no construtor; ninguém escolhe `k` no `lock`; abaixo do rol mínimo do nível, o `lock` é recusado | `k_vem_da_gravidade`, `rol_abaixo_do_minimo_do_nivel_e_recusado` |
| INV-20 | `afastado` é verdadeiro só para a pessoa julgada, do `lock` até `lock + afastamento`, e nada o prorroga | `afastado_so_para_o_acusado_e_so_no_prazo` |
| INV-21 | `revela_em ≥ lock + espera_minima`; e o que `reveal` lê vive pelo menos até `revela_em + janela`, ou até o máximo da rede, o que vier antes | `revelar_antes_da_espera_minima_e_recusado`, `revelar_depois_de_ttl_minimo_funciona` |

A INV-19 tira do escrivão a escolha de `k`, que hoje é dele. A INV-18 muda a assinatura do `lock`, que é a interface pública: precisa de um DEC novo, como as anteriores.

---

## 6. Limites deste estudo

- **O modelo é hipergeométrico puro.** Não modela recusas, ausências, a ampliação do limiar nem simpatia parcial. O bloco disciplinado é o pior caso, e superestima a ameaça real.
- **Os pisos de rol e os `k` da tabela são escolhas.** Seguem faixas que nós mesmos traçamos (`limite-inferior-do-rol.md`); nenhum estudo externo os sustenta.
- **O afastamento só tira poder de quem lê.** Fora do Pinakion, nada acontece sem a governança ligar `afastado` às permissões reais.
- **A denúncia como arma fica auditável, não impedida.**
- **Ainda sem resposta:** o conflito de quem denunciou (Q-27) e "o painel não pode ser do corpo a que a pessoa pertence" (`desenho-sorteio-e-sigilo.md` §1). O estudo não resolve nenhum dos dois.
- **A revelação tardia adia a retaliação, não a impede.** Depende de TTL, e dois fatos dela estão marcados `[VERIFY]`.
- **Nada aqui foi construído, testado no contrato ou rodado no testnet.**
