import Ajv2020 from 'ajv/dist/2020.js';
const root='docs/helix/02-design/contracts/';
const ajv=new Ajv2020({strict:false,allErrors:true});
for(const name of ['logical-plan-v0.3.schema.json','backend-manifest-v0.2.schema.json'])ajv.addSchema(await Bun.file(root+name).json());
const schema=await Bun.file(root+'compile-response-v0.3.schema.json').json();
const validate=ajv.compile(schema);
const path=Bun.argv[2];if(!path)throw new Error('Supply an actual positioned compiled artifact');
const original=await Bun.file(path).json();
const check=(id:string,value:unknown,expected:boolean)=>{if(Boolean(validate(value))!==expected)throw new Error(id+': '+JSON.stringify(validate.errors));};
check('actual-positioned-artifact',original,true);
if(!original.logicalPlan.requiredCapabilities.includes('project.positionedOutputs'))throw new Error('No positional admission in actual artifact');
const mutate=(id:string,edit:(v:any)=>void)=>{const v=structuredClone(original);edit(v);check(id,v,false);};
mutate('missing-carrier-name',v=>delete v.columns[0].carrierName);
mutate('empty-carrier-name',v=>v.columns[0].carrierName='');
mutate('float-position',v=>v.columns[0].position=1.5);
mutate('unknown-column-member',v=>v.columns[0].unknown=true);
mutate('no-positioned-admission',v=>v.logicalPlan.requiredCapabilities=v.logicalPlan.requiredCapabilities.filter((c:string)=>c!=='project.positionedOutputs'));
for(const version of ['0.1.0','0.2.0']) {
 const old=await Bun.file(root+`compile-response${version==='0.1.0'?'':'-v0.2'}.schema.json`).json();
 const item=old.oneOf[1].properties.columns.items;
 const oldColumn=ajv.compile(item);
 if(oldColumn(original.columns[0]))throw new Error('Older response schema accepted carrierName');
}
console.log('Actual 0.3 positioned artifact and five schema refusals pass; both older column schemas remain closed. Native ordered/result/custody checks remain host obligations.');
