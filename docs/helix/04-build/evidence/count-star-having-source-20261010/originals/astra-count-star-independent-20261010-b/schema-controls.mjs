import Ajv2020 from '/private/tmp/ashlar-weft-distribution-d2d/node_modules/ajv/dist/2020.js';
import fs from 'node:fs';
const root='/private/tmp/ashlar-weft-distribution-d2d/docs/helix/02-design/contracts/';
const load=(name)=>JSON.parse(fs.readFileSync(root+name));
const ajv=new Ajv2020({strict:false,allErrors:true});
for(const v of ['0.4','0.4.1']) ajv.addSchema(load(`logical-plan-v${v}.schema.json`));
const responses=Object.fromEntries(['0.4','0.4.1'].map(v=>[v,ajv.compile(load(`compile-response-v${v}.schema.json`))]));
const request=ajv.compile(load('compile-request-v0.4.1.schema.json'));
const data=JSON.parse(fs.readFileSync('/private/tmp/astra-count-star-independent-20261010-b/run.stdout.json'));
const checks=[];
function check(id, fn, value, expected) {if(Boolean(fn(value))!==expected)throw Error(id+': '+JSON.stringify(fn.errors));checks.push({id,expected,passed:true});}
for(const item of data.checks.filter(x=>x.response)) {
 const v=item.response.interfaceVersion==='weft-compile/0.4.1'?'0.4.1':'0.4';
 check(item.case+'-response',responses[v],item.response,true);
 if(item.response.status==='compiled') {
  check(item.case+'-plan',ajv.getSchema(root+'unused')??ajv.getSchema('https://github.com/DocumentDrivenDX/weft/raw/main/docs/helix/02-design/contracts/logical-plan-v0.4.1.schema.json'),item.response.logicalPlan,true);
 }
}
const item=data.checks[0];check('original041-request',request,item.request,true);
for(const [id,change]of[
 ['extra-target-interface',r=>{r.target.interfaceVersion='weft-backend/0.3.0'}],
 ['mixed-dialect',r=>{r.dialect='weft-sql/0.4.0'}],
 ['unknown-setting',r=>{r.options.countStarEnabled=true}],
]){const r=structuredClone(item.request);change(r);check(id,request,r,false);}
const planValidate=ajv.getSchema('https://github.com/DocumentDrivenDX/weft/raw/main/docs/helix/02-design/contracts/logical-plan-v0.4.1.schema.json');
for(const [id,change]of[
 ['mixed-ir',p=>{p.irVersion='weft-ir/0.4.0'}],
 ['count-unknown-member',p=>{p.having[0].count.argument={}}],
 ['count-nullable',p=>{p.having[0].count.type.nullable=true}],
 ['count-unknown-operation',p=>{p.having[0].count.op='countUnbound'}],
]){const p=structuredClone(item.response.logicalPlan);change(p);check(id,planValidate,p,false);}
console.log(JSON.stringify({scope:'offline public request/response/plan schema controls; no native execution',checks},null,2));
