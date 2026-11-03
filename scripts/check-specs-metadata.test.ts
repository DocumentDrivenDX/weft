import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';

// Exercise the checker's actual metadata assertion without running unrelated
// schema/allocation checks or changing any repository fixture/frontmatter.
const source = readFileSync(new URL('./check-specs.ts', import.meta.url), 'utf8');
const assertion = source.match(/^  assert\((.+), `\$\{file\}: incomplete metadata`\);$/m);
if (!assertion) throw new Error('Cannot locate the actual metadata assertion');
const admitted = new Function('meta', `return Boolean(${assertion[1]});`);
const valid = () => ({ id: 'release-test', type: 'release-notes', activity: 'deploy', authoring: { home: 'repo' } });

describe('actual HELIX metadata admission', () => {
  test('omitted optional status passes', () => {
    expect(admitted(valid())).toBe(true);
  });

  test('arbitrary legacy status passes unchanged', () => {
    for (const status of ['released', 'accepted', 'historical-label', '', null, 42, false, { legacy: true }]) {
      const meta = { ...valid(), status };
      expect(admitted(meta)).toBe(true);
      expect(meta.status).toBe(status);
    }
  });

  test('missing required metadata refuses even with draft status', () => {
    for (const key of ['id', 'type', 'activity', 'authoring']) {
      const meta: Record<string, unknown> = { ...valid(), status: 'draft' };
      delete meta[key];
      expect(admitted(meta)).toBe(false);
    }
    expect(admitted(undefined)).toBe(false);
    expect(admitted(null)).toBe(false);
    expect(admitted({ ...valid(), authoring: {} })).toBe(false);
  });

  test('invalid required values refuse regardless of legacy status', () => {
    for (const key of ['id', 'type', 'activity']) {
      expect(admitted({ ...valid(), status: 'draft', [key]: '' })).toBe(false);
    }
    expect(admitted({ ...valid(), status: 'draft', authoring: { home: 'external-tool' } })).toBe(false);
  });
});
