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
  const files={'/spike.js':process.env.WEFT_PROBE_JS || 'target/b002/web/weft_frontend_probe.js','/spike.wasm':process.env.WEFT_PROBE_WASM || 'target/b002/web/weft_frontend_probe_bg.wasm','/wrapper.js':'target/b004/wrapper/index.js'};
  if(!files[path]) return route.abort();
  return route.fulfill({contentType:path.endsWith('.wasm')?'application/wasm':'text/javascript',body:await readFile(resolve(root,files[path]))});
 });
 await page.goto(base);
 const corpus=JSON.parse(await readFile(resolve(root,process.env.WEFT_FRONTEND_CORPUS || 'docs/helix/03-test/fixtures/cases.json'),'utf8'));
 const cases=corpus.map(c=>({id:c.id,request:JSON.stringify(c.request),configuration:c.configuration}));
 const reports=JSON.parse(await readFile(resolve(root,process.env.WEFT_FRONTEND_REPORTS || 'target/b002/reports.json'),'utf8'));
 const expected=reports.map(r=>r.raw);
 const configured=process.env.WEFT_PROBE_API==='compile_json_with_conformance_configuration';
 const batchSize=configured ? 8 : cases.length;
 const results=await page.evaluate(async ({base,cases,apiName})=> {
  const bytes=await (await fetch(base+'/spike.wasm')).arrayBuffer();
  const module=await WebAssembly.compile(bytes);const imports=WebAssembly.Module.imports(module);
  const api=await import(base+'/spike.js');const instance=await api.default({module_or_path:module});
  const initialMemoryBytes=instance.memory.buffer.byteLength;
  const io=()=>{throw new Error('Host network IO attempted during compile')};
  globalThis.fetch=io;globalThis.XMLHttpRequest=io;globalThis.WebSocket=io;globalThis.Worker=io;
  const {bindCompiler}=await import(base+'/wrapper.js');
  const compiler=bindCompiler(api);
  const compileBatch=batch=>batch.map(c=>apiName==='compile_json_with_conformance_configuration' ? api[apiName](c.request,c.configuration) : compiler.compileJson(c.request));
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
  return {responses,imports,initialMemoryBytes,finalMemoryBytes:instance.memory.buffer.byteLength,
   hasNodeProcess:typeof globalThis.process!=='undefined',hasRequire:typeof globalThis.require!=='undefined'};
 },{base,cases:cases.slice(0,batchSize),apiName:process.env.WEFT_PROBE_API || 'resolve_json'});
 for(let start=batchSize;start<cases.length;start+=batchSize) {
  const batch=await page.evaluate(cases=>({responses:globalThis.__weftConfiguredProbe.compileBatch(cases),memoryBytes:globalThis.__weftConfiguredProbe.memory.buffer.byteLength}),cases.slice(start,start+batchSize));
  results.responses.push(...batch.responses);results.finalMemoryBytes=batch.memoryBytes;
 }
 if(failures.length) throw new Error(failures.join('\n'));
 if(results.hasNodeProcess||results.hasRequire) throw new Error('Node globals exposed');
 results.responses.forEach((r,i)=>{if(r!==expected[i])throw new Error('WASM/native byte mismatch: '+cases[i].id)});
 // Imports must be generated string/memory/error interop, never WASI/system/network.
 if(results.imports.some(i=>i.module!=='wbg'||!(i.name==='__wbindgen_init_externref_table'||i.name.startsWith('__wbg___wbindgen_throw_'))||i.kind!=='function')) throw new Error('Unexpected WASM host imports '+JSON.stringify(results.imports));
 if(requests.some(r=>!r.startsWith(base))) throw new Error('External network request');
 const wasm=await readFile(resolve(root,process.env.WEFT_PROBE_WASM || 'target/b002/web/weft_frontend_probe_bg.wasm'));
 const summary={cases:cases.length,batchSize,browser:await browser.version(),playwrightVersion,wasmBytes:wasm.length,wasmSha256:createHash('sha256').update(wasm).digest('hex'),jsGlueBytes:(await stat(resolve(root,process.env.WEFT_PROBE_JS || 'target/b002/web/weft_frontend_probe.js'))).size,imports:results.imports,initialMemoryBytes:results.initialMemoryBytes,finalMemoryBytes:results.finalMemoryBytes,networkRequests:requests,nodeGlobals:false,byteParity:true};
 await writeFile(resolve(root,process.env.WEFT_FRONTEND_BROWSER_SUMMARY || 'target/b002/browser-summary.json'),JSON.stringify(summary,null,2)+'\n');
 console.log(JSON.stringify(summary,null,2));
} finally {await browser.close();}
