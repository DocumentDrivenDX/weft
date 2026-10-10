"""Full scoped input closure; retained history never becomes current authority."""
import hashlib,json,pathlib,subprocess,re,tomllib
ROOTS=('.',)
FILES=()
OUTPUTS=tuple('docs/helix/04-build/evidence/reliability/'+name for name in ('r6-inputs.json','r6-tests.json','r6-hosts.json','r6-review.json'))+('docs/helix/04-build/evidence/B-007-retained-evidence-replay/summary.json',)
BUILD_OUTPUTS={'.git','node_modules','target','dist','.venv','packages/weft-browser/dist','scripts/reliability/__pycache__','tests/qualify-and-evolve/__pycache__','tests/ashlar-databricks/__pycache__','tests/truss-postgresql/__pycache__','docs/helix/04-build/evidence/B-007-ashlar-warehouse-count-native/__pycache__'}
CHECKPOINT='f81565a1addaa6d2c83561f62d3805d1167233ee'
HISTORICAL_MANIFEST_SHA='5ea7ed12913b9e12def63411e80793a6327257b8621986bcc82ad280af5edfda'
ARCHIVE_ROOTS=('docs/helix/04-build/evidence/','distributions/realizations/')
class CustodyError(Exception):pass

def rust_tokens(source):
 tokens=[];literals={};i=0
 while i<len(source):
  if source[i].isspace():i+=1;continue
  if source.startswith('//',i):
   end=source.find('\n',i);i=len(source) if end<0 else end+1;continue
  if source.startswith('/*',i):
   depth=1;i+=2
   while depth and i<len(source):
    if source.startswith('/*',i):depth+=1;i+=2
    elif source.startswith('*/',i):depth-=1;i+=2
    else:i+=1
   if depth:raise CustodyError()
   continue
  raw=re.match(r'(?:br|cr|r)(#*)"',source[i:])
  if raw:
   start=i+len(raw[0]);end=source.find('"'+raw[1],start)
   if end<0:raise CustodyError()
   literals[len(tokens)]=source[start:end] if raw[0].startswith('r') else None;tokens.append('LITERAL');i=end+1+len(raw[1]);continue
  if source[i]=='"':
   start=i;i+=1;closed=False
   while i<len(source):
    if source[i]=='\\':i+=2
    elif source[i]=='"':i+=1;closed=True;break
    else:i+=1
   if not closed:raise CustodyError()
   try:value=json.loads(source[start:i])
   except ValueError:value=None
   literals[len(tokens)]=value;tokens.append('LITERAL');continue
  char=re.match(r"'(?:\\(?:u\{[0-9a-fA-F_]+\}|x[0-9a-fA-F]{2}|.)|[^'\\])'",source[i:])
  if char:i+=len(char[0]);tokens.append('LITERAL');continue
  word=re.match(r'(?:r#)?[A-Za-z_][A-Za-z_0-9]*',source[i:])
  if word:tokens.append(word[0].removeprefix('r#'));i+=len(word[0]);continue
  tokens.append(source[i]);i+=1
 return tokens,literals

def embedded_inputs(root,path,entries):
 root=root.resolve();path=path.resolve()
 tokens,literals=rust_tokens(path.read_text())
 def owned(base,index,executable=False):
  value=literals.get(index)
  if type(value)!=str or not value or '\x00' in value:raise CustodyError()
  target=(base/value).resolve()
  if not target.is_relative_to(root) or target.relative_to(root).as_posix() not in entries:raise CustodyError()
  if executable and any(target.relative_to(root).as_posix().startswith(prefix) for prefix in ARCHIVE_ROOTS):raise CustodyError()
 for i,token in enumerate(tokens):
  if token in ('include_str','include_bytes','include') and tokens[i+1:i+2]==['as']:raise CustodyError()
  if token in ('include_str','include_bytes','include') and tokens[i+1:i+2]==['!']:
   end=i+4
   if tokens[end:end+1]==[',']:end+=1
   if tokens[i+2:i+4]!=['(','LITERAL'] or tokens[end:end+1]!=[')']:raise CustodyError()
   owned(path.parent,i+3,token=='include')
  if token=='#' and tokens[i+1:i+2]==['[']:
   depth=1;end=i+2
   while end<len(tokens) and depth:
    if tokens[end]=='[':depth+=1
    elif tokens[end]==']':depth-=1
    end+=1
   if depth:raise CustodyError()
   attribute=tokens[i+2:end-1]
   schema=attribute[:4]==['jsonschema',':',':','validator']
   if not schema and any(attribute[k:k+4]==['jsonschema',':',':','validator'] for k in range(len(attribute))):raise CustodyError()
   base=path.parent
   if schema:
    while not (base/'Cargo.toml').is_file():
     if base==root or not base.is_relative_to(root):raise CustodyError()
     base=base.parent
   path_keys=[k for k in range(i+2,end-1) if tokens[k]=='path' and tokens[k+1:k+2]==['=']]
   if path_keys and not schema:
    direct=attribute==['path','=','LITERAL']
    conditional=False
    if attribute[:2]==['cfg_attr','('] and attribute[-1:]==[')']:
     depth=0;comma=None
     for index,part in enumerate(attribute[2:-1],2):
      if part in ('(','[','{'):depth+=1
      elif part in (')',']','}'):depth-=1
      elif part==',' and depth==0:comma=index;break
     conditional=comma is not None and attribute[comma+1:-1]==['path','=','LITERAL']
    if not direct and not conditional:raise CustodyError()
   for k in path_keys:owned(base,k+2,not schema)

