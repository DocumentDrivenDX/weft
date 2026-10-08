import {bindCompiler, type Compiler, type WasmCompiler} from '@documentdrivendx/weft';
const api: WasmCompiler={compile_json:(request:string)=>request};
const compiler: Compiler=bindCompiler(api);
const output:string=compiler.compileJson('{}');
// @ts-expect-error scalar string API must reject numbers
compiler.compileJson(1);
