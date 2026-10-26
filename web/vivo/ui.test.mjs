import test from 'node:test';
import assert from 'node:assert/strict';
import { resumirConferencia } from './ui.mjs';

test('verification failure cannot be presented as a successful verdict', () => {
  const r = resumirConferencia({ conferencias: [['rol', true], ['assinatura', false]] }, { eventos: [{ evento: 'Decidido', procedente: true, sim: 5, nao: 2, exigidos: 4 }] });
  assert.equal(r.confere, false);
  assert.match(r.titulo, /divergências/i);
  assert.match(r.detalhe, /assinatura/);
});
test('an adverse verdict still has a successful verification', () => {
  const r = resumirConferencia({ conferencias: [['rol', true]] }, { eventos: [{ evento: 'Decidido', procedente: false, sim: 2, nao: 5, exigidos: 4 }] });
  assert.equal(r.confere, true);
  assert.equal(r.titulo, 'Improcedente');
});
test('missing decision or empty checks cannot report success', () => {
  assert.throws(() => resumirConferencia({ conferencias: [['rol', true]] }, { eventos: [] }));
  assert.throws(() => resumirConferencia({ conferencias: [] }, { eventos: [{ evento: 'Decidido', procedente: true }] }));
});

test('the displayed demo rules agree with the recorded testnet case', async () => {
  const { regrasDoPlano } = await import('./ui.mjs');
  const { PLANO } = await import('./motor.mjs');
  const { readFile } = await import('node:fs/promises');
  const registro = JSON.parse(await readFile(new URL('../../docs/evidencia-processo-registro.json', import.meta.url)));
  const aberto = registro.eventos.find((e) => e.evento === 'CasoAberto');
  assert.deepEqual(regrasDoPlano(PLANO), { quorum: aberto.quorum, exigidos: aberto.exigidos });
});
