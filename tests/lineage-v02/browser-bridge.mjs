// SPDX-License-Identifier: MIT OR Apache-2.0
/** Actual initialized Chromium WASM; source-only, no native authority.
 * @covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
 */
import {readFile,writeFile,readdir} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createHash} from 'node:crypto';
import {chromium} from 'playwright';
const [root,web,output]=process.argv.slice(2);if(!root||!web||!output)throw Error('absolute root/web/output required');
const cases=[];
for(const folder of ['application-lineage','lineage-v02'])for(const name of (await readdir(resolve(root,'tests',folder,'fixtures'))).sort().filter(x=>x.endsWith('.request.json'))){
 const path=resolve(root,'tests',folder,'fixtures',name);let expected;try{expected=await readFile(path.replace('.request.json','.report.json'),'utf8')}catch{continue}
 cases.push({id:folder+':'+name,request:await readFile(path,'utf8'),expected});
}
const bad=JSON.parse(cases.find(c=>c.id==='lineage-v02:distinct-required.request.json').request);bad.readRequirements={};
for(const raw of ['{}','[]','{"interfaceVersion":1,"interfaceVersion":2}',JSON.stringify(bad),' '.repeat(4_000_001)])cases.push({id:'refusal:'+cases.length,request:raw,expected:null});
const js=await readFile(resolve(web,'weft_wasm.js'));const wasm=await readFile(resolve(web,'weft_wasm_bg.wasm'));
const browser=await chromium.launch({headless:true,timeout:30000});let result;
try{
 const page=await browser.newPage();const errors=[],requests=[];page.on('pageerror',e=>errors.push(String(e)));page.on('request',r=>requests.push(r.url()));
 const origin='http://weft.lineage.test';await page.route('**/*',async route=>{const u=new URL(route.request().url());if(u.origin!==origin)return route.abort();if(u.pathname==='/')return route.fulfill({contentType:'text/html',body:'<!doctype html><title>Weft lineage</title>'});if(u.pathname==='/weft.js')return route.fulfill({contentType:'text/javascript',body:js});if(u.pathname==='/weft_wasm_bg.wasm')return route.fulfill({contentType:'application/wasm',body:wasm});return route.abort()});
 await page.goto(origin,{timeout:30000});
 result=await page.evaluate(async({cases,origin})=>{
  const api=await import(origin+'/weft.js');await api.default({module_or_path:origin+'/weft_wasm_bg.wasm'});
  const deny=()=>{throw Error('compiler attempted host I/O')};globalThis.fetch=deny;globalThis.XMLHttpRequest=deny;globalThis.WebSocket=deny;globalThis.Worker=deny;
  const observations=[];
  for(const c of cases){if(c.expected===null){let refused=false;try{api.prepare_application_lineage_json(c.request)}catch(e){if(!String(e).startsWith('WFT-')||String(e).length>8192)throw Error('bad-refusal');refused=true}if(!refused)throw Error('unexpected-report');observations.push({id:c.id,refused:true})}
   else{const report=api.prepare_application_lineage_json(c.request);if(report!==c.expected||api.prepare_application_lineage_json(c.request)!==report)throw Error('byte-correspondence');const r=JSON.parse(report);if(r.authority!==false||r.securityReady!==false)throw Error('authority');observations.push({id:c.id,report})}}
  const ordinary=JSON.parse(api.compile_json(cases.find(c=>c.id==='lineage-v02:distinct-required.request.json').request));if(ordinary.status==='compiled'||'sql'in ordinary)throw Error('ordinary-activation');return observations;
 },{cases,origin});
 if(errors.length||requests.some(x=>!x.startsWith(origin+'/')))throw Error('browser-host-custody');
 const receipt={scope:'Real Chromium source-only WASM bridge; no data/current authority',browser:browser.version(),wasmSha256:createHash('sha256').update(wasm).digest('hex'),jsSha256:createHash('sha256').update(js).digest('hex'),checks:result.map(({report,...r})=>report?{...r,reportSha256:createHash('sha256').update(report).digest('hex')}:r),requests,pageErrors:errors};await writeFile(output,JSON.stringify(receipt,null,2)+'\n',{flag:'wx'});console.log(JSON.stringify({status:'passed',checks:result.length,browser:browser.version()}));
}finally{await browser.close()}
