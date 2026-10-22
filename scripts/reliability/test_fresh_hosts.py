# SPDX-License-Identifier: MIT OR Apache-2.0
"""@covers US-008-AC7 Actual helper isolation and receipt corruption refusals."""
import copy,hashlib,json,os,pathlib,subprocess,sys,tempfile,unittest,zipfile
from unittest.mock import patch
from reliability.fresh_hosts import BASELINE,validate_reports
ROOT=pathlib.Path(__file__).resolve().parents[2]
class FreshHostTests(unittest.TestCase):
 def test_security_blocked_qualification_refuses_executable_members(self):
  from reliability.fresh_cases import validate_blocked_security
  version='weft-security-compile/0.2.0';code='WFT-SECURITY-BACKEND-REQUIRED'
  blocked={'interfaceVersion':version,'status':'blocked','diagnostics':[{'code':code,'severity':'error'}]}
  validate_blocked_security(blocked,version,code)
  for member in ('lowering','ownerPlan','resultContract','nativeAdmission','sql'):
   leaked=copy.deepcopy(blocked);leaked[member]={'execution':{'sql':'SELECT secret FROM protected'}}
   with self.assertRaises(RuntimeError):validate_blocked_security(leaked,version,code)
 def test_optimized_helpers_refuse_before_qualification(self):
  helpers=[ROOT/'tests/qualify-and-evolve'/name for name in ('prepare-qualified-host-corpus.py','qualified-host-python.py','host-resources.py')]
  helpers.extend([ROOT/'docs/helix/04-build/evidence/main-integration-20261010/baseline-generator.py',ROOT/'docs/helix/04-build/evidence/main-ed2-integration-20261010/baseline/baseline-generator.py',ROOT/BASELINE/'baseline-generator.py'])
  for helper in helpers:
   result=subprocess.run([sys.executable,'-O',str(helper)],cwd=ROOT,capture_output=True,text=True)
   self.assertNotEqual(result.returncode,0);self.assertEqual(result.stdout,'');self.assertIn('Qualification requires nonoptimized Python',result.stderr)
 def test_actual_isolated_helper_rejects_wrong_compiler_under_optimized_environment(self):
  with tempfile.TemporaryDirectory() as work:
   work=pathlib.Path(work);binary=work/'wrong-compiler';binary.write_text('#!'+sys.executable+'\nimport sys\nfor line in sys.stdin:print("{}")\n');binary.chmod(0o700)
   env=dict(os.environ,PYTHONOPTIMIZE='1',PYTHONPATH=str(work),WEFT_QUALIFIED_PUBLIC_BINARY=str(binary),WEFT_QUALIFIED_HOST_OUT=str(work/'out'))
   (work/'weft.py').write_text('raise RuntimeError("shadow module loaded")')
   isolated=subprocess.run([sys.executable,'-I','-c','import sys,json;print(json.dumps([sys.flags.optimize,sys.flags.isolated,sys.path]))'],env=env,capture_output=True,text=True)
   flags=json.loads(isolated.stdout);self.assertEqual(flags[:2],[0,1]);self.assertNotIn(str(work),flags[2])
   result=subprocess.run([sys.executable,'-I',str(ROOT/'tests/qualify-and-evolve/prepare-qualified-host-corpus.py')],env=env,capture_output=True,text=True)
   self.assertNotEqual(result.returncode,0);self.assertFalse((work/'out/cli-summary.json').exists())
 def test_receipt_parity_counts_identifiers_and_wheel_corruptions_refuse(self):
  with tempfile.TemporaryDirectory() as work:
   out=pathlib.Path(work);native=out/'weft.abi3.so';native.write_bytes(b'fresh-native');digest=hashlib.sha256(native.read_bytes()).hexdigest();wheel=out/'fresh.whl'
   with zipfile.ZipFile(wheel,'w') as package:package.writestr('weft/weft.abi3.so',native.read_bytes())
   raw='{}';response_sha=hashlib.sha256(raw.encode()).hexdigest();rows=[{'id':str(i),'actualSha256':response_sha,'expectedSha256':response_sha} for i in range(2181)]
   baseline=json.loads((ROOT/BASELINE/'baseline.json').read_bytes())
   values={'cli-summary.json':{'status':'passed','cases':2181,'retiredInputRefusals':2181,'freshCompiledInputs':0,'currentCoreVersion':'0.10.0','mainBaselineSha256':baseline['baselineSha256'],'mainCheckpoint':baseline['checkpoint']},'python-summary.json':{'status':'passed','cases':2181,'byteParity':True,'subprocessDisabled':True,'nativeModule':str(native),'extensionSha256':digest},'browser-summary.json':{'cases':2181,'byteParity':True,'wasmSha256':'wasm'},'resource-summary.json':{'status':'passed','cases':7,'extensionSha256':digest},'security-summary.json':{'status':'passed','cases':17,'securityCases':10,'resourceCases':7,'libraryByteParity':True,'cliResponseParityCases':16,'cliInputLimitRefusals':1,'extensionSha256':digest},'browser-resource-security-summary.json':{'status':'passed','cases':17,'byteParity':True,'wasmSha256':'wasm'},'cli-reports.json':[{'id':str(i),'raw':raw} for i in range(2181)],'python-receipts.json':{'cases':copy.deepcopy(rows)},'browser-receipts.json':{'cases':copy.deepcopy(rows)}}
   def write(data):
    for name,value in data.items():(out/name).write_text(json.dumps(value))
   write(values)
   with patch.object(sys,'prefix',str(out)):
    self.assertEqual(validate_reports(out,wheel)['python-summary.json']['cases'],2181)
    changes=[('cli-summary.json','retiredInputRefusals',0),('cli-summary.json','freshCompiledInputs',1),('cli-summary.json','mainCheckpoint','wrong'),('python-summary.json','cases',0),('python-summary.json','byteParity',False),('security-summary.json','securityCases',0),('security-summary.json','cliInputLimitRefusals',0),('security-summary.json','libraryByteParity',False),('resource-summary.json','extensionSha256','stale'),('browser-summary.json','wasmSha256','stale')]
    for name,key,value in changes:
     corrupt=copy.deepcopy(values);corrupt[name][key]=value;write(corrupt)
     with self.assertRaises(RuntimeError):validate_reports(out,wheel)
    for mode in ('missing','duplicate','wrong_hash'):
     corrupt=copy.deepcopy(values);records=corrupt['python-receipts.json']['cases']
     if mode=='missing':records.pop()
     elif mode=='duplicate':records[0]['id']=records[1]['id']
     else:records[0]['actualSha256']='wrong'
     write(corrupt)
     with self.assertRaises(RuntimeError):validate_reports(out,wheel)
    write(values);native.write_bytes(b'stale-native')
    with self.assertRaises(RuntimeError):validate_reports(out,wheel)
