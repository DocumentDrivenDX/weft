"""Pure closed current release event selector; no API or build side effects."""
import re

def select(event,ref,inputs):
 result=dict(produce=False,publish=False)
 # Tags/release-published/workflow_run never schedule current producers.
 if event!='workflow_dispatch' or ref!='refs/heads/main' or inputs.get('current_final') not in (True,'true'):return result
 source=inputs.get('source_sha','');source_run=inputs.get('current_source_run_id','')
 if re.fullmatch('[0-9a-f]{40}',source) is None or re.fullmatch('[1-9][0-9]*',str(source_run)) is None:raise ValueError('Current explicit selection requires frozen source/source-run')
 tag=inputs.get('tag','');linux=inputs.get('packaging_run_id','');mac=inputs.get('macos_run_id','')
 if not tag and not linux and not mac:return dict(produce=True,publish=False)
 if tag=='v0.4.57' and re.fullmatch('[1-9][0-9]*',str(linux)) and re.fullmatch('[1-9][0-9]*',str(mac)) and inputs.get('macos_source_sha')==source:return dict(produce=False,publish=True)
 raise ValueError('Current publication must reuse the completed exact producer cohort')
