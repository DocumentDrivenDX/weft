import fs from 'node:fs';
import Ajv from 'ajv/dist/2020.js';
const base = 'docs/helix/02-design/contracts/';
const evidence = 'docs/helix/04-build/evidence/generated-ordinary-page-v6-source-review-20261105/';
const ajv = new Ajv({ strict: false, allErrors: true });
for (const file of ['query-codec-instructions.schema.json', 'generated-ordinary-page.schema.json']) {
  ajv.addSchema(JSON.parse(fs.readFileSync(base + file, 'utf8')));
}
const validate = ajv.compile(JSON.parse(fs.readFileSync(base + 'generated-read-algebra-v6.schema.json', 'utf8')));
for (const file of ['issued-v6.json', 'issued-v6-escaped-identifiers.json', 'issued-v6-unsupported-root.json']) {
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
const unsupported = JSON.parse(fs.readFileSync(evidence + 'issued-v6-unsupported-root.json', 'utf8'));
unsupported.roots['demo/Metric'].ordinaryPage.key = {};
if (validate(unsupported)) throw new Error('unsupported branch accepted eligible-only members');
console.log('PASS: 3 generated /6 instances accepted; 5 schema-drift negatives refused.');
