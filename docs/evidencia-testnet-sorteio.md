# Evidência: sorteio verificável por beacon, na testnet

**25 de outubro de 2026.** Tudo abaixo é transação real na testnet Stellar, com hash. Nada é projeção.

## O contrato

| | |
|---|---|
| Contrato | `CCLWP7WOESC46JLHKGASYQ5R7XKBHEBIJCKWT74R7GPO7P5ZW3N5PCGN` |
| Hash do Wasm | `fc7f70473fb1e44c937b72d140bae0d7db52328970044c063a17efeea39cabaa` |
| Tamanho | 2.182 bytes otimizado |
| Funções | `verify`, `draw`, `randomness` |
| Fonte | `contracts/beacon-verifier/` |

## O beacon

drand **quicknet**, esquema `bls-unchained-g1-rfc9380` — assinatura em G1, chave pública em G2, período de 3 segundos. Chain hash `52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971`.

A construção foi descoberta por força bruta contra rodadas reais, não assumida:

- mensagem assinada = `sha256(round como u64 big-endian)`
- DST = `BLS_SIG_BLS12381G1_XMD:SHA-256_SSWU_RO_NUL_`
- `randomness = sha256(assinatura)`

## As transações

Rodada usada: **33380955**, obtida ao vivo da API pública no momento do teste.
`randomness = 104870a5f58e570f7bb7d3f63331a752e42245543c17d1628c75bf775ff268d1`

| Ato | Resultado | Transação |
|---|---|---|
| `verify` com a rodada real | **`true`** | [9de19747…](https://stellar.expert/explorer/testnet/tx/9de197478cd68e93a1f2d7a16c9b3f997580584239c0e5e83fddc0c322a2238d) |
| `verify` com a mesma assinatura na rodada seguinte | **`false`** | [2ebda22b…](https://stellar.expert/explorer/testnet/tx/2ebda22bc89b0f8f72133381e9b30d94047d6fecc028c42a68a59b08053ec5ea) |
| `draw` — rol de 200, painel de 25 | painel abaixo | [dda87cc0…](https://stellar.expert/explorer/testnet/tx/dda87cc05c16beed6e2720f579990d15c79e46dd9dc9f920b24917e8e1d87d65) |

Painel sorteado on-chain:

```
[146, 187, 57, 172, 113, 46, 106, 152, 12, 124, 56, 193, 101, 99, 88,
 169, 63, 167, 108, 135, 18, 197, 148, 97, 73]
```

## A recomputação independente

O mesmo painel, recomputado fora da cadeia por uma implementação escrita do zero — ChaCha20, amostragem de Lemire e Fisher-Yates à mão, **sem nenhuma dependência**:

```bash
python3 scripts/verificar_sorteio.py \
  104870a5f58e570f7bb7d3f63331a752e42245543c17d1628c75bf775ff268d1 200 25
```

Saída idêntica ao painel on-chain.

A receita foi lida do host (`rs-soroban-env`), não da documentação:

1. `chave = HMAC-SHA256(SALT, semente)` — é o que `unbias_prng_seed` faz, em `soroban-env-host/src/crypto/mod.rs`
2. `ChaCha20Rng::from_seed(chave)` — `rand_chacha` 0.3.1
3. Fisher-Yates de `rand` 0.8.5, índices por amostragem de Lemire com rejeição

`SALT = sha256("Public Global Stellar Network ; September 2015")`, conferido. O comentário no próprio host diz que o sal é parte do protocolo e precisa ser igual em todas as implementações.

## Custo medido

No host de testes do Soroban, via `env.cost_estimate().budget()`:

| Operação | CPU | Memória |
|---|---|---|
| `verify` de uma rodada | **26.718.596** | 161.546 |
| `draw`, rol de 50 | 128.952 | 26.379 |
| `draw`, rol de 200 | 347.784 | 192.579 |
| `draw`, rol de 1.000 | 2.274.884 | 4.118.979 |

O pairing responde por 20,4M dos 26,7M. `hash_to_g1` custa 2,6M; as checagens de subgrupo somam 3,6M.

## Testes

Dez, em `contracts/beacon-verifier/src/test.rs`, incluindo três **vetores de ouro** produzidos pela implementação independente. Se o host mudar o algoritmo numa versão futura, os testes quebram.

## Limitações declaradas

- **A chave pública do beacon entra como parâmetro neste spike.** Na versão de produção ela precisa ser constante pinada no contrato; caso contrário qualquer um passa a própria chave e forja qualquer rodada.
- A descompressão dos pontos acontece fora da cadeia. É seguro: ponto errado faz o pairing falhar.
- Os custos foram medidos no host de testes, não extraídos das transações de testnet.
- O verificador independente está em Python. Para auditoria pública de verdade falta a versão em JavaScript, que roda no navegador.
- Sem auditoria externa.
