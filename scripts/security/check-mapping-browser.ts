/** Actual browser WASM/CLI correspondence; no native execution admission.
 * @covers US-009-AC1
 * @covers US-009-AC2
 */
import {createHash} from 'node:crypto';
const [folder,requestPath,receiptPath,playwrightPath,nativeResponsePath,nativeReceiptPath]=Bun.argv.slice(2);
if(!folder||!requestPath||!receiptPath||!playwrightPath||!nativeResponsePath||!nativeReceiptPath)throw Error('WASM folder, request, receipt, Playwright module, native response and native receipt required');
const {chromium}=await import(playwrightPath);
const original=JSON.parse(await Bun.file(requestPath).text()),baseline=await Bun.file(nativeResponsePath).text();
const nativeReceipt=JSON.parse(await Bun.file(nativeReceiptPath).text());
const cases:any[]=[{id:'five-operators',value:structuredClone(original),status:'prepared'}];
for(const key of ['bindingJson','policyJson','ontologyJson','backendId','backendVersion','targetProfile']){const value=structuredClone(original);value[key]+=' ';cases.push({id:key+'-drift',value,status:'refused'});}
for(const key of ['originalAction','handoff']){const value=structuredClone(original);value[key]='caller-forged';cases.push({id:key+'-forgery',value,status:'refused'});}
{const value=structuredClone(original);value.parameters={p:{family:'integer',value:100}};cases.push({id:'numeric-carrier',value,status:'refused'});}
{const value=structuredClone(original),profile=JSON.parse(value.queryProfileJson);profile.bindings=[];value.queryProfileJson=JSON.stringify(profile);cases.push({id:'missing-bindings',value,status:'refused'});}
{const value=structuredClone(original);value.queryProfileJson+=' ';cases.push({id:'profile-bytes-not-authentication',value,status:'prepared'});}
{const value=structuredClone(original);value.parameters={p:{family:'string',value:'x'.repeat(65536)}};value.sql='SELECT r.resourceId FROM Resource r WHERE '+Array(300).fill('r.resourceId=:p').join(' AND ');cases.push({id:'repeated-parameter-budget',value,status:'refused'});}
{const value=structuredClone(original);value.readProfile={version:'weft-truss-entity-read/0.1.0',subset:'complete-entity'};value.sql='SELECT r.* FROM Resource r';cases.push({id:'complete-entity-not-admitted',value,status:'refused'});}
{const value=structuredClone(original),document=JSON.parse(value.modules[0].documentJson);document.modules[0].relationships=[{id:'resource-peer',name:'peer',source:[{module:'m',element:'Resource'}],target:[{module:'m',element:'Resource',key:'pk'}],sourceMultiplicity:{min:0,max:'*'},targetMultiplicity:{min:0,max:'*'},targetLifecycle:'independent',directed:true}];value.modules[0].documentJson=JSON.stringify(document);value.modules[0].pin.sha256=createHash('sha256').update(value.modules[0].documentJson).digest('hex');const profile=JSON.parse(value.queryProfileJson);profile.modelPins=[value.modules[0].pin];value.queryProfileJson=JSON.stringify(profile);value.sql='SELECT r.resourceId FROM Resource r';value.parameters={};cases.push({id:'relationship-model-baseline',value:structuredClone(value),status:'prepared'});value.sql='SELECT r.resourceId FROM Resource r WHERE r.peer=:p';value.parameters={p:{family:'string',value:'r1'}};cases.push({id:'relationship-shorthand-not-admitted',value,status:'refused'});}
const files=['weft_wasm.js','weft_wasm_bg.wasm'];
const sourceDigests=Object.fromEntries(await Promise.all([import.meta.path,requestPath,nativeReceiptPath,nativeResponsePath,...files.map(f=>folder+'/'+f)].map(async p=>[p,createHash('sha256').update(new Uint8Array(await Bun.file(p).arrayBuffer())).digest('hex')])));
const server=Bun.serve({hostname:'127.0.0.1',port:0,fetch(req){const name=new URL(req.url).pathname.slice(1);if(files.includes(name))return new Response(Bun.file(folder+'/'+name),{headers:{'content-type':name.endsWith('.wasm')?'application/wasm':'text/javascript'}});return new Response('<!doctype html><title>Source-bound mapping WASM</title>');}});
let browser;
try{
 browser=await chromium.launch({headless:true});const page=await browser.newPage(),origin=`http://127.0.0.1:${server.port}`;let external=0;
 await page.route('**/*',(route:any)=>{if(new URL(route.request().url()).origin!==origin){external++;return route.abort();}return route.continue();});
 await page.goto(origin);
 const results=await page.evaluate(async({cases}:any)=>{const path='/weft_wasm.js',wasm=await import(path);await wasm.default();return cases.map((c:any)=>({id:c.id,raw:wasm.prepare_security_mapping_json(JSON.stringify(c.value))}));},{cases});
 if(external||results[0].raw!==baseline.trimEnd())throw Error('Browser/native correspondence or network boundary failed');
 const observations=results.map((r:any,i:number)=>{const value=JSON.parse(r.raw);if(value.status!==cases[i].status||value.securityReady!==false||(value.status==='refused'&&'handoff'in value))throw Error('Browser control failed: '+r.id);return{id:r.id,status:value.status,responseSha256:createHash('sha256').update(r.raw).digest('hex')};});
 const nativeHashes=new Map(nativeReceipt.observations.map((r:any)=>[r.id,r.responseSha256]));
 if(nativeHashes.size!==observations.length||observations.some((r:any)=>nativeHashes.get(r.id)!==r.responseSha256))throw Error('Full native/browser correspondence failed');
 await Bun.write(receiptPath,JSON.stringify({status:'passed',chromium:browser.version(),bun:Bun.version,sourceDigests,observations,externalRequests:external,baselineCorrespondence:'byte-identical to imported native Python/CLI',allResponseCorrespondence:'all16 response hashes identical',scope:'Actual Chromium WASM logical mapping preparation with opaque fixture sources. No native SQL/issuer/profile/caller/guard/release admission.'},null,2)+'\n');
 console.log(JSON.stringify({passed:observations.length,chromium:browser.version()}));
}finally{await browser?.close();server.stop(true);}
