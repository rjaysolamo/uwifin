import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseAmount, decimalAmount, validateAddress } from '../src/lib/money.ts';
test('USDC precision and supported boundary are exact', () => {
  for (const value of ['0.000001', '1.500001', '9223372036854.775807'])
    assert.equal(decimalAmount(parseAmount(value)), value);
  for (const value of [
    '0',
    '-1',
    '1e6',
    '0.0000001',
    '9223372036854.775808',
    '1.',
    '.1',
    'Infinity',
  ])
    assert.throws(() => parseAmount(value));
});
test('recipient syntax rejects empty and zero addresses', () => {
  assert.equal(validateAddress('0x' + '1'.repeat(40)), true);
  for (const value of ['', '0x' + '0'.repeat(40), '0x123', '0x' + 'g'.repeat(40)])
    assert.equal(validateAddress(value), false);
});
