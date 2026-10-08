"""Join engine-pinned registrations to retained native-tested compiler artifacts.
@covers US-002-AC2 @covers US-006-AC1 @covers US-006-AC3
Does not promote candidate support or execute SQL.
"""
import copy,gzip,hashlib,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
E=ROOT/'docs/helix/04-build/evidence'
OUT=Path(os.environ.get('WEFT_REGISTRATION_OUT','/private/tmp/weft-b007-registration-join'))
BINARY=Path(os.environ.get('WEFT_REGISTRATION_BINARY','/private/tmp/weft-b007-parser-target/debug/examples/compile_native_review'))
sha=lambda b:hashlib.sha256(b).hexdigest()
def load(p):return json.loads(gzip.decompress(p.read_bytes()) if p.suffix=='.gz' else p.read_bytes())
def corpus():
 cases=load(E/'B-007-current-initial-hosts/cases.json.gz')
 reports=load(E/'B-007-current-initial-hosts/cli-reports.json.gz')
 expected={r['id']:json.loads(r['raw']) for r in reports}
 for c in cases:c['response']=expected[c['id']]
 for line in gzip.decompress((E/'B-007-decimal-domains-native/compile-artifacts.jsonl.gz').read_bytes()).decode().splitlines():
  a=json.loads(line);a['id']='ashlar-decimal-'+a['id'];cases.append(a)
 assert len(cases)==len({c['id'] for c in cases})==1025
 return cases
def check(before,after,backend):
 assert before['status']==after['status'],(before['status'],after)
 if before['status']=='blocked':
  assert before==after,(before,after)
  return
 profile='pg17.9-native-review' if backend=='truss.postgresql' else 'dbsql2026.39-native-review'
 obligation_id='truss.nativeProfile' if backend=='truss.postgresql' else 'ashlar.nativeProfile'
 assert after['backend']==dict(before['backend'],backendVersion='0.1.0-native-review',targetProfile=profile)
 for k,v in before.items():
  if k not in {'backend','qualification','obligations','targetContext'}:assert after[k]==v,k
 assert set(before)==set(after)
 settings=({'server_encoding':'UTF8','client_encoding':'UTF8','standard_conforming_strings':'on','transaction_isolation':'repeatable read','lc_collate':'C','lc_ctype':'C','comparison':'explicit C','arithmetic':'exact-or-error'} if backend=='truss.postgresql' else {'ansi_mode':True,'comparison':'explicit UTF8_BINARY','arithmetic':'ANSI exact-or-error','resultTransport':'JSON_ARRAY exact STRING','warehouseBuild':{'u_build_hash':'15b447529a1f55ca1ad8c73a89b8d196f6ed4904','r_build_hash':'3909d148af5cb6b560624c9255f5a727f7a17a4c'}})
 assert after['targetContext']=={'id':profile,'engine':before['targetContext']['engine'],'engineVersion':'17.9' if backend=='truss.postgresql' else '2026.39','sessionSettings':settings,'storageLayoutRevision':'weft-truss-fixtures/0.1' if backend=='truss.postgresql' else 'ashlar-delta/0.3','publicationRevision':'host-verified-native-review/0.1'}
 old=before['obligations'];new=after['obligations']
 assert all(o in new for o in old)
 extra=[o for o in new if o not in old];assert len(extra)==1 and extra[0]['id']==obligation_id
 profile_obligation=extra[0]
 assert profile_obligation['owner']=='host' and profile_obligation['failureCode']=='WFT-OBLIGATION'
 assert profile_obligation['parameters']['targetProfile']==profile
 assert profile_obligation['parameters']['engineVersion']==('17.9' if backend=='truss.postgresql' else '2026.39')
 assert len(profile_obligation['parameters']['requirements'])==3
 assert profile_obligation['parameters']['settings']==settings
 a=before['qualification'];b=after['qualification']
 assert b['status']==a['status']=='candidate' and b['evidence']==a['evidence']==[]
 assert b['assumptions']==a['assumptions']
 assert len(a['operations'])==len(b['operations'])
 for x,y in zip(a['operations'],b['operations'],strict=True):
  assert x['assessment']==y['assessment']
  oldd=x['declaration'];newd=y['declaration']
  assert newd['status']=='candidate' and newd['evidence']==[]
  assert newd['targetProfiles']==[profile]
  assert newd['id']==oldd['id'] and newd['languageProfiles']==oldd['languageProfiles']
  assert newd['constraints']==oldd['constraints']+['Exact engine/settings review scope; candidate opt-in still required']
  assert newd['obligations']==oldd['obligations']+[profile_obligation]
  if backend=='ashlar.databricks':
   assert newd['logicalDomain']==oldd['logicalDomain'] and newd['resultDomain']==oldd['resultDomain']
  else:
   assert newd['logicalDomain']=={'subset':'admitted weft-sql/0.1 scalar and weft-sql/0.2 application plans','storageHomes':['JSONB properties','typed scalar rows','complete compound row trees'],'meaning':'only binding-admitted type, presence, key, relationship and comparison semantics','nativeNull':'refused for selected non-nullable values'}
   assert newd['resultDomain']=={'transport':'exact text; compound numeric values are base-ten strings','arithmetic':'PostgreSQL exact NUMERIC or error','emptyAggregates':{'sum':'nullable','count':'non-null integer'},'qualification':'candidate native review'}
