"""Bounded pure event selection and authenticated-byte publication controls."""
import tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import select_current_events_0457 as events
import current_publication_0457 as publication
R='a'*40
class Route(unittest.TestCase):
 def setUp(self):self.i=dict(current_final=True,source_sha=R,current_source_run_id='1',tag='',packaging_run_id='',macos_run_id='',macos_source_sha=R)
 def test_explicit_producer(self):self.assertEqual(events.select('workflow_dispatch','refs/heads/main',self.i),dict(produce=True,publish=False))
 def test_explicit_publisher_reuses(self):
  self.i.update(tag='v0.4.57',packaging_run_id='2',macos_run_id='3');self.assertEqual(events.select('workflow_dispatch','refs/heads/main',self.i),dict(produce=False,publish=True))
 def test_tag_never_replays(self):self.assertEqual(events.select('push','refs/tags/v0.4.57',self.i),dict(produce=False,publish=False))
 def test_published_never_replays(self):self.assertEqual(events.select('release','refs/tags/v0.4.57',self.i),dict(produce=False,publish=False))
 def test_workflow_run_never_replays(self):self.assertEqual(events.select('workflow_run','refs/heads/main',self.i),dict(produce=False,publish=False))
 def test_default_off(self):self.i['current_final']=False;self.assertFalse(any(events.select('workflow_dispatch','refs/heads/main',self.i).values()))
 def test_tag_without_qualified_inputs_refuses(self):
  self.i['tag']='v0.4.57'
  with self.assertRaises(ValueError):events.select('workflow_dispatch','refs/heads/main',self.i)
 def test_wrong_source_refuses(self):
  self.i.update(tag='v0.4.57',packaging_run_id='2',macos_run_id='3',macos_source_sha='b'*40)
  with self.assertRaises(ValueError):events.select('workflow_dispatch','refs/heads/main',self.i)
class ExactBytes(unittest.TestCase):
 def test_authenticated_reuse_preserves_wheel(self):
  with tempfile.TemporaryDirectory() as t:
   d=Path(t);w=d/'one.whl';w.write_bytes(b'qualified-wheel');proof={'wheelSha256':{w.name:publication.sha(w.read_bytes())}}
   with patch.object(publication,'assemble',return_value=({w.name:w.read_bytes(),'source.tar.gz':b'qualified-source'},proof)):
    self.assertEqual(publication.stage(R,1,2,3,d,d/'external'),proof);self.assertEqual(w.read_bytes(),b'qualified-wheel')
 def test_wrong_wheel_refuses_before_source_staging(self):
  with tempfile.TemporaryDirectory() as t:
   d=Path(t);(d/'one.whl').write_bytes(b'rebuilt-wrong-wheel')
   with patch.object(publication,'assemble',return_value=({'source.tar.gz':b'source'},{'wheelSha256':{'one.whl':'0'*64}})):
    with self.assertRaisesRegex(ValueError,'Publisher wheels'):publication.stage(R,1,2,3,d,d/'external')
   self.assertFalse((d/'source.tar.gz').exists())
class OriginalSourceAttempts(unittest.TestCase):
 def call(self,attempt,claim=None):
  from admit_current_sources_0457 import actual_job
  t='2026-11-03T10:00:00Z';end='2026-11-03T10:01:00Z'
  job=dict(id=attempt,run_id=7,head_sha=R,run_attempt=attempt,name='source-archives',status='completed',conclusion='success',steps=[dict(name='Retain source archives and correspondence immediately',conclusion='success',started_at=t,completed_at=end)])
  manifest=dict(producerRunId=7,producerRunAttempt=attempt if claim is None else claim,producerJobName='source-archives')
  return actual_job([job],manifest,dict(created_at=t),7,R,'source-archives',attempt)
 def test_original_successes_keep_original_attempt(self):self.assertEqual([self.call(n)['attempt'] for n in (1,2)],[1,2])
 def test_wrong_claimed_attempt_refuses(self):
  with self.assertRaisesRegex(ValueError,'Wrong claimed'):self.call(1,claim=2)
if __name__=='__main__':unittest.main()
