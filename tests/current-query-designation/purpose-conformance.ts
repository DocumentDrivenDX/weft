// SPDX-License-Identifier: MIT OR Apache-2.0
/** Independent original UMF purpose checker vs Weft's bounded operation-role subset.
 * @covers US-009-AC1 @covers US-009-AC2
 */
import {readFile,writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const [owner,binary,expectedSha,output]=process.argv.slice(2);if(!owner||!binary||!output||!/^[a-f0-9]{64}$/.test(expectedSha??''))throw Error('Original owner/binary/SHA/output required');
const sha=(s:string|Uint8Array)=>createHash('sha256').update(s).digest('hex');if(sha(await readFile(binary))!==expectedSha)throw Error('Producer mismatch');
const paths=['src/extensions/security/action-purposes.ts','src/extensions/security/action-purpose-declarations.ts','src/extensions/security/ontology-checks-candidate.ts','spec/extensions/security-v0.2/ontology.schema.json'];
const pins=async()=>Object.fromEntries(await Promise.all(paths.map(async p=>[p,sha(await readFile(resolve(owner,p)))])));const before=await pins();
const {inspectSecurityActionPurposes}=await import(pathToFileURL(resolve(owner,paths[0]!)).href);
const base=JSON.parse(await readFile(resolve(import.meta.dir,'fixtures/mixed-original-uses.request.json'),'utf8'));
const original=JSON.parse(await readFile(resolve(import.meta.dir,'fixtures/original-owner-source.json'),'utf8'));
const cases:[string,(o:any,p:any)=>void][]=[
 ['valid-source',()=>{}],
 ['missing-purpose-closure',o=>delete o.actionPurposes],
 ['missing-entry',o=>o.actionPurposes.pop()],
 ['duplicate-entry',o=>o.actionPurposes.push(structuredClone(o.actionPurposes[0]))],
 ['duplicate-action',o=>o.actions.push(o.actions[0])],
 ['duplicate-purpose',o=>o.actionPurposes[0].purposes.push(o.actionPurposes[0].purposes[0])],
 ['unknown-purpose',o=>o.actionPurposes[0].purposes=['unknown']],
 ['mixed-reverse',o=>o.actionPurposes[0].purposes=['read','reverse']],
 ['mixed-other',o=>o.actionPurposes[0].purposes=['read','other']],
 ['wrong-original-purpose',o=>o.actionPurposes.find((x:any)=>x.action==='query-original-salary-predicate').purposes=['field']],
 ['wrong-primary-purpose',o=>o.actionPurposes.find((x:any)=>x.action==='read').purposes=['field']],
 ['unrelated-field-action',(_o,p)=>p.bindings[0].originalAction='update'],
 ['unrelated-reverse-action',(_o,p)=>p.bindings[0].originalAction='reverse'],
 ['primary-action-as-original',(_o,p)=>p.bindings[0].originalAction='read'],
 ['unknown-original-action',(_o,p)=>p.bindings[0].originalAction='missing'],
 ['ordinary-purpose-combination',o=>o.actionPurposes.find((x:any)=>x.action==='update').purposes=['field','read']],
 ['opaque-action-name',(o,p)=>{const old='query-original-salary-predicate',name='reverse-looking-original-action';o.actions[o.actions.indexOf(old)]=name;o.actionPurposes.find((x:any)=>x.action===old).action=name;p.bindings[0].originalAction=name;}],
];
const observations=[];
for(const [id,edit]of cases){
 const request=structuredClone(base),source=request.opaqueSources.find((s:any)=>s.role==='ontology'),profileSource=request.opaqueSources.find((s:any)=>s.role==='query-designation');
 const ontology=JSON.parse(source.utf8),profile=JSON.parse(profileSource.utf8);edit(ontology,profile);
 source.utf8=JSON.stringify(ontology);source.sha256=sha(source.utf8);profile.sources.find((s:any)=>s.role==='ontology').sha256=source.sha256;profileSource.utf8=JSON.stringify(profile);profileSource.sha256=sha(profileSource.utf8);
 const inspection=inspectSecurityActionPurposes(ontology,original.closure.documents),purposes=new Map((inspection.purposes??[]).map((x:any)=>[x.action,x.purposes]));
 const compatible=inspection.status==='resolved-purposes'&&(purposes.get(profile.primaryAction)as string[]|undefined)?.includes('read')===true&&profile.bindings.every((e:any)=>e.originalAction!==profile.primaryAction&&(purposes.get(e.originalAction)as string[]|undefined)?.includes('original-authorized')===true);
 const proc=Bun.spawn([binary],{stdin:'pipe',stdout:'pipe',stderr:'pipe'});proc.stdin.write(JSON.stringify(request));proc.stdin.end();const [stdout,stderr,exit]=await Promise.all([new Response(proc.stdout).text(),new Response(proc.stderr).text(),proc.exited]);
 if((exit===0)!==compatible||(compatible?stderr!==''||JSON.parse(stdout).authority!==false:stdout!==''||!stderr.startsWith('WFT-CURRENT-QUERY-DESIGNATION:')))throw Error(id+': correspondence failed');
 observations.push({id,ownerStatus:inspection.status,ownerRoleCompatible:compatible,weftIssued:exit===0,passed:true});
}
const after=await pins();if(JSON.stringify(before)!==JSON.stringify(after)||sha(await readFile(binary))!==expectedSha)throw Error('Source changed');
await writeFile(output,JSON.stringify({scope:'Original UMF purpose declaration + Weft role compatibility only; not whole policy/current source/native permission. Opaque-name mutation is not an admitted policy revision.',producerSha256:expectedSha,selectedOwnerSourceDigests:after,controls:observations.length,observations},null,2)+'\n',{flag:'wx'});console.log(JSON.stringify({status:'passed',controls:observations.length}));
