import {chromium} from '/Users/erik/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules/playwright/index.mjs';
import {createHash} from 'node:crypto';
const native=JSON.parse(await Bun.file('/private/tmp/subject-consumer-python.json').text());
const folder='/private/tmp/subject-consumer-wasm';
const server=Bun.serve({hostname:'127.0.0.1',port:0,fetch(req){const name=new URL(req.url).pathname.slice(1);return ['weft_wasm.js','weft_wasm_bg.wasm'].includes(name)?new Response(Bun.file(folder+'/'+name),{headers:{'content-type':name.endsWith('wasm')?'application/wasm':'text/javascript'}}):new Response('<!doctype html>');}});
let browser;
try{
 browser=await chromium.launch({headless:true});const page=await browser.newPage();const origin=`http://127.0.0.1:${server.port}`;let external=0;
 await page.route('**/*',route=>{if(new URL(route.request().url()).origin!==origin){external++;return route.abort();}return route.continue();});
 await page.goto(origin);
 const results=await page.evaluate(async(cases)=>{const path='/weft_wasm.js';const owner=await import(path);await owner.default();return cases.map(c=>({id:c.id,response:owner.inspect_security_mapping_subject_key_json(c.request,c.selection)}));},native.cases);
 if(external||results.length!==native.cases.length||results.some((r,i)=>r.response!==native.cases[i].response))throw Error('Actual browser/Python byte correspondence failed');
 const hashes={};for(const f of ['weft_wasm.js','weft_wasm_bg.wasm'])hashes[f]=createHash('sha256').update(new Uint8Array(await Bun.file(folder+'/'+f).arrayBuffer())).digest('hex');
 await Bun.write('/private/tmp/subject-consumer-browser.json',JSON.stringify({status:'passed',chromium:browser.version(),cases:results,artifactHashes:hashes,externalRequests:external,scope:'Actual browser WASM source inspector;14 byte-identical Python responses; no database/native admission'},null,2)+'\n');
 console.log(JSON.stringify({passed:results.length,chromium:browser.version()}));
}finally{await browser?.close();server.stop(true);}
