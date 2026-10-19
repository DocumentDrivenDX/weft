# SPDX-License-Identifier: MIT OR Apache-2.0
import copy,importlib.util,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('promotion',Path(__file__).with_name('verify-promotion.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class PromotionTests(unittest.TestCase):
 def fixture(self):
  run={'id':123,'repository':{'full_name':'DocumentDrivenDX/weft'},'path':'.github/workflows/release.yml','head_sha':'a'*40,'head_branch':'v0.4.33','event':'push','status':'completed','conclusion':'failure'}
  jobs=[{'name':n,'status':'completed','conclusion':'success'} for n in sorted(m.EXPECTED)]+[{'name':'publish','status':'completed','conclusion':'failure'}]
  return run,jobs
 def check(self,run,jobs):return m.verify(run,jobs,run_id='123',repository='DocumentDrivenDX/weft',tag='v0.4.33',commit='a'*40)
 def test_successful_wheels_after_failed_publication_are_promotable(self):self.assertTrue(self.check(*self.fixture()))
 def test_wrong_provenance_or_incomplete_jobs_refuse(self):
  mutations=[lambda r,j:r.update(head_sha='b'*40),lambda r,j:r.update(head_branch='main'),lambda r,j:r.update(event='pull_request'),lambda r,j:r.update(status='in_progress'),lambda r,j:r.update(path='.github/workflows/other.yml'),lambda r,j:r['repository'].update(full_name='outsider/weft'),lambda r,j:r.update(id=124),lambda r,j:j.pop(),lambda r,j:j[0].update(conclusion='failure'),lambda r,j:j[0].update(conclusion='skipped'),lambda r,j:j[0].update(status='in_progress'),lambda r,j:j[0].update(name=j[1]['name']),lambda r,j:j.append({'name':'unknown','status':'completed','conclusion':'success'}),lambda r,j:r.update(conclusion='cancelled')]
  for mutation in mutations:
   with self.subTest(mutation=mutations.index(mutation)):
    r,j=self.fixture();mutation(r,j)
    with self.assertRaises(AssertionError):self.check(r,j)
if __name__=='__main__':unittest.main()
