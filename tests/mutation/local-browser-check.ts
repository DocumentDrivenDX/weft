/** Basic actual WASM local UPDATE compilation; no provider or execution claim. */
import {createRequire} from 'node:module';
import {join} from 'node:path';
const {chromium}=createRequire(Bun.argv[2])('playwright');
const directory=Bun.argv[3];
if(!directory)throw Error('Playwright package and wasm-bindgen web output required');
const fixture=await Bun.file(new URL('./fixtures/local-products-input.json',import.meta.url)).json();
const server=Bun.serve({hostname:'127.0.0.1',port:0,fetch(request){
 const path=new URL(request.url).pathname;
 if(path==='/')return new Response('<!doctype html><title>Weft local UPDATE</title>');
 if(path==='/weft_wasm.js'||path==='/weft_wasm_bg.wasm')return new Response(Bun.file(join(directory,path.slice(1))));
 return new Response('Missing',{status:404});
}});
const browser=await chromium.launch({headless:true});
try{
 const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.port}`);
 const result=await page.evaluate(async(fixture:any)=>{
  const url='/weft_wasm.js';const wasm=await import(url);await wasm.default();
  const compiler=new wasm.MutationCompiler(JSON.stringify(fixture.configuration));
  try{
   const response=JSON.parse(compiler.compile_json(JSON.stringify(fixture.request)));
   if(response.status!=='compiled')throw Error(JSON.stringify(response));
   const plan=JSON.parse(response.plan.utf8);
   if(plan.action.invocation.id!=='umf.actions.local-tx'||plan.inputs.assignments[0].value.string!=='Foo')throw Error('Original local plan mismatch');
   const hash=Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',new TextEncoder().encode(response.plan.utf8))),x=>x.toString(16).padStart(2,'0')).join('');
   if(hash!==response.plan.sha256)throw Error('Original plan hash mismatch');
   return {status:response.status,planSha256:hash,invocation:plan.action.invocation,nodeGlobalsAbsent:!('process' in globalThis)&&!('Buffer' in globalThis)};
  }finally{compiler.free();}
 },fixture);
 if(!result.nodeGlobalsAbsent)throw Error('Browser globals mismatch');
 const receipt={scope:'Original Rust WASM local UPDATE plan compilation only',chromium:browser.version(),...result,wasmSha256:new Bun.CryptoHasher('sha256').update(await Bun.file(join(directory,'weft_wasm_bg.wasm')).bytes()).digest('hex'),physicalLowering:false,executionAuthority:false,effects:false};
 const text=JSON.stringify(receipt,null,2)+'\n';if(Bun.argv[4])await Bun.write(Bun.argv[4],text);console.log(text);
}finally{await browser.close();server.stop(true);}
