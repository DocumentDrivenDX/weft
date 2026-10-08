import {bindCompiler} from '@documentdrivendx/weft';
const compiler=bindCompiler({compile_json:value=>value});
if(compiler.compileJson('é😀')!=='é😀')throw new Error('scalar transport changed');
let refused=false;try{compiler.compileJson('\ud800')}catch(e){refused=e instanceof TypeError}
if(!refused)throw new Error('surrogate accepted');
const trap=bindCompiler({compile_json:()=>{throw new WebAssembly.RuntimeError('trap')}});
if(JSON.parse(trap.compileJson('{}')).status!=='blocked')throw new Error('trap escaped');
console.log('packed ESM import and transport controls passed');
