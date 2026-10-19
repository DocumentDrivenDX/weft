// SPDX-License-Identifier: MIT OR Apache-2.0
import {readdir,readFile,writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import Ajv2020 from 'ajv/dist/2020.js';
const root=resolve(import.meta.dir,'../..');
const input=process.argv[2],output=process.argv[3];
if(!input||!output)throw Error('Usage: bun check-transport-schemas.ts ACTUAL_TEST_CAPTURE RECEIPT');
const raw=await readFile(input,'utf8');const captured=JSON.parse(raw);
if(!Array.isArray(captured)||captured.length!==41)throw Error('Exact actual producer capture required');
const ajv=new Ajv2020({strict:false,allErrors:true});
ajv.addSchema(JSON.parse(await readFile(resolve(root,'spec/upstream/umf-0.8.0.schema.json'),'utf8')));
async function walk(path:string):Promise<string[]>{
 if(path===resolve(root,'docs/helix/04-build/evidence'))return [];
 return (await Promise.all((await readdir(path,{withFileTypes:true})).map(e=>e.isDirectory()?walk(resolve(path,e.name)):Promise.resolve([resolve(path,e.name)])))).flat();
}
const schemas=new Map<string,any>();
for(const path of await walk(resolve(root,'docs/helix'))){if(!path.endsWith('.schema.json'))continue;const s=JSON.parse(await readFile(path,'utf8'));schemas.set(s.$id,s);ajv.addSchema(s);}
const profiles={binding:'urn:weft:postgresql:current-presence:binding:0.1-candidate',compiled:'urn:weft:postgresql:current-presence:sql:0.1-candidate',frame:'urn:weft:current-presence:transport-input:0.1-candidate',report:'urn:weft:current-presence:transport-result:0.1-candidate'};
const observations:Array<{id:string;valid:boolean;expected:boolean}>=[];
function check(id:string,profile:keyof typeof profiles,value:any,expected:boolean){const v=ajv.getSchema(profiles[profile]);if(!v)throw Error('Missing schema');const valid=Boolean(v(value));observations.push({id,valid,expected});if(valid!==expected)throw Error(`${id}: ${JSON.stringify(v.errors)}`);}
for(const record of captured)for(const profile of Object.keys(profiles) as Array<keyof typeof profiles>)check(`${record.id}:${profile}`,profile,record[profile],true);
const first=captured[0];
for(const profile of Object.keys(profiles) as Array<keyof typeof profiles>){
 const original=first[profile];const schema=schemas.get(profiles[profile]);
 for(const field of schema.required){const value=structuredClone(original);delete value[field];check(`${profile}:missing:${field}`,profile,value,false);}
 let value=structuredClone(original);value.unrecognized=true;check(`${profile}:unknown`,profile,value,false);
 value=structuredClone(original);value.version='weft.old-scalar/0.1-candidate';check(`${profile}:old-version`,profile,value,false);
 if('authority' in original){value=structuredClone(original);value.authority=true;check(`${profile}:authority`,profile,value,false);value=structuredClone(original);value.nativeAdmission='accepted';check(`${profile}:admission`,profile,value,false);}
 if(profile==='binding'||profile==='frame'||profile==='compiled'){value=structuredClone(original);const header=profile==='compiled'?value.disclosureHeader:value.header;header.columns[0].unrecognized=true;check(`${profile}:unknown-header-column`,profile,value,false);}
}
const receipt={scope:'Schema data consistency for actual Rust lowering/checker outputs over supplied UMF logical producer packets; no invocation/native authority proof',inputSha256:createHash('sha256').update(raw).digest('hex'),producerPackets:41,checks:observations.length,passed:observations.every(x=>x.valid===x.expected),observations};await writeFile(output,JSON.stringify(receipt,null,2)+'\n');console.log(JSON.stringify({producerPackets:41,checks:receipt.checks,passed:receipt.passed}));
