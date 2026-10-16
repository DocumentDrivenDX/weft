"""Compile actual old APIs with old/new types; no SQL or native execution."""
import argparse,json,pathlib,subprocess,os,shutil
p=argparse.ArgumentParser();p.add_argument('--cargo',required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();root=pathlib.Path(__file__).resolve().parents[2];a.output.mkdir(exist_ok=False)
api={'weft_postgresql':['lower','lower_rows','lower_contributors','lower_original_sum','lower_bounded_rows','lower_bounded_contributors','check_rows','check_contributors','check_original_sum','check_bounded_rows','check_bounded_contributors'],'weft_databricks':['lower','lower_original_sum','lower_bounded_contributors','lower_spark_contributors','check_contributors','check_original_sum','check_bounded_contributors','check_spark_contributors']}
calls=[f'let _={crate}::current_scalar::{name}(c,""'+(',""' if name.startswith('check') else '')+');' for crate,names in api.items() for name in names]
assert len(calls)==19
results=[]
for label,features,body,expected in [
 ('old-positive',['current-presence-context-candidate'],'use weft_core::current_query_operands::CurrentQueryOperands;fn test<\'a>(c:&\'a CurrentQueryOperands<\'a>){'+''.join(calls)+'}fn main(){}',None),
 ('new-rejected',['current-presence-context-candidate'],'use weft_core::current_presence::CurrentPresencePreparation;fn test<\'a>(c:&\'a CurrentPresencePreparation<\'a>){'+''.join(calls)+'}fn main(){}','E0308'),
 ('feature-off',[],'use weft_core::current_presence::CurrentPresencePreparation;fn main(){}','E0432'),
 ('lineage-only',['tagged-current-projection-candidate'],'use weft_core::current_presence::CurrentPresencePreparation;fn main(){}','E0432')]:
 work=a.output/label;(work/'src').mkdir(parents=True)
 manifest='[package]\nname="presence-boundary-probe"\nversion="0.0.0"\nedition="2021"\n[dependencies]\n'
 manifest+=f'weft-core={{path={json.dumps(str(root/"crates/weft-core"))},features={json.dumps(features)}}}\n'
 for name,flags in [('weft-postgresql',['bounded-current-string-candidate']),('weft-databricks',['bounded-current-string-candidate','current-spark-scalar-candidate'])]:manifest+=f'{name}={{path={json.dumps(str(root/"crates"/name))},features={json.dumps(flags)}}}\n'
 manifest+='[patch.crates-io]\nahash={path='+json.dumps(str(root/'vendor/ahash'))+'}\n';(work/'Cargo.toml').write_text(manifest);(work/'Cargo.lock').write_bytes((root/'Cargo.lock').read_bytes());(work/'src/main.rs').write_text(body)
 run=subprocess.run([a.cargo,'check','--manifest-path',str(work/'Cargo.toml'),'--offline','--message-format=json'],capture_output=True,text=True);(work/'stdout.jsonl').write_text(run.stdout);(work/'stderr.log').write_text(run.stderr)
 errors=[]
 for line in run.stdout.splitlines():
  try:value=json.loads(line)
  except ValueError:continue
  if value.get('reason')=='compiler-message' and value['message']['level']=='error':errors.append(value['message'])
 if expected is None:assert run.returncode==0 and not errors,label
 else:
  assert run.returncode!=0 and errors,label
  assert all(e.get('code',{}).get('code')==expected for e in errors),(label,[e.get('code')for e in errors])
  if expected=='E0308':assert len(errors)==19 and all('CurrentQueryOperands' in (e.get('rendered')or'') and 'CurrentPresencePreparation' in (e.get('rendered')or'')for e in errors)
 results.append({'id':label,'exitCode':run.returncode,'expectedDiagnostic':expected,'errors':len(errors)})
(a.output/'receipt.json').write_text(json.dumps({'scope':'Actual Rust type/feature rejection, not execution/authority','oldApiCount':19,'results':results,'nativeAdmission':False},indent=2)+'\n');print(json.dumps({'scenarios':4,'oldApiCount':19,'passed':True}))
