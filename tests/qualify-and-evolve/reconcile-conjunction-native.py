"""Reconcile exact native conjunction statements against independent truth tables.
@covers US-006-AC1 @covers US-003-AC1 @covers US-004-AC1
"""
import csv,hashlib,io,json,os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=Path(os.environ.get('WEFT_CONJUNCTION_EVIDENCE',str(ROOT/'docs/helix/04-build/evidence/B-007-conjunction-native')))
# Independently enumerated relation with a duplicate truth cell.
DATA=[{'name':'hit','active':True},{'name':'hit','active':False},{'name':'miss','active':True},{'name':'miss','active':False},{'name':'hit','active':True}]
ENGINE={'dbsql_version':'2026.39','u_build_hash':'15b447529a1f55ca1ad8c73a89b8d196f6ed4904','r_build_hash':'3909d148af5cb6b560624c9255f5a727f7a17a4c','dbr_version':None}
PG={'version':'17.9 (Debian 17.9-1.pgdg13+1)','server_encoding':'UTF8','client_encoding':'UTF8','standard_conforming_strings':'on','transaction_isolation':'repeatable read','lc_collate':'C','lc_ctype':'C'}
def verify(base):
 custody=json.loads((base/'custody.json').read_text())
 for name,digest in custody['files'].items():assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest,name
 summary=json.loads((base/'summary.json').read_text());assert summary['status']=='passed' and summary['cases']==32
 assert summary['harnessSha256']==hashlib.sha256((base/'harness.py').read_bytes()).hexdigest()
 artifacts=[json.loads(l) for l in (base/'compile-artifacts.jsonl').read_text().splitlines()];assert len(artifacts)==len({a['id'] for a in artifacts})==32
 native=[json.loads(l) for l in (base/'ashlar/statements.jsonl').read_text().splitlines()];assert len(native)==summary['ashlarStatements']==51
 assert len({n['response']['statement_id'] for n in native})==51
 statements={n['label']:n for n in native};assert len(statements)==51
 pg=json.loads((base/'postgresql-receipts.json').read_text());assert len(pg)==len({p['id'] for p in pg})==16
 pg={p['id']:p for p in pg};outcomes={r['id']:r for r in summary['results']};assert len(outcomes)==32
 for n in native:
  r=n['response'];assert r['status']['state']=='SUCCEEDED' and not r['manifest'].get('truncated')
  assert r['manifest']['total_chunk_count']<=1
 for label in ['engine-before','engine-after']:
  n=statements[label];assert n['sql']=="SELECT to_json(current_version(), map('ignoreNullFields','false'))";assert json.loads(n['response']['result']['data_array'][0][0])==ENGINE
 assert statements['ansi-mode']['sql']=='SET ansi_mode' and statements['ansi-mode']['response']['result']['data_array']==[['ansi_mode','true']]
 seen=set()
 for a in artifacts:
  key=(a['backend'],a['home'],a['dialect'],a['active'],a['name']);assert key not in seen;seen.add(key)
  matching=[r['name'] for r in DATA if r['active']==a['active'] and r['name']==a['name']]
  expected=[[v] for v in matching] if a['dialect']=='0.1' else [[str(len(matching))]]
  assert a['expectedRows']==expected and a['expectedCount']==len(matching)
  request=a['request'];response=a['response'];entity='Customer' if a['backend']=='truss' else 'Thing';projection='c.name' if a['dialect']=='0.1' else 'COUNT(*) AS total'
  assert request['sql']==f"SELECT {projection} FROM {entity} c WHERE c.active = {str(a['active']).upper()} AND c.name = '{a['name']}'"
  assert response['status']=='compiled' and response['qualification']['status']=='candidate'
  assert any(op['assessment']['id']=='and' for op in response['qualification']['operations'])
  for module in request['modules']:assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
  assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256']
  checks=[c for o in response['obligations'] if o['id'] in ['truss.candidate.scalarIntegrity','ashlar.candidate.scalarIntegrity'] for c in o['parameters']['checks']]
  if a['backend']=='truss':
   receipt=pg[a['id']];rows=list(csv.reader(io.StringIO(receipt['stdout'])));assert rows[0]==['profile'] and json.loads(rows[1][0])==receipt['profile']==PG
   assert receipt['sql'].startswith('BEGIN ISOLATION LEVEL REPEATABLE READ;\nSET standard_conforming_strings=on;\n') and receipt['sql'].endswith('ROLLBACK;\n')
   assert len(checks)==1 and rows[2:4]==[['count'],['0']]
   types=','.join({'string':'text','boolean':'bool','integer':'numeric','decimal':'numeric'}[p['logicalType']['family']] for p in response['parameters'])
   quote=lambda s:"'"+s.replace("'","''")+"'";values=','.join(quote(p['value']) for p in response['parameters'])
   for i,c in enumerate(checks):assert f"PREPARE check_{i}({types}) AS {c['sql']};\nEXECUTE check_{i}({values});\n" in receipt['sql']
   assert f"PREPARE query({types}) AS {response['sql']};\nEXECUTE query({values});\n" in receipt['sql']
   assert rows[4]==[c['outputName'] for c in response['columns']];actual=rows[5:];profile=PG
  else:
   binding=json.loads(request['target']['bindingJson']);record=binding['records'][0];table=binding['publication']['tables'][0]
   physical='.'.join('`'+x.replace('`','``')+'`' for x in table['name'])+' VERSION AS OF '+str(table['version'])
   owner='('+' UNION ALL '.join("SELECT "+str(i)+" AS id, '"+record['sourceSystem']+"' AS source_system, CAST("+record['typeId']+" AS BIGINT) AS type_id, '"+json.dumps({'24':row['name'],'25':row['active'],'23':i})+"' AS props_json, '"+row['name']+"' AS group_value, true AS group_present, "+str(row['active']).lower()+" AS active_value, true AS active_present" for i,row in enumerate(DATA,1))+')'
   params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
   assert len(checks)==2
   for i,c in enumerate(checks):
    n=statements[a['id']+'-guard-'+str(i)];assert physical in c['sql'];assert n['sql']==c['sql'].replace(physical,owner) and n['parameters']==params;assert n['response']['result']['data_array']==[['0']]
   n=statements[a['id']+'-query'];assert physical in response['sql']
   assert n['sql']=="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_engine, observed.* FROM ("+response['sql'].replace(physical,owner)+") observed" and n['parameters']==params
   rows=n['response']['result']['data_array'];assert rows and all(json.loads(row[0])==ENGINE for row in rows);actual=[row[1:] for row in rows];profile=ENGINE
  assert actual==expected==outcomes[a['id']]['actual'] and outcomes[a['id']]['profile']==profile,a['id']
 assert seen=={(b,h,d,v,n) for b in ['truss','ashlar'] for h in ['props','typed'] for d in ['0.1','0.2'] for v in [True,False] for n in ['hit','miss']}
 return dict(status='passed',cases=32,postgresqlTransactions=16,ashlarStatements=51,scope='Independent truth-table expectations and exact emitted SQL/parameters/guards/pins matched to terminal native receipts. Truss typed Boolean/string rows; Ashlar admitted typed string plus props Boolean. Same-query/transaction engine context; no support promotion.')
if __name__=='__main__':print(json.dumps(verify(BASE)))
