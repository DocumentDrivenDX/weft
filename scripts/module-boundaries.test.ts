import {test,expect} from 'bun:test';
import {checkGraph,checkSurface,edgeIdentity,javascriptDependencies,checkJavascriptDependencies} from './module-boundaries';
const metadata={workspace_members:['a','b'],packages:[{id:'a',name:'a',dependencies:[{name:'b',kind:null,optional:true}]},{id:'b',name:'b',dependencies:[]}]};
const map={packages:[{name:'a',dependencies:[{name:'b',kind:'normal',optional:true}]},{name:'b',dependencies:[]}]};
test('allowed actual graph passes',()=>expect(()=>checkGraph(metadata,map)).not.toThrow());
test('forbidden optional edge fails',()=>{const x=structuredClone(metadata);x.packages[1].dependencies.push({name:'a',kind:null,optional:true});expect(()=>checkGraph(x,map)).toThrow('Dependency boundary');});
test('unowned package fails',()=>{const x=structuredClone(metadata);x.packages[1].name='new';expect(()=>checkGraph(x,map)).toThrow('Unowned');});
test('missing package fails',()=>{const x=structuredClone(metadata);x.packages.pop();expect(()=>checkGraph(x,map)).toThrow();});
test('cycle fails even when edges individually allowed',()=>{const x=structuredClone(metadata),y=structuredClone(map);x.packages[1].dependencies.push({name:'a',kind:null,optional:true});y.packages[1].dependencies.push({name:'a',kind:'normal',optional:true});expect(()=>checkGraph(x,y)).toThrow('cycle');});
test('new public type fails',()=>expect(()=>checkSurface([{path:'a.rs',publicDeclarations:['New']}],[{path:'a.rs',publicDeclarations:[]}])).toThrow());
test('new source fails',()=>expect(()=>checkSurface([{path:'new.rs',publicDeclarations:[]}],[])).toThrow());
test('removed public type fails',()=>expect(()=>checkSurface([{path:'a.rs',publicDeclarations:[]}],[{path:'a.rs',publicDeclarations:['Old']}])).toThrow());

import {mkdtemp, realpath, mkdir, writeFile, symlink, rm, readFile, cp} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {resolve} from 'node:path';
import {publicDeclarations,scanSurface} from './module-boundaries';
test('actual file scanner refuses same-line, reexport, extern and static additions',async()=>{
 const root=await realpath(await mkdtemp(resolve(tmpdir(),'weft-boundary-')));
 try {
  await mkdir(resolve(root,'src'));
  const baseline='pub struct Owned; // pub struct Comment;\nconst TEXT: &str=r#"pub struct Secret;"#;';
  await writeFile(resolve(root,'src/lib.rs'),baseline);
  const expected=await scanSurface(root,'Cargo.toml');
  for(const added of ['const _: () = (); pub struct UnownedExport;','pub use model::Catalog as UnownedCatalog;','pub extern "C" fn unowned_export() {}','pub static UNOWNED: u8=0;']) {
   await writeFile(resolve(root,'src/lib.rs'),baseline+'\n'+added);
   const actual=await scanSurface(root,'Cargo.toml');expect(()=>checkSurface(actual,expected)).toThrow();
  }
 } finally {await rm(root,{recursive:true,force:true});}
});
test('unsupported public form refuses',()=>expect(()=>publicDeclarations('pub macro exported {}')).toThrow());
test('actual Cargo metadata detects same-name external source substitution',async()=>{
 const root=await realpath(await mkdtemp(resolve(tmpdir(),'weft-metadata-')));
 try {
  for(const dir of ['core/src','runtime/src','external/src']) {await mkdir(resolve(root,dir),{recursive:true});await writeFile(resolve(root,dir,'lib.rs'),'');}
  const pkg=(name:string,version='0.1.0')=>`[package]\nname="${name}"\nversion="${version}"\nedition="2021"\n`;
  await writeFile(resolve(root,'Cargo.toml'),'[workspace]\nmembers=["core","runtime"]\nexclude=["external"]\nresolver="2"\n');
  await writeFile(resolve(root,'core/Cargo.toml'),pkg('weft-core'));
  await writeFile(resolve(root,'external/Cargo.toml'),pkg('weft-core','0.1.1'));
  await writeFile(resolve(root,'runtime/Cargo.toml'),pkg('weft-runtime')+'[dependencies]\nweft-core={path="../core"}\n');
  async function metadata() {
   const p=Bun.spawn([process.env.WEFT_CARGO??'cargo','metadata','--no-deps','--format-version','1','--offline'],{cwd:root,stdout:'pipe',stderr:'pipe'});
   const raw=await new Response(p.stdout).text();await new Response(p.stderr).text();expect(await p.exited).toBe(0);return JSON.parse(raw);
  }
  const good=await metadata();
  const rules={packages:good.packages.map((p:any)=>({name:p.name,dependencies:p.dependencies.map((d:any)=>edgeIdentity(d,root)).sort((a:any,b:any)=>a.name.localeCompare(b.name)||a.kind.localeCompare(b.kind))}))};
  expect(()=>checkGraph(good,rules,root)).not.toThrow();
  await writeFile(resolve(root,'runtime/Cargo.toml'),pkg('weft-runtime')+'[dependencies]\nweft-core={path="../external"}\n');
  const bad=await metadata();expect(()=>checkGraph(bad,rules,root)).toThrow('Dependency boundary');
 } finally {await rm(root,{recursive:true,force:true});}
});

