// SPDX-License-Identifier: MIT OR Apache-2.0
// Refresh the generated parts of docs/helix/02-design/module-boundaries.json:
// each package's dependency edges and Rust source/public-declaration inventory,
// and each host area's source list. Narrative fields (responsibility, ownedTypes,
// constructionAndIntegration, coverage) stay human-owned: a workspace package
// without an entry refuses until one is written. Review the resulting diff; the
// checker (scripts/module-boundaries.ts) then enforces it.
//
// Usage: WEFT_CARGO="$(command -v cargo)" bun scripts/module-boundaries-update.ts
import {readFile, writeFile, readdir} from 'node:fs/promises';
import {resolve} from 'node:path';
import {edgeIdentity, scanSurface} from './module-boundaries';

async function walk(path: string): Promise<string[]> {
  const entries = await readdir(path, {withFileTypes: true});
  return (await Promise.all(entries.map(e => e.isDirectory() ? walk(resolve(path, e.name)) : [resolve(path, e.name)]))).flat();
}

const root = resolve(import.meta.dir, '..');
const mapPath = resolve(root, 'docs/helix/02-design/module-boundaries.json');
const map = JSON.parse(await readFile(mapPath, 'utf8'));
const cargo = Bun.spawn([process.env.WEFT_CARGO || 'cargo', 'metadata', '--no-deps', '--format-version', '1', '--locked', '--offline'], {cwd: root, stdout: 'pipe', stderr: 'inherit'});
const [raw, code] = await Promise.all([new Response(cargo.stdout).text(), cargo.exited]);
if (code !== 0) throw Error('cargo metadata failed');
const metadata = JSON.parse(raw);
const actual = metadata.packages.filter((p: any) => metadata.workspace_members.includes(p.id));
const missing = actual.filter((p: any) => !map.packages.some((m: any) => m.name === p.name)).map((p: any) => p.name);
if (missing.length) throw Error(`Write a narrative entry first for: ${missing.join(', ')}`);
map.packages = map.packages.filter((m: any) => actual.some((p: any) => p.name === m.name));
for (const entry of map.packages) {
  const pkg = actual.find((p: any) => p.name === entry.name);
  entry.manifest = pkg.manifest_path.slice(root.length + 1);
  entry.dependencies = pkg.dependencies.map((d: any) => edgeIdentity(d, root))
    .sort((a: any, b: any) => a.name.localeCompare(b.name) || a.kind.localeCompare(b.kind));
  entry.sources = await scanSurface(root, entry.manifest);
}
for (const area of map.hostAreas) {
  area.sources = (await Promise.all(area.roots.map((path: string) => walk(resolve(root, path))))).flat()
    .filter((f: string) => area.extensions.some((e: string) => f.endsWith(e)))
    .map((f: string) => f.slice(root.length + 1)).concat(area.configs ?? []).sort();
}
await writeFile(mapPath, JSON.stringify(map, null, 2) + '\n');
console.log(`Updated ${map.packages.length} package inventories and ${map.hostAreas.length} host areas.`);