def main():
 cases=corpus();requests=[]
 for c in cases:
  request=copy.deepcopy(c['request']);backend=request['target']['backendId']
  assert backend in {'truss.postgresql','ashlar.databricks'}
  request['target']['backendVersion']='0.1.0-native-review'
  request['target']['targetProfile']='pg17.9-native-review' if backend=='truss.postgresql' else 'dbsql2026.39-native-review'
  requests.append(request)
 raw=''.join(json.dumps(r,separators=(',',':'))+'\n' for r in requests)
 run=subprocess.run([str(BINARY)],input=raw,text=True,capture_output=True,cwd=ROOT)
 assert run.returncode==0,run.stderr
 responses=[json.loads(l) for l in run.stdout.splitlines()];assert len(responses)==1025
 operations={};compiled=blocked=0;records=[]
 for c,request,response in zip(cases,requests,responses,strict=True):
  check(c['response'],response,request['target']['backendId'])
  if response['status']=='compiled':
   compiled+=1
   backend=request['target']['backendId'];ops=operations.setdefault(backend,set())
   ops.update(o['assessment']['id'] for o in response['qualification']['operations'])
  else:blocked+=1
  records.append({'id':c['id'],'request':request,'response':response})
 OUT.mkdir(parents=True,exist_ok=True)
 with gzip.open(OUT/'joined-artifacts.jsonl.gz','wb') as f:
  f.write(''.join(json.dumps(r,separators=(',',':'))+'\n' for r in records).encode())
 inputs=['docs/helix/04-build/evidence/B-007-current-initial-hosts/cases.json.gz','docs/helix/04-build/evidence/B-007-current-initial-hosts/cli-reports.json.gz','docs/helix/04-build/evidence/B-007-decimal-domains-native/compile-artifacts.jsonl.gz']
 sources=[str(p.relative_to(ROOT)) for base in ['crates/weft-core/src','crates/weft-postgresql/src','crates/weft-databricks/src'] for p in sorted((ROOT/base).rglob('*.rs'))]
 manifests_run=subprocess.run([str(BINARY),'--manifests'],capture_output=True,text=True,cwd=ROOT)
 assert manifests_run.returncode==0,manifests_run.stderr
 manifests=json.loads(manifests_run.stdout)
 gaps={m['backendId']:sorted(set(c['id'] for c in m['capabilities'])-operations[m['backendId']]) for m in manifests}
 (OUT/'manifests.json').write_text(json.dumps(manifests,indent=2)+'\n')
 summary=dict(unobservedDeclaredOperations=gaps,manifestSha256=sha((OUT/'manifests.json').read_bytes()),status='passed',cases=1025,compiled=compiled,blocked=blocked,operationIds={k:sorted(v) for k,v in operations.items()},binarySha256=sha(BINARY.read_bytes()),sourceHashes={p:sha((ROOT/p).read_bytes()) for p in sources},inputHashes={p:sha((ROOT/p).read_bytes()) for p in inputs},artifactSha256=sha((OUT/'joined-artifacts.jsonl.gz').read_bytes()),harnessSha256=sha(Path(__file__).read_bytes()),scope='NativeReview equivalence to retained native-tested artifacts. Exact SQL/guards/parameters/decoders/pins and previous obligations preserved; only declared profile metadata and host obligation differ. No new SQL execution, native-engine recapture or supported promotion.')
 (OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
if __name__=='__main__':main()
