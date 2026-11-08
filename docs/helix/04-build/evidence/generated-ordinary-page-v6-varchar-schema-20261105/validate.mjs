import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import fs from 'node:fs';
import Ajv from 'ajv/dist/2020.js';

const base = 'docs/helix/02-design/contracts/';
const requestPath = 'docs/helix/04-build/evidence/generated-ordinary-page-v6-wheel-20261105/request-v6.json';
const python = process.env.WEFT_V6_PYTHON ?? '/private/tmp/umf-build-v6-dev-20261105/bin/python';
const wheelSource = `import hashlib,json,pathlib,weft
assert weft.__version__ == '0.4.65.dev1', weft.__version__
request=json.loads(pathlib.Path(${JSON.stringify(requestPath)}).read_text())
model=json.loads(request['binding']['modelJson'])
model['modules'][0]['extensions']['umf.tables']['tables'][0]['columns'][0]['nativeType']={'kind':'varchar','length':32}
request['binding']['modelJson']=json.dumps(model,sort_keys=True,separators=(',',':'),ensure_ascii=False)
request['binding']['modelSha256']=hashlib.sha256(request['binding']['modelJson'].encode()).hexdigest()
print(weft.prepare_generated_read_algebra_v6_json(json.dumps(request,separators=(',',':'))))`;
const issued = JSON.parse(execFileSync(python, ['-c', wheelSource], { encoding: 'utf8' }));
const root = issued.roots['demo/Note'];
assert.equal(issued.version, 'weft.generated-read-algebra/6');
assert.equal(root.ordinaryPage.status, 'eligible');
assert.deepEqual(root.ordinaryPage.key.output.nativeType, { kind: 'varchar', length: 32 });
assert.deepEqual(root.ordinaryPage.page.hiddenKeyOutput.nativeType, { kind: 'varchar', length: 32 });
assert.equal(root.ordinaryPage.key.output.oid, 1043);
assert.equal(root.ordinaryPage.page.hiddenKeyOutput.oid, 1043);

const ajv = new Ajv({ strict: false, allErrors: true });
for (const file of ['query-codec-instructions.schema.json', 'generated-ordinary-page.schema.json']) {
  ajv.addSchema(JSON.parse(fs.readFileSync(base + file, 'utf8')));
}
const validate = ajv.compile(JSON.parse(fs.readFileSync(base + 'generated-read-algebra-v6.schema.json', 'utf8')));
assert.equal(validate(issued), true, JSON.stringify(validate.errors));
let refused = 0;
function reject(type, oid) {
  const value = structuredClone(issued);
  value.roots['demo/Note'].ordinaryPage.key.output.nativeType = type;
  value.roots['demo/Note'].ordinaryPage.key.output.oid = oid;
  value.roots['demo/Note'].ordinaryPage.page.hiddenKeyOutput.nativeType = type;
  value.roots['demo/Note'].ordinaryPage.page.hiddenKeyOutput.oid = oid;
  assert.equal(validate(value), false, JSON.stringify({ type, oid }));
  refused++;
}
reject({ kind: 'varchar' }, 1043);
reject({ kind: 'varchar', length: 0 }, 1043);
reject({ kind: 'varchar', length: '32' }, 1043);
reject({ kind: 'char', length: 32 }, 1042);
reject({ kind: 'varchar', length: 32 }, 25);
reject({ kind: 'text' }, 1043);
const schemaSha256 = createHash('sha256').update(fs.readFileSync(base + 'generated-ordinary-page.schema.json')).digest('hex');
console.log(JSON.stringify({ status: 'pass', wheel: 'weft-sql 0.4.65.dev1', issued: 'eligible varchar(32)', schemaSha256, rejected: refused }));
