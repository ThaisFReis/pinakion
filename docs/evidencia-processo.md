# Evidência: um caso inteiro na testnet

**Caso fictício, transações reais.** Gerado por `python3 scripts/evidencia_processo.py docs/evidencia-processo-registro.json`; o G6 confere que este arquivo é o que o comando imprime. Sequência: `scripts/sequencia_processo.sh`.

> A votação roda no **Tessera**, o módulo de voto secreto anterior da mesma autora: o contrato no commit `356c22f`, sem mudança, e a CLI no mesmo commit com uma correção de uma linha (`scripts/tessera-cli.patch`). O que é deste projeto é a ligação: o contrato `processo` só aceitou a votação porque a lista de votantes do Tessera é exatamente a dos jurados presentes.

## O que a conferência diz

`python3 scripts/verificar_processo.py docs/evidencia-processo-registro.json`: **CONFERE, 217 conferências, 0 falha(s)**, sem confiar em nenhum dos contratos. A página [`web/processo.html`](../web/processo.html) faz as mesmas conferências no navegador.

## Os contratos

| Papel | Endereço |
|---|---|
| o rol e o sorteio selado | [`CCVLAM…R6F2`](https://stellar.expert/explorer/testnet/contract/CCVLAMSF7SXEVU4EJIHN7P24HHBGKJPKLJ26DXR7DAWPLEG66IYYR6F2) |
| o caso | [`CCX3X4…EO5Q`](https://stellar.expert/explorer/testnet/contract/CCX3X4IH5PNELSDAF2WOBR27IJ4PYBQFH5CQPF25YPZU5PBHEYBSEO5Q) |
| a votação (Tessera, 356c22f) | [`CDWBWA…XQT7`](https://stellar.expert/explorer/testnet/contract/CDWBWAHUZ3MC3BJXJIW5O74MNUXH76VCGBIVFQU3XJJFGML3VKFIXQT7) |
| o verificador Groth16 | [`CBEX5A…NI5F`](https://stellar.expert/explorer/testnet/contract/CBEX5AEAUDKK4SUZGXQPM26QZHNZZ5X3ELCSIQJ3V6SZKUCQ4B57NI5F) |
| o SAC do bilhete PINAKION | [`CDRT2X…J7ER`](https://stellar.expert/explorer/testnet/contract/CDRT2XH6JFSZLPOSFOU2SCBRNLNTAAJAMXUMCK5HSBOBGYN34VHKJ7ER) |
| o emissor, com peso 0 | [`GBNSGL…BZYH`](https://stellar.expert/explorer/testnet/account/GBNSGLFMQ6KK2L7DEJ4A42A5RSTNH7QG2Y5TYZ3ODZKVPV2V55MJBZYH) |

## O caso, em números

| | |
|---|---|
| Inscritos no rol | 40 |
| Nível de gravidade · assentos | 3 · 7 |
| A pessoa julgada estava no rol e foi impedida no fechamento | sim |
| Rodada drand | 33394730 |
| Reivindicações (provas em conhecimento zero) | 13 |
| Painel | 7, completo |
| Quórum · presentes | 5 · 6 |
| Documentos nos autos | 6 |
| Placar do Tessera (sim · não) · exigidos | 5 · 1 · 4 |
| Veredito | procedente |
| Revelaram · em silêncio | 38 · 1 |

## Os autos

| # | Tipo | Por | Documento | Transação |
|---|---|---|---|---|
| 0 | denúncia | escrivão | [`01-denuncia.md`](caso-demo/01-denuncia.md) | [`c32d963a2e83…`](https://stellar.expert/explorer/testnet/tx/c32d963a2e836a7e6558a38932673e58b7a8ca2a4fb0c1f6257a19353c13681f) |
| 1 | prova | escrivão | [`02-prova.md`](caso-demo/02-prova.md) | [`2368cecfccd9…`](https://stellar.expert/explorer/testnet/tx/2368cecfccd9937a821ad38c63f7a6c8cf5e20c59c845d361052453194017a8e) |
| 2 | defesa | pessoa julgada | [`03-defesa.md`](caso-demo/03-defesa.md) | [`9304f35ea546…`](https://stellar.expert/explorer/testnet/tx/9304f35ea5463d8ac8b99a85ab5c4f8dc785fbb62b3985e19a91021c9afa7ff2) |
| 3 | ata | jurado (conta de assento) | [`04-ata-jurado.md`](caso-demo/04-ata-jurado.md) | [`0ee70e51c85d…`](https://stellar.expert/explorer/testnet/tx/0ee70e51c85dea1bd9f6125d464a8a0abd8db1b6aaa4653d4976c92a8feb01f2) |
| 4 | ata | escrivão | [`05-ata-sessao.md`](caso-demo/05-ata-sessao.md) | [`454e14d56cf4…`](https://stellar.expert/explorer/testnet/tx/454e14d56cf47d32930f58c099a17b4786ef5c47f7d76a5883b2c4dd7bdfa8ea) |
| 5 | ata | escrivão | [`06-ata-decisao.md`](caso-demo/06-ata-decisao.md) | [`77df0a460f04…`](https://stellar.expert/explorer/testnet/tx/77df0a460f04c3cd605ff349f09796b523feca025226a810e4fe49f062490251) |

## Cada passo, com a transação

| Passo | Transação |
|---|---|
| fechamento do rol (e o impedimento da parte) | [`50052e2a3a6b…`](https://stellar.expert/explorer/testnet/tx/50052e2a3a6bf64470d9a0e574a69176989a3b0700115411885264973faa9c42) |
| abertura do caso | [`c32d963a2e83…`](https://stellar.expert/explorer/testnet/tx/c32d963a2e836a7e6558a38932673e58b7a8ca2a4fb0c1f6257a19353c13681f) |
| selo (assinatura drand verificada na cadeia) | [`cb031537116e…`](https://stellar.expert/explorer/testnet/tx/cb031537116e0e79433cf4449c80018935e396d6e088827c286b6f8c8c6f8df5) |
| painel | [`d645b60401c9…`](https://stellar.expert/explorer/testnet/tx/d645b60401c9bc6a046028ebd080e5a7ada99dbf2150cdedc806e310b7bf9e40) |
| sessão instalada | [`b93ca660c114…`](https://stellar.expert/explorer/testnet/tx/b93ca660c114d329bc9bf150e528cecd2db7f35452e2759204f25a6b125d611b) |
| abertura da votação no Tessera | [`1ff99cac8d90…`](https://stellar.expert/explorer/testnet/tx/1ff99cac8d90c0164e4e778fc1ca6bab7d06dbd799affccfb5652b318842ee05) |
| votação vinculada ao caso | [`133c93369a0a…`](https://stellar.expert/explorer/testnet/tx/133c93369a0acfb4aa67f58257857d20bf9405957eb196064c1d3a7b233c366a) |
| apuração no Tessera | [`43444e8d9d84…`](https://stellar.expert/explorer/testnet/tx/43444e8d9d84e5dd15a059bfd94bfb009512da085cfae4e58932e4fcc1c93f4b) |
| veredito | [`14fda9a0313f…`](https://stellar.expert/explorer/testnet/tx/14fda9a0313f7f73ad727d5920a65cf7358bc6971d0550a52bd210554000f397) |

## O que esta rodada não mostra

- **Uma máquina fez o papel de todos.** Todas as contas vieram do friendbot; o financiamento da conta de assento sem rastro precisa de um relayer, que não existe.
- **Os prazos são de demonstração.** A espera mínima da revelação foi de 300 ledgers, cerca de 25 minutos a 5 s por ledger (estimativa); o padrão aprovado é 30 dias.
- **Os autos são públicos aqui.** Um caso real precisa de autos sigilosos com acesso justificado (desenhados, não construídos).
- **A correção do placar é do Tessera.** Este projeto confere a ligação e a regra; a apuração é conferida por `tessera verificar`, cuja saída está em `docs/evidencia-processo-tessera.txt`.
- **É só testnet, e nada foi auditado.**
