/** Genuine compiler source/use/designation correspondence with original UMF routes.
 * @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
 * Diagnostic interoperability only; no selected branch, SQL or native permission.
 */
import {readFile,writeFile,readdir} from 'node:fs/promises';
import {resolve} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const [owner,lineageBinary,designationBinary,output]=process.argv.slice(2);
if(!owner||!lineageBinary||!designationBinary||!output)throw Error('Original owner and two actual producers required');
const sha=(x:string|Uint8Array)=>createHash('sha256').update(x).digest('hex');
const paths=['src/extensions/security/query-value-routes-candidate.ts','src/extensions/security/catalog-program-candidate.ts','src/extensions/security/catalog-storage-candidate.ts','src/extensions/security/protection/queries.ts'];
const pins=async()=>Object.fromEntries(await Promise.all(paths.map(async p=>[p,sha(await readFile(resolve(owner,p)))])));
const manifestRaw=await readFile(resolve(import.meta.dir,'umf-source-manifest.json'));
if(sha(manifestRaw)!=='3e7851b0b24afe50981b1f89103fe6595a8d29a44b2a769c1541477549963897')throw Error('Original owner manifest changed');
const manifest=JSON.parse(manifestRaw.toString());
if(manifest.sourceCommit!=='c57d283ece30326bcb06607ca5d2384fc69ddef4'||manifest.files[paths[0]!]!=='2eccc51f05a3e34280a1b1e5b44dbfdb9b43acd739e96d20236a649d02347f59')throw Error('Original owner version mismatch');
async function sourceInventory(path:string,prefix:string):Promise<string[]>{const out:string[]=[];for(const item of await readdir(path,{withFileTypes:true})){const name=prefix+'/'+item.name;if(item.isSymbolicLink())throw Error('Source symlink');if(item.isDirectory())out.push(...await sourceInventory(resolve(path,item.name),name));else if(item.isFile())out.push(name);else throw Error('Unknown source entry');}return out;}
async function verifySource(){
 const actual=[...(await sourceInventory(resolve(owner,'src'),'src')),...(await sourceInventory(resolve(owner,'spec'),'spec')),'package.json','bun.lock'].sort(),expected=Object.keys(manifest.files).sort();
 if(JSON.stringify(actual)!==JSON.stringify(expected))throw Error('Original source inventory mismatch');
 for(let i=0;i<actual.length;i+=16)await Promise.all(actual.slice(i,i+16).map(async path=>{if(sha(await readFile(resolve(owner,path)))!==manifest.files[path])throw Error('Original owner source changed: '+path);}));
}
await verifySource();
const before=await pins(),lineageSha=sha(await readFile(lineageBinary)),designationSha=sha(await readFile(designationBinary));
if(lineageSha!=='113c1dab3f51a5646a04e545573264d071289bd147d641f235fc455f1820f6d9'||designationSha!=='418a5f3b33231fd000eaec1d8bfdde1fba8f85385cb4f937c76a5c9c1722b41e')throw Error('Original compiler producer mismatch');
const {compileCandidateCatalogProgram}=await import(pathToFileURL(resolve(owner,paths[1]!)).href);
const {inspectCandidateQueryValueRoutes}=await import(pathToFileURL(resolve(owner,paths[0]!)).href);
const originalRaw=await readFile(resolve(import.meta.dir,'fixtures/original-owner-source.json'));
if(sha(originalRaw)!=='2f826621bd3187857f1b4fbab5d9c818891b0df96d55d6a1d379638906e2ebda')throw Error('Original source changed');
const original=JSON.parse(originalRaw.toString()),program=compileCandidateCatalogProgram(original.policy,original.catalog,original.closure,original.ontology);
async function produce(binary:string,raw:string){const p=Bun.spawn([binary],{stdin:'pipe',stdout:'pipe',stderr:'pipe'});p.stdin.write(raw);p.stdin.end();const [body,error,exit]=await Promise.all([new Response(p.stdout).text(),new Response(p.stderr).text(),p.exited]);if(exit||error)throw Error('Producer refused');return JSON.parse(body);}
const names=['disclosed-token-equality','disclosed-token-selfjoin','original-token-order','read-dto-hidden-original','mixed-original-uses','prohibited-salary-order-empty','unsupported-salary-sum'];
const observations:any[]=[],overrideRefusals:any[]=[];
for(const name of names){
 const raw=await readFile(resolve(import.meta.dir,'fixtures/'+name+'.request.json'),'utf8');
 const lineage=await produce(lineageBinary,raw),designation=await produce(designationBinary,raw);
 const expected=JSON.parse(await readFile(resolve(import.meta.dir,'fixtures/'+name+'.report.json'),'utf8'));if(JSON.stringify(designation)!==JSON.stringify(expected))throw Error('Frozen designation changed');
 const request=JSON.parse(raw);for(const role of ['policy','ontology','catalog','owner']){const bodies=request.opaqueSources.filter((s:any)=>s.role===role);if(bodies.length!==1||bodies[0].sha256!==sha(bodies[0].utf8)||JSON.stringify(JSON.parse(bodies[0].utf8))!==JSON.stringify(original[role]))throw Error('Original source mismatch');}
 for(const input of request.modules){if(input.pin.sha256!==sha(input.documentJson))throw Error('Model digest mismatch');const entry=original.closure.documents.find((d:any)=>d.document.id===input.pin.documentId);if(!entry||JSON.stringify(JSON.parse(input.documentJson))!==JSON.stringify(entry.document)||!original.closure.pins.some((p:any)=>p.documentId===input.pin.documentId&&p.revision===input.pin.revision))throw Error('Original model closure mismatch');}
 if(lineage.sourceRequestSha256!==sha(raw)||designation.sourceRequestSha256!==sha(raw)||lineage.authority!==false||designation.authority!==false||lineage.securityReady!==false||designation.securityReady!==false)throw Error('Source correspondence');
 if(lineage.uses.length!==designation.uses.length)throw Error('Omitted designation occurrence');
 for(const use of lineage.uses){
  const ds=designation.uses.filter((d:any)=>d.path===use.path&&d.scan===use.scan);
  if(ds.length!==1||JSON.stringify(ds[0].fieldIdentity)!==JSON.stringify(use.fieldIdentity)||JSON.stringify(ds[0].target)!==JSON.stringify(use.type)||ds[0].operator!==use.operator||ds[0].operation!==use.operation)throw Error('Original occurrence correspondence');
  if(use.propertyStatus!=='derived'){if(name!=='unsupported-salary-sum'||!lineage.readRequirements.unsupportedSources.length)throw Error('Unsupported operation lost');observations.push({name,path:use.path,status:'unsupported-original-operation'});continue;}
  const requirement={scan:use.scan,path:use.path,type:use.type,field:use.field,operator:use.operator,operation:use.operation,required:use.required};
  const route=inspectCandidateQueryValueRoutes(program,original.owner,requirement);
  if(route.status!=='inspected'||route.complete!==false||route.authority!==false||route.branches.length!==1||route.composition.branchActivationUnverified!==true)throw Error('Conditional route correspondence');
  if(JSON.stringify(route.use)!==JSON.stringify(requirement))throw Error('Original route requirement changed');
  const expectedConditions=original.policy.rules.flatMap((rule:any,ruleIndex:number)=>rule.actions.includes(original.owner.disclosure.readAction)&&rule.target.some((target:any)=>JSON.stringify(target)===JSON.stringify(original.ontology.entities.find((e:any)=>e.type.elementId===use.type.elementId).type))?[{ruleIndex,ruleId:rule.id,effect:rule.effect,condition:rule.condition}]:[]);
  if(JSON.stringify(route.composition.readConditions)!==JSON.stringify(expectedConditions))throw Error('Original read/membership conditions lost');
  const branch=route.branches[0];
  const expected=name==='prohibited-salary-order-empty'?'prohibited':use.field.elementId==='salary'||use.operator==='order'?'requires-original-action':'compatible-disclosed-properties';
  if(branch.status!==expected)throw Error('Wrong joint mode');
  if(expected==='requires-original-action'&&!ds[0].originalAction)throw Error('Missing exact separate designation');
  if(expected==='prohibited'){if(branch.queryValue.field!==null||branch.queryValue.origin!=='none'||ds[0].originalAction!==null)throw Error('Prohibition defaulted');}
  else if(JSON.stringify(branch.queryValue.field)!==JSON.stringify(use.field)||branch.queryValue.origin!=='live-field')throw Error('Original live domain replaced');
  if(use.field.elementId==='salary'&&branch.projection.field.elementId!=='salaryMaskLabel')throw Error('Read DTO lost');
  const domains=route.domains as any[];for(const domain of domains){const entry=original.closure.documents.find((d:any)=>d.document.id===domain.field.documentId),module=entry?.document.modules.find((m:any)=>m.id===domain.field.moduleId),element=module?.elements.find((e:any)=>e.id===domain.field.elementId);if(!element||JSON.stringify(element)!==JSON.stringify(domain.element)||domain.identity.documentId!==domain.field.documentId||domain.identity.module!==domain.field.moduleId||domain.identity.element!==domain.field.elementId||domain.identity.revision!==original.closure.pins.find((p:any)=>p.documentId===domain.field.documentId)?.revision)throw Error('Domain source lost');}
  if(use.field.elementId==='salary'){const rawDomain=domains.find((d:any)=>d.field.elementId==='salary'),dto=domains.find((d:any)=>d.field.elementId==='salaryMaskLabel');if(rawDomain?.element.scalarType!=='integer'||dto?.element.scalarType!=='string'||use.logicalType.family!=='integer')throw Error('Read DTO replaced original scalar domain');}
  if(use.field.elementId==='token'&&route.sources.preStorageInput.selectable!==false)throw Error('Pre-storage selectable');
  for(const key of ['originalAction','mode','authority','securityReady']){const bad=inspectCandidateQueryValueRoutes(program,original.owner,{...requirement,[key]:'read'});if(bad.status!=='refused'||bad.branches.length)throw Error('Caller override accepted');overrideRefusals.push({case:name,path:use.path,scan:use.scan,key,refused:true});}
  observations.push({name,path:use.path,scan:use.scan,status:branch.status,queryValue:branch.queryValue,projection:branch.projection,originalAction:ds[0].originalAction,branchActivationUnverified:true});
 }
 if(name==='prohibited-salary-order-empty'&&(!lineage.readRequirements.scans.length||!lineage.readRequirements.fields.length))throw Error('Empty selection elided preflight');
}
const ref=(elementId:string)=>({documentId:'domain',moduleId:'m',elementId});
const target=(f:any)=>f.ontology.entities.find((e:any)=>e.type.elementId==='Resource');
const constraints:any[]=[];
for(const [id,edit,expected]of [
 ['ontology-storage-conflict',(f:any)=>target(f).fields.find((x:any)=>x.ref.elementId==='token').queryUse.join='original-authorized','conflicting-modes'],
 ['ontology-prohibition-dominates',(f:any)=>target(f).fields.find((x:any)=>x.ref.elementId==='token').queryUse.join='prohibited','prohibited'],
 ['storage-prohibition-dominates',(f:any)=>f.catalog.transforms.find((t:any)=>t.outputDomain.elementId==='token').queryUse.join='prohibited','prohibited'],
]as [string,(f:any)=>void,string][]){
 const f=structuredClone(original);edit(f);const p=compileCandidateCatalogProgram(f.policy,f.catalog,f.closure,f.ontology);
 const route=inspectCandidateQueryValueRoutes(p,f.owner,{scan:'s0',path:'joins/0/on/0/left',type:ref('Resource'),field:ref('token'),operator:'join',operation:'equal',required:['equality']});
 if(route.status!=='inspected'||route.branches.length!==1||route.branches[0].status!==expected||route.branches[0].queryValue.field!==null||route.authority!==false)throw Error('Joint constraint lost: '+id);
 constraints.push({id,status:route.branches[0].status,passed:true});
}
{
 const raw=await readFile(resolve(import.meta.dir,'fixtures/mixed-original-uses.request.json'),'utf8'),request=JSON.parse(raw),source=request.opaqueSources.find((s:any)=>s.role==='query-designation'),profile=JSON.parse(source.utf8);
 profile.bindings=profile.bindings.filter((e:any)=>e.field.elementId!=='salary');source.utf8=JSON.stringify(profile);source.sha256=sha(source.utf8);
 const designation=await produce(designationBinary,JSON.stringify(request));
 if(designation.uses.find((u:any)=>u.field.elementId==='salary').originalAction!==null||designation.uses.find((u:any)=>u.field.elementId==='token').originalAction!=='query-original-token-order'||designation.authority!==false)throw Error('Missing action borrowed from another use');
 const route=inspectCandidateQueryValueRoutes(program,original.owner,{scan:'s0',path:'filters/0/left',type:ref('Resource'),field:ref('salary'),operator:'predicate',operation:'equal',required:['equality']});
 if(route.status!=='inspected'||route.branches[0].status!=='requires-original-action'||route.authority!==false)throw Error('Missing designation became permission');
 constraints.push({id:'missing-per-use-designation-remains-unresolved',passed:true});
}
{
 const path=resolve(import.meta.dir,'fixtures/original-token-order.request.json'),request=JSON.parse(await readFile(path,'utf8'));
 request.sql='SELECT r.token FROM Resource r ORDER BY r.token DESC, r.salary ASC ';const raw=JSON.stringify(request);
 request.sql='SELECT r.token FROM Resource r ORDER BY r.token ASC , r.salary DESC';const other=JSON.stringify(request);
 const a=await produce(lineageBinary,raw),b=await produce(lineageBinary,other),da=await produce(designationBinary,raw),db=await produce(designationBinary,other);
 if(a.applicationPlanSha256!==b.applicationPlanSha256||a.applicationPlanJson!==b.applicationPlanJson||JSON.stringify(a.orderDirections)===JSON.stringify(b.orderDirections)||da.sourceRequestSha256===db.sourceRequestSha256)throw Error('Direction/source lifetime binding lost');
 constraints.push({id:'byte-identical-plan-opposite-order-needs-original-context',samePlanSha256:a.applicationPlanSha256,directions:[a.orderDirections,b.orderDirections],distinctSourceRequestDigests:true,passed:true});
}
await verifySource();
if(JSON.stringify(before)!==JSON.stringify(await pins())||lineageSha!==sha(await readFile(lineageBinary))||designationSha!==sha(await readFile(designationBinary)))throw Error('Sources/producers changed');
await writeFile(output,JSON.stringify({scope:'Original producer-to-owner conditional route correspondence only; no branch activation, native permission, SQL or acceptance promotion. Counterfactual mode-source edits are not admitted policy revisions.',ownerSourceCommit:manifest.sourceCommit,ownerSourceManifestSha256:sha(manifestRaw),ownerSourceFilesVerified:Object.keys(manifest.files).length,bunVersion:Bun.version,selectedOwnerSourceDigests:before,lineageProducerSha256:lineageSha,designationProducerSha256:designationSha,controls:observations.length+constraints.length+overrideRefusals.length,constraints,overrideRefusals,observations},null,2)+'\n',{flag:'wx'});
console.log(JSON.stringify({status:'passed',occurrences:observations.length,controls:observations.length+constraints.length+overrideRefusals.length}));
