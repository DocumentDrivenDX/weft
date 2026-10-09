// Public UMF source/presence evidence; this is a Bun-only harness, not a validator.
import {createHash} from 'node:crypto';
import {readFileSync,writeFileSync} from 'node:fs';
import {resolve} from 'node:path';
import {execFileSync} from 'node:child_process';
const [umfRoot,packRoot,output]=process.argv.slice(2);if(!umfRoot||!packRoot||!output)throw new Error('Explicit clean UMF source, pack source and fresh output required');
const umfRevision='a95c3ec18a8f904decde884a4fa252988d2a5b0f';
if(execFileSync('git',['-C',umfRoot,'rev-parse','HEAD'],{encoding:'utf8'}).trim()!==umfRevision||execFileSync('git',['-C',umfRoot,'diff','--name-only','HEAD','--','src'],{encoding:'utf8'}).trim())throw new Error('Exact clean original public API pin required');
const api=await import(resolve(umfRoot,'src/model/schema-properties.ts'));
const record=await import(resolve(umfRoot,'src/model/record-values.ts'));
const requests=[['archaeology','samples.parent_id',[null,{string:''},{string:'original'}]],['ecology','observations.value',[null,{decimalToken:'0.00'},{string:'invalid-number'}]],['ecology','effort.amount',[null,{decimalToken:'0.00'}]],['ecology','occurrences.count',[null,{integerToken:'0'}]]];
const probes=[];
for(const [pack,element,values]of requests){
 const bytes=readFileSync(resolve(packRoot,String(pack),'upstream/ontology.json'));const source=JSON.parse(bytes.toString());const identity={module:'domain',element:String(element)};
 for(const value of values as unknown[]){const validation=api.validateCoreFieldValue(source,identity,value);probes.push({pack,sourceSha256:createHash('sha256').update(bytes).digest('hex'),identity,value,validation});}
 const receipt=record.validateCoreRecordValues(source,{module:'domain',element:String(element).split('.')[0]},[]);
 const selected=receipt.fields.find((r:any)=>r.field.element===element);if(!selected)throw new Error('Selected original member absent');
 probes.push({pack,sourceSha256:createHash('sha256').update(bytes).digest('hex'),identity,operation:receipt.operation,source:receipt.source,recordIdentity:receipt.identity,values:receipt.values,documentValidation:receipt.documentValidation,globalValidation:receipt.validation,selectedOriginalField:selected});
 const absence=api.resolveCoreDefault(source,identity,{state:'missing'});if(absence.result.state!=='missing'||absence.applied!==false)throw new Error('Original missing state changed');probes.push({pack,identity,originalDefaultReceipt:absence});
}
const requiredNull=probes.find((r:any)=>r.identity.element==='occurrences.count'&&r.value===null);if((requiredNull as any).validation.valid!==false)throw new Error('Required original null must be invalid');
for(const field of ['samples.parent_id','observations.value','effort.amount']){const nullProbe=probes.find((r:any)=>r.identity.element===field&&r.value===null);if((nullProbe as any).validation.valid!==true)throw new Error('Original optional null not valid');const absent=probes.find((r:any)=>r.identity.element===field&&r.selectedOriginalField);if((absent as any).selectedOriginalField.state!=='absent'||(absent as any).selectedOriginalField.validation.valid!==true)throw new Error('Original optional absence not retained');}
writeFileSync(output,JSON.stringify({format:'weft-original-public-presence-probes/0.1',umfRevision,qualification:'Original source public UMF Field/Record/default receipts; global empty-Record validity is retained, never claimed admitted. Missing, present-null and present scalar remain distinct.',probes},null,2)+'\n',{flag:'wx'});
