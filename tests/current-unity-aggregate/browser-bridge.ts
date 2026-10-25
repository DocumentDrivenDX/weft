// SPDX-License-Identifier: MIT OR Apache-2.0
/** Actual Chromium/WASM parity for public generic Unity SUM/AVG dispatch.
 * This is source-object compilation only; it performs no database admission.
 */
import {createHash} from 'node:crypto';
const [web,packetPath,playwrightPath,output]=Bun.argv.slice(2);
if(!web||!packetPath||!playwrightPath||!output)throw Error('web, parity packet, Playwright and output required');
const {chromium}=await import(playwrightPath);
const packet=JSON.parse(await Bun.file(packetPath).text());
if(packet.profile!=='weft.unity-current-sum-avg-host-parity/0.1'||!Array.isArray(packet.vectors)||packet.vectors.length!==9)throw Error('closed SUM/AVG parity packet required');
const js=await Bun.file(web+'/weft_wasm.js').arrayBuffer();
const wasm=await Bun.file(web+'/weft_wasm_bg.wasm').arrayBuffer();
const browser=await chromium.launch({headless:true});
let receipt;
try{
 const page=await browser.newPage(),origin='http://weft.unity-sum-avg.test',errors:string[]=[],requests:string[]=[];
 page.on('pageerror',(error:any)=>errors.push(String(error)));
 page.on('request',(request:any)=>requests.push(request.url()));
 await page.route('**/*',(route:any)=>{
  const url=new URL(route.request().url());
  if(url.origin!==origin)return route.abort();
  if(url.pathname==='/')return route.fulfill({contentType:'text/html',body:'<!doctype html><title>Unity aggregate parity</title>'});
  if(url.pathname==='/weft.js')return route.fulfill({contentType:'text/javascript',body:Buffer.from(js)});
  if(url.pathname==='/weft_wasm_bg.wasm')return route.fulfill({contentType:'application/wasm',body:Buffer.from(wasm)});
  return route.abort();
 });
 await page.goto(origin);
 const checks=await page.evaluate(async({origin,vectors}:any)=>{
  const api=await import(origin+'/weft.js');
  await api.default({module_or_path:origin+'/weft_wasm_bg.wasm'});
  const deny=()=>{throw Error('host-I/O')};
  globalThis.fetch=deny;globalThis.XMLHttpRequest=deny as any;globalThis.WebSocket=deny as any;globalThis.Worker=deny as any;
  const checks:any[]=[];
  for(const vector of vectors){
   const actual=api.compile_raw_read_json(vector.request);
   const expected=JSON.parse(vector.response);
   const observed=JSON.parse(actual);
   if(JSON.stringify(observed)!==JSON.stringify(expected))throw Error('public generic dispatcher mismatch:'+vector.id);
   checks.push({id:vector.id,matched:true,status:observed.status,version:observed.version});
  }
  return checks;
 },{origin,vectors:packet.vectors});
 if(errors.length||requests.some(url=>!url.startsWith(origin+'/')))throw Error('browser host boundary violation');
 receipt={profile:'weft.unity-current-sum-avg-wasm-parity/0.1',scope:'Actual Chromium initialized WASM through compile_raw_read_json; no database, native authority or release qualification.',browser:browser.version(),packetSha256:createHash('sha256').update(await Bun.file(packetPath).text()).digest('hex'),wasmSha256:createHash('sha256').update(new Uint8Array(wasm)).digest('hex'),jsSha256:createHash('sha256').update(new Uint8Array(js)).digest('hex'),checks,requests,pageErrors:errors,nativeDatabase:false,nativeAuthority:false};
 await Bun.write(output,JSON.stringify(receipt,null,2)+'\n');
 console.log(JSON.stringify({vectors:checks.length,browser:browser.version()}));
}finally{await browser.close();}
