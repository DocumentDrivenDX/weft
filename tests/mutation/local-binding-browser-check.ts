import {createRequire} from 'node:module';
import {join} from 'node:path';
const {chromium}=createRequire(Bun.argv[2])('playwright'),wasm=Bun.argv[3],truss=Bun.argv[4];
if(!wasm||!truss)throw Error('Playwright package, WASM web output and Truss source checkout required');
const pglite=join(truss,'packages/browser/node_modules/@electric-sql/pglite/dist');
const built=await Bun.build({entrypoints:[new URL('./local-binding-browser-probe.ts',import.meta.url).pathname],target:'browser',plugins:[{name:'original-owners',setup(b){
 b.onResolve({filter:/^truss-browser-catalog$/},()=>({path:join(truss,'packages/browser/catalog.ts')}));
 b.onResolve({filter:/^@electric-sql\/pglite$/},()=>({path:'./pglite/index.js',external:true}));
}}]});if(!built.success)throw Error(String(built.logs));
const server=Bun.serve({hostname:'127.0.0.1',port:0,fetch(request){const path=new URL(request.url).pathname;
 if(path==='/')return new Response('<!doctype html><title>Weft original Truss selection</title>');
 if(path==='/probe.js')return new Response(built.outputs[0],{headers:{'content-type':'text/javascript'}});
 if(path==='/weft_wasm.js'||path==='/weft_wasm_bg.wasm')return new Response(Bun.file(join(wasm,path.slice(1))));
 if(path.startsWith('/pglite/')&&!path.includes('..'))return new Response(Bun.file(join(pglite,path.slice(8))));
 return new Response('Missing',{status:404});
}});
const browser=await chromium.launch({headless:true});
try{
 const page=await browser.newPage();await page.goto(`http://127.0.0.1:${server.port}`);await page.evaluate(async()=>{const path='/probe.js';await import(path);});
 const fixture=await Bun.file(new URL('./fixtures/local-products-input.json',import.meta.url)).json();
 const runtime=join(truss,'packages/python/src/truss/_catalog_runtime');
 const result=await page.evaluate(async({fixture,layout,scalars})=>(globalThis as any).runBinding(fixture,layout,scalars),{fixture,layout:await Bun.file(join(runtime,'layout.sql')).text(),scalars:await Bun.file(join(runtime,'scalars.sql')).text()});
 const sourceHashes:Record<string,string>={};for(const file of ['packages/browser/catalog.ts','packages/postgresql/src/acceptance-json.ts','packages/postgresql/src/original-byte-custody.ts','packages/umf-bun/src/catalog-declarations.ts','packages/python/src/truss/_umf_core08/producer.js',...['layout','scalars','lineage','containers','plan'].map(n=>'packages/python/src/truss/_catalog_runtime/'+n+'.sql')])sourceHashes[file]=new Bun.CryptoHasher('sha256').update(await Bun.file(join(truss,file)).bytes()).digest('hex');
 const receipt={sourceHashes,scope:'Original Rust WASM lowering executed against actual Truss catalog/layout and scalar validators in PGlite memory; administrative fixture seeds only',chromium:browser.version(),wasmSha256:new Bun.CryptoHasher('sha256').update(await Bun.file(join(wasm,'weft_wasm_bg.wasm')).bytes()).digest('hex'),...result};
 const text=JSON.stringify(receipt,null,2)+'\n';if(Bun.argv[5])await Bun.write(Bun.argv[5],text);console.log(text);
}finally{await browser.close();server.stop(true);}
