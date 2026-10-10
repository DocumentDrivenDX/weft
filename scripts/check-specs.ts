import { readdir, readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { createHash } from 'node:crypto';
import YAML from 'yaml';
import Ajv2020 from 'ajv/dist/2020.js';

const root = resolve(import.meta.dir, '..');
const read = (p: string) => readFile(resolve(root, p), 'utf8');
const json = async (p: string) => JSON.parse(await read(p));
const assert = (ok: unknown, message: string): asserts ok => { if (!ok) throw new Error(message); };
async function walk(p: string): Promise<string[]> {
  // Byte-preserved historical copies are evidence, not governing instances or schemas.
  if (p === 'docs/helix/04-build/evidence') return [];
  const entries = await readdir(resolve(root, p), { withFileTypes: true });
  return (await Promise.all(entries.map(e => e.isDirectory() ? walk(`${p}/${e.name}`) : [`${p}/${e.name}`]))).flat();
}
const marker = YAML.parse(await read('.helix.yml'));
assert(marker.flows?.some((f: any) => f.id === 'helix' && f.root === 'docs/helix/'), 'Missing active HELIX scope');
const files = await walk('docs/helix');
const artifacts = new Map<string, { file: string; meta: any; body: string }>();
for (const file of files.filter(f => f.endsWith('.md'))) {
  const body = await read(file);
  if (!body.startsWith('---\n')) continue;
  const end = body.indexOf('\n---\n', 4);
  assert(end > 0, `${file}: unclosed frontmatter`);
  const meta = YAML.parse(body.slice(4, end)).ddx;
  assert(meta?.id && meta.type && meta.activity && (meta.status === 'draft' || (['ADR-001','ADR-002'].includes(meta.id) && meta.status === 'accepted')) && meta.authoring?.home === 'repo', `${file}: incomplete metadata`);
  assert(!artifacts.has(meta.id), `Duplicate artifact ${meta.id}`);
  artifacts.set(meta.id, {file, meta, body});
}
for (const [id, a] of artifacts) {
  for (const edge of a.meta.links ?? []) {
    assert(edge.kind === 'informed_by' && artifacts.has(edge.id), `${id}: broken link ${edge.id}`);
  }
  // Resolve local Markdown links without trying network citations or anchors.
  for (const match of a.body.matchAll(/\]\(([^)]+)\)/g)) {
    const target = match[1].split('#')[0];
    if (!target || /^(https?:|app:|codex:|\/)/.test(target)) continue;
    const absolute = resolve(root, dirname(a.file), target);
    assert(absolute.startsWith(`${root}/`), `${a.file}: link escapes repository`);
    await readFile(absolute);
  }
}
const allocation = await json('docs/helix/03-test/story-test-allocation.json');
const criteria = new Set<string>();
for (const story of allocation) {
  const us = [...artifacts.values()].find(a => a.meta.id === story.story);
  const stp = [...artifacts.values()].find(a => a.meta.id === `STP-${story.story.slice(3)}`);
  assert(us && stp && story.criteria.length > 0, `${story.story}: missing story/test allocation`);
  for (const criterion of story.criteria) {
    assert(!criteria.has(criterion.id), `Duplicate criterion ${criterion.id}`);
    assert(us.body.includes(criterion.id) && stp.body.includes(criterion.id) && stp.body.includes(criterion.plannedTest), `Missing trace ${criterion.id}`);
    assert(criterion.state === 'planned', 'Bootstrap must not claim executed tests');
    criteria.add(criterion.id);
  }
}
assert(criteria.size === 30, 'Expected 30 criteria after the application-read input');
const schemaDir = 'docs/helix/02-design/contracts';
const ajv = new Ajv2020({ allErrors: true, strict: false });
// Security contract literals reference the separately pinned UMF 0.8 envelope.
const core08Text = await read('spec/upstream/umf-0.8.0.schema.json');
const core08Pin = await json('spec/upstream/umf-0.8.0.source.json');
assert(createHash('sha256').update(core08Text, 'utf8').digest('hex') === core08Pin.sha256,
  'Pinned UMF 0.8 schema bytes differ');