class MainBaselineTests(unittest.TestCase):
 def test_main_compatibility_cannot_change_requests_or_hide_native_gaps(self):
  from reliability.fresh_hosts import reconcile_main_baseline
  historical=[{'id':str(i),'request':{'sql':'SELECT'},'response':{'diagnostics':[{'phase':'sql'}],'nullable':False}} for i in range(2181)]
  current=copy.deepcopy(historical);current[0]['response']['diagnostics'][0]['phase']='input'
  receipt={'status':'passed','requests':2181,'requestsUnchanged':True,'changedOutputs':1,'historicalIdenticalOutputs':2180}
  differences=[{'id':'0','differences':[{'path':'/diagnostics/0/phase','historical':'sql','main':'input'}],'qualification':'Current main compatibility and fresh cross-host parity only; native requalification remains open.'}]
  reconcile_main_baseline(historical,current,receipt,differences)
  for mode in ('request','identifier','duplicate','count','gap','missing-difference','wrong-difference','boolean-number','missing-null-key'):
   records=copy.deepcopy(current);summary=copy.deepcopy(receipt);changes=copy.deepcopy(differences)
   if mode=='request':records[0]['request']['sql']='CHANGED'
   elif mode=='identifier':records[0]['id']='unowned'
   elif mode=='duplicate':records[0]['id']=records[1]['id']
   elif mode=='count':records.pop()
   elif mode=='gap':summary['changedOutputs']=0
   elif mode=='missing-difference':changes=[]
   elif mode=='boolean-number':records[0]['response']['nullable']=0
   elif mode=='missing-null-key':records[0]['response']['unaccounted']=None
   else:changes[0]['differences'][0]['main']='wrong'
   with self.assertRaises(RuntimeError):reconcile_main_baseline(historical,records,summary,changes)


