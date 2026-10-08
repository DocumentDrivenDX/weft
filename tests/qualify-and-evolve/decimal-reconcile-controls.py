"""Semantic receipt-corruption controls after valid custody rehashing.
@covers US-006-AC1 @covers US-004-AC3
"""
import copy,gzip,hashlib,json,os,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-007-decimal-domains-native'
artifacts=[json.loads(l) for l in gzip.decompress((BASE/'compile-artifacts.jsonl.gz').read_bytes()).decode().splitlines()]
receipts=[json.loads(l) for l in gzip.decompress((BASE/'statements.jsonl.gz').read_bytes()).decode().splitlines()]

def result_row(records,suffix):
    return next(row for r in records for row in r['response']['result']['data_array'] if len(row)==3 and row[1].endswith(suffix))
def change_sum(a,r):result_row(r,'-valid-sum')[2]='0'
def change_guard(a,r):result_row(r,'-invalid-guard')[2]='4'
def change_engine(a,r):
    row=result_row(r,'-valid-sum');v=json.loads(row[0]);v['dbsql_version']='changed';row[0]=json.dumps(v)
def change_parameter(a,r):a[0]['response']['parameters'][0]['value']='untrusted-source'
def change_pin(a,r):a[0]['request']['modules'][0]['pin']['sha256']='0'*64
def change_scale(a,r):a[0]['response']['columns'][0]['logicalType']['facets']['scale']=1
def duplicate_native(a,r):r[-1]=copy.deepcopy(r[-2])
controls=[('rounded-sum',change_sum),('lost-invalid-value',change_guard),('wrong-warehouse',change_engine),('changed-slot',change_parameter),('stale-module',change_pin),('changed-result-scale',change_scale),('duplicate-native-receipt',duplicate_native)]
with tempfile.TemporaryDirectory() as folder:
    folder=Path(folder)
    for name,edit in controls:
        case=folder/name;case.mkdir()
        for fixed in ['summary.json','harness.py','transport.py','compiler-build.log','run.log','source-inputs.json']:
            (case/fixed).write_bytes((BASE/fixed).read_bytes())
        a=copy.deepcopy(artifacts);r=copy.deepcopy(receipts);edit(a,r)
        for file,value in [('compile-artifacts.jsonl.gz',a),('statements.jsonl.gz',r)]:
            raw=('\n'.join(json.dumps(x) for x in value)+'\n').encode()
            (case/file).write_bytes(gzip.compress(raw,mtime=0))
        custody=json.loads((BASE/'custody.json').read_text())
        for file in ['compile-artifacts.jsonl','statements.jsonl']:
            raw=gzip.decompress((case/(file+'.gz')).read_bytes())
            custody['archives'][file]=dict(bytes=len(raw),uncompressedSha256=hashlib.sha256(raw).hexdigest())
        custody['inputHashes']={file:hashlib.sha256((case/file).read_bytes()).hexdigest() for file in custody['inputHashes']}
        (case/'custody.json').write_text(json.dumps(custody))
        env=dict(os.environ,WEFT_DECIMAL_RECONCILE_INPUT=str(case),WEFT_DECIMAL_RECONCILE_OUTPUT=str(case/'result'))
        run=subprocess.run([sys.executable,str(Path(__file__).with_name('reconcile-decimal-domains.py'))],env=env,capture_output=True,text=True)
        assert run.returncode!=0 and 'AssertionError' in run.stderr and 'Traceback' in run.stderr,(name,run.stdout,run.stderr)
        assert not (case/'result'/'summary.json').exists(),name
print(json.dumps(dict(status='passed',corruptionsRejected=len(controls),controls=[n for n,_ in controls],scope='Semantically corrupted archived native receipts with recomputed custody digests refuse. No native execution or support promotion.')))
