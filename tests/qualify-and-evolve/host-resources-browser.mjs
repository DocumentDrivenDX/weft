// @covers US-006-AC4: exact raw refusal parity in real Chromium.
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {chromium} from 'playwright';
const cases=JSON.parse(await readFile(process.env.WEFT_RESOURCE_CASES,'utf8'));
const files={'/compiler.js':process.env.WEFT_RESOURCE_JS,'/compiler.wasm':process.env.WEFT_RESOURCE_WASM,'/wrapper.js':process.env.WEFT_RESOURCE_WRAPPER || 'packages/weft-browser/dist/index.js'};
const browser=await chromium.launch({headless:true,executablePath:process.env.WEFT_CHROMIUM_EXECUTABLE});
try{
 const page=await browser.newPage();
 await page.route('**/*',async route=>{
  const path=new URL(route.request().url()).pathname;
  if(path==='/')return route.fulfill({contentType:'text/html',body:'<!doctype html>'});
  if(!files[path])return route.abort();
  return route.fulfill({contentType:path.endsWith('.wasm')?'application/wasm':'text/javascript',body:await readFile(files[path])});
 });
 await page.goto('http://weft.resources.test/');
 await page.evaluate(async()=>{
  const api=await import('/compiler.js');await api.default({module_or_path:await(await fetch('/compiler.wasm')).arrayBuffer()});
  const {bindCompiler}=await import('/wrapper.js');globalThis.resourceCompiler=bindCompiler(api);
  const refuse=()=>{throw new Error('Unexpected compilation I/O')};globalThis.fetch=globalThis.XMLHttpRequest=globalThis.WebSocket=globalThis.Worker=refuse;
 });
 for(const c of cases){
  const response=await page.evaluate(raw=>globalThis.resourceCompiler.compileJson(raw),c.raw);
  if(response!==c.response)throw new Error('Resource parity mismatch '+c.id);
 }
 const sha=async p=>createHash('sha256').update(await readFile(p)).digest('hex');
 const summary={status:'passed',cases:cases.length,browser:await browser.version(),byteParity:true,wasmSha256:await sha(files['/compiler.wasm']),wrapperSha256:await sha(files['/wrapper.js']),casesAndResponsesSha256:await sha(process.env.WEFT_RESOURCE_CASES),scope:process.env.WEFT_RESOURCE_SCOPE || 'Raw resource/malicious-input host checks; retained test-third WASM, built transport, no database execution.'};
 await writeFile(process.env.WEFT_RESOURCE_SUMMARY,JSON.stringify(summary,null,2)+'\n');console.log(JSON.stringify(summary));
}finally{await browser.close()}
