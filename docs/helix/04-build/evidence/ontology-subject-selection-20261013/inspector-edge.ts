import {createOrdinarySubjectKeyBinding} from '/private/tmp/truss-security-main-integration/packages/pg-runtime/src/subject-key.ts';
import * as owner from '/private/tmp/truss-security-main-integration/packages/python/src/truss/_umf_keys08/producer.js';
import vm from 'node:vm';import {types} from 'node:util';
const sourceText=JSON.parse(await Bun.file('/private/tmp/truss-security-main-integration/packages/python/tests/fixtures/security-mapping-request.json').text()).modules[0].documentJson;
let entered=0,species=0;const mode=process.argv[2];
const options:any={host:{observeOrdinarySubject(){throw Error('must not observe');}},owner,sourceText,identity:{module:'m',element:'Staff',key:'pk'},keyColumns:['id'],encodings:['string']};
if(mode==='foreign-async') options.inspectSubjectKey=vm.runInNewContext('(async()=>{ mark(); throw new Error("foreign rejection") })',{mark(){entered++;}});
else if(mode==='promise-species') {class Rejected extends Promise<unknown>{static get [Symbol.species](){species++;throw Error('species executed');}} options.inspectSubjectKey=()=>new Rejected((_,reject)=>reject(Error('original rejected')));}
else if(mode==='promise-constructor') options.inspectSubjectKey=()=>{const p=Promise.reject(Error('original rejected'));Object.defineProperty(p,'constructor',{get(){species++;throw Error('constructor executed');}});return p;};
try{createOrdinarySubjectKeyBinding(options);}catch(e){console.log(JSON.stringify({mode,refused:true,entered,species,isAsyncFunction:types.isAsyncFunction(options.inspectSubjectKey),error:String(e)}));}
await new Promise(resolve=>setTimeout(resolve,20));
