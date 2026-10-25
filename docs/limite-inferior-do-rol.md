# O limite inferior do rol

**Cálculo · 25 de outubro de 2026**

> Todos os números aqui foram calculados, não estimados. O modelo e os limites dele estão na §6.
> A pergunta: a partir de que tamanho de rol o sorteio ainda dá garantia?

---

## 1. O resultado que eu não esperava

Simulei o procedimento inteiro — sorteio, comparecimento, recusas alternadas com substituto pela ordem — e medi **quantas pessoas um adversário precisa controlar para ter 50% de chance de maioria no painel**.

| Rol | Sorteados | Capturados necessários | Fração |
|---|---|---|---|
| 15 | 15 | 8 | 53% |
| 20 | 20 | 10 | 50% |
| 30 | 25 | 15 | 50% |
| 60 | 25 | 31 | 52% |
| 100 | 25 | 50 | 50% |
| 200 | 25 | 101 | 50% |
| 400 | 25 | 196 | 49% |

**A fração é constante em ~50%, de 15 a 400.** Rol pequeno não enfraquece a resistência à captura em termos proporcionais.

O motivo, visto depois: sorteio é **amostragem representativa**. Uma facção precisa de cerca de metade do rol para obter com confiança metade do painel. Isso vale para rol de 15 e para rol de 400.

Eu esperava que a garantia estatística degradasse com o tamanho. Não degrada.

---

## 2. O que degrada é o preço, não a proporção

A fração é a mesma. O **número absoluto** não é.

| Rol | Precisa capturar | O que isso é, na prática |
|---|---|---|
| 15 | 7 | um grupo de amigos |
| 30 | 15 | uma mesa de bar grande |
| 50 | 25 | uma turma |
| 100 | 50 | um bairro pequeno |
| 200 | 100 | difícil de coordenar em segredo |
| 400 | 200 | praticamente um partido |

**A proteção do sorteio é econômica, não estatística.** Ela não vem de o adversário não conseguir prever o painel — vem de ele ter que comprar metade da comunidade **antes de saber qual será o caso**, e manter isso em segredo.

Então o parâmetro de segurança não é a fração. É `f* ≈ R/2`, em pessoas.

---

## 3. A restrição que realmente manda: a rotação

O art. 426 §4º exclui do rol quem serviu nos 12 meses anteriores. É a regra anticaptura — impede o jurado profissional. E é ela que esgota um rol pequeno.

O rol precisa comportar, ao mesmo tempo, o sorteio e as exclusões do ciclo:

```
R  ≥  N_necessário(K)  +  K × casos_por_ciclo
```

| Painel K | Sortear N | 1 caso | 2 casos | 4 casos | 8 casos | 12 casos |
|---|---|---|---|---|---|---|
| 3 | 13 | 16 | 19 | 25 | 37 | 49 |
| 5 | 19 | 24 | 29 | 39 | 59 | 79 |
| **7** | **25** | **32** | **39** | **53** | **81** | **109** |
| 9 | 30 | 39 | 48 | 66 | 102 | 138 |

*(N calculado para 99% de chance de atingir o quórum com 80% de comparecimento)*

**Com painel de 7, um rol de 30 não sustenta nem um caso por ciclo.** Precisa de 32 para um, 53 para quatro.

E aqui está a tensão que vale nomear: **a regra que protege contra captura é a mesma que esgota o rol pequeno.** As duas garantias brigam entre si embaixo de certa escala.

---

## 4. A saída óbvia, e o que ela custa

Comunidade pequena? Diminui o painel. Com K=3, um rol de 25 sustenta quatro casos por ciclo.

O preço é volatilidade. Probabilidade de uma facção minoritária obter maioria do painel, em rol de 100:

| Facção | K=3 | K=5 | K=7 | K=9 | K=12 |
|---|---|---|---|---|---|
| 20% | 10% | 5% | 3% | 1% | 0% |
| 30% | 21% | 16% | 12% | 9% | 3% |
| 40% | 35% | 31% | 28% | 26% | 14% |
| 45% | 42% | 40% | 39% | 37% | 25% |

Uma facção de 30% leva a maioria em **1 de cada 5 painéis de 3**, contra **1 de cada 8 painéis de 7**.

Painel pequeno não é só menos deliberação — é menos representativo, de forma mensurável. É uma troca explícita, não um detalhe.

---

## 5. A carga, que é o terceiro piso

Chance de uma pessoa qualquer ser convocada ao menos uma vez no ciclo:

| Rol | 1 caso | 4 casos | 12 casos |
|---|---|---|---|
| 20 | 100% | 100% | 100% |
| 30 | 83% | 100% | 100% |
| 60 | 42% | 88% | 100% |
| 100 | 25% | 68% | 97% |
| 400 | 6% | 23% | 54% |

**Abaixo de ~30, todo mundo é convocado todo ciclo.** Aí o sorteio deixa de significar alguma coisa: o rol *é* a comunidade, e "você não escolheu quem julga" vira "todo mundo julga todo mundo".

Não é falha matemática — é perda de sentido. Sorteio pressupõe que há mais gente elegível do que assentos.

---

## 6. O limite, em números

Juntando as quatro restrições:

| Faixa do rol | Veredito |
|---|---|
| **R < 16** | **O contrato deve recusar.** Não dá nem para sortear um painel de 3 com quórum. |
| **16 – 40** | Só K=3, carga baixa. Capturar custa de 8 a 20 pessoas — barato demais para qualquer coisa consequente. Serve para decisão de baixo risco, e o sistema deve **dizer isso**. |
| **40 – 100** | K=5 ou 7 com carga moderada. Capturar custa de 20 a 50. É a primeira faixa em que a palavra garantia se sustenta. |
| **R ≥ 100** | O desenho completo funciona. Capturar custa 50+ e exige coordenação em segredo, antes de saber o caso. |

**A recomendação:** o contrato recusa operar abaixo de 16, e **publica `f*` junto com cada painel** — o número de pessoas que teriam que estar combinadas para aquele painel ter sido capturado. Quem lê a decisão vê, ao lado dela, quanto custaria tê-la comprado.

Isso é melhor do que um piso escondido. Transforma a limitação em informação.

---

## 7. O modelo, e onde ele é frágil

- **Comparecimento independente.** Na prática há correlação — feriado, fuso, um caso que ninguém quer tocar. A margem de 2 no quórum existe por isso, e é pequena.
- **O adversário vota em bloco e sabe quem é dele.** É o pior caso, o que é adequado para desenho de segurança, mas superestima a ameaça real.
- **As recusas são gulosas.** Cada parte tira o pior do painel prospectivo, alternando. É aproximação razoável do que advogados fazem, não teoria de jogos resolvida.
- **Monte Carlo com 1.000 amostras e busca binária.** O `f*` tem ruído de mais ou menos um.
- **Não modelei coalizão parcial** — gente que simpatiza sem estar combinada. O mundo real está entre o sorteio puro e o bloco disciplinado.
- **Não modelei o adversário que entra no rol.** Se ele consegue inscrever gente no alistamento, o custo de captura cai. É aí que a janela de reclamação pública do art. 426 trabalha, e eu não quantifiquei quanto ela ajuda.
