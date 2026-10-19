// SPDX-License-Identifier: MIT OR Apache-2.0
/** Real Chromium and actual WASM object/lower/check parity. No database admission.
 * @covers US-009-AC3 @covers US-009-AC4
 */
import {createHash} from 'node:crypto';
const [web,packetPath,playwrightPath,output,mode="bounded"]=Bun.argv.slice(2);
if(!web||!packetPath||!playwrightPath||!output)throw Error('web, native packet, Playwright and output required');
const {chromium}=await import(playwrightPath),packet=JSON.parse(await Bun.file(packetPath).text());
const js=await Bun.file(web+'/weft_wasm.js').arrayBuffer(),wasm=await Bun.file(web+'/weft_wasm_bg.wasm').arrayBuffer();
const browser=await chromium.launch({headless:true});let receipt;
try{
 const page=await browser.newPage(),origin='http://weft.bounded.test',errors:string[]=[],requests:string[]=[];
 page.on('pageerror',(e:any)=>errors.push(String(e)));page.on('request',(r:any)=>requests.push(r.url()));
 await page.route('**/*',(route:any)=>{const u=new URL(route.request().url());if(u.origin!==origin)return route.abort();if(u.pathname==='/')return route.fulfill({contentType:'text/html',body:'<!doctype html><title>Bounded current String</title>'});if(u.pathname==='/weft.js')return route.fulfill({contentType:'text/javascript',body:Buffer.from(js)});if(u.pathname==='/weft_wasm_bg.wasm')return route.fulfill({contentType:'application/wasm',body:Buffer.from(wasm)});return route.abort();});
 await page.goto(origin);
 const controls=await page.evaluate(async({origin,vectors,mode}:any)=>{
  const api=await import(origin+'/weft.js');await api.default({module_or_path:origin+'/weft_wasm_bg.wasm'});
  const deny=()=>{throw Error('host-I/O')};globalThis.fetch=deny;globalThis.XMLHttpRequest=deny as any;globalThis.WebSocket=deny as any;globalThis.Worker=deny as any;
  const controls:any[]=[];function assert(value:any,label:string){if(!value)throw Error(label);controls.push({id:label,passed:true});}
  function refuses(label:string,fn:()=>unknown,prefix='WFT-'){try{fn();}catch(e){assert(String(e).startsWith(prefix)&&!String(e).includes('PRIVATE-SENTINEL'),label);return;}throw Error('unexpected-accept:'+label);}
  if(mode==='standard') {
   assert(typeof api.prepare_application_lineage==='undefined'&&typeof api.CurrentQueryPreparation==='undefined','feature-off-no-prepared-bounded-object-API');
   for(const v of vectors){assert(api.prepare_current_query_designations_json(v.request)===v.designation,v.id+'/standard-source-inspection-only');assert(JSON.parse(api.compile_json(v.request)).status!=='compiled',v.id+'/standard-no-activation');}
  }
  if(mode==='bounded') for(const v of vectors){
   const parent=api.prepare_application_lineage(v.request),child=parent.prepare_current_query(),other=api.prepare_application_lineage(v.request);
   assert(child.is_from(parent)&&!child.is_from(other),v.id+'/object-custody');other.free();
   assert(child.designation_json()===v.designation&&child.native_request_json()===v.nativeRequest,v.id+'/source-byte-parity');
   const suffix=v.backend+'_bounded_current_'+v.kind+'_candidate_json';
   assert(child['lower_'+suffix](v.binding)===v.artifact,v.id+'/artifact-byte-parity');
   assert(child['check_'+suffix](v.binding,v.input)===v.report,v.id+'/check-byte-parity');
   for(const [label,value] of [['null',null],['undefined',undefined],['number',42],['object',{}],['boxed-string',new String(v.binding)]] as const)refuses(v.id+'/nonprimitive/'+label,()=>child['lower_'+suffix](value),'WFT-CURRENT-QUERY-TEXT:');
   const badBinding=v.binding.replace('current_values','current_\ud800_values');refuses(v.id+'/raw-surrogate-binding',()=>child['lower_'+suffix](badBinding),'WFT-CURRENT-QUERY-TEXT:');
   const frame=JSON.parse(v.input);frame.capabilities=null;refuses(v.id+'/null-capability',()=>child['check_'+suffix](v.binding,JSON.stringify(frame)));
   assert(JSON.parse(api.compile_json(v.request)).status!=='compiled',v.id+'/ordinary-no-activation');
   parent.free();assert(child.designation_json()===v.designation,v.id+'/original-retained-after-parent-free');child.free();
  }
  const first=vectors[0],bad=first.request.replace('"value":"😀"','"value":"\ud800"');assert(bad!==first.request,'actual-raw-surrogate-source-injection');
  for(const name of (mode==='bounded'?['prepare_application_lineage','prepare_application_lineage_json','prepare_current_query_designations_json']:['prepare_application_lineage_json','prepare_current_query_designations_json'])){
   refuses(name+'/raw-high-surrogate',()=>api[name](bad),'WFT-CURRENT-QUERY-TEXT:');
   refuses(name+'/raw-low-surrogate',()=>api[name](bad.replace('\ud800','\udc00')),'WFT-CURRENT-QUERY-TEXT:');
   refuses(name+'/boxed-string',()=>api[name](new String(first.request)),'WFT-CURRENT-QUERY-TEXT:');
   refuses(name+'/ASCII-frame-budget',()=>api[name](' '.repeat(4_000_001)),'WFT-CURRENT-QUERY-TEXT:');
  }
  refuses('UTF8-byte-budget-before-conversion',()=>api.prepare_application_lineage_json('😀'.repeat(1_000_001)),'WFT-CURRENT-QUERY-TEXT:');
  for(const name of ['prepare_security_mapping_json','inspect_security_mapping_subject_key_json']) {
   const refused=JSON.parse(api[name](bad,'{}'));assert(refused.status==='refused'&&refused.securityReady===false&&!('handoff'in refused),name+'/original-Unicode-JSON-refusal');
   assert(api[name](' '.repeat(4_000_001),'{}')===api[name]('','{}'),name+'/legacy-oversize-JSON-refusal-channel');
  }
  if(new Set(controls.map(c=>c.id)).size!==controls.length)throw Error('duplicate-control-id');
  return controls;
 },{origin,vectors:packet.vectors,mode});
 if(errors.length||requests.some(r=>!r.startsWith(origin+'/')))throw Error('browser-host-boundary');
 receipt={mode,scope:'Actual Chromium initialized WASM, typed object custody and complete native compiler/checker byte parity; no database/native caller/authority/release or P03 acceptance.',browser:browser.version(),nativePacketSha256:createHash('sha256').update(await Bun.file(packetPath).text()).digest('hex'),jsSha256:createHash('sha256').update(new Uint8Array(js)).digest('hex'),wasmSha256:createHash('sha256').update(new Uint8Array(wasm)).digest('hex'),controls,requests,pageErrors:errors};
 await Bun.write(output,JSON.stringify(receipt,null,2)+'\n');console.log(JSON.stringify({controls:controls.length,browser:browser.version()}));
}finally{await browser.close();}
