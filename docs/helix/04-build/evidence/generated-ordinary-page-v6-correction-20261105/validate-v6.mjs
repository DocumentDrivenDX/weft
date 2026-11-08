import fs from 'node:fs';
import Ajv from 'ajv/dist/2020.js';
const base = 'docs/helix/02-design/contracts/';
const evidence = 'docs/helix/04-build/evidence/generated-ordinary-page-v6-correction-20261105/';
const ajv = new Ajv({ strict: false, allErrors: true });
for (const file of ['query-codec-instructions.schema.json', 'generated-ordinary-page.schema.json']) {
  ajv.addSchema(JSON.parse(fs.readFileSync(base + file, 'utf8')));
}
const validate = ajv.compile(JSON.parse(fs.readFileSync(base + 'generated-read-algebra-v6.schema.json', 'utf8')));
for (const file of ['issued-v6.json']) {
  const value = JSON.parse(fs.readFileSync(evidence + file, 'utf8'));
  if (!validate(value)) throw new Error(`${file}: ${JSON.stringify(validate.errors)}`);
}
const eligible = JSON.parse(fs.readFileSync(evidence + 'issued-v6.json', 'utf8'));
const mutations = [
  (v) => { v.unexpected = true; },
  (v) => { v.roots['demo/Note'].unexpected = true; },
  (v) => { v.roots['demo/Note'].ordinaryPage.unexpected = true; },
  (v) => { v.roots['demo/Note'].ordinaryPage.key.codec.unexpected = true; },
];
for (const mutate of mutations) {
  const invalid = structuredClone(eligible);
  mutate(invalid);
  if (validate(invalid)) throw new Error('schema accepted a top/root/descriptor/codec drift negative');
}
const inventoryDrift = structuredClone(eligible);
inventoryDrift.roots['demo/Note'].ordinaryPage.page.fieldOutputs[0].output.oid = 9999;
if (validate(inventoryDrift)) throw new Error('field output inventory accepted invalid OID shape');
console.log('PASS: corrected generated /6 instance accepted; 5 schema-drift negatives refused.');