class DeclarationBaselineTests(unittest.TestCase):
 def test_reviewed_change_is_exact_additive_ashlar_metadata_only(self):
  import importlib.util
  spec=importlib.util.spec_from_file_location('baseline_refresh',ROOT/BASELINE/'baseline-generator.py');module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
  source={'backend':{'backendId':'ashlar.databricks','backendVersion':'0.1.0-qualified','interfaceVersion':'weft-backend/0.2.0','targetProfile':'dbsql2026.39-qualified'},'sql':'SELECT exact_original','qualification':{'operations':[{'declaration':{'languageProfiles':copy.deepcopy(module.OLD),'unknown':{'retained':True}}}]}}
  original=copy.deepcopy(source);expected=copy.deepcopy(source);expected['qualification']['operations'][0]['declaration']['languageProfiles'].append(module.ADDITION)
  self.assertEqual(module.advance_declarations(source),expected);self.assertEqual(source,original)
  for key,value in [('backendId','truss.postgresql'),('backendVersion','future'),('targetProfile','future')]:
   other=copy.deepcopy(source);other['backend'][key]=value;self.assertEqual(module.advance_declarations(other),other)
  for profiles in [[],module.OLD+[module.ADDITION],[module.OLD[0]],None]:
   other=copy.deepcopy(source);other['qualification']['operations'][0]['declaration']['languageProfiles']=profiles
   with self.assertRaises(RuntimeError):module.advance_declarations(other)
 def test_current_baseline_preserves_requests_and_historical_native_gaps(self):
  import gzip
  from reliability.fresh_hosts import load_main_baseline
  path=ROOT/'docs/helix/04-build/evidence/B-007-qualified-registration/compiled-artifacts.jsonl.gz'
  historical=[json.loads(line) for line in gzip.decompress(path.read_bytes()).decode().splitlines()]
  records,receipt=load_main_baseline(ROOT,path,historical)
  self.assertEqual(len(records),2181);self.assertEqual(receipt['changedOutputs'],965);self.assertEqual(receipt['historicalIdenticalOutputs'],1216)
  self.assertEqual(receipt['reviewedDeclarationCases'],964);self.assertFalse(receipt['independentlyBuiltReference'])
  reviewed=ROOT/BASELINE/'reviewed-declaration-differences.json.gz'
  self.assertEqual(hashlib.sha256(reviewed.read_bytes()).hexdigest(),receipt['reviewedDeclarationDifferencesSha256'])



class CurrentCoreAdmissionTests(unittest.TestCase):
 def test_original_retired_inputs_refuse_without_relabeling_or_native_promotion(self):
  import gzip
  from reliability.fresh_hosts import retired_core_expectations
  path=ROOT/BASELINE/'main-reference-artifacts.jsonl.gz'
  original=[json.loads(line) for line in gzip.decompress(path.read_bytes()).decode().splitlines()]
  before=copy.deepcopy(original)
  expected=retired_core_expectations(original)
  self.assertEqual(original,before)
  self.assertEqual(len(expected),2181)
  self.assertTrue(all(r['response']['status']=='blocked' and r['response']['diagnostics'][0]['code']=='WFT-INPUT' and 'sql' not in r['response'] for r in expected))
  self.assertEqual([r['request'] for r in expected],[r['request'] for r in original])
  changed=copy.deepcopy(original[:1]);changed[0]['request']['modules'][0]['pin']['umfVersion']='0.10.0'
  with self.assertRaises(RuntimeError):retired_core_expectations(changed)

if __name__=='__main__':unittest.main()
