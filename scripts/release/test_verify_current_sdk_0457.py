"""Pure protocol/byte-fixture controls; synthetic modules are never executed."""
import copy,hashlib,io,json,struct,unittest,zipfile
from pathlib import Path
import verify_current_sdk_0457 as gate
R='a'*40;TREE='b'*40;CONTROLLER='c'*64
ACTUAL=Path(__file__).parent/'fixtures/current-sdk-0457/actual42-checks.json'

def digest(b):return hashlib.sha256(b).hexdigest()
def identity(run,path):return dict(repository='DocumentDrivenDX/weft',runId=run,workflowPath=path,sourceCommit=R,workflowCommit=R,headBranch='main',event='workflow_dispatch',conclusion='success')
def fixtures():
 actual=dict(version='0.4.57',sourceCommit=R,wheel='',pinsBeforeAfter={},controls=42,checks=json.loads(ACTUAL.read_text()),nativeAdmission=False,securityReady=False,temporalQualification='held');out=[]
 for target,(tag,machine) in gate.TARGETS.items():
  native=bytearray(64)
  if target=='aarch64-apple-darwin':native[:4]=b'\xcf\xfa\xed\xfe';native[4:8]=machine.to_bytes(4,'little')
  else:native[:6]=b'\x7fELF\x02\x01';native[18:20]=machine.to_bytes(2,'little')
  shim=b'# synthetic byte fixture; no execution\n';stream=io.BytesIO()
  with zipfile.ZipFile(stream,'w') as z:z.writestr('weft/weft.abi3.so',native);z.writestr('weft/__init__.py',shim)
  raw=stream.getvalue();wheel=f'umf_weft-0.4.57-cp39-abi3-{tag}.whl'
  report=copy.deepcopy(actual);report.update(sourceCommit=R,wheel=wheel,pinsBeforeAfter=dict(wheelSha256=digest(raw),nativeModuleSha256=digest(native),shimSha256=digest(shim)))
  report_raw=json.dumps(report,separators=(',',':')).encode()
  packaging=json.dumps(dict(status='passed',version='0.4.57',wheel=wheel,wheelSha256=digest(raw),nativeModuleSha256=digest(native),python='synthetic protocol byte fixture',scope='exact installed archive, RECORD, metadata, ABI tag and notices; native semantics separately gated')).encode()
  e=dict(profile=gate.PROFILE,sourceCommit=R,sourceTree=TREE,distribution='umf-weft',version='0.4.57',target=target,wheel=wheel,wheelSha256=digest(raw),nativeModuleSha256=digest(native),shimSha256=digest(shim),sdkReportSha256=digest(report_raw),packagingReportSha256=digest(packaging),producer=identity(2 if target=='aarch64-apple-darwin' else 1,'.github/workflows/release-hosted-macos.yml' if target=='aarch64-apple-darwin' else '.github/workflows/release.yml'),controllerCodeSha256=CONTROLLER,controller=identity(2 if target=='aarch64-apple-darwin' else 1,'.github/workflows/release-hosted-macos.yml' if target=='aarch64-apple-darwin' else '.github/workflows/release.yml'))
  out.append([e,report_raw,raw,packaging])
 return out
class CurrentSDK(unittest.TestCase):
 def setUp(self):self.rows=fixtures()
 def reject(self):
  with self.assertRaises(gate.InvalidCurrentProfile):gate.validate(self.rows,R,TREE,CONTROLLER)
 def report_change(self,fn):
  e,raw,*_=self.rows[0];r=json.loads(raw);fn(r);raw=json.dumps(r).encode();self.rows[0][1]=raw;e['sdkReportSha256']=digest(raw)
 def test_positive_exact_three_independent_42_case_inventory(self):
  r=gate.validate(self.rows,R,TREE,CONTROLLER);self.assertEqual(len(r['wheels']),3);self.assertFalse(r['publicationAdmission'])
 def test_missing_platform(self):self.rows.pop();self.reject()
 def test_duplicate_platform(self):self.rows[1]=copy.deepcopy(self.rows[0]);self.reject()
 def test_old_producer_cannot_be_called_R(self):self.rows[0][0]['producer']['sourceCommit']='d52644f4b2fb5579f9a303012cfc3a98ac6c174e';self.reject()
 def test_old_report_cannot_be_called_R(self):self.report_change(lambda r:r.update(sourceCommit='9de5315b5612288151180e9752a8a694d63a6c45'));self.reject()
 def test_source_tree_mismatch(self):self.rows[0][0]['sourceTree']='c'*40;self.reject()
 def test_unknown_envelope_field(self):self.rows[0][0]['accepted']=True;self.reject()
 def test_unknown_report_field(self):self.report_change(lambda r:r.update(accepted=True));self.reject()
 def test_old26_inventory_refuses(self):self.report_change(lambda r:r.update(controls=26,checks=r['checks'][:26]));self.reject()
 def test_reordered_cases_refuse(self):self.report_change(lambda r:r['checks'].reverse());self.reject()
 def test_negative_relabel_refuses(self):self.report_change(lambda r:r['checks'][16].update(status='compiled'));self.reject()
 def test_boolean_control_count_refuses(self):self.report_change(lambda r:r.update(controls=True));self.reject()
 def test_scope_promotion_refuses(self):self.report_change(lambda r:r.update(nativeAdmission=True));self.reject()
 def test_stale_wheel_hash_refuses(self):self.rows[0][0]['wheelSha256']='0'*64;self.reject()
 def test_report_bytes_hash_refuses(self):self.rows[0][1]+=b' ';self.reject()
 def test_native_module_hash_refuses(self):self.rows[0][0]['nativeModuleSha256']='0'*64;self.reject()
 def test_shim_hash_refuses(self):self.rows[0][0]['shimSha256']='0'*64;self.reject()
 def test_controller_must_be_complete(self):self.rows[0][0]['controller']['conclusion']='in_progress';self.reject()
 def test_controller_sha_mismatch(self):self.rows[0][0]['controller']['workflowCommit']='c'*40;self.reject()
 def test_producer_main_guard(self):self.rows[0][0]['producer']['headBranch']='candidate';self.reject()
 def test_wrong_producer_workflow(self):self.rows[0][0]['producer']['workflowPath']='.github/workflows/conformance.yml';self.reject()
 def test_split_linux_producer_cohort(self):self.rows[1][0]['producer']['runId']=4;self.reject()
 def test_split_controller_cohort(self):self.rows[0][0]['controller']['runId']=4;self.reject()
 def test_controller_code_hash_mismatch(self):self.rows[0][0]['controllerCodeSha256']='0'*64;self.reject()
 def test_fresh_packaging_bytes_mismatch(self):self.rows[0][3]+=b' ';self.reject()
 def test_fresh_packaging_failure(self):
  e,_,_,raw=self.rows[0];r=json.loads(raw);r['status']='failed';raw=json.dumps(r).encode();self.rows[0][3]=raw;e['packagingReportSha256']=digest(raw);self.reject()
 def test_packaging_cannot_claim_other_module(self):
  e,_,_,raw=self.rows[0];r=json.loads(raw);r['nativeModuleSha256']='0'*64;raw=json.dumps(r).encode();self.rows[0][3]=raw;e['packagingReportSha256']=digest(raw);self.reject()
 def test_duplicate_json_key(self):
  with self.assertRaises(gate.InvalidCurrentProfile):gate.load_json(b'{"x":1,"x":2}')
 def test_non_json_numeric_constant(self):
  with self.assertRaises(gate.InvalidCurrentProfile):gate.load_json(b'{"x":NaN}')
if __name__=='__main__':unittest.main()
