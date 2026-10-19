// SPDX-License-Identifier: MIT OR Apache-2.0
// Real Chromium runs the same Rust compiler; no host query or execution shim.
import {readFile,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {resolve} from 'node:path';
const [corpusPath,artifactDirectory,reportPath]=process.argv.slice(2);
if(!corpusPath||!artifactDirectory||!reportPath)throw Error('Expected corpus, wasm-bindgen directory and report paths');
const playwright=process.env.WEFT_PLAYWRIGHT_MODULE||'playwright';
const {chromium}=await import(playwright);
const cases=JSON.parse(await readFile(corpusPath,'utf8'));
const js=await readFile(resolve(artifactDirectory,'weft_wasm.js'));
const wasm=await readFile(resolve(artifactDirectory,'weft_wasm_bg.wasm'));
const browser=await chromium.launch({headless:true,...(process.env.WEFT_CHROMIUM_EXECUTABLE?{executablePath:process.env.WEFT_CHROMIUM_EXECUTABLE}:{})});
try{
 const page=await browser.newPage();const errors=[];
 page.on('pageerror',e=>errors.push(String(e)));
 await page.route('**/*',async route=>{
  const pathname=new URL(route.request().url()).pathname;
  if(pathname==='/')return route.fulfill({contentType:'text/html',body:'<!doctype html><title>Declared table operations</title>'});
  if(pathname==='/weft.js')return route.fulfill({contentType:'text/javascript',body:js});
  if(pathname==='/weft.wasm')return route.fulfill({contentType:'application/wasm',body:wasm});
  return route.abort();
 });
 await page.goto('http://weft.operation.test');
 const observed=await page.evaluate(async cases=>{
  const api=await import('/weft.js');
  const bytes=await(await fetch('/weft.wasm')).arrayBuffer();
  const module=await WebAssembly.compile(bytes);
  await api.default({module_or_path:module});
  const denied=()=>{throw Error('Unexpected host IO during compile')};
  globalThis.fetch=denied;globalThis.XMLHttpRequest=denied;globalThis.WebSocket=denied;globalThis.Worker=denied;
  const actual=cases.map(c=>{
   const result=JSON.parse(api.compile_table_operation_json(c.request));
   const checked=result.status==='compiled'?JSON.parse(api.check_table_operation_json(JSON.stringify(result))):null;
   return {name:c.name,result,checked};
  });
  const program=actual.find(c=>c.result.status==='compiled').result;
  const forged=structuredClone(program);forged.sql='SELECT forged';
  const tamper=JSON.parse(api.check_table_operation_json(JSON.stringify(forged)));
  let surrogates=0;
  for(const bad of [String.fromCharCode(0xd800),String.fromCharCode(0xdc00)]){
   let refused=false;try{api.compile_table_operation_json(bad);}catch{refused=true;}
   if(!refused)throw Error('Invalid Unicode transport accepted');surrogates++;
  }
  return {actual,tamper,surrogates,imports:WebAssembly.Module.imports(module),nodeProcess:typeof globalThis.process!=='undefined',require:typeof globalThis.require!=='undefined'};
 },cases);
 if(errors.length)throw Error(errors.join('\n'));
 if(observed.nodeProcess||observed.require)throw Error('Unexpected Node runtime');
 const canonical=value=>Array.isArray(value)?value.map(canonical):value&&typeof value==='object'?Object.fromEntries(Object.keys(value).sort().map(k=>[k,canonical(value[k])])):value;
 const canonicalJson=value=>JSON.stringify(canonical(value));
 for(let i=0;i<cases.length;i++){
  if(canonicalJson(observed.actual[i].result)!==canonicalJson(cases[i].expected))throw Error('Native/WASM result differs: '+cases[i].name);
  if(cases[i].expected.status==='compiled'&&observed.actual[i].checked.status!=='checked')throw Error('WASM program verification failed');
 }
 if(observed.tamper.status!=='blocked')throw Error('WASM program mutation accepted');
 const sha=bytes=>createHash('sha256').update(bytes).digest('hex');
 const report={scope:'real Chromium shared compiler parity; no native database authority',browser:browser.version(),cases:cases.map(c=>({name:c.name,status:c.expected.status,nativeWasmEqual:true,requestSha256:sha(c.request)})),tamperControls:1,invalidUnicodeTransportControls:observed.surrogates,wasmSha256:sha(wasm),javascriptSha256:sha(js),imports:observed.imports,nodeProcess:observed.nodeProcess,require:observed.require,nativeExecutionQualified:false};
 await writeFile(reportPath,JSON.stringify(report,null,2)+'\n');
 console.log(JSON.stringify({cases:cases.length,tamperControls:1,unicodeControls:observed.surrogates,browser:report.browser}));
}finally{await browser.close();}
