"""Read-only exact-main current producer input gate; never resolves or builds."""
from pathlib import Path
import hashlib,json,os,re,subprocess,sys,tomllib

def need(v,message):
 if not v:raise ValueError(message)
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def verify(root):
 need(not sys.flags.optimize,'Producer guards require unoptimized Python')
 R=os.environ['GITHUB_SHA'];need(re.fullmatch('[0-9a-f]{40}',R) is not None,'Malformed final R')
 need(os.environ['GITHUB_REF']=='refs/heads/main' and os.environ['GITHUB_EVENT_NAME']=='workflow_dispatch','Exact-main explicit dispatch required')
 need(os.environ['GITHUB_REPOSITORY']=='DocumentDrivenDX/weft','Wrong repository')
 need(os.environ.get('SOURCE_SHA',R)==R,'Caller source differs from exact workflow R')
 git=lambda *a:subprocess.check_output(['git','-C',str(root),*a],text=True).strip()
 need(git('rev-parse','HEAD')==R,'Checkout differs from workflow R')
 need(not git('status','--porcelain','--untracked-files=all'),'Source checkout is dirty')
 e=json.loads(Path(__file__).with_name('current-inputs-0457.json').read_text())
 need(sha(root/'Cargo.lock')==e['CargoLockSHA256'],'Frozen Cargo.lock differs')
 lock=tomllib.loads((root/'Cargo.lock').read_text());ids=sorted((p['name'],p['version'],p['source']) for p in lock['package'] if p.get('source','').startswith('git+'))
 need(ids==sorted((p['name'],p['version'],p['sourceID']) for p in e['lockGitPackages']),'Canonical S Git identities differ')
 for item in e['resources']:need(sha(root/item['path'])==item['sha256'],'Canonical S resource differs')
 core=tomllib.loads((root/'crates/weft-core/Cargo.toml').read_text());pins=[]
 for section in ['dependencies','build-dependencies']:
  pins.extend(v for k,v in core.get(section,{}).items() if k in ['umf-core','umf-codegen'])
 patches=tomllib.loads((root/'Cargo.toml').read_text())['patch']['crates-io']
 pins.extend(v for v in patches.values() if isinstance(v,dict) and v.get('git')=='https://github.com/DocumentDrivenDX/umf.git')
 need(len(pins)==8 and all(p['git']=='https://github.com/DocumentDrivenDX/umf.git' and p['rev']==e['canonicalSourceID'].split('?rev=')[1].split('#')[0] and 'path' not in p for p in pins),'Exact2 dependencies+6 patches differ')
 project=tomllib.loads((root/'crates/weft-python/pyproject.toml').read_text());crate=tomllib.loads((root/'crates/weft-python/Cargo.toml').read_text())
 need(project['tool']['maturin']['features']==['ashlar-sdk-ordered-page-candidate', 'raw-postgresql-reads', 'ashlar-databricks-paths-keys', 'ashlar-databricks-qualified', 'truss-postgresql-pg18', 'current-postgresql-scalar-candidate', 'current-databricks-scalar-candidate', 'current-query-refinement-candidate', 'current-presence-graph-requirements-candidate'],'Frozen wheel feature selection differs')
 need(project['project']['name']=='umf-weft' and project['project']['version']==crate['package']['version']=='0.4.57','Current distribution/version differs')
 return dict(sourceCommit=R,sourceTree=git('rev-parse','HEAD^{tree}'),canonicalUmf=e['canonicalSourceID'],cargoLockSha256=e['CargoLockSHA256'],resources=9,features=project['tool']['maturin']['features'])
if __name__=='__main__':print(json.dumps(verify(Path(__file__).resolve().parents[2])))
