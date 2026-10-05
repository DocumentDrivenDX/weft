import Ajv2020 from 'ajv/dist/2020.js';
const dir='docs/helix/02-design/contracts/';
const ajv=new Ajv2020({strict:false,allErrors:true});
const plan=ajv.compile(await Bun.file(dir+'logical-plan-v0.2.schema.json').json());
const request=ajv.compile(await Bun.file(dir+'compile-request-v0.2.schema.json').json());
const original=await Bun.file('docs/helix/03-test/fixtures/cases.json').json();
const cases=await Bun.file('tests/application/fixtures/cases.json').json();
for(const c of cases){const transport={...c.request,interfaceVersion:'weft-compile/0.2.0',target:original[0].request.target};if(!request(transport))throw new Error(c.id+': malformed request '+JSON.stringify(request.errors));}
const result=ajv.compile(await Bun.file(dir+'application-result-v0.2.schema.json').json());
const reports=await Bun.file('target/b002a/reports.json').json();let count=0;
for(const {id,response} of reports){if(response.status==='resolved'){if(!plan(response.logicalPlan))throw new Error(id+': '+JSON.stringify(plan.errors));count++;}}
for(const valid of [{state:'absent'},{state:'null'},{state:'value',value:[]},{state:'value',value:'18446744073709551615'},{items:[],truncated:false},{items:[['1'],['1']],truncated:true}])if(!result(valid))throw new Error('Carrier schema rejected '+JSON.stringify(valid));
for(const invalid of [{state:'absent',value:null},{state:'value'},{state:'null',value:null},{items:[1],truncated:false},{items:[['1']]},{items:[],truncated:'false'}])if(result(invalid))throw new Error('Carrier schema accepted malformed value '+JSON.stringify(invalid));
console.log(`Validated ${count} application plans and presence/related-key carrier shapes.`);
