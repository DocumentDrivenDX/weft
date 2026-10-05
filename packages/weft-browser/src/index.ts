/** Transport wrapper only; all JSON, model, language and backend semantics stay in Rust. */
export interface WasmCompiler {
  compile_json(request: string): string;
}
export interface Compiler {
  compileJson(requestJson: string): string;
}
const fatalResponse='{"diagnostics":[{"code":"WFT-BACKEND-FAILURE","message":"Compiler host trapped; initialize a fresh instance","phase":"host","recoverability":"host-action","severity":"error"}],"interfaceVersion":"weft-compile/0.1.0","status":"blocked"}';
function scalarText(value: unknown): asserts value is string {
  if(typeof value!=='string')throw new TypeError('compileJson requires a UTF-8 scalar string');
  for(let i=0;i<value.length;i++){
    const c=value.charCodeAt(i);
    if(c>=0xd800&&c<=0xdbff){const next=value.charCodeAt(++i);if(!(next>=0xdc00&&next<=0xdfff))throw new TypeError('Unpaired UTF-16 surrogate');}
    else if(c>=0xdc00&&c<=0xdfff)throw new TypeError('Unpaired UTF-16 surrogate');
  }
}
/** Bind an explicitly initialized, trusted WASM module; no executable URL comes from content. */
export function bindCompiler(api: WasmCompiler): Compiler {
  let poisoned=false;
  return Object.freeze({compileJson(requestJson: string):string{
    scalarText(requestJson);
    if(poisoned)return fatalResponse;
    try{return api.compile_json(requestJson);}
    catch(error){
      if(!(error instanceof WebAssembly.RuntimeError))throw error;
      poisoned=true;return fatalResponse;
    }
  }});
}
/** Host calls wasm-bindgen init first and supplies its module object to bindCompiler. */
