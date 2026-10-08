"""Verify saved read-only warehouse setting receipts, not corpus qualification."""
import hashlib,json
from pathlib import Path
from evidence_audit import strict
ROOT=Path(__file__).resolve().parents[2];BASE=ROOT/'docs/helix/04-build/evidence/B-007-warehouse-settings-native'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
custody=strict((BASE/'custody.json').read_text())
for name,digest in custody['inputHashes'].items():assert sha(BASE/name)==digest,name
summary=strict((BASE/'summary.json').read_text());assert summary['status']=='passed' and summary['nativeStatements']==3 and summary['observedAnsiMode'] is True
assert sha(BASE/'warehouse-settings-native.py')==summary['harnessSha256'] and sha(BASE/'native_transport.py')==summary['transportSha256'] and sha(BASE/'statements.jsonl')==summary['statementsSha256']
records=[strict(line) for line in (BASE/'statements.jsonl').read_text().splitlines()]
assert len(records)==len({r['response']['statement_id'] for r in records})==3
assert [r['label'] for r in records]==['warehouse-before-settings','read-ansi-mode','warehouse-after-settings']
for record in records:
 assert record['parameters'] is None
 response=record['response'];manifest=response['manifest'];result=response['result']
 assert response['status']['state']=='SUCCEEDED' and not manifest['truncated']
 assert manifest['total_row_count']==result['row_count']==manifest['total_chunk_count']==1
 assert result['row_offset']==result['chunk_index']==0
 assert all(c['type_name']=='STRING' for c in manifest['schema']['columns'])
for record in [records[0],records[2]]:
 assert record['sql']=="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS warehouse"
 data=record['response']['result']['data_array'];assert len(data)==1 and len(data[0])==1 and strict(data[0][0])==summary['warehouseIdentity']
assert records[1]['sql']=='SET ansi_mode'
setting=records[1]['response']['result']['data_array'];assert len(setting)==1 and len(setting[0])==2 and [v.lower() for v in setting[0]]==['ansi_mode','true']
print(json.dumps(dict(status='passed',nativeStatements=3,observedAnsiMode=True,warehouseIdentity=summary['warehouseIdentity'],scope='Saved separate-statement setting observation between matching warehouse builds; not retroactive settings evidence for corpus queries or arbitrary host sessions.')))
