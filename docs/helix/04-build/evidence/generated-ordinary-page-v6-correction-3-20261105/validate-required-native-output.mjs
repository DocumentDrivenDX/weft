import fs from 'node:fs';
import crypto from 'node:crypto';
import Ajv from 'ajv/dist/2020.js';
const base = 'docs/helix/02-design/contracts/';
const prior = 'docs/helix/04-build/evidence/generated-ordinary-page-v6-correction-2-20261105/';
const ajv = new Ajv({strict:false,allErrors:true});
const page = JSON.parse(fs.readFileSync(base+'generated-ordinary-page.schema.json','utf8'));
for (const file of ['query-codec-instructions.schema.json','generated-ordinary-page.schema.json']) ajv.addSchema(JSON.parse(fs.readFileSync(base+file,'utf8')));
const validateRoot = ajv.compile(JSON.parse(fs.readFileSync(base+'generated-read-algebra-v6.schema.json','utf8')));
const bytea = JSON.parse(fs.readFileSync(prior+'issued-v6-bytea.json','utf8'));
if (!validateRoot(bytea)) throw new Error(JSON.stringify(validateRoot.errors));
const validatePageOutput = ajv.compile(page.$defs.eligible.properties.page.properties.fieldOutputs.items.properties.output);
const char = JSON.parse(fs.readFileSync(prior+'synthetic-char-field-output.json','utf8'));
if (!validatePageOutput(char.output)) throw new Error(JSON.stringify(validatePageOutput.errors));
let omissions = 0;
const fields = bytea.roots['demo/Note'].ordinaryPage.page.fieldOutputs;
for (let i=0; i<fields.length; i++) {
  for (const removed of [['nativeType'],['oid'],['nativeType','oid']]) {
    const changed = structuredClone(bytea);
    const output = changed.roots['demo/Note'].ordinaryPage.page.fieldOutputs[i].output;
    for (const name of removed) delete output[name];
    if (validateRoot(changed)) throw new Error(`accepted omitted ${removed} on ${fields[i].fieldId}`);
    omissions++;
  }
}
for (const removed of [['nativeType'],['oid'],['nativeType','oid']]) {
  const changed = structuredClone(char.output);
  for (const name of removed) delete changed[name];
  if (validatePageOutput(changed)) throw new Error(`accepted helper output omission ${removed}`);
  omissions++;
}
console.log(JSON.stringify({status:'pass',scope:'schema-only omission correction; no new issuer bytes/native consumer qualification',fullByteaInstance:true,syntheticCharHelper:true,omissionNegatives:omissions,schemaSha256:crypto.createHash('sha256').update(fs.readFileSync(base+'generated-ordinary-page.schema.json')).digest('hex')}));
