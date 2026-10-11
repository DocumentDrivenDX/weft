import {readFile, readdir, realpath} from 'node:fs/promises';
import {resolve, dirname} from 'node:path';
export function edgeIdentity(d:any, root:string) {
  return {name:d.name,kind:d.kind ?? 'normal',optional:d.optional,source:d.source ?? null,req:d.req ?? null,rename:d.rename ?? null,target:d.target ?? null,features:[...(d.features ?? [])].sort(),defaultFeatures:d.uses_default_features ?? true,path:d.path ? resolve(d.path).slice(root.length+1) : null};
}
export function checkGraph(metadata: any, map: any, root:string='') {
  const expected = new Map<string, any>(map.packages.map((p: any) => [p.name,p]));
  const actual = metadata.packages.filter((p:any) => metadata.workspace_members.includes(p.id));
  if (expected.size !== map.packages.length || actual.length !== expected.size || new Set(actual.map((p:any)=>p.name)).size !== expected.size) throw Error('Workspace ownership set differs');
  const graph = new Map<string,string[]>();
  for (const p of actual) {
    const rule = expected.get(p.name);
    if (!rule) throw Error(`Unowned package: ${p.name}`);
    const edges = p.dependencies.map((d:any)=>edgeIdentity(d,root)).sort((a:any,b:any)=>a.name.localeCompare(b.name)||a.kind.localeCompare(b.kind));
    const allowed = rule.dependencies.map((d:any)=>('defaultFeatures' in d ? d : edgeIdentity(d,root))).sort((a:any,b:any)=>a.name.localeCompare(b.name)||a.kind.localeCompare(b.kind));
    if (JSON.stringify(edges)!==JSON.stringify(allowed)) throw Error(`Dependency boundary differs: ${p.name}`);
    graph.set(p.name,edges.filter((d:any)=>expected.has(d.name)).map((d:any)=>d.name));
  }
  const active = new Set<string>(), done = new Set<string>();
  function visit(name:string) {
    if(active.has(name)) throw Error(`Dependency cycle: ${name}`);
    if(done.has(name)) return;
    active.add(name); for(const next of graph.get(name) ?? []) visit(next);
    active.delete(name);done.add(name);
  }
  for(const name of graph.keys()) visit(name);
}
// Conservative lexical inventory, not a Rust parser/name-resolution proof.
// Comments and quoted/raw literals are skipped before inspecting every pub token.
function rustTokens(source:string) {
  const tokens:string[]=[]; const literals=new Map<number,{raw:boolean,value:string}>(); let i=0;
  while(i<source.length) {
    if(/\s/.test(source[i])) {i++;continue;}
    if(source.startsWith('//',i)) {const end=source.indexOf('\n',i);i=end<0?source.length:end+1;continue;}
    if(source.startsWith('/*',i)) {
      let depth=1;i+=2;
      while(depth && i<source.length) {if(source.startsWith('/*',i)){depth++;i+=2;}else if(source.startsWith('*/',i)){depth--;i+=2;}else i++;}
      if(depth) throw Error('Unclosed Rust comment');continue;
    }
    const raw=source.slice(i).match(/^(?:br|cr|r)(#*)"/);
    if(raw) {const start=i+raw[0].length;const end=source.indexOf('"'+raw[1],i+raw[0].length);if(end<0)throw Error('Unclosed raw literal');i=end+1+raw[1].length;literals.set(tokens.length,{raw:true,value:source.slice(start,end)});tokens.push('LITERAL');continue;}
    if(source[i]==='"') {
      const start=i;i++;let closed=false;while(i<source.length){if(source[i]==='\\'){i+=2;}else if(source[i++]==='"'){closed=true;break;}}
      if(!closed)throw Error('Unclosed literal');literals.set(tokens.length,{raw:false,value:source.slice(start,i)});tokens.push('LITERAL');continue;
    }
    const char=source.slice(i).match(/^'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\])'/u);
    if(char){i+=char[0].length;tokens.push('LITERAL');continue;}
    const word=source.slice(i).match(/^(?:r#)?[A-Za-z_][A-Za-z_0-9]*/);
    if(word){tokens.push(word[0]);i+=word[0].length;continue;}
    tokens.push(source[i++]);
  }
  return {tokens,literals};
}
export function publicDeclarations(source:string):string[] {
  const {tokens}=rustTokens(source);
  const result:string[]=[];
  for(let n=0;n<tokens.length;n++) {
    if(tokens[n]!=='pub')continue;
    let k=n+1;
    if(tokens[k]==='(') { // pub(crate), pub(super), pub(in ...) are not exported.
      let depth=1;k++;while(k<tokens.length&&depth){if(tokens[k]==='(')depth++;if(tokens[k]===')')depth--;k++;}
      if(depth)throw Error('Malformed visibility');n=k-1;continue;
    }
    while((['unsafe','async','default'].includes(tokens[k]) && tokens[k+1]!==':') || (tokens[k]==='const' && ['unsafe','async','extern','fn'].includes(tokens[k+1])))k++;
    if(tokens[k]==='extern'){k++;if(tokens[k]==='LITERAL')k++;}
    if(tokens[k]==='const'&&tokens[k+1]==='fn')k++;
    const kind=tokens[k++];
    if(kind==='use') {
      const start=k;while(k<tokens.length&&tokens[k]!==';')k++;
      if(k===tokens.length)throw Error('Unclosed public reexport');
      result.push('use:'+tokens.slice(start,k).join(' '));n=k;continue;
    }
    if(kind==='static'&&tokens[k]==='mut')k++;
    if(['struct','enum','trait','type','const','static','fn','mod','union'].includes(kind)) {
      if(!/^(?:r#)?[A-Za-z_][A-Za-z_0-9]*$/.test(tokens[k]??''))throw Error('Unsupported public declaration');
      result.push(kind+':'+tokens[k]);continue;
    }
    if(tokens[k]===':'&&/^[A-Za-z_][A-Za-z_0-9]*$/.test(kind)) {result.push('field:'+kind);continue;}
    throw Error('Unsupported public surface');
  }
  return result;
}

export function checkSurface(actual:any[], expected:any[]) {
  if(JSON.stringify(actual)!==JSON.stringify(expected)) throw Error('Unowned Rust source or public declaration');
}
async function walk(path:string):Promise<string[]> {
  if(await realpath(path)!==resolve(path))throw Error('Unowned symlink directory');
  const entries = await readdir(path,{withFileTypes:true});
  if(entries.some(e=>e.isSymbolicLink()))throw Error('Unowned symlink entry');
  return (await Promise.all(entries.map(e=>e.isDirectory()?walk(resolve(path,e.name)):[resolve(path,e.name)]))).flat();
}
export function javascriptDependencies(pkg:any) {
  return Object.fromEntries(['dependencies','devDependencies','optionalDependencies','peerDependencies'].map(group=>[group,Object.fromEntries(Object.entries(pkg[group] ?? {}).sort(([a],[b])=>a.localeCompare(b)))]));
}
export function checkJavascriptDependencies(pkg:any,expected:any) {
  if(JSON.stringify(javascriptDependencies(pkg))!==JSON.stringify(expected))throw Error('JavaScript dependency boundary differs');
}
export async function scanSurface(root:string,manifest:string) {
  const actual=[];
  for(const path of (await walk(resolve(root,dirname(manifest),'src'))).filter(f=>f.endsWith('.rs')).sort()) {
    if(await realpath(path)!==path)throw Error('Unowned symlink source');
    const source=await readFile(path,'utf8');
    const pathReferences=[];
    const {tokens,literals}=rustTokens(source);
    for(let n=0;n<tokens.length;n++) {
      if(tokens[n]!=='#' || tokens[n+1]!=='[')continue;
      let depth=1,end=n+2;
      while(end<tokens.length&&depth){if(tokens[end]==='[')depth++;if(tokens[end]===']')depth--;end++;}
      if(depth)throw Error('Malformed Rust attribute');
      if(!['path','cfg_attr'].includes(tokens[n+2].replace(/^r#/,''))){n=end-1;continue;}
      for(let k=n+2;k<end-1;k++) {
        if(tokens[k].replace(/^r#/,'')!=='path')continue;
        if(tokens[k+1]!=='='){if(k===n+2)throw Error('Unsupported module path');continue;}
        const literal=literals.get(k+2);if(!literal)throw Error('Unsupported module path');
        let value:string;try{value=literal.raw?literal.value:JSON.parse(literal.value);}catch{throw Error('Unsupported path escape');}
        const target=resolve(dirname(path),value);
      if(!target.startsWith(root+'/') || await realpath(target)!==target)throw Error('Unowned module path');
        pathReferences.push(target.slice(root.length+1));
      }
      n=end-1;
    }
    actual.push({path:path.slice(root.length+1),publicDeclarations:publicDeclarations(source),pathReferences});
  }
  return actual;
}
export async function checkRepository(root:string,cargo:string) {
  const map=JSON.parse(await readFile(resolve(root,'docs/helix/02-design/module-boundaries.json'),'utf8'));
  const process=Bun.spawn([cargo,'metadata','--no-deps','--format-version','1','--locked','--offline'],{cwd:root,stdout:'pipe',stderr:'pipe'});
  const [raw,_stderr,code]=await Promise.all([new Response(process.stdout).text(),new Response(process.stderr).text(),process.exited]);
  if(code!==0) throw Error('Cargo metadata failed'); // do not echo host paths or stderr
  const metadata=JSON.parse(raw); checkGraph(metadata,map,root);
  for (const p of map.packages) {
    const actual=metadata.packages.find((a:any)=>a.name===p.name);
    if(resolve(actual.manifest_path)!==resolve(root,p.manifest) || await realpath(actual.manifest_path)!==resolve(root,p.manifest)) throw Error('Package manifest ownership differs');
  }
  for(const p of map.packages) {
    const packageMetadata=metadata.packages.find((a:any)=>a.name===p.name);
    for(const dependency of packageMetadata.dependencies.filter((d:any)=>d.path)) {
      const owned=map.packages.find((a:any)=>a.name===dependency.name);
      if(!owned || await realpath(dependency.path)!==resolve(root,dirname(owned.manifest))) throw Error('Unowned path dependency');
    }
    const actual=await scanSurface(root,p.manifest);
    checkSurface(actual,p.sources);
  }
  for(const area of map.hostAreas) {
    const actual=(await Promise.all(area.roots.map((path:string)=>walk(resolve(root,path))))).flat().filter((f:string)=>area.extensions.some((e:string)=>f.endsWith(e))).map((f:string)=>f.slice(root.length+1)).concat(area.configs ?? []).sort();
    for(const file of actual) {if(await realpath(resolve(root,file))!==resolve(root,file))throw Error('Unowned host symlink source');}
    if(JSON.stringify(actual)!==JSON.stringify(area.sources))throw Error('Unowned host tooling source');
  }
  for(const [file,expected] of Object.entries(map.javascriptDependencies)) {
    const pkg=JSON.parse(await readFile(resolve(root,file),'utf8'));
    checkJavascriptDependencies(pkg,expected);
  }
  return map.packages.length;
}
if(import.meta.main) {
  try {const count=await checkRepository(resolve(import.meta.dir,'..'),process.env.WEFT_CARGO ?? ''); console.log(`Module boundaries: ${count} workspace packages checked`);}
  catch {console.error('Module boundary check failed');process.exitCode=1;}
}
