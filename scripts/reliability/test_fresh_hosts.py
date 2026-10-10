"""@covers US-008-AC7 Actual helper isolation and receipt corruption refusals."""
import copy,hashlib,json,os,pathlib,subprocess,sys,tempfile,unittest,zipfile
from unittest.mock import patch
from reliability.fresh_hosts import validate_reports
ROOT=pathlib.Path(__file__).resolve().parents[2]
class FreshHostTests(unittest.TestCase):
 def test_optimized_helpers_refuse_before_qualification(self):
  for name in ('prepare-qualified-host-corpus.py','qualified-host-python.py','host-resources.py'):
   result=subprocess.run([sys.executable,'-O',str(ROOT/'tests/qualify-and-evolve'/name)],capture_output=True,text=True)
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
   values={'cli-summary.json':{'status':'passed','cases':2181},'python-summary.json':{'status':'passed','cases':2181,'byteParity':True,'subprocessDisabled':True,'nativeModule':str(native),'extensionSha256':digest},'browser-summary.json':{'cases':2181,'byteParity':True,'wasmSha256':'wasm'},'resource-summary.json':{'status':'passed','cases':7,'extensionSha256':digest},'security-summary.json':{'status':'passed','cases':13,'securityCases':6,'resourceCases':7,'libraryByteParity':True,'cliResponseParityCases':12,'cliInputLimitRefusals':1,'extensionSha256':digest},'browser-resource-security-summary.json':{'status':'passed','cases':13,'byteParity':True,'wasmSha256':'wasm'},'cli-reports.json':[{'id':str(i),'raw':raw} for i in range(2181)],'python-receipts.json':{'cases':copy.deepcopy(rows)},'browser-receipts.json':{'cases':copy.deepcopy(rows)}}
   def write(data):
    for name,value in data.items():(out/name).write_text(json.dumps(value))
   write(values)
   with patch.object(sys,'prefix',str(out)):
    self.assertEqual(validate_reports(out,wheel)['python-summary.json']['cases'],2181)
    changes=[('python-summary.json','cases',0),('python-summary.json','byteParity',False),('security-summary.json','securityCases',0),('security-summary.json','cliInputLimitRefusals',0),('security-summary.json','libraryByteParity',False),('resource-summary.json','extensionSha256','stale'),('browser-summary.json','wasmSha256','stale')]
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
if __name__=='__main__':unittest.main()