ajv.addSchema(JSON.parse(core08Text));
const schemas = [];
for (const file of files.filter(f => f.endsWith('.schema.json'))) {
  const schema = await json(file);
  ajv.addSchema(schema);
  schemas.push(schema);
}
for (const schema of schemas) assert(ajv.getSchema(schema.$id), `Cannot compile ${schema.$id}`);
const id = (name: string) => schemas.find(s => s.$id.endsWith(`/${name}.schema.json`)).$id;
const validateCase = ajv.getSchema(id('conformance-case'))!;
const validateRequest = ajv.getSchema(id('compile-request'))!;
const cases = await json('docs/helix/03-test/fixtures/cases.json');
const ids = new Set<string>();
let structuralNegatives = 0;
const sha256 = (s: string) => createHash('sha256').update(s, 'utf8').digest('hex');
for (const c of cases) {
  assert(validateCase(c), `${c.id}: case schema ${JSON.stringify(validateCase.errors)}`);
  assert(!ids.has(c.id), `Duplicate case ${c.id}`); ids.add(c.id);
  for (const criterion of c.covers) assert(criteria.has(criterion), `${c.id}: unknown ${criterion}`);
  const valid = validateRequest(c.request);
  if (c.setup.schemaNegative) { assert(!valid, `${c.id}: deliberate invalid request accepted`); structuralNegatives++; }
  else assert(valid, `${c.id}: request schema ${JSON.stringify(validateRequest.errors)}`);
  for (const module of c.request.modules) {
    const document = JSON.parse(module.documentJson);
    const pinValid = module.pin.sha256 === sha256(module.documentJson) && module.pin.documentId === document.id;
    assert(pinValid || c.expected.code === 'WFT-PIN', `${c.id}: unexpected pin mismatch`);
    assert(module.selectedModuleIds.every((id: string) => document.modules.some((m: any) => m.id === id)), `${c.id}: invalid selected module`);
  }
  assert(c.request.target.bindingSha256 === sha256(c.request.target.bindingJson), `${c.id}: binding digest`);
  const binding = JSON.parse(c.request.target.bindingJson);
  assert(binding.fixtureOnly === true && c.request.target.targetProfile === 'fixture-only', `${c.id}: fixture claims production binding`);
  if (c.setup.rows) await read(`docs/helix/03-test/fixtures/${c.setup.rows}`);
}
assert(cases.length >= 636 && structuralNegatives > 0, 'Corpus floor / deliberate negatives missing');
// Nested declarations must admit only explicitly paired language versions.
for (const version of ['0.1.0','0.2.0','0.3.0']) {
  const schema = schemas.find(s => s.$id.endsWith(`/compile-response${version === '0.1.0' ? '' : '-v'+version.slice(0,3)}.schema.json`));
  const profiles = schema.oneOf[1].properties.qualification.properties.operations.items.properties.declaration.properties.languageProfiles;
  const validate = ajv.compile(profiles);
  for (const dialect of ['0.1.0','0.2.0','0.3.0']) for (const ir of ['0.1.0','0.2.0','0.3.0']) {
    const supported = dialect === ir && dialect <= (version === '0.3.0' ? '0.3.0' : '0.2.0');
    assert(validate([{dialectProfile:`weft-sql/${dialect}`,irVersion:`weft-ir/${ir}`}]) === supported, `Response ${version} declaration pair ${dialect}/${ir}`);
  }
}

console.log(`Checked ${artifacts.size} governed artifacts, ${schemas.length} schemas, ${criteria.size} planned criteria and ${cases.length} fixture scenarios (${structuralNegatives} deliberate schema negatives).`);
console.log('No compiler, database or embedding tests were executed.');