test('directory symlink cannot hide source from actual scanner',async()=>{
 const root=await realpath(await mkdtemp(resolve(tmpdir(),'weft-link-')));
 try {
  await mkdir(resolve(root,'src'));await mkdir(resolve(root,'outside'));
  await writeFile(resolve(root,'src/lib.rs'),'#[path="link/owned.rs"] pub mod owned;');
  await writeFile(resolve(root,'outside/owned.rs'),'pub struct Unowned;');
  await symlink('../outside',resolve(root,'src/link'));
  await expect(scanSurface(root,'Cargo.toml')).rejects.toThrow('symlink');
 } finally {await rm(root,{recursive:true,force:true});}
});
test('const unsafe functions retain actual API identity',()=>{
 expect(publicDeclarations('pub const unsafe fn before() {}')).toEqual(['fn:before']);
 expect(publicDeclarations('pub const unsafe extern "C" fn after() {}')).toEqual(['fn:after']);
});

test('new JavaScript dependency groups cannot bypass ownership',()=>{
 const baseline={devDependencies:{owned:'1.0.0'}};const expected=javascriptDependencies(baseline);
 for(const group of ['dependencies','devDependencies','optionalDependencies','peerDependencies']) {
  const changed=structuredClone(baseline) as any;changed[group]={...changed[group],unowned:'1.0.0'};
  expect(()=>checkJavascriptDependencies(changed,expected)).toThrow();
 }
});

test('raw and commented Rust path attributes cannot redirect owned modules',async()=>{
 const root=await realpath(await mkdtemp(resolve(tmpdir(),'weft-path-')));
 try {
  await mkdir(resolve(root,'src'));await mkdir(resolve(root,'outside'));
  await writeFile(resolve(root,'src/owned.rs'),'pub struct Owned;');
  await writeFile(resolve(root,'outside/owned.rs'),'pub struct Unowned;');
  await writeFile(resolve(root,'src/lib.rs'),'pub mod owned;');
  const expected=await scanSurface(root,'Cargo.toml');
  for(const attribute of ['#[path = r"../outside/owned.rs"]','#[path /* comment */ = "../outside/owned.rs"]','#[r#path="../outside/owned.rs"]','#[r#cfg_attr(all(),r#path="../outside/owned.rs")]']) {
   await writeFile(resolve(root,'src/lib.rs'),attribute+' pub mod owned;');
   const actual=await scanSurface(root,'Cargo.toml');expect(()=>checkSurface(actual,expected)).toThrow();
  }
 } finally {await rm(root,{recursive:true,force:true});}
});
