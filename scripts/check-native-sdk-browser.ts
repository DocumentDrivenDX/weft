/** Same original native programs through installed Python and actual Chromium WASM. */
import {chromium} from 'playwright';
import {resolve} from 'node:path';
const [wasmDirectory,python,output]=process.argv.slice(2);if(!wasmDirectory||!python||!output)throw Error('Required: WASM directory, installed Python, output JSON');
const cases:{name:string;method:string;request:string;arguments?:string[];expected?:string}[]=[];
const action=await Bun.file('tests/native-actions/action-request.json').json();
for(const name of ['approve','guarded','postFail','createOrder','firstFalse']){const r=structuredClone(action);r.source.action=name;const doc=JSON.parse(r.binding.modelJson);r.source.actionPointer='/modules/0/extensions/umf.actions/actions/'+doc.modules[0].extensions['umf.actions'].actions.findIndex((a:any)=>a.id===name);cases.push({name,method:'compile_native_action_json',request:JSON.stringify(r)});}
const bulk={version:'weft.native-bulk/0.1',selection:{version:'weft.raw-read/0.1',target:'postgresql18',sql:'SELECT o.id AS id, o.status AS status FROM sales.order AS o ORDER BY o.id ASC',binding:action.binding,parameters:[],rowCap:4},action,maxItems:3,actionInputs:{},parameters:[]};cases.push({name:'bulk',method:'compile_native_bulk_json',request:JSON.stringify(bulk)});
const deltaAction=structuredClone(action);deltaAction.target='spark-delta4';
cases.push({name:'delta-action',method:'compile_native_action_json',request:JSON.stringify(deltaAction)});
const deltaBulk=structuredClone(bulk);deltaBulk.action=deltaAction;deltaBulk.selection.target='spark-delta4';
cases.push({name:'delta-bulk',method:'compile_native_bulk_json',request:JSON.stringify(deltaBulk)});
for(const name of ['keyword','vector'])cases.push({name,method:'compile_search_json',request:await Bun.file('tests/native-search/'+name+'-request.json').text()});
const vector086=await Bun.file('tests/native-search/vector-request.json').json();vector086.vectorBinding.extensionVersion='0.8.6';cases.push({name:'vector-0.8.6',method:'compile_search_json',request:JSON.stringify(vector086)});
const vectorUnknown=structuredClone(vector086);vectorUnknown.vectorBinding.extensionVersion='unknown';cases.push({name:'vector-version-refusal',method:'compile_search_json',request:JSON.stringify(vectorUnknown),expected:'blocked'});
const spark={version:'weft.raw-read/0.1',target:'spark-delta4',sql:'SELECT o.id AS id, o.status AS status FROM sales.order AS o ORDER BY o.id ASC',binding:{...action.binding,schema:'sdk_spark'},parameters:[],rowCap:10,keyset:false,snapshots:{orders:7}};cases.push({name:'spark',method:'compile_raw_read_json',request:JSON.stringify(spark)});
const refused=structuredClone(action);refused.source.documentSha256='0'.repeat(64);cases.push({name:'source-refusal',method:'compile_native_action_json',request:JSON.stringify(refused)});
for(const [name,method] of [['truss','truss-runtime'],['ashlar-spark4','ashlar-spark4'],['ashlar-dbsql-20261016','ashlar-dbsql-20261016']]){
 const request=await Bun.file('tests/engine-catalog/native-'+name+'-request.json').json();
 const input=JSON.stringify({version:'weft.engine-read/0.1',compiler:method,request});
 cases.push({name:'engine-'+name,method:'compile_engine_read_json',request:input});
 const compile=Bun.spawn([python,'-I','-B','-c','import sys,weft; print(weft.compile_engine_read_json(sys.stdin.read()))'],{stdin:new Blob([input]),stdout:'pipe',stderr:'pipe'});
 const program=await new Response(compile.stdout).text();if(await compile.exited!==0)throw Error('Original engine fixture compilation failed');
 const current=structuredClone(request);for(const value of Object.values(current.parameters) as any[])if(value.family==='string')value.value='different bound data';
 cases.push({name:'engine-current-'+name,method:'check_engine_read_json',request:'',arguments:[program,JSON.stringify(current)]});
 current.target.bindingSha256='0'.repeat(64);
 cases.push({name:'engine-drift-'+name,method:'check_engine_read_json',request:'',arguments:[program,JSON.stringify(current)],expected:'blocked'});
}
const child=Bun.spawn([python,'-I','-B','-c','import sys,json,weft\nprint(json.dumps([json.loads(getattr(weft,c["method"])(*c.get("arguments",[c["request"]]))) for c in json.load(sys.stdin)]))'],{stdin:new Blob([JSON.stringify(cases)]),stdout:'pipe',stderr:'pipe'});const expected=JSON.parse(await new Response(child.stdout).text());if(await child.exited!==0)throw Error(await new Response(child.stderr).text());
if(expected.some((e:any,i:number)=>e.status!==(cases[i].expected??(cases[i].name==='source-refusal'?'blocked':'compiled'))))throw Error('Native expected fixtures did not exercise success/refusal');
const server=Bun.serve({hostname:'127.0.0.1',port:0,fetch(req){const name=new URL(req.url).pathname.slice(1);if(name==='weft_wasm.js'||name==='weft_wasm_bg.wasm')return new Response(Bun.file(resolve(wasmDirectory,name)),{headers:{'content-type':name.endsWith('.wasm')?'application/wasm':'text/javascript'}});return new Response('<script type="module">import init,* as api from "/weft_wasm.js";await init();window.api=api;</script>',{headers:{'content-type':'text/html'}});}});
const browser=await chromium.launch({headless:true});try{const page=await browser.newPage();await page.goto('http://127.0.0.1:'+server.port);await page.waitForFunction(()=>!!(window as any).api);const actual=await page.evaluate(cases=>cases.map(c=>JSON.parse((window as any).api[c.method](...(c.arguments??[c.request])))),cases);
function canonical(value:any):string{return JSON.stringify(value,(_,v)=>v&&typeof v==='object'&&!Array.isArray(v)?Object.fromEntries(Object.keys(v).sort().map(k=>[k,v[k]])):v);}
if(canonical(actual)!==canonical(expected))throw Error('WASM differs from installed Rust Python output');
const unicode=await page.evaluate(()=>['compile_raw_read_json','compile_native_action_json','compile_native_bulk_json','compile_search_json','compile_engine_read_json'].every(name=>{try{(window as any).api[name]('\ud800');return false;}catch{return true;}}));const engineUnicode=await page.evaluate(()=>{const bad=String.fromCharCode(0xd800);for(const args of [[bad,'{}'],['{}',bad]]){try{(window as any).api.check_engine_read_json(...args);return false;}catch{}}return true;});if(!engineUnicode)throw Error('Engine comparison replaced malformed UTF16');
if(!unicode)throw Error('Original malformed UTF16 was silently replaced');
await Bun.write(output,JSON.stringify({status:'pass',browser:browser.version(),cases:cases.map(c=>c.name),exactPythonWasmPrograms:true,malformedUnicodeRefused:true,temporal:'HELD'},null,2)+'\n');console.log('native SDK WASM parity: pass');}finally{await browser.close();server.stop(true);}
