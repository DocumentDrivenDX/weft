import {chromium} from 'playwright';
import {mkdirSync,writeFileSync,readFileSync} from 'node:fs';
import {createHash} from 'node:crypto';
const root=Bun.argv[2], casesDir=Bun.argv[3];
if(!root||!casesDir)throw new Error('Supply generated WASM directory and immutable request/artifact directory');
const requests=[];
for(const p of Array.from(new Bun.Glob('*-request.json').scanSync(casesDir)).sort())requests.push({name:p.replace('-request.json',''),request:readFileSync(casesDir+'/'+p,'utf8'),expected:JSON.parse(readFileSync(casesDir+'/'+p.replace('-request.json','-stdout.json'),'utf8'))});
const build=await Bun.build({entrypoints:['packages/weft-browser/src/index.ts'],target:'browser'});if(!build.success)throw new Error('wrapper build failed');
const wrapper=await build.outputs[0].text();writeFileSync(root+'/wrapper.js',wrapper);writeFileSync(root+'/cases.json',JSON.stringify(requests));
const html=`<script type="module">import init,*as wasm from '/pkg/weft_wasm.js';import{bindCompiler}from'/wrapper.js';await init();const compiler=bindCompiler(wasm);const cases=await(await fetch('/cases.json')).json();const outputs=[];for(const c of cases){const raw=compiler.compileJson(c.request);const actual=JSON.parse(raw);if(JSON.stringify(actual)!==JSON.stringify(c.expected))throw new Error(c.name+' differs');outputs.push({name:c.name,raw});}window.__proof={outputs};</script>`;
const paths=new Map([['/','index'],['/wrapper.js',root+'/wrapper.js'],['/cases.json',root+'/cases.json'],['/pkg/weft_wasm.js',root+'/pkg/weft_wasm.js'],['/pkg/weft_wasm_bg.wasm',root+'/pkg/weft_wasm_bg.wasm']]);
const server=Bun.serve({hostname:'127.0.0.1',port:0,fetch(req){const p=paths.get(new URL(req.url).pathname);if(!p)return new Response('missing',{status:404});if(p==='index')return new Response(html,{headers:{'content-type':'text/html'}});return new Response(Bun.file(p));}});
let browser;let proof;
try{browser=await chromium.launch({headless:true});const page=await browser.newPage();const errors=[];page.on('pageerror',e=>errors.push(String(e)));await page.goto(`http://127.0.0.1:${server.port}/`);await page.waitForFunction(()=>Boolean((window as any).__proof),{},{timeout:60000});proof=await page.evaluate(()=> (window as any).__proof);if(errors.length)throw new Error(errors.join('\n'));proof.browser=await browser.version();}
finally{try{if(browser)await browser.close();}finally{server.stop(true);}}
for(const item of proof.outputs)writeFileSync(root+'/'+item.name+'-browser.json',item.raw+'\n');
writeFileSync(root+'/report.json',JSON.stringify({format:'weft-positioned-browser-parity/0.1',browser:proof.browser,checks:proof.outputs.map(c=>({case:c.name,sha256:createHash('sha256').update(c.raw+'\n').digest('hex'),json_identical:true})),qualification:'Actual Chromium shared Rust WASM/compiler transport17requests identical; no native data or positionedresult decoder execution.'},null,2)+'\n');
console.log(JSON.stringify({browser:proof.browser,cases:proof.outputs.length}));
