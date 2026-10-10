/** Actual initialized browser WASM transport parity; no engine execution.
 * The enclosing bounded launcher owns deadlines, environment and artifact custody.
 * lstat/readFile is cooperative regular-file custody, not adversarial no-follow fencing.
 */
import {readFile,writeFile,lstat} from 'node:fs/promises';
import {isAbsolute} from 'node:path';
import {pathToFileURL} from 'node:url';
import {createHash} from 'node:crypto';
const args=process.argv.slice(2);
if(args.length!==6||args.some(p=>!isAbsolute(p)))throw new Error('absolute-config');
const [casesPath,wasmJs,wasmBinary,wrapperJs,playwrightPath,output]=args;
const maximum=4*1024*1024;
async function read(path,limit=maximum){const info=await lstat(path);if(!info.isFile()||info.size>limit)throw new Error('regular-bounded-input');const raw=await readFile(path);if(raw.length>limit)throw new Error('input-bound');return raw;}
const [caseBytes,js,wasm,wrapper]=await Promise.all([read(casesPath,32*1024*1024),read(wasmJs),read(wasmBinary,16*1024*1024),read(wrapperJs)]);
const rows=JSON.parse(caseBytes);
if(!Array.isArray(rows)||!rows.length||rows.length>128)throw new Error('case-inventory');
for(const row of rows)if(typeof row.request!=='string'||typeof row.expected!=='string'||Buffer.byteLength(row.request)>16*1024*1024||Buffer.byteLength(row.expected)>maximum)throw new Error('case-bound');
const {chromium}=await import(pathToFileURL(playwrightPath).href);
const browser=await chromium.launch({headless:true,timeout:30000});
let primary;
try{
 const page=await browser.newPage();const errors=[];const requests=[];
 page.on('pageerror',e=>errors.push(String(e)));page.on('request',r=>requests.push(r.url()));
 const origin='http://weft.parity.test';
 await page.route('**/*',async route=>{
  const url=new URL(route.request().url());
  if(url.origin!==origin)return route.abort();
  if(url.pathname==='/')return route.fulfill({contentType:'text/html',body:'<!doctype html><title>PathsKeys transport parity</title>'});
  const selected={'/weft.js':js,'/weft_bg.wasm':wasm,'/wrapper.js':wrapper};
  if(!(url.pathname in selected))return route.abort();
  return route.fulfill({contentType:url.pathname.endsWith('.wasm')?'application/wasm':'text/javascript',body:selected[url.pathname]});
 });
 await page.goto(origin,{timeout:30000});
 const result=await page.evaluate(async({rows,origin})=>{
  const api=await import(origin+'/weft.js');await api.default({module_or_path:origin+'/weft_bg.wasm'});
  const {bindCompiler}=await import(origin+'/wrapper.js');
  const compiler=bindCompiler({compile_json:request=>api.compile_paths_keys_json(request)});
  const deny=()=>{throw new Error('compile-host-io')};globalThis.fetch=deny;globalThis.XMLHttpRequest=deny;globalThis.WebSocket=deny;globalThis.Worker=deny;
  const responses=rows.map(row=>{const response=compiler.compileJson(row.request);if(response!==row.expected||compiler.compileJson(row.request)!==response)throw new Error('exact-response-parity');return {id:row.id,response};});
  const selected=rows.find(row=>JSON.parse(row.request).interfaceVersion==='weft-compile/0.4.1'&&JSON.parse(row.expected).status==='compiled');if(!selected)throw new Error('new-profile-positive-required');
  const generic=api.compile_json(selected.request);const value=JSON.parse(generic);if(value.status!=='blocked'||!value.diagnostics.some(d=>d.code==='WFT-VERSION'))throw new Error('generic-entrypoint-profile');
  for(const bad of [null,{},1,String.fromCharCode(0xd800),String.fromCharCode(0xdc00)]){let refused=false;try{compiler.compileJson(bad);}catch(e){if(!(e instanceof TypeError))throw e;refused=true;}if(!refused)throw new Error('scalar-transport');}
  const trap=new Uint8Array([0,97,115,109,1,0,0,0,1,4,1,96,0,0,3,2,1,0,7,8,1,4,116,114,97,112,0,0,10,5,1,3,0,0,11]);
  const instance=await WebAssembly.instantiate(trap);let calls=0;
  const poisoned=bindCompiler({compile_json(){calls++;instance.instance.exports.trap();return '';}});
  const failure=poisoned.compileJson('{}');if(JSON.parse(failure).diagnostics[0].code!=='WFT-BACKEND-FAILURE'||poisoned.compileJson('{}')!==failure||calls!==1)throw new Error('trap-retirement');
  return {responses,generic,trapRetired:true};
 },{rows,origin});
 if(errors.length||requests.some(url=>!url.startsWith(origin+'/')))throw new Error('browser-custody');
 const receipt={format:'weft-browser-paths-embedding-parity/0.1',browserVersion:browser.version(),cases:result.responses.map(row=>({...row,responseSha256:createHash('sha256').update(row.response).digest('hex')})),genericResponse:result.generic,trapRetired:result.trapRetired,requests,qualification:'Actual real-browser WASM transport only; no native SQL or host obligations.'};
 await writeFile(output,JSON.stringify(receipt,null,2)+'\n',{flag:'wx'});
 process.stdout.write(JSON.stringify({state:'browser-wasm-parity-passed',cases:rows.length})+'\n');
}catch(error){primary=error;}
finally{try{await browser.close();}catch(error){if(primary)primary.cleanupFailed=true;else primary=error;}}
if(primary)throw primary;
