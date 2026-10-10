/** Closed metadata shape does not make any index or realization trusted. */
import {test,expect} from 'bun:test';
import Ajv2020 from 'ajv/dist/2020.js';
import schema from '../../docs/helix/02-design/contracts/distribution-index.schema.json';
const ajv=new Ajv2020({strict:true,allErrors:true});
const validate=ajv.compile(schema);
const artifact=(path:string)=>({path,sha256:'a'.repeat(64),bytes:10});
const entry=()=>({realizationId:'fixture-only-cli',manifest:artifact('manifest.json'),executable:artifact('bin/weft-runtime'),target:'aarch64-apple-darwin',assemblyCustody:artifact('assembly-custody.json')});
const index=()=>({format:'weft-distribution-index/0.1',entries:[entry()]});
test('closed inert index schema compiles and preserves original input',()=>{
 expect(ajv.validateSchema(schema)).toBe(true);const value=index(),before=JSON.stringify(value);
 expect(validate(value)).toBe(true);expect(JSON.stringify(value)).toBe(before);
});
test('no self-admission, executable locator, omitted member or unknown metadata',()=>{
 for(const path of [['format'],['entries'],['entries',0,'realizationId'],['entries',0,'manifest'],['entries',0,'executable'],['entries',0,'target'],['entries',0,'assemblyCustody']]){
  const value:any=index();let owner=value;for(const key of path.slice(0,-1))owner=owner[key];delete owner[path.at(-1)!];expect(validate(value)).toBe(false);
 }
 for(const mutate of [(v:any)=>v.trusted=true,(v:any)=>v.entries[0].registered=true,(v:any)=>v.entries[0].executable.url='https://example.invalid/run',(v:any)=>v.entries[0].manifest.approved=true]){
  const value=index();mutate(value);expect(validate(value)).toBe(false);
 }
});
test('empty/duplicate whole entries and unsafe descriptor or identity forms refuse',()=>{
 for(const mutate of [(v:any)=>v.entries=[],(v:any)=>v.entries.push({...v.entries[0]}),(v:any)=>v.entries[0].manifest.path='../manifest.json',(v:any)=>v.entries[0].executable.path='/tmp/run',(v:any)=>v.entries[0].manifest.sha256='A'.repeat(64),(v:any)=>v.entries[0].executable.bytes=true,(v:any)=>v.entries[0].realizationId='bad id',(v:any)=>v.entries[0].target='native']){
  const value=index();mutate(value);expect(validate(value)).toBe(false);
 }
});
test('schema cannot establish unique IDs, target correspondence or index trust',()=>{
 const value=index();value.entries.push({...entry(),target:'x86_64-pc-windows-msvc'});
 expect(validate(value)).toBe(true); // Consumer admission rejects repeated identity regardless of distinct metadata.
});