def snapshot(root):
 root=root.resolve()
 entries={}
 def add(p):
  name=p.relative_to(root).as_posix()
  if name in OUTPUTS or name=='.DS_Store' or p.suffix=='.pyc':return
  if p.is_symlink() or not p.is_file():raise CustodyError()
  entries[name]=hashlib.sha256(p.read_bytes()).hexdigest()
 def walk(p):
  if p.is_symlink():raise CustodyError()
  if p.is_dir():
   for child in sorted(p.iterdir()):
    if child.relative_to(root).as_posix() not in BUILD_OUTPUTS:walk(child)
  else:add(p)
 for name in FILES:add(root/name)
 for name in ROOTS:walk(root/name)
 for name in list(entries):
  # Frozen source excerpts remain byte-bound inputs, not current build graphs.
  if any(name.startswith(prefix) for prefix in ARCHIVE_ROOTS):continue
  if name.endswith('.rs'):embedded_inputs(root,root/name,entries)
  if pathlib.PurePosixPath(name).name=='Cargo.toml':
   manifest=tomllib.loads((root/name).read_text());targets=([manifest['lib']] if 'lib' in manifest else [])+[v for kind in ('bin','example','test','bench') for v in manifest.get(kind,[])]
   build=manifest.get('package',{}).get('build')
   if type(build)==str:targets.append({'path':build})
   for target in targets:
    if 'path' in target:
     path=((root/name).parent/target['path']).resolve()
     if not path.is_relative_to(root) or path.relative_to(root).as_posix() not in entries:raise CustodyError()
     if any(path.relative_to(root).as_posix().startswith(prefix) for prefix in ARCHIVE_ROOTS):raise CustodyError()
 return {'version':'weft-inputs/1','scope':'Complete compiler/binding/schema/corpus/oracle/test/host/build/lock/config/checker and governing HELIX input closure, including vendor and embedded inputs. Archived evidence/distribution realizations are byte-hashed, not reinterpreted as current build trees. Named generated qualification outputs/build products excluded.','archivalByteOnlyRoots':list(ARCHIVE_ROOTS),'roots':list(ROOTS),'rootFiles':list(FILES),'excludedOutputFiles':list(OUTPUTS),'excludedBuildDirectories':sorted(BUILD_OUTPUTS),'excludedGeneratedSuffixes':['.pyc'],'excludedGeneratedFiles':['.DS_Store'],'files':entries}

def verify(root,manifest):
 actual=snapshot(root)
 if manifest!=actual:raise CustodyError()
 return len(actual['files'])

def historical(root):
 path=root/'docs/helix/04-build/evidence/B-007-workspace-qualified-final/sources.json';raw=path.read_bytes()
 if hashlib.sha256(raw).hexdigest()!=HISTORICAL_MANIFEST_SHA:raise CustodyError()
 hashes=json.loads(raw)
 if len(hashes)!=111:raise CustodyError()
 for name,digest in hashes.items():
  if pathlib.PurePosixPath(name).is_absolute() or '..' in pathlib.PurePosixPath(name).parts:raise CustodyError()
  result=subprocess.run(['git','show',CHECKPOINT+':'+name],cwd=root,capture_output=True)
  if result.returncode or hashlib.sha256(result.stdout).hexdigest()!=digest:raise CustodyError()
 return {'checkpoint':CHECKPOINT,'sourceHashesVerified':len(hashes),'sourceManifestSha256':hashlib.sha256(path.read_bytes()).hexdigest(),'scope':'Historical Git object custody only; current input closure verified separately.'}
