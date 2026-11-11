// SPDX-License-Identifier: MIT OR Apache-2.0
import { readdir, readFile } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { createHash } from 'node:crypto';
import Ajv2020 from 'ajv/dist/2020.js';

const root = resolve(import.meta.dir, '..');
const read = (p: string) => readFile(resolve(root, p), 'utf8');
const json = async (p: string) => JSON.parse(await read(p));
const assert = (ok: unknown, message: string): asserts ok => { if (!ok) throw new Error(message); };
async function walk(p: string): Promise<string[]> {
  const entries = await readdir(resolve(root, p), { withFileTypes: true });
  return (await Promise.all(entries.map(e => e.isDirectory() ? walk(`${p}/${e.name}`) : [`${p}/${e.name}`]))).flat();
}
const files = await walk('docs/helix');
const schemaDir = 'docs/helix/02-design/contracts';
const ajv = new Ajv2020({ allErrors: true, strict: false });
// Security contract literals reference the separately pinned UMF 0.8 envelope.
const core08Text = await read('spec/upstream/umf-0.8.0.schema.json');
const core08Pin = await json('spec/upstream/umf-0.8.0.source.json');
assert(createHash('sha256').update(core08Text, 'utf8').digest('hex') === core08Pin.sha256,
  'Pinned UMF 0.8 schema bytes differ');
ajv.addSchema(JSON.parse(core08Text));
// Current security contracts use the owner-pinned 0.10 bundle, including its
// semantic-type reference resource. Retain 0.8 for historical contracts.
const core010Pin = await json('spec/upstream/umf-0.10.0-bundled.source.json');
const core010Text = await read(core010Pin.bundlePath);
assert(createHash('sha256').update(core010Text, 'utf8').digest('hex') === core010Pin.bundleSha256,
  'Pinned UMF 0.10 schema bytes differ');
ajv.addSchema(JSON.parse(core010Text));
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

console.log(`Checked ${schemas.length} schemas and ${cases.length} fixture scenarios (${structuralNegatives} deliberate schema negatives).`);
console.log('No compiler, database or embedding tests were executed.');

