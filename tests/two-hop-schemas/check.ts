import Ajv2020 from 'ajv/dist/2020.js';
import assert from 'node:assert/strict';
const root=new URL('../../docs/helix/02-design/contracts/',import.meta.url);
const names=['logical-plan-v0.4','compile-response-v0.4','application-result-v0.4','backend-manifest-v0.3'];
const schemas=await Promise.all(names.map(async n=>JSON.parse(await Bun.file(new URL(n+'.schema.json',root)).text())));
const ajv=new Ajv2020({strict:false,allErrors:true});schemas.forEach(s=>ajv.addSchema(s));
let controls=0;function check(v:any,x:any,expected:boolean){assert.equal(!!v(x),expected,JSON.stringify(v.errors));controls++}
const id={documentId:'original',revision:'r1',module:'m',element:'Root'},type={family:'string',facets:{},nullable:false};
const key={id:'key',fields:[id],types:[type]},rid={documentId:'original',revision:'r1',module:'m',relationship:'link'};
const hop={identity:rid,inverse:false,from:id,to:id,sourceKey:key,targetKey:key,sourceMultiplicity:{min:0,max:'*'},targetMultiplicity:{min:0,max:'*'},targetLifecycle:'independent'};
const path={startScan:'root',hops:[hop,hop],span:{start:0,end:20},hopSpans:[{start:1,end:5},{start:6,end:10}]};
const pin={documentId:'original',revision:'r1',umfVersion:'0.8.0',sha256:'a'.repeat(64)};
const plan={irVersion:'weft-ir/0.4.0',modulePins:[pin],readProfile:null,requiredCapabilities:[],typeGraph:[],source:{occurrence:'root',record:id,pin},pageKey:null,joins:[],filters:[],groups:[],aggregate:false,outputs:[{name:'paths',expression:{op:'relatedPaths',path,bound:2}}],order:[],limit:null};
const pv=ajv.getSchema(schemas[0].$id)!;check(pv,plan,true);
for(const f of [(x:any)=>x.irVersion='weft-ir/0.3.0',(x:any)=>x.pathExpansion={occurrence:'p',path},(x:any)=>x.outputs[0].expression.bound=0,(x:any)=>x.outputs[0].expression.path.hops.pop(),(x:any)=>x.outputs[0].expression.path.nativeId=1,(x:any)=>x.readProfile={kind:'entityPage'}]){let x=structuredClone(plan);f(x);check(pv,x,false)}
let count=structuredClone(plan);count.aggregate=true;count.outputs=[{name:'n',expression:{op:'countDistinctPathTargets',pathOccurrence:'p',type:{family:'integer',facets:{},nullable:false}}}];check(pv,count,false);count.pathExpansion={occurrence:'p',path};check(pv,count,true);
const av=ajv.getSchema(schemas[2].$id)!;
const collection={items:[{intermediate:['true'],terminal:['17'],edges:['-1','0']},{intermediate:['true'],terminal:['17'],edges:['-1','0']}],truncated:false};
for(const x of [collection,{state:'absent'},{state:'value',value:collection}])check(av,x,true);
for(const x of [null,{state:'null'},{state:'value',value:{}},{items:[{intermediate:[true],terminal:['17'],edges:['1','2']}],truncated:false},{items:[{intermediate:['x'],terminal:['17'],edges:['01','2']}],truncated:false},{...collection,extra:1}])check(av,x,false);
const response=schemas[1].oneOf[1];
function fragment(fragment:any){return ajv.compile({$defs:schemas[1].$defs,...fragment})}
const rv=fragment(response.properties.columns.items.properties.representation);
const representation={kind:'relatedPaths',path,startRecord:id,bound:2,edgeEncoding:'signed64-decimal/0.1'};check(rv,representation,true);check(rv,{...representation,nativeNull:false},false);
const target={kind:'scalar',logicalType:{family:'integer',facets:{},nullable:false},carrier:'text',decoder:'exact-integer',pathTarget:{pathOccurrence:'p',record:id,key}};check(rv,target,true);check(rv,{...target,decoder:'text'},false);
const ov=fragment(response.properties.obligations.items);
const occurrence={id:'ashlar.path.occurrenceIntegrity',owner:'host',failureCode:'WFT-BINDING',parameters:{phase:'before-user-query',samePublicationRequired:true,noPartialPublication:true,success:'one exact STRING count equal to 0 per check',paths:[{path,edgeEncoding:'signed64-decimal/0.1',bound:2}],edgeSchemas:[0,1].map(hop=>({pathIndex:0,hop,relationship:rid,table:{name:['cat','schema','edges'],uuid:'uuid',version:0},identityColumn:'id',nativeType:'BIGINT'})),checks:[{pathIndex:0,kind:'edgeEncoding',sql:'SELECT 0',failureCode:'WFT-BINDING'}]}};
check(ov,occurrence,true);let missing=structuredClone(occurrence);delete missing.parameters.edgeSchemas;check(ov,missing,false);check(ov,{...occurrence,parameters:{...occurrence.parameters,extra:true}},false);
// Schema cannot attest SQL correctness, cross-reference equality, check coverage,
// native types/values, signed64 range or count semantics; those need owning admission.
for(const f of [(x:any)=>x.aggregate=true,(x:any)=>x.groups=[{scan:'root',identity:id,type}],(x:any)=>x.having=[{count:{op:'count',type:{family:'integer',facets:{},nullable:false}},threshold:{kind:'literal',value:'1'}}]]){let x=structuredClone(plan);f(x);check(pv,x,false)}
check(pv,{...count,aggregate:false},false);
check(av,{items:Array(1001).fill(collection.items[0]),truncated:true},false);
check(av,{items:[{...collection.items[0],edges:['1'.repeat(21),'0']}],truncated:false},false);
// Complete envelopes exercise nested language/interface fences, not fragments alone.
function sample(s:any,defs:any={}):any{
 if(s.$ref){if(s.$ref.startsWith('#/$defs/'))return sample(defs[s.$ref.split('/').at(-1)!],defs);if(s.$ref.includes('logical-plan-v0.4'))return structuredClone(plan);throw Error(s.$ref)}
 if('const'in s)return structuredClone(s.const);if(s.enum)return s.enum[0];if(s.oneOf)return sample(s.oneOf[0],defs);if(s.anyOf)return sample(s.anyOf[0],defs);
 if(s.type==='object'){const x:any={};for(const k of s.required??[])x[k]=sample(s.properties[k],defs);return x}
 if(s.type==='array')return Array.from({length:s.minItems??0},()=>sample(s.items,defs));if(s.type==='integer'||s.type==='number')return s.minimum??0;if(s.type==='boolean')return false;if(s.type==='null')return null;
 if(s.type==='string')return s.pattern?.includes('64')?'a'.repeat(64):s.pattern?.includes('semver')?'1.0.0':'x';return null;
}
const full=sample(response,schemas[1].$defs);full.logicalPlan=structuredClone(plan);full.columns=[{position:1,outputName:'paths',representation,sourceIdentities:[id],nullable:false}];full.obligations=[occurrence];
const fv=ajv.getSchema(schemas[1].$id)!;check(fv,full,true);
for(const f of [(x:any)=>x.backend.interfaceVersion='weft-backend/0.2.0',(x:any)=>x.obligations=[],(x:any)=>x.columns[0].nullable=true]){const x=structuredClone(full);f(x);check(fv,x,false)}
const blocked={interfaceVersion:'weft-compile/0.4.0',diagnostics:[],status:'blocked'};check(fv,blocked,true);check(fv,{...blocked,interfaceVersion:'weft-compile/0.3.0'},false);
const manifest=sample(schemas[3]);manifest.capabilities[0].logicalDomain={subset:"fixture"};manifest.capabilities[0].resultDomain={subset:"fixture"};const mv=ajv.getSchema(schemas[3].$id)!;check(mv,manifest,true);check(mv,{...manifest,interfaceVersion:'weft-backend/0.2.0'},false);check(mv,{...manifest,languageProfiles:[{dialectProfile:'weft-sql/0.3.0',irVersion:'weft-ir/0.3.0'}]},false);

