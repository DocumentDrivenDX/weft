// SPDX-License-Identifier: MIT OR Apache-2.0
import { createReadStream, writeFileSync } from 'node:fs';
import { createInterface } from 'node:readline';
import { readFile, writeFile, stat } from 'node:fs/promises';
import { resolve, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { createHash } from 'node:crypto';
import { createRequire } from 'node:module';
const root=resolve(dirname(fileURLToPath(import.meta.url)),'../..');
const playwright = process.env.WEFT_PLAYWRIGHT_MODULE || 'playwright';
const packagePath=playwright==='playwright' ? createRequire(import.meta.url).resolve('playwright/package.json') : resolve(dirname(playwright),'package.json');
const playwrightVersion=JSON.parse(await readFile(packagePath,'utf8')).version;
if(playwrightVersion!=='1.62.1') throw new Error('Frontend probe requires Playwright 1.62.1');
const {chromium}=await import(playwright);
const browser=await chromium.launch({headless:true, ...(process.env.WEFT_CHROMIUM_EXECUTABLE ? {executablePath:process.env.WEFT_CHROMIUM_EXECUTABLE} : {})});
try {
 const page=await browser.newPage();const requests=[];
 const failures=[];page.on('pageerror',e=>failures.push(String(e)));page.on('request',r=>requests.push(r.url()));
 const base='http://weft.spike.test';
 await page.route('**/*',async route=> {
  const path=new URL(route.request().url()).pathname;
  if(path==='/') return route.fulfill({contentType:'text/html',body:'<!doctype html><title>Weft B-002</title>'});
  const files={'/spike.js':process.env.WEFT_PROBE_JS || 'target/b002/web/weft_frontend_probe.js','/spike.wasm':process.env.WEFT_PROBE_WASM || 'target/b002/web/weft_frontend_probe_bg.wasm','/wrapper.js':process.env.WEFT_BROWSER_WRAPPER || 'target/b004/wrapper/index.js'};
  if(!files[path]) return route.abort();
  return route.fulfill({contentType:path.endsWith('.wasm')?'application/wasm':'text/javascript',body:await readFile(resolve(root,files[path]))});
 });
 await page.goto(base);
 const corpusPath=resolve(root,process.env.WEFT_FRONTEND_CORPUS || 'docs/helix/03-test/fixtures/cases.json');
 const reports=JSON.parse(await readFile(resolve(root,process.env.WEFT_FRONTEND_REPORTS || 'target/b002/reports.json'),'utf8'));
 const expected=reports.map(r=>r.raw);
 const configured=process.env.WEFT_PROBE_API==='compile_json_with_conformance_configuration';
 const batchSize=configured ? 8 : 64;
 async function* inputCases() {
  if(corpusPath.endsWith('.jsonl')) {
   for await(const line of createInterface({input:createReadStream(corpusPath),crlfDelay:Infinity})) {
    if(line.trim()) yield JSON.parse(line);
   }
  } else {
   yield* JSON.parse(await readFile(corpusPath,'utf8'));
  }
 }
 const iterator=inputCases()[Symbol.asyncIterator]();
 async function nextBatch() {
  const batch=[];
  while(batch.length<batchSize) {
   const next=await iterator.next();if(next.done)break;
   const c=next.value;batch.push({id:c.id,request:JSON.stringify(c.request),configuration:c.configuration,pageSize:c.pageSize??null,after:c.after??null});
  }
  return batch;
 }
 let checked=0;
 const receipts=[];
 function verifyBatch(cases,responses) {
  if(responses.length!==cases.length)throw new Error('Browser response count differs');
  responses.forEach((r,i)=>{if(r!==expected[checked+i]){if(process.env.WEFT_BROWSER_MISMATCH)writeFileSync(process.env.WEFT_BROWSER_MISMATCH,JSON.stringify({id:cases[i].id,actual:r,expected:expected[checked+i]},null,2)+'\n');throw new Error('WASM/native byte mismatch: '+cases[i].id);}receipts.push({id:cases[i].id,requestSha256:createHash('sha256').update(cases[i].request).digest('hex'),actualSha256:createHash('sha256').update(r).digest('hex'),expectedSha256:createHash('sha256').update(expected[checked+i]).digest('hex')})});
  checked+=cases.length;
 }
 const firstBatch=await nextBatch();if(!firstBatch.length)throw new Error('Empty browser corpus');
 const results=await page.evaluate(async ({base,cases,apiName})=> {
  const bytes=await (await fetch(base+'/spike.wasm')).arrayBuffer();
  const module=await WebAssembly.compile(bytes);const imports=WebAssembly.Module.imports(module);
  const api=await import(base+'/spike.js');const instance=await api.default({module_or_path:module});
  const initialMemoryBytes=instance.memory.buffer.byteLength;
  const io=()=>{throw new Error('Host network IO attempted during compile')};
  globalThis.fetch=io;globalThis.XMLHttpRequest=io;globalThis.WebSocket=io;globalThis.Worker=io;
  const {bindCompiler,bindTrussPageCompiler}=await import(base+'/wrapper.js');
  // The fixed runtime compiler has its own explicit registration; generic
  // compile_json must continue to refuse this backend when not registered.
  const entitySelection=apiName==='compile_truss_runtime_entity_selection';
  const compiler=bindCompiler(apiName==='compile_truss_runtime_json'||entitySelection?{compile_json:api.compile_truss_runtime_json}:api);
  const pageCompiler=entitySelection?bindTrussPageCompiler(api):null;
  const compileBatch=batch=>batch.map(c=>entitySelection&&c.pageSize!==null?pageCompiler.compilePageJson(c.request,c.pageSize,c.after):apiName==='compile_json_with_conformance_configuration' ? api[apiName](c.request,c.configuration) : compiler.compileJson(c.request));
  globalThis.__weftConfiguredProbe={compileBatch,memory:instance.memory};
  const responses=compileBatch(cases);
  for(const bad of [null,{},1]) {try{compiler.compileJson(bad);throw new Error('Bad transport accepted')}catch(e){if(!(e instanceof TypeError))throw e}}
  for(const bad of [String.fromCharCode(0xd800),String.fromCharCode(0xdc00)]) {try{compiler.compileJson(bad);throw new Error('Surrogate accepted')}catch(e){if(!(e instanceof TypeError))throw e}}
  // Real WebAssembly unreachable instruction: host must retire a trapped instance.
  const trapBytes=new Uint8Array([0,97,115,109,1,0,0,0,1,4,1,96,0,0,3,2,1,0,7,8,1,4,116,114,97,112,0,0,10,5,1,3,0,0,11]);
  const trapInstance=await WebAssembly.instantiate(trapBytes);let calls=0;
  const trapped=bindCompiler({compile_json(){calls++;trapInstance.instance.exports.trap();return ''}});
  const failure=trapped.compileJson('{}');
  if(JSON.parse(failure).diagnostics[0].code!=='WFT-BACKEND-FAILURE'||trapped.compileJson('{}')!==failure||calls!==1)throw new Error('Trap recovery violated');
  if(entitySelection){
   for(const bad of [0,1000,1.5,NaN,Infinity,'1']){try{pageCompiler.compilePageJson('{}',bad);throw new Error('Invalid page bound accepted')}catch(e){if(!(e instanceof TypeError))throw e}}
   for(const bad of [false,{},[String.fromCharCode(0xd800)]]){try{pageCompiler.compilePageJson('{}',1,bad);throw new Error('Invalid page continuation accepted')}catch(e){if(!(e instanceof TypeError))throw e}}
   let boundaryCalls=0;const boundedPage=bindTrussPageCompiler({compile_truss_runtime_page_json(){boundaryCalls++;return '{}'}});
   boundedPage.compilePageJson('x'.repeat(16_777_216),999,null);
   boundedPage.compilePageJson('{}',1,['x'.repeat(65_536)]);
   const exactAfter=[...Array(15).fill('x'.repeat(65_536)),'x'.repeat(65_487)];
   if(JSON.stringify(exactAfter).length!==1_048_576)throw new Error('Invalid independent continuation boundary fixture');
   boundedPage.compilePageJson('{}',1,exactAfter);
   const admittedCalls=boundaryCalls;
   for(const [request,after] of [['x'.repeat(16_777_217),null],['{}',['x'.repeat(65_537)]],['{}',[...exactAfter.slice(0,-1),'x'.repeat(65_488)]],['{}',Array(1025).fill('')]]){
    try{boundedPage.compilePageJson(request,1,after);throw new Error('One-over page transport bound accepted')}catch(e){if(!(e instanceof RangeError)&&!(e instanceof TypeError))throw e}
   }
   if(boundaryCalls!==admittedCalls)throw new Error('Rejected page transport reached compiler');
   const direct=JSON.parse(api.compile_truss_runtime_page_json('{}',1.5,'null'));
   if(direct.status!=='blocked'||direct.diagnostics[0].code!=='WFT-INPUT')throw new Error('Raw WASM page bound truncated');
   for(const bad of [true,'1',null]){if(JSON.parse(api.compile_truss_runtime_page_json('{}',bad,'null')).status!=='blocked')throw new Error('Raw WASM page bound coerced');}
   let pageCalls=0;const trappedPage=bindTrussPageCompiler({compile_truss_runtime_page_json(){pageCalls++;trapInstance.instance.exports.trap();return ''}});
   const first=trappedPage.compilePageJson('{}',1);
   if(JSON.parse(first).diagnostics[0].code!=='WFT-BACKEND-FAILURE'||trappedPage.compilePageJson('{}',1)!==first||pageCalls!==1)throw new Error('Page trap recovery violated');
  }
  return {responses,imports,initialMemoryBytes,finalMemoryBytes:instance.memory.buffer.byteLength,
   hasNodeProcess:typeof globalThis.process!=='undefined',hasRequire:typeof globalThis.require!=='undefined'};
 },{base,cases:firstBatch,apiName:process.env.WEFT_PROBE_API || 'resolve_json'});
 verifyBatch(firstBatch,results.responses);
 while(true) {
  const cases=await nextBatch();if(!cases.length)break;
  const batch=await page.evaluate(cases=>({responses:globalThis.__weftConfiguredProbe.compileBatch(cases),memoryBytes:globalThis.__weftConfiguredProbe.memory.buffer.byteLength}),cases);
  verifyBatch(cases,batch.responses);results.finalMemoryBytes=batch.memoryBytes;
 }
 if(checked!==expected.length)throw new Error('Corpus/report count differs');
 if(failures.length) throw new Error(failures.join('\n'));
 if(results.hasNodeProcess||results.hasRequire) throw new Error('Node globals exposed');
 // Imports must be generated string/memory/error interop, never WASI/system/network.
 // wasm-bindgen0.2.105 adds this exact Ref(String)->Externref cast for browser-local Result errors.
 const stringInteropImports=new Set(['__wbg_length_a95b69f903b746c4','__wbg_charCodeAt_291b921e27833f8e','__wbg___wbindgen_is_string_fbb76cb2940daafd','__wbg___wbindgen_string_get_e4f06c90489ad01b',
  // Pinned primitive-only number admission for the typed page bound; raw bool,
  // string, fractional and nonfinite controls precede this import census.
  '__wbg___wbindgen_number_get_a20bf9b85341449d']);
 // Pinned wasm-bindgen string validation: length, UTF-16 code units, type check and string transfer.
 if(results.imports.some(i=>i.module!=='wbg'||!(i.name==='__wbindgen_init_externref_table'||i.name==='__wbindgen_cast_2241b6af4c4b2941'||i.name.startsWith('__wbg___wbindgen_throw_')||stringInteropImports.has(i.name))||i.kind!=='function')) throw new Error('Unexpected WASM host imports '+JSON.stringify(results.imports));
 if(requests.some(r=>!r.startsWith(base))) throw new Error('External network request');
 const wasm=await readFile(resolve(root,process.env.WEFT_PROBE_WASM || 'target/b002/web/weft_frontend_probe_bg.wasm'));
 const summary={cases:checked,batchSize,streamingCorpus:corpusPath.endsWith('.jsonl'),browser:await browser.version(),playwrightVersion,wasmBytes:wasm.length,wasmSha256:createHash('sha256').update(wasm).digest('hex'),jsGlueBytes:(await stat(resolve(root,process.env.WEFT_PROBE_JS || 'target/b002/web/weft_frontend_probe.js'))).size,imports:results.imports,initialMemoryBytes:results.initialMemoryBytes,finalMemoryBytes:results.finalMemoryBytes,networkRequests:requests,nodeGlobals:false,byteParity:true};
 if(process.env.WEFT_BROWSER_RECEIPTS)await writeFile(resolve(root,process.env.WEFT_BROWSER_RECEIPTS),JSON.stringify({summary,cases:receipts},null,2)+'\n');
 await writeFile(resolve(root,process.env.WEFT_FRONTEND_BROWSER_SUMMARY || 'target/b002/browser-summary.json'),JSON.stringify(summary,null,2)+'\n');
 console.log(JSON.stringify(summary,null,2));
} finally {await browser.close();}
