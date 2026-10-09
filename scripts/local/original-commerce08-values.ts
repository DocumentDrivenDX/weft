/** Explicit original graph lexical carriers, validated by actual public UMF. */
import {resolve,join} from 'node:path';
import {pathToFileURL} from 'node:url';
const [repoArg,outArg]=process.argv.slice(2);if(!repoArg||!outArg)throw Error('UMF_SOURCE FRESH_OUTPUT required');
const repo=resolve(repoArg),pin='c7c95e1c4ea5b72541f47fa0350ca467ff02f395';
for(const [args,expected]of [[['rev-parse','HEAD'],pin],[['status','--porcelain'],'']]as const){const r=Bun.spawnSync(['git','-C',repo,...args]);if(r.exitCode||new TextDecoder().decode(r.stdout).trim()!==expected)throw Error('Clean exact public UMF required');}
const load=(p:string)=>import(pathToFileURL(join(repo,'src',p+'.ts')).href);
const {readDocument}=await load('model/document'),{validateCoreRecordValues}=await load('model/record-values'),{validateCoreFieldValue}=await load('model/schema-properties');
const base=resolve(import.meta.dir,'../../tests/fixtures/original-commerce-0.8');
const sourceText=await Bun.file(join(base,'ontology.json')).text(),source=readDocument(sourceText,'json'),graph=await Bun.file(join(base,'graph.json')).json();
const elements=new Map(source.modules.flatMap((m:any)=>m.elements.map((e:any)=>[m.id+'\0'+e.id,e])));
const records=graph.objects.map((o:any)=>{const record:any=elements.get(o.type.module+'\0'+o.type.element);const values=record.members.map((ref:any)=>{const field:any=elements.get(ref.module+'\0'+ref.element),token=o.values[ref.element];if(typeof token!=='string')throw Error('Original lexical carrier required');const value=field.scalarType==='integer'?{integerToken:token}:field.scalarType==='decimal'?{decimalToken:token}:field.scalarType==='string'?{string:token}:undefined;if(!value)throw Error('Explicit unsupported source carrier');return {field:ref,state:'present',value};});const result=validateCoreRecordValues(source,{module:o.type.module,element:o.type.element},values);if(!result.validation.valid||result.fields.some((f:any)=>!f.validation.valid||!f.validation.complete))throw Error('Original public source-value validation failed');return {original:o,values,result};});
const controls=['10','9007199254740993','9'.repeat(38),'1'+'0'.repeat(38),'-'+'9'.repeat(38),'12.5'].map(token=>({token,result:validateCoreFieldValue(source,{module:'domain',element:'order_lines.quantity'},{integerToken:token})}));
for(let i=0;i<controls.length;i++)if(controls[i].result.valid!==(i<5))throw Error('Public source-validity classification differs');
const output=resolve(outArg);if(await Bun.file(output).exists())throw Error('Fresh receipt path required');await Bun.write(output,JSON.stringify({umfRevision:pin,sourceText,records,controls,qualification:'Original public source value receipts; separate context obligations remain. No native/backend representability claim.'})+'\n');
