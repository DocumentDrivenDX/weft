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
  const files={'/spike.js':'target/b002/web/weft_frontend_probe.js','/spike.wasm':'target/b002/web/weft_frontend_probe_bg.wasm'};
  if(!files[path]) return route.abort();
  return route.fulfill({contentType:path.endsWith('.wasm')?'application/wasm':'text/javascript',body:await readFile(resolve(root,files[path]))});
 });
 await page.goto(base);
 const corpus=JSON.parse(await readFile(resolve(root,'docs/helix/03-test/fixtures/cases.json'),'utf8'));
 const cases=corpus.map(c=>({id:c.id,request:JSON.stringify(c.request)}));
 const reports=JSON.parse(await readFile(resolve(root,'target/b002/reports.json'),'utf8'));
 const expected=reports.map(r=>r.raw);
 const results=await page.evaluate(async ({base,cases})=> {
  const bytes=await (await fetch(base+'/spike.wasm')).arrayBuffer();
  const module=new WebAssembly.Module(bytes);const imports=WebAssembly.Module.imports(module);
  const api=await import(base+'/spike.js');const instance=await api.default({module_or_path:bytes});
  const initialMemoryBytes=instance.memory.buffer.byteLength;
  const io=()=>{throw new Error('Host network IO attempted during compile')};
  globalThis.fetch=io;globalThis.XMLHttpRequest=io;globalThis.WebSocket=io;globalThis.Worker=io;
  const responses=cases.map(c=>api.resolve_json(c.request));
  return {responses,imports,initialMemoryBytes,finalMemoryBytes:instance.memory.buffer.byteLength,
   hasNodeProcess:typeof globalThis.process!=='undefined',hasRequire:typeof globalThis.require!=='undefined'};
 },{base,cases});
 if(failures.length) throw new Error(failures.join('\n'));
 if(results.hasNodeProcess||results.hasRequire) throw new Error('Node globals exposed');
 results.responses.forEach((r,i)=>{if(r!==expected[i])throw new Error('WASM/native byte mismatch: '+cases[i].id)});
 // Imports must be generated string/memory/error interop, never WASI/system/network.
 if(results.imports.some(i=>i.module!=='wbg'||!(i.name==='__wbindgen_init_externref_table'||i.name.startsWith('__wbg___wbindgen_throw_'))||i.kind!=='function')) throw new Error('Unexpected WASM host imports '+JSON.stringify(results.imports));
 if(requests.some(r=>!r.startsWith(base))) throw new Error('External network request');
 const wasm=await readFile(resolve(root,'target/b002/web/weft_frontend_probe_bg.wasm'));
 const summary={cases:cases.length,browser:await browser.version(),playwrightVersion,wasmBytes:wasm.length,wasmSha256:createHash('sha256').update(wasm).digest('hex'),jsGlueBytes:(await stat(resolve(root,'target/b002/web/weft_frontend_probe.js'))).size,imports:results.imports,initialMemoryBytes:results.initialMemoryBytes,finalMemoryBytes:results.finalMemoryBytes,networkRequests:requests,nodeGlobals:false,byteParity:true};
 await writeFile(resolve(root,'target/b002/browser-summary.json'),JSON.stringify(summary,null,2)+'\n');
 console.log(JSON.stringify(summary,null,2));
} finally {await browser.close();}
