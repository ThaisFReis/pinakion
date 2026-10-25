# O rol, o sorteio, as recusas e a máquina de estados

**Documento de desenho · 25 de outubro de 2026**

> Cada regra abaixo tem fonte. O que eu calculei está marcado como cálculo. O que não verifiquei está marcado como não verificado.
>
> Ressalva de fonte: a maior parte do texto legal veio de sites jurídicos e doutrina, não do Planalto. Para citação formal, conferir a redação vigente.

---

## 1. A aritmética que organiza tudo

O Tribunal do Júri sorteia **25**, exige **15 presentes** para instalar, dá **3 recusas imotivadas por parte**, e senta **7**. Esses números parecem arbitrários. Não são.

Calculando a probabilidade de instalar a sessão, com comparecimento independente:

| Comparecimento | P(ao menos 15 dos 25) |
|---|---|
| 90% | 100,0% |
| **80%** | **99,4%** |
| 75% | 97,0% |
| 70% | 90,2% |
| 65% | 77,1% |
| 60% | 58,6% |
| 50% | 21,2% |

**O CPP está calibrado para cerca de 80% de comparecimento.** Abaixo de 70% o sistema começa a falhar, e a 60% mais de quatro sessões em dez não instalam.

E os dados reais de comparecimento ao júri: Canadá cerca de 80%, Nova Zelândia cerca de 80%, Filadélfia entre 58% e 64%. Ou seja: a regra brasileira funciona onde o comparecimento é o do Canadá, e quebraria na Filadélfia — que, não por acaso, tem uma crise documentada de júri.

**Consequência de desenho: o fator de sobre-sorteio não é constante a copiar, é parâmetro a calibrar.** Quanto sortear para ter 99% de chance de atingir o quórum:

| Comparecimento | Sortear para 15 presentes |
|---|---|
| 90% | 21 |
| 80% | **25** (é o número do CPP) |
| 70% | 30 |
| 60% | 36 |
| 50% | 45 |
| 40% | 58 |

E para painéis menores, que é o caso provável de uma comunidade pequena — exigindo sempre `sentados + recusas + 2 de margem`:

| Sentar | Recusas | Exigir presentes | p=80% | p=70% | p=60% | p=50% |
|---|---|---|---|---|---|---|
| 7 | 6 | 15 | 25 | 30 | 36 | 45 |
| 5 | 4 | 11 | 19 | 23 | 28 | 35 |
| 3 | 2 | 7 | 13 | 16 | 20 | 25 |

*(cálculo próprio, distribuição binomial, comparecimento independente)*

**A regra que o contrato aplica:** o fator de sobre-sorteio é recalculado a cada ciclo a partir do **comparecimento medido no ciclo anterior**. A comunidade não escolhe o número; ela o descobre. Se o comparecimento cai, o rol sorteado cresce sozinho.

**Limite honesto do cálculo:** comparecimento independente é simplificação. Na prática há correlação — feriado, fuso, um caso que ninguém quer tocar. Então a margem de 2 existe, e o `min_presentes` é piso duro, não meta.

---

## 2. O rol

### Como a instituição faz

| Regra | Fonte |
|---|---|
| Número de jurados escalonado por população: comarca com mais de 1 milhão de habitantes, de 800 a 1.500; mais de 100 mil, de 300 a 700; menores, de 80 a 400 | art. 425 |
| O §1º permite ampliar o número e organizar **lista de suplentes** | art. 425 §1º |
| Lista geral publicada pela imprensa **até 10 de outubro**, com editais afixados à porta do Tribunal | art. 426 |
| **Qualquer pessoa pode reclamar**; a lista pode ser alterada de ofício ou por reclamação até **10 de novembro**, data da lista definitiva | art. 426 |
| **Quem integrou o Conselho de Sentença nos 12 meses anteriores à publicação fica excluído** da lista geral — a finalidade declarada é evitar o "jurado profissional" | art. 426 §4º |
| A lista geral é completada todo ano | art. 426 §5º |

