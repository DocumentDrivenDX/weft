/** Structural metadata only; actual artifact/index admission is a separate boundary. */
import {test,expect} from 'bun:test';
import Ajv2020 from 'ajv/dist/2020.js';
import schema from '../../docs/helix/02-design/contracts/distribution-manifest.schema.json';
const ajv=new Ajv2020({strict:true,allErrors:true});
const validate=ajv.compile(schema);
const artifact=(path:string)=>({path,sha256:'a'.repeat(64),bytes:10});
function candidate(){return {
 format:'weft-distribution/0.1',realizationId:'weft-cli-candidate-a',
 source:{commit:'b'.repeat(40),inventory:{artifact:artifact('source-inventory.json.gz'),decodedSha256:'c'.repeat(64),decodedBytes:1000,trackedFiles:1588}},
 build:{platform:{binaryFormat:'mach-o',machine:'arm64',minimumOS:'11.0',sdk:'27.0',observedOS:'27.0.1'},release:true,target:'aarch64-apple-darwin',features:['ashlar-databricks-candidate'],command:['cargo','build','--release','--features','ashlar-databricks-candidate'],tools:artifact('tools.json'),lockfiles:[artifact('Cargo.lock')],toolchain:artifact('observed-toolchain.json'),effectiveEnvironment:{observed:{CARGO_HOME:'/private/tmp/cargo',RUSTUP_HOME:'/private/tmp/rustup',CARGO_TARGET_DIR:'/private/tmp/build',PATHPrefix:'/private/tmp/toolchain/bin'},unknowns:['linker selection not independently observed','cache provenance not independently observed']}},
 executable:artifact('bin/weft-runtime'),backendManifests:[artifact('backend-manifest.json')],publicSchemas:[artifact('compile-request.schema.json')],
 conformance:{corpus:{cases:463,compiled:462,blocked:1,responses:artifact('corpus/responses.jsonl'),summary:artifact('corpus/summary.json'),custody:artifact('corpus/custody.json')},controls:{cases:19,responses:artifact('controls/responses.jsonl'),summary:artifact('controls/summary.json'),custody:artifact('controls/custody.json')},transport:artifact('transport.json')}
};}
function changed(mutate:(value:any)=>void){const value=candidate();mutate(value);return value;}
test('closed schema and one exact initial candidate shape validate without mutation',()=>{
 expect(ajv.validateSchema(schema)).toBe(true);const value=candidate(),before=JSON.stringify(value);expect(validate(value)).toBe(true);expect(JSON.stringify(value)).toBe(before);
});
test('every required top-level and nested custody member must exist',()=>{
 const paths=[...Object.keys(candidate()).map(key=>[key]),['source','commit'],['source','inventory','decodedSha256'],['source','inventory','decodedBytes'],['source','inventory','trackedFiles'],['source','inventory','artifact'],['build','platform'],['build','platform','binaryFormat'],['build','platform','machine'],['build','platform','minimumOS'],['build','platform','sdk'],['build','platform','observedOS'],['build','release'],['build','target'],['build','features'],['build','command'],['build','tools'],['build','lockfiles'],['build','toolchain'],['build','effectiveEnvironment','observed'],['build','effectiveEnvironment','unknowns'],['executable','sha256'],['executable','path'],['executable','bytes'],['conformance','corpus','responses'],['conformance','corpus','custody'],['conformance','controls','cases'],['conformance','transport']];
 for(const path of paths){const value:any=candidate();let owner=value;for(const key of path.slice(0,-1))owner=owner[key];delete owner[path.at(-1)!];expect(validate(value),path.join('.')).toBe(false);}
});
test('unknown properties and self-promoting trust or pass flags refuse',()=>{
 for(const mutate of [(v:any)=>v.registered=true,(v:any)=>v.trustedIndex='self.json',(v:any)=>v.source.inventory.complete=true,(v:any)=>v.build.hermetic=true,(v:any)=>v.executable.url='https://example.invalid/binary',(v:any)=>v.conformance.passed=true,(v:any)=>v.conformance.corpus.passed=true,(v:any)=>v.build.effectiveEnvironment.secret='hidden'])expect(validate(changed(mutate))).toBe(false);
});
test('descriptor paths reject unsafe lexical forms and preserve relative Unicode file names',()=>{
 for(const path of ['', '/', '/bin/weft', '../binary','./binary','a/../binary','a/./binary','a//binary','a/','C:/binary','C:\\binary','a\\binary','a\u0000b','a\nb','a\u007fb'])expect(validate(changed(v=>v.executable.path=path)),path).toBe(false);
 expect(validate(changed(v=>v.executable.path='artifacts/ráw-binary'))).toBe(true);
});
test('exact hash shapes and safe integer counts refuse malformed values',()=>{
 for(const sha of ['', 'A'.repeat(64),'a'.repeat(63),'a'.repeat(65),'a'.repeat(64)+'\n'])expect(validate(changed(v=>v.executable.sha256=sha))).toBe(false);
 for(const commit of ['', 'A'.repeat(40),'b'.repeat(39),'b'.repeat(41)])expect(validate(changed(v=>v.source.commit=commit))).toBe(false);
 for(const bytes of [-1,0.5,true,'10',9007199254740992])expect(validate(changed(v=>v.executable.bytes=bytes))).toBe(false);
 for(const count of [0,-1,0.5,true,'1',9007199254740992])expect(validate(changed(v=>v.conformance.corpus.cases=count))).toBe(false);
 expect(validate(changed(v=>v.executable.bytes=0))).toBe(true);
});
test('release target features argv and observed environment have a closed bounded shape',()=>{
 for(const mutate of [(v:any)=>v.build.platform.supported=true,(v:any)=>v.build.platform.machine='',(v:any)=>v.build.platform.observedOS='macOS\ncredential',(v:any)=>v.build.release=false,(v:any)=>v.build.target='',(v:any)=>v.build.target='native',(v:any)=>v.build.target='aarch64 apple darwin',(v:any)=>v.build.features=[],(v:any)=>v.build.features=['a','a'],(v:any)=>v.build.features=['bad feature'],(v:any)=>v.build.command=[],(v:any)=>v.build.command=['cargo\ncredential'],(v:any)=>v.build.effectiveEnvironment.observed.TOKEN='hidden',(v:any)=>v.build.effectiveEnvironment.observed.PATHPrefix=3,(v:any)=>v.build.effectiveEnvironment.unknowns=['x','x'],(v:any)=>v.build.effectiveEnvironment.unknowns=['']])expect(validate(changed(mutate))).toBe(false);
});
test('generic well-shaped candidates remain inert when exact realization/count equations differ',()=>{
 const value=changed(v=>{v.build.target='x86_64-pc-windows-msvc';v.build.features=['other-backend'];v.conformance.corpus.cases=1;v.conformance.corpus.compiled=100;v.conformance.corpus.blocked=2;v.conformance.controls.cases=2;v.source.inventory.trackedFiles=1;});
 expect(validate(value)).toBe(true); // Independent index/actual receipts must refuse this as the initial realization.
});
test('empty inventories and exact duplicate artifacts refuse but path-wise correspondence is external',()=>{
 for(const field of ['backendManifests','publicSchemas'])expect(validate(changed(v=>v[field]=[]))).toBe(false);
 expect(validate(changed(v=>v.build.lockfiles=[]))).toBe(false);
 expect(validate(changed(v=>v.publicSchemas.push({...v.publicSchemas[0]})))).toBe(false);
 expect(validate(changed(v=>v.publicSchemas.push({...v.publicSchemas[0],sha256:'d'.repeat(64)})))).toBe(true); // Same path/different bytes is a semantic inventory conflict, not admitted here.
});
