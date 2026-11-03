import { expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
const source = readFileSync(process.env.WEFT_LINK_SOURCE ?? new URL('./check-specs.ts', import.meta.url), 'utf8');
const assertion = source.match(/^    assert\((.+), `\$\{id\}: broken link \$\{edge.id\}`\);$/m);
if (!assertion) throw new Error('Cannot locate actual link assertion');
const admitted = new Function('edge', 'artifacts', `return Boolean(${assertion[1]});`);
const artifacts = new Map([['release-test', {}]]);
test('existing informed_by with an actual target passes', () => {
  expect(admitted({ kind: 'informed_by', id: 'release-test' }, artifacts)).toBe(true);
});
test('governing informs with an actual target passes', () => {
  expect(admitted({ kind: 'informs', id: 'release-test' }, artifacts)).toBe(true);
});
test('unknown relation refuses even with an actual target', () => {
  for (const kind of ['', 'invented', null, undefined]) expect(admitted({ kind, id: 'release-test' }, artifacts)).toBe(false);
});
test('missing targets refuse for both governed directions', () => {
  for (const kind of ['informs', 'informed_by']) expect(admitted({ kind, id: 'absent' }, artifacts)).toBe(false);
});