Fontes: [art. 425–426 comentados](https://www.aurum.com.br/blog/cpp-comentado/art-425-a-426-cpp/), [Projuris](https://www.projuris.com.br/cpp/art-425-ao-art-426-do-cpp/), [Flavio Meirelles Medeiros sobre o art. 426](https://flaviomeirellesmedeiros.com.br/artigo-426o-cpp/), [Dizer o Direito sobre lista de jurados](https://www.buscadordizerodireito.com.br/jurisprudencia/detalhes/f1e2b2c9255d552500a833ac828cd635).

### On-chain

```
Ciclo {
  raiz_rol:         BytesN<32>,   // Merkle de (membro, metadados)
  publicado_em:     u32,          // ledger
  definitivo_em:    u32,          // fim da janela de reclamação
  rodada_minima:    u64,          // nenhum sorteio pode usar rodada anterior a esta
  excluidos:        BytesN<32>,   // raiz de quem serviu no ciclo anterior
  comparecimento:   u32,          // medido no ciclo anterior, em pontos de %
}
```

Três propriedades que vêm das regras acima:

**O rol precede o caso, e isso é verificável.** `rodada_minima` é fixada quando a lista vira definitiva. Nenhum sorteio aceita rodada do beacon anterior a ela. A anterioridade deixa de ser declaração e vira aritmética.

**Reclamação é pública e tem prazo.** Entre `publicado_em` e `definitivo_em`, qualquer um contesta uma entrada. Depois disso a raiz congela. Sem janela, o rol é só uma lista que alguém montou.

**Rotação é obrigatória.** Quem serviu sai do ciclo seguinte. Não é cortesia com o jurado — é o que impede um subgrupo de virar tribunal permanente. A regra brasileira tem nome para o que ela previne: jurado profissional.

---

## 3. O sorteio e o quórum

### Como a instituição faz

| Regra | Fonte |
|---|---|
| O juiz preside o sorteio **a portas abertas**, retirando cédulas até completar **25**, entre o 15º e o 10º dia útil antes da reunião | art. 433 |
| MP, OAB e Defensoria são intimados para acompanhar | art. 432 |
| A sessão instala com **ao menos 15 presentes** | art. 463 |
| **Jurados excluídos por impedimento ou suspeição continuam contando para o número legal** | art. 463 §2º |
| Faltando o número, o juiz sorteia **tantos suplentes quantos necessários** e marca nova data | art. 464 |
| Julgar sem os 15 presentes é **nulidade** | art. 564, III, "i" |

Fontes: [art. 463](https://flaviomeirellesmedeiros.com.br/artigo-463o-cpp/), [art. 464](https://flaviomeirellesmedeiros.com.br/artigo-464o-cpp/), [Juspodivm sobre o número mínimo](https://meusitejuridico.editorajuspodivm.com.br/2017/09/07/qual-o-numero-minimo-de-jurados-para-que-se-inicie-sessao-de-julgamento-no-tribunal-juri/).

### O detalhe que eu não teria inventado

O **§2º do art. 463** é a peça mais fina do conjunto: quem é excluído por impedimento **ainda conta** para os 15.

Pensando como engenheiro, o impulso é o contrário — se a pessoa não pode julgar, não deveria contar. Mas aí o impedimento vira ferramenta: basta arranjar impedimentos suficientes para derrubar o quórum e adiar o julgamento indefinidamente.

A regra brasileira separa duas coisas que parecem uma: **o quórum mede comparecimento, não disponibilidade.** Quem apareceu, contou. O que o impedimento faz é tirar a pessoa daquele conselho, não desfazer a sessão.

No contrato: `presentes` incrementa no comparecimento e **nunca decrementa** por impedimento.

---

## 4. As recusas

### Como a instituição faz

| Regra | Fonte |
|---|---|
| Acusação e defesa recusam, **sem motivar**, até **3 jurados cada** | art. 468 |
| O recusado sai **apenas daquela sessão**, e o sorteio prossegue com os remanescentes | art. 468 |
| Com dois ou mais acusados, as recusas podem ser feitas por um só defensor | art. 469 |
| O STJ entende que o direito é **de cada réu**, não de cada defensor | REsp 1.540.151/MT |
| Se as recusas deixarem **menos de 7** para o Conselho — o "estouro de urna" — os julgamentos são **separados**, julgando-se primeiro o acusado a quem se atribui a autoria | art. 469 §1º e §2º |

Fontes: [art. 468](https://flaviomeirellesmedeiros.com.br/artigo-468o-cpp/), [art. 469](https://flaviomeirellesmedeiros.com.br/1818-2/), [recusa peremptória explicada](https://www.jusbrasil.com.br/artigos/o-que-se-entende-por-recusa-peremptoria-no-tribunal-do-juri/121927357), [REsp 1.540.151/MT](https://www.stj.jus.br/websecstj/cgi/revista/REJ.cgi/ATC?seq=50142195&tipo=51&nreg=201500677452&SeqCgrmaSessao=&CodOrgaoJgdr=&dt=20150929&formato=PDF&salvar=false).

### Por que recusar não vira escolher

Este era o problema que eu não sabia resolver, e a resposta está no art. 468: **o sorteio prossegue com os remanescentes.**

O substituto não é selecionado. Ele é **o próximo da ordem que o beacon já fixou**. Recusar só avança na fila — nunca produz um julgador novo.

Três regras que o contrato impõe, e cada uma fecha um buraco:

**Número fixo e pequeno, por parte.** Recusas ilimitadas são seleção disfarçada: basta recusar até chegar em quem se quer.

**Substituto pela ordem, sem exceção.** A sequência sorteada é a autoridade. O contrato nunca re-sorteia para substituir.

**Impedimento não consome recusa.** São institutos diferentes: impedimento é declarado e tem causa; recusa é imotivada e limitada. Confundir os dois ou dá recusas infinitas a quem alega impedimento, ou pune quem declara um conflito legítimo.

### O estouro de urna, que é um estado, não um erro

Se as recusas deixarem menos do que o necessário, **o julgamento não prossegue**. A instituição não improvisa: ela separa os julgamentos ou adia.

On-chain, o equivalente honesto é um estado terminal `SemPainel`: o caso volta para nova sessão, com nova rodada do beacon e novo sorteio sobre o mesmo rol. Não é falha do sistema — é o sistema recusando-se a julgar com painel insuficiente.

Inventar um preenchimento automático aqui seria pior que o original.

---

## 5. A obrigatoriedade, e por que ela funciona mal

| Regra | Fonte |
|---|---|
| O serviço do júri é obrigatório; o alistamento alcança maiores de 18 anos de notória idoneidade | art. 436 |
| Recusa injustificada: **multa de 1 a 10 salários mínimos**, conforme a condição econômica | art. 436 §2º |
| Não comparecer sem causa legítima, **ou sair antes de ser dispensado**: mesma multa | art. 442 |
| Isenções dos incisos I a VIII valem de pleno direito; maiores de 70 anos e justo impedimento dependem de requerimento | art. 437 |
| Objeção de consciência leva a **serviço alternativo**, sob pena de suspensão dos direitos políticos | art. 438 |

Fontes: [art. 436–446 comentados](https://www.aurum.com.br/blog/cpp-comentado/art-436-a-446-cpp/), [art. 436](https://www.cognijus.com/blog/art-436-cpp), [art. 437](https://modeloinicial.com.br/lei/CPP/codigo-processo-penal/art-437).

Duas observações que mudam o desenho:

**A sanção quase nunca é aplicada.** Na Irlanda, 25.649 faltas em três anos contra 20 multas aplicadas. Na Nova Zelândia, a multa de mil dólares foi imposta uma vez, em 2007. A multa existe no papel, e quem decide aplicá-la é uma pessoa com mais o que fazer.

**O art. 438 é mais interessante que a multa.** Quem recusa por convicção não paga — **presta serviço alternativo**. A obrigação não desaparece, muda de forma. É um desenho melhor que punição pecuniária, e traduz bem: quem não serve num caso entra na fila do próximo, ou assume uma função de menor custo.

**On-chain, a diferença é que a consequência não depende de alguém decidir aplicá-la.** Ela é graduada e automática: primeiro aviso, depois perda de prioridade, depois exclusão do ciclo seguinte. Nunca em um passo, porque silêncio também é chave perdida, doença e notificação que não chegou. E a dispensa com motivo tem que ser barata, senão a pessoa some em vez de declarar.

---

## 6. As duas máquinas de estados

### O ciclo do rol — anual

```
   RASCUNHO
      │ publicar(raiz, janela)
      ▼
   EM RECLAMAÇÃO ─── qualquer um contesta uma entrada ───┐
      │                                                   │
      │ vence a janela                        corrige e republica
      ▼                                                   │
   DEFINITIVO ◄────────────────────────────────────────────┘
      │ fixa rodada_minima; nenhum sorteio usa rodada anterior
      │
      │ fim do ciclo: mede comparecimento, exclui quem serviu,
      │ recalcula o fator de sobre-sorteio
      ▼
   ENCERRADO ──► novo ciclo
```

### O caso

```
   ABERTO                    acusação registrada, fundamento por hash
      │ convoca()            fixa a rodada do beacon, futura
      ▼
   AGUARDANDO O BEACON       ninguém pode agir; a rodada ainda não existe
      │ rodada publicada
      ▼
   SORTEADO                  N posições do rol, na ordem do embaralhamento
      │                      ├─ vence o prazo sem quórum ──► ADIADO ──► novo sorteio
      │ comparecimento ≥ min_presentes
      ▼
   EM FORMAÇÃO               impedimentos declarados (não consomem recusa,
      │                      não reduzem o número de presentes)
      │                      recusas, limitadas, substituto pela ordem
      │                      ├─ painel não fecha ──► SEM PAINEL ──► novo sorteio
      │ painel completo
      ▼
   INSTRUÇÃO                 defesa protocola por hash; prazo corre
      │                      ├─ prazo vence sem defesa ──► segue com marca "sem defesa"
      ▼
   DELIBERAÇÃO               prazo para decidir
      │                      ├─ prazo vence sem decisão ──► CADUCO (arquiva, sem mérito)
      ▼
   DECIDIDO                  ├─ relator sorteado entre os presentes publica a fundamentação
      │                      └─ janela de recurso
      ▼
   TRANSITADO
```

**Invariantes que o contrato impõe:**

1. Nenhum sorteio usa rodada de beacon anterior ao `definitivo_em` do rol.
2. `presentes` nunca decresce por impedimento (art. 463 §2º).
3. Substituto vem sempre da ordem já sorteada; o contrato não tem função de re-sorteio para substituição.
4. Recusas são limitadas por parte e contadas separadamente de impedimentos.
5. Todo estado não terminal tem prazo, e o vencimento leva a estado terminal ou a novo sorteio — nunca a esperar indefinidamente.
6. **Quem serviu entra na lista de exclusão do ciclo seguinte**, automaticamente, no momento em que o painel fecha.
7. O relator que publica a fundamentação é sorteado **depois** da decisão, entre quem participou — então o nome dele não carrega informação sobre o voto.

---

## 7. Onde isto é melhor que o original, e onde é pior

**Melhor:**

- O sorteio é **verificável por qualquer um**, não só assistido por quem estava na sala. É o art. 433 sem exigir presença física — já construído e provado na testnet.
- A consequência do silêncio é **automática**, em vez de depender de alguém decidir multar.
- O fator de sobre-sorteio **se recalibra** com o comparecimento medido, em vez de ser constante fixada em 1941.
- O relator sorteado **depois** do voto é melhoria sobre a prática brasileira, em que o relator é designado.

**Pior, e precisa estar escrito:**

- Não há juiz presidente para resolver o imprevisto. Tudo que o contrato não previu vira estado terminal.
- A notificação depende de canal que pode falhar, e a pessoa que não foi avisada parece, para o contrato, igual à que ignorou.
- Comunidade pequena: abaixo de certo tamanho o sorteio não produz garantia nenhuma, e a regra dos 12 meses pode esgotar o rol. **O contrato deve recusar operar abaixo do mínimo em vez de fingir que sorteia.**

---

## 8. O que não está resolvido

- **O limite inferior de comunidade.** Qual o menor rol em que esse desenho ainda significa alguma coisa. Depende do tamanho do painel e das recusas, e precisa ser calculado, não chutado.
- **Correlação no comparecimento.** O cálculo da §1 assume independência. Falta um modelo com correlação, ou uma margem empírica medida nos primeiros ciclos.
- **Impedimento verificável.** Hoje é declaração. Como provar relação entre membros sem expor a rede social inteira da comunidade é problema em aberto — e é onde ZK poderia servir, sem que eu tenha verificado se cabe.
- **Serviço alternativo (art. 438).** Traduz bem conceitualmente, mas qual é a função de menor custo numa comunidade on-chain, não sei.
- **Quem opera o rol no primeiro ciclo.** Alguém publica a primeira lista. Essa é a mesma pergunta do "quem calcula a root" que já apareceu antes, e continua sem resposta boa.
