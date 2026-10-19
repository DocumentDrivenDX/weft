# SPDX-License-Identifier: MIT OR Apache-2.0
"""Actual pre-import refusal controls; no native authority or SQL.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import argparse,json,pathlib,shutil,subprocess,tempfile
p=argparse.ArgumentParser();p.add_argument('--owner',required=True);p.add_argument('--lineage',required=True);p.add_argument('--designation',required=True);p.add_argument('--unqualified-producer',required=True);p.add_argument('--output',required=True);p.add_argument('--bun',default=shutil.which('bun'));args=p.parse_args()
if not args.bun:raise ValueError('Bun required')
root=pathlib.Path(__file__).resolve().parent;owner=pathlib.Path(args.owner);output=pathlib.Path(args.output)
if output.exists():raise ValueError('Fresh evidence path required')
checks=[]
with tempfile.TemporaryDirectory(prefix='weft-route-refusals-') as tmp:
 tmp=pathlib.Path(tmp);bad=tmp/'source';shutil.copytree(owner,bad,symlinks=True)
 def refuses(name,source,producer,expected,script=root/'value-route-conformance.ts'):
  out=tmp/(name+'.json')
  q=subprocess.run([args.bun,str(script),str(source),args.lineage,producer,str(out)],capture_output=True,timeout=60)
  if not(q.returncode and not q.stdout and expected.encode() in q.stderr and not out.exists()):raise AssertionError(name)
  checks.append({'id':name,'refusedBeforeReport':True,'diagnosticMatched':True})
 refuses('unqualified-producer',owner,args.unqualified_producer,'Original compiler producer mismatch')
 helper=bad/'src/extensions/security/query-value-routes-candidate.ts';original=helper.read_bytes();helper.write_bytes(original+b'\n// Counterfactual source drift.\n')
 refuses('source-drift',bad,args.designation,'Original owner source changed');helper.write_bytes(original)
 extra=bad/'src/extensions/security/unregistered-helper.ts';extra.write_text('throw Error("must not execute");\n')
 refuses('extra-source',bad,args.designation,'Original source inventory mismatch');extra.unlink()
 harness=tmp/'harness';harness.mkdir();shutil.copyfile(root/'value-route-conformance.ts',harness/'value-route-conformance.ts')
 manifest=json.loads((root/'umf-source-manifest.json').read_text());manifest['sourceCommit']='counterfactual';(harness/'umf-source-manifest.json').write_text(json.dumps(manifest))
 refuses('manifest-drift',owner,args.designation,'Original owner manifest changed',harness/'value-route-conformance.ts')
with output.open('x') as f:json.dump({'scope':'Actual pre-import provenance refusals; no report/native authority','controls':len(checks),'temporaryCopiesRemoved':True,'checks':checks},f,indent=2);f.write('\n')
print(json.dumps({'status':'passed','controls':len(checks)}))
