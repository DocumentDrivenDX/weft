import fs from 'node:fs';
import Ajv from 'ajv/dist/2020.js';
const base = 'docs/helix/02-design/contracts/';
const evidence = 'docs/helix/04-build/evidence/generated-ordinary-page-v6-correction-2-20261105/';
const ajv = new Ajv({strict:false,allErrors:true});
for (const file of ['query-codec-instructions.schema.json','generated-ordinary-page.schema.json']) ajv.addSchema(JSON.parse(fs.readFileSync(base+file,'utf8')));
const schema = JSON.parse(fs.readFileSync(base+'generated-read-algebra-v6.schema.json','utf8'));
const validateRoot = ajv.compile(schema);
const bytea = JSON.parse(fs.readFileSync(evidence+'issued-v6-bytea.json','utf8'));
if (!validateRoot(bytea)) throw new Error(JSON.stringify(validateRoot.errors));
const validateOutput = ajv.compile(schema.$defs.output);
const char = JSON.parse(fs.readFileSync(evidence+'synthetic-char-field-output.json','utf8'));
if (!validateOutput(char.output)) throw new Error(`char helper output: ${JSON.stringify(validateOutput.errors)}`);
const byteaField=bytea.roots['demo/Note'].ordinaryPage.page.fieldOutputs.find(x=>x.fieldId==='demo/opaque');
for (const [label, source, mutate] of [
  ['char OID drift', char, x=>x.output.oid=25],
  ['char carrier drift', char, x=>x.output.carrier='bits128'],
  ['bytea carrier drift', byteaField, x=>x.output.carrier='string'],
  ['bytea OID drift', byteaField, x=>x.output.oid=1042],
]) {
  const x=structuredClone(source); mutate(x);
  if (validateOutput(x.output)) throw new Error(`schema accepted ${label}`);
}
console.log('PASS: bytea complete /6 root and synthetic char helper output validate; 4 native metadata drift negatives refuse.');
