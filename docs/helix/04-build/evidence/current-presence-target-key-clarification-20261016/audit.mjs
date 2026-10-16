import Ajv2020 from 'ajv/dist/2020.js';
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
const root=process.cwd(),schemaPath='spec/upstream/umf-0.8.0.schema.json',fixturePath='docs/helix/03-test/fixtures/security-graph-source/adoc.json';
const schemaRaw=await readFile(resolve(root,schemaPath)),fixtureRaw=await readFile(resolve(root,fixturePath));
const schema=JSON.parse(schemaRaw),original=JSON.parse(fixtureRaw),sha=b=>createHash('sha256').update(b).digest('hex');
if(JSON.stringify(schema.$defs.relationshipTarget.required)!==JSON.stringify(['module','element','key']))throw Error('Canonical target requirements changed');
if(JSON.stringify(schema.$defs.relationshipEndpoint.required)!==JSON.stringify(['module','element']))throw Error('Canonical source requirements changed');
const validate=new Ajv2020({allErrors:true,strict:false,coerceTypes:false,useDefaults:false,removeAdditional:false}).compile(schema),checks=[];
const relationship=d=>d.modules.flatMap(m=>m.relationships??[])[0];
function check(name,edit,expected,missingKey=false){const model=structuredClone(original);edit(model);const valid=validate(model);const errors=structuredClone(validate.errors);if(valid!==expected)throw Error(name+': unexpected shape result');if(missingKey&&!errors?.some(e=>e.keyword==='required'&&e.params.missingProperty==='key'&&e.instancePath.includes('/target/')))throw Error(name+': required-target-key refusal missing');checks.push({name,schemaShapeAdmitted:valid,errors});}
check('original graph model shape',()=>{},true);
check('missing authored target Key refuses',d=>{delete relationship(d).target[0].key},false,true);
check('null target Key refuses',d=>{relationship(d).target[0].key=null},false);
check('source endpoint has no authored Key',d=>{if('key' in relationship(d).source[0])throw Error('Fixture has unexpected source Key')},true);
check('unknown Key ID needs semantic resolution beyond schema shape',d=>{relationship(d).target[0].key='missing-key-needs-original-semantic-resolution'},true);
const receipt={version:'weft-target-key-schema-audit/0.1',baseline:'51459bd7b5533adec8e40022c0428813f5f4537d',schema:{path:schemaPath,sha256:sha(schemaRaw)},fixture:{path:fixturePath,sha256:sha(fixtureRaw)},checks,semanticAdmission:false,nativeAuthority:false,scope:'Original canonical schema shape only; target Key existence/domain and whole-source semantic admission require original UMF interpretation'};
await writeFile(resolve(root,'docs/helix/04-build/evidence/current-presence-target-key-clarification-20261016/audit.json'),JSON.stringify(receipt,null,2)+'\n');console.log(JSON.stringify({passed:checks.length,semanticAdmission:false,nativeAuthority:false}));
