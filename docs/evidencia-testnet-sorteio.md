# Evidência: sorteio verificável por beacon, na testnet

**25 de outubro de 2026.** Tudo abaixo é transação real na testnet Stellar, com hash. Nada é projeção.

## O contrato

| | |
|---|---|
| Contrato | `CBPHNZLGQIUEC4BEC53ZP7P65LLIBT5JTWTHMZVAMCGZCEN3PSUHTPQN` |
| Tamanho | 2.576 bytes otimizado |
| Funções | `verify`, `beacon`, `randomness`, `draw` |
| Fonte | `contracts/beacon-verifier/` |

A chave pública do beacon é **constante pinada no contrato**, não argumento de chamada (T-007). `beacon()` publica o hash da chain em que o contrato confia, para que isso se confira sem ler o Wasm.

## O beacon

drand **quicknet**, esquema `bls-unchained-g1-rfc9380` — assinatura em G1, chave pública em G2, período de 3 segundos. Chain hash `52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971`.

A construção foi descoberta por força bruta contra rodadas reais, não assumida:

- mensagem assinada = `sha256(round como u64 big-endian)`
- DST = `BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_`
- `randomness = sha256(assinatura)`

## As transações

Rodada usada: **33382274**, obtida ao vivo da API pública no momento do teste.
`randomness = 7fe86201ed0c0edcf570b52e55382e4ce981a4ec8dddb9f10fc454e33a2719f1`

| Ato | Resultado | Transação |
|---|---|---|
| `beacon()` — em que beacon o contrato confia | hash da quicknet | [804ba7a1…](https://stellar.expert/explorer/testnet/tx/804ba7a190e4e8b9ee1d593888fe8f35cc69e16580ca69e6e40a6a386a1ccbb5) |
| `verify` com a rodada real, **sem passar chave** | **`true`** | [c74150ee…](https://stellar.expert/explorer/testnet/tx/c74150eeb7b73d0a196d20b44ed851a140812b3ac67e579b670b87ee2c3ffeea) |
| `verify` com assinatura **forjada por outro par de chaves** | **`false`** | [30de94fe…](https://stellar.expert/explorer/testnet/tx/30de94feaf1ac2e6dea42a03edc8fe9be158b10716f03b4be707eb4328559375) |
| `draw` — rol de 200, painel de 25 | painel abaixo | [d077a6a9…](https://stellar.expert/explorer/testnet/tx/d077a6a953a4c5c48a75306754e838a89bf00c92bd00a261c02cc6b03f2f1a1d) |

**Sobre a terceira linha.** Essa assinatura não é um byte trocado: é uma assinatura **válida**, produzida por uma chave secreta real sobre a mensagem correta da rodada 1000000. Ela verifica sob a chave do adversário e é recusada aqui, porque este contrato só aceita a chave da quicknet. É a prova de que pinar a chave resolveu o problema que T-007 descreve.

Painel sorteado on-chain:

```
[23, 81, 114, 169, 188, 195, 142, 52, 44, 13, 0, 60, 7, 124, 42,
 37, 121, 10, 192, 174, 155, 177, 140, 49, 135]
```

## A recomputação independente

O mesmo painel, recomputado fora da cadeia por uma implementação escrita do zero — ChaCha20, amostragem de Lemire e Fisher-Yates à mão, **sem nenhuma dependência**:

```bash
python3 scripts/verificar_sorteio.py \
  7fe86201ed0c0edcf570b52e55382e4ce981a4ec8dddb9f10fc454e33a2719f1 200 25
```

Saída idêntica ao painel on-chain. A mesma recomputação roda no navegador, em `web/`.

A receita foi lida do host (`rs-soroban-env`), não da documentação:

1. `chave = HMAC-SHA256(SALT, semente)` — é o que `unbias_prng_seed` faz, em `soroban-env-host/src/crypto/mod.rs`
2. `ChaCha20Rng::from_seed(chave)` — `rand_chacha` 0.3.1
3. Fisher-Yates de `rand` 0.8.5, índices por amostragem de Lemire com rejeição

`SALT = sha256("Public Global Stellar Network ; September 2015")`, conferido. O comentário no próprio host diz que o sal é parte do protocolo e precisa ser igual em todas as implementações.

## Custo medido

No host de testes do Soroban, via `env.cost_estimate().budget()`. **Não** são números extraídos das transações de testnet.

| Operação | CPU | Memória |
|---|---|---|
| `verify` de uma rodada | **26.719.483** | 161.818 |
| `draw`, rol de 50 | 128.952 | 26.379 |
| `draw`, rol de 200 | 347.784 | 192.579 |
| `draw`, rol de 1.000 | 2.274.884 | 4.118.979 |

O pairing responde pela maior parte do custo de `verify`. A diferença de 887 instruções em relação à medição anterior é o carregamento da chave pinada, que antes vinha como argumento.

Reproduzir: `cargo test -p beacon-verifier rodada_real_verifica_e_mede_custo -- --nocapture`

## Testes

Doze, em `contracts/beacon-verifier/src/test.rs`, incluindo três **vetores de ouro** produzidos pela implementação independente e a recusa da assinatura forjada. Se o host mudar o algoritmo numa versão futura, os testes quebram.

## Limitações declaradas

- O contrato confia em **uma única chain da drand**. Se o grupo da quicknet rotacionar a chave, isso é uma chain nova e exige um contrato novo — consequência aceita de pinar.
- A descompressão dos pontos acontece fora da cadeia. É seguro: ponto errado faz o pairing falhar.
- Os custos foram medidos no host de testes, não extraídos das transações de testnet.
- O verificador independente está em Python e em JavaScript; nenhum dos dois é exercitado por um gate ainda (issues #4 e #5).
- Sem auditoria externa. Nada em mainnet.
