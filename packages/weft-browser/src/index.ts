// SPDX-License-Identifier: MIT OR Apache-2.0
/** Transport wrapper only; all JSON, model, language and backend semantics stay in Rust. */
export interface WasmCompiler {
  compile_json(request: string): string;
}
export interface Compiler {
  compileJson(requestJson: string): string;
}
export interface WasmTrussPageCompiler {
  compile_truss_runtime_page_json(request: string, pageSize: number, afterJson: string): string;
}
export interface TrussPageCompiler {
  compilePageJson(requestJson: string, pageSize: number, after?: readonly string[] | null): string;
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
function boundedScalarText(value: unknown,maximum: number): number {
  if(typeof value!=='string')throw new TypeError('UTF-8 scalar string required');
  if(value.length>maximum)throw new RangeError('Text byte bound exceeded');
  scalarText(value);
  let bytes=0;
  for(let i=0;i<value.length;i++){
    const unit=value.charCodeAt(i);
    bytes+=unit<0x80?1:unit<0x800?2:unit>=0xd800&&unit<=0xdbff?(i++,4):3;
    if(bytes>maximum)throw new RangeError('Text byte bound exceeded');
  }
  return bytes;
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
export function bindTrussPageCompiler(api: WasmTrussPageCompiler): TrussPageCompiler {
  let poisoned=false;
  return Object.freeze({compilePageJson(requestJson: string,pageSize: number,after: readonly string[] | null=null): string {
    boundedScalarText(requestJson,16_777_216);
    if(typeof pageSize!=='number'||!Number.isInteger(pageSize)||pageSize<1||pageSize>999)throw new TypeError('Integer page size from 1 through 999 required');
    if(after!==null){
      if(!Array.isArray(after)||after.length>1024)throw new TypeError('Bounded typed continuation required');
      let bytes=after.length===0?2:1;
      for(const value of after){bytes+=boundedScalarText(value,65_536)+3;if(bytes>1_048_576)throw new RangeError('Continuation byte bound exceeded');}
    }
    const afterJson=JSON.stringify(after);boundedScalarText(afterJson,1_048_576);
    if(poisoned)return fatalResponse;
    try{return api.compile_truss_runtime_page_json(requestJson,pageSize,afterJson);}
    catch(error){if(!(error instanceof WebAssembly.RuntimeError))throw error;poisoned=true;return fatalResponse;}
  }});
}
