import { test, expect } from 'bun:test';
import { readFile, writeFile, unlink } from 'node:fs/promises';
import { resolve } from 'node:path';

const root = resolve(import.meta.dir, '../..');
async function check() {
  const process = Bun.spawn([processExecutable(), 'scripts/check-specs.ts'], { cwd: root, stdout: 'pipe', stderr: 'pipe' });
  const [status, stdout, stderr] = await Promise.all([process.exited, new Response(process.stdout).text(), new Response(process.stderr).text()]);
  return {status, stdout, stderr};
}
function processExecutable() { return process.execPath; }

test('actual checker accepts archived copies and still refuses duplicate governing IDs', async () => {
  const baseline = await check();
  expect(baseline.status).toBe(0);
  expect(baseline.stdout).toContain('governed artifacts');
  const schema = await readFile(resolve(root, 'docs/helix/02-design/contracts/compile-response-v0.3.schema.json'));
  const schemaCopy = resolve(root, 'docs/helix/04-build/specification-scope-control-' + crypto.randomUUID() + '.schema.json');
  try {
    await writeFile(schemaCopy, schema, {flag: 'wx'});
    const refusal = await check();
    expect(refusal.status).not.toBe(0);
    expect(refusal.stderr).toContain('already exists');
  } finally { await unlink(schemaCopy); }
  const original = await readFile(resolve(root, 'docs/helix/02-design/architecture.md'));
  const duplicate = resolve(root, 'docs/helix/02-design/specification-scope-control-' + crypto.randomUUID() + '.md');
  try {
    await writeFile(duplicate, original, {flag: 'wx'});
    const refusal = await check();
    expect(refusal.status).not.toBe(0);
    expect(refusal.stderr).toContain('Duplicate artifact weft.architecture');
  } finally { await unlink(duplicate); }
}, 30000);
