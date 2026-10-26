import initTessera, * as tessera from './tessera/tessera_cliente.js';
import { rodarCaso, PLANO } from './motor.mjs';
import * as provar from '../../circuits/navegador.mjs';
import { criarPoseidon } from '../poseidon.mjs';
import { raiz } from '../arvore.mjs';
import { verificarProcesso } from '../processo.mjs';
import { montarDemo } from './ui.mjs';

let P;
async function buscar(caminho, json = false) {
  const r = await fetch(caminho);
  if (!r.ok) throw new Error(`Falha ao carregar ${caminho} (HTTP ${r.status}).`);
  return json ? r.json() : r.text();
}
montarDemo({
  plano: PLANO,
  async carregar() {
    if (!window.StellarSdk || !window.snarkjs) throw new Error('As bibliotecas da demonstração não carregaram.');
    const [textos, parametros] = await Promise.all([
      Promise.all(['01-denuncia.md', '02-prova.md', '03-defesa.md', '04-ata-jurado.md'].map((nome) => buscar(`../../docs/caso-demo/${nome}`))),
      buscar('../../tests/poseidon-bn254.json', true),
      initTessera(),
    ]);
    P = criarPoseidon(parametros);
    return textos;
  },
  executar(opcoes, aviso) {
    return rodarCaso({sdk: window.StellarSdk, snarkjs: window.snarkjs, tessera, P, provar,
      circuito: {wasm: '../../circuits/setup/pinakion_claim.wasm', zkey: '../../circuits/setup/claim_final.zkey'}}, opcoes, aviso);
  },
  conferir(registro, docs) {
    const mapa = Object.fromEntries(Object.entries(docs).map(([hash, d]) => [hash, d.nome]));
    return verificarProcesso(registro, {P, raiz, docs: mapa});
  },
});