const wrongCapability=structuredClone(manifest);wrongCapability.capabilities[0].languageProfiles=[{dialectProfile:"weft-sql/0.3.0",irVersion:"weft-ir/0.3.0"}];check(mv,wrongCapability,false);
const nullableTarget=structuredClone(full);nullableTarget.logicalPlan=count;nullableTarget.columns[0].representation=target;nullableTarget.columns[0].nullable=true;check(fv,nullableTarget,false);
const completeTarget=structuredClone(full);completeTarget.logicalPlan=count;completeTarget.columns[0].representation=target;completeTarget.obligations.push({id:'ashlar.path.countCapacity',owner:'host',failureCode:'WFT-CAPABILITY',parameters:{phase:'before-user-query',samePublicationRequired:true,noPartialPublication:true,nativeRepresentation:'signed64',checks:[{pathOccurrence:'p',kind:'targetDistinct',sql:'SELECT 0',failureCode:'WFT-CAPABILITY'}],success:'one exact STRING count equal to 0 per check'}});check(fv,completeTarget,true);
check(fv,{...completeTarget,columns:[{...completeTarget.columns[0],nullable:true}]},false);
const withOperation=structuredClone(full);withOperation.qualification.operations=[sample(response.properties.qualification.properties.operations.items,schemas[1].$defs)];withOperation.qualification.operations[0].declaration.logicalDomain={subset:'fixture'};withOperation.qualification.operations[0].declaration.resultDomain={subset:'fixture'};check(fv,withOperation,true);const oldOperation=structuredClone(withOperation);oldOperation.qualification.operations[0].declaration.languageProfiles=[{dialectProfile:'weft-sql/0.3.0',irVersion:'weft-ir/0.3.0'}];check(fv,oldOperation,false);
const forged={id:'ashlar.path.occurrenceIntegrity',owner:'host',failureCode:'WFT-BINDING',parameters:{forged:true}};
for(const where of ['assessment','declaration']){const x=structuredClone(withOperation);x.qualification.operations[0][where].obligations=[forged];check(fv,x,false)}
const badManifest=structuredClone(manifest);badManifest.capabilities[0].obligations=[forged];check(mv,badManifest,false);
const rowCount=structuredClone(completeTarget);rowCount.logicalPlan.outputs=[{name:'n',expression:{op:'count',type:{family:'integer',facets:{},nullable:false}}}];delete rowCount.columns[0].representation.pathTarget;rowCount.obligations[1].parameters.checks[0].kind='pathRows';check(fv,rowCount,true);
const missingRowCapacity=structuredClone(rowCount);missingRowCapacity.obligations.pop();check(fv,missingRowCapacity,false);
console.log(JSON.stringify({totalControls:controls,scope:"structural schema controls only; no compiler/native proof"}));
