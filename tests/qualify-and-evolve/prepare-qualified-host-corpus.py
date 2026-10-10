"""Reconcile fresh public runtime artifacts to qualified native-registration receipts.
@covers US-005-AC1 @covers US-005-AC2 @covers US-005-AC3
"""
import gzip,hashlib,json,os,subprocess
import sys
if sys.flags.optimize:raise RuntimeError('Qualification requires nonoptimized Python')
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];SOURCE=ROOT/'docs/helix/04-build/evidence/B-007-qualified-registration/compiled-artifacts.jsonl.gz'
OUT=Path(os.environ.get('WEFT_QUALIFIED_HOST_OUT','/private/tmp/weft-b007-qualified-hosts'));OUT.mkdir(parents=True,exist_ok=True)
BINARY=Path(os.environ.get('WEFT_QUALIFIED_PUBLIC_BINARY','/private/tmp/weft-b007-parser-target/debug/examples/compile_public_batch'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
records=[json.loads(l) for l in gzip.decompress(SOURCE.read_bytes()).decode().splitlines()];assert len(records)==len({r['id'] for r in records})==2181
sys.path.insert(0,str(ROOT/'scripts'))
from reliability.fresh_hosts import load_main_baseline
records,receipt=load_main_baseline(ROOT,SOURCE,records)
assert subprocess.run(['git','merge-base','--is-ancestor',receipt['checkpoint'],'HEAD'],cwd=ROOT).returncode==0
binary_sha=sha(BINARY)
run=subprocess.run([str(BINARY)],input=''.join(json.dumps(r['request'],ensure_ascii=False)+'\n' for r in records),text=True,capture_output=True);assert run.returncode==0,run.stderr
raw=run.stdout.splitlines();assert len(raw)==2181
for record,response in zip(records,raw,strict=True):assert json.loads(response)==record['response'],record['id']
assert sha(BINARY)==binary_sha
cases=[{'id':r['id'],'request':r['request']} for r in records];reports=[{'id':r['id'],'raw':s} for r,s in zip(records,raw,strict=True)]
(OUT/'cases.jsonl').write_text(''.join(json.dumps(c,ensure_ascii=False)+'\n' for c in cases));(OUT/'cli-reports.json').write_text(json.dumps(reports,ensure_ascii=False)+'\n')
summary=dict(status='passed',cases=2181,historicalNativeArtifactSha256=sha(SOURCE),mainBaselineSha256=receipt['baselineSha256'],mainCheckpoint=receipt['checkpoint'],historicalIdenticalOutputs=receipt['historicalIdenticalOutputs'],nativeRequalificationOpen=receipt['changedOutputs'],compilerSha256=binary_sha,harnessSha256=sha(Path(__file__)),scope='Fresh public qualified runtime matches all2181 pinned-main reference artifacts from unchanged retained fixture requests. Unchanged outputs retain historical native byte correspondence; changed outputs have main compatibility only and native requalification remains open. No SQL execution or metadata normalization.')
(OUT/'cli-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
