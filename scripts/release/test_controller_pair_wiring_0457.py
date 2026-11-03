"""Pure controller wiring/budget controls; no tool/API/build/publication calls."""
from pathlib import Path
from unittest.mock import patch
import ast,io,json,os,tempfile,unittest
import current_macos_pair_0457 as policy
import run_macos_pair_0457 as mac
import run_arm64_pair_0457 as arm
import publish_macos_pair_0457 as publish
ROOT=Path(__file__).resolve().parents[2]
class BeforeCommand(RuntimeError):pass
class Wiring(unittest.TestCase):
 def test_actual_Mac_first_build_command_keeps_locked_args_and_1800_bound(self):
  with tempfile.TemporaryDirectory() as t,patch.dict(os.environ,{'GITHUB_SHA':'2'*40}),patch.object(mac,'bounded',side_effect=BeforeCommand) as run:
   product=Path(t)/'product';out=Path(t)/'out'
   with self.assertRaises(BeforeCommand):mac.build_mac(product,out)
   args=run.call_args.args
   self.assertEqual(args[:4],(product/'crates/weft-python',out,'macos-wheel-build',1800))
   self.assertEqual(args[4],['maturin','build','--release','--locked','--target','aarch64-apple-darwin','--out',out])
   self.assertEqual(os.environ['GITHUB_SHA'],'2'*40)
 def test_ARM_preserves_actual_action_selection_and_truthful_GitHub_head(self):
  with patch.dict(os.environ,{'GITHUB_SHA':'4'*40,'GITHUB_WORKFLOW_REF':'actual-controller-ref'}):
   e=arm.action_environment(Path('/product/R'))
   self.assertEqual(e['GITHUB_SHA'],'4'*40);self.assertEqual(e['GITHUB_WORKFLOW_REF'],'actual-controller-ref')
   self.assertEqual(e['INPUT_ARGS'],'--manifest-path crates/weft-python/Cargo.toml --release --locked --out dist')
   self.assertEqual((e['INPUT_TARGET'],e['INPUT_MANYLINUX'],e['INPUT_RUST-TOOLCHAIN'],e['INPUT_MATURIN-VERSION'],e['INPUT_SCCACHE']),('aarch64-unknown-linux-gnu','2014','1.90.0','1.9.6','false'))
   self.assertEqual(e['CARGO_TARGET_DIR'],'/product/R/target');self.assertEqual(e['CARGO_BUILD_JOBS'],'1')
 def test_closed_platform_workflows_default_off_and_original_global_budgets(self):
  for path,label in [('release-macos-pair-0457.yml','macos-14'),('release-arm64-pair-0457.yml','ubuntu-24.04-arm')]:
   s=(ROOT/'.github/workflows'/path).read_text()
   self.assertIn('default: false',s);self.assertIn('if: inputs.prepare_pair',s);self.assertIn('runs-on: '+label,s)
   self.assertIn('timeout-minutes: 90',s);self.assertIn('time.monotonic()+5400',s)
   self.assertIn('path: controller',s);self.assertIn('path: product',s);self.assertIn('ref: '+policy.R,s)
   self.assertNotIn('actions/cache',s)
  a=(ROOT/'.github/workflows/release-arm64-pair-0457.yml').read_text()
  self.assertIn('--stage arm-cold-acquisition-build --seconds 4200 --grace-seconds 30 --reserve-seconds 60',a)
  self.assertIn('-- python controller/scripts/release/run_arm64_pair_0457.py',a)
  self.assertNotIn('uses: PyO3/maturin-action',a) # cannot acquire outside bounded child
 def test_initial_product_ignored_addition_refuses_before_source_inventory(self):
  with tempfile.TemporaryDirectory() as t:
   root=Path(t)
   def git(path,*a):
    if a==('rev-parse','HEAD'):return policy.R
    if a==('rev-parse','HEAD^{tree}'):return policy.RTREE
    if a[0]=='diff':return ''
    if a[0]=='status':return '!! .cargo/unadmitted-config'
    raise AssertionError(a)
   with patch.object(mac,'git',side_effect=git),patch.object(mac.subprocess,'check_output',side_effect=AssertionError('inventory/process must not start')):
    with self.assertRaisesRegex(ValueError,'initially clean'):mac.source_snapshot(root,initial=True)
 def test_registration_remains_unadmitted_and_public_qualification_cannot_bypass(self):
  record=json.loads((ROOT/policy.REGISTRY).read_text());self.assertEqual(record['status'],'unadmitted');self.assertIsNone(record['macos']);self.assertIsNone(record['arm64'])
  with tempfile.TemporaryDirectory() as t,patch.object(publish,'assemble',side_effect=AssertionError('no API before exact pair validation')):
   root=Path(t);(root/'release-pins.json').write_text(json.dumps(dict(tag='v0.4.57',commit=policy.R,sourceCommit=policy.R,version='0.4.57',distribution='umf-weft',currentQualification=dict(profile=policy.COHORT_PROFILE,sourceCommit=policy.R,sourceTree=policy.RTREE,registration=record))))
   with self.assertRaisesRegex(ValueError,'tracked exact registration'):publish.verify_public(policy.R,root,root/'output')
 def test_wrong_publisher_head_refuses_before_API_tag_or_directory_creation(self):
  with tempfile.TemporaryDirectory() as t,patch.dict(os.environ,{'GITHUB_EVENT_NAME':'workflow_dispatch','GITHUB_REF':'refs/heads/main','GITHUB_SHA':policy.R}),patch.object(publish,'assemble',side_effect=AssertionError('no API')):
   dest=Path(t)/'new'
   with self.assertRaisesRegex(ValueError,'registration main publisher'):publish.publish('f'*40,1,2,3,4,dest,execute=True)
   self.assertFalse(dest.exists())
 def test_publication_selection_uses_current_controller_without_rebuilding(self):
  text=(ROOT/'.github/workflows/release.yml').read_text();self.assertIn('current_pair_publish:',text)
  self.assertIn('from publish_macos_pair_0457 import select_publication',text)
  self.assertIn('inputs.current_pair_publish && github.sha',text);self.assertIn('if: ${{ !inputs.current_pair_publish }}',text)
  self.assertIn("if: inputs.macos_run_id != '' && !inputs.current_pair_publish",text)
  self.assertIn('--publish-reviewed',text);self.assertIn('--arm64-run',text)
 def test_actual_pair_event_selection_never_replays_qualified_producers(self):
  i=dict(current_pair_publish=True,current_final=False,tag='v0.4.57',source_sha=policy.R,macos_source_sha=policy.R,packaging_run_id='1',macos_run_id='2',current_arm64_run_id='3',current_source_run_id='4')
  registration=dict(status='reviewed',sourceCommit=policy.R,sourceTree=policy.RTREE)
  self.assertEqual(publish.select_publication('workflow_dispatch','refs/heads/main',i,registration),dict(produce=False,publish=True))
  for event,ref in [('push','refs/tags/v0.4.57'),('release','refs/tags/v0.4.57'),('workflow_run','refs/heads/main')]:self.assertEqual(publish.select_publication(event,ref,i,registration),dict(produce=False,publish=False))
  i['current_pair_publish']=False;self.assertEqual(publish.select_publication('workflow_dispatch','refs/heads/main',i,registration),dict(produce=False,publish=False));i['current_pair_publish']=True
  for key,value in [('current_final',True),('tag','v0.4.58'),('source_sha','f'*40),('current_arm64_run_id','')]:
   altered=dict(i);altered[key]=value
   with self.assertRaises(ValueError):publish.select_publication('workflow_dispatch','refs/heads/main',altered,registration)
 def test_bounded_upload_refuses_symlink_and_never_reads_oversized_file(self):
  with tempfile.TemporaryDirectory() as t:
   root=Path(t);out=root/'out';out.mkdir();(out/'link').symlink_to(root/'outside')
   with self.assertRaisesRegex(ValueError,'symlink'):mac.retain(out,root/'upload')
  with tempfile.TemporaryDirectory() as t:
   root=Path(t);out=root/'out';out.mkdir()
   with (out/'oversized.whl').open('wb') as f:f.truncate(60*1024*1024+1)
   with patch.object(mac.hashlib,'file_digest',side_effect=AssertionError('oversized file must not be hashed')):
    with self.assertRaisesRegex(ValueError,'caps exceeded'):mac.retain(out,root/'upload')
   self.assertFalse((root/'upload/oversized.whl').exists())
   self.assertEqual(json.loads((root/'upload/component.json').read_text())['status'],'failed')
if __name__=='__main__':unittest.main()
