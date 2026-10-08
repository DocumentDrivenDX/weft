"""Actual psycopg sales-query transport and publication proof.
@covers US-003-AC1: independently authored exact grouped decimal results.
@covers US-003-AC4: explicit text parameter/result OIDs and prepublication refusal.
Synthetic fixtures and injected authority views, not production policy adoption.
"""
import copy,hashlib,json,os
from pathlib import Path
import psycopg
from psycopg import RawCursor
from psycopg.types.string import StrDumper
from host_obligation_fixture import Refused,execute
class TextSlotDumper(StrDumper):oid=25
ROOT=Path(__file__).resolve().parents[2];source=Path(os.environ['WEFT_SALES_DRIVER_SETUPS']);setups=json.loads(source.read_text());results=[]
assert len(setups)==128
with psycopg.connect(host='127.0.0.1',port=int(os.environ['WEFT_DRIVER_PORT']),dbname='postgres',user='postgres',autocommit=True) as connection:
 version=connection.execute('SHOW server_version').fetchone()[0];assert version.split()[0]=='17.9'
 assert connection.execute('SHOW server_encoding').fetchone()[0]=='UTF8'
 connection.adapters.register_dumper(str,TextSlotDumper)
 for setup in setups:
  artifact=setup['artifact']
  assert [c['outputName'] for c in artifact['columns']]==['name','total']
  assert artifact['columns'][0]['representation']==dict(kind='scalar',carrier='text',decoder='text',logicalType=dict(family='string',facets={},nullable=False))
  assert artifact['columns'][1]['representation']==dict(kind='scalar',carrier='text',decoder='exact-decimal',logicalType=dict(family='decimal',facets={'scale':2},nullable=False))
  base=dict(bindingSha256=artifact['bindingSha256'],modelPins=artifact['modelPins'],targetContext=artifact['targetContext'],completeVisibility=True,authority='fixture-authority')
  for scenario in ['stored-data']+(['authority-revoked','binding-revoked','visibility-revoked'] if not setup['corrupt'] else []):
   events=[];snapshots=[copy.deepcopy(base),copy.deepcopy(base)];oids=[]
   if scenario=='authority-revoked':snapshots[1]['authority']='other'
   if scenario=='binding-revoked':snapshots[1]['bindingSha256']='0'*64
   if scenario=='visibility-revoked':snapshots[1]['completeVisibility']=False
   def snapshot():events.append('context');return snapshots.pop(0)
   with connection.transaction():
    connection.execute(setup['setupSql'],prepare=False)
    def query(sql,parameters):
     data=sql==artifact['sql'];events.append('query' if data else 'guard')
     with RawCursor(connection) as cursor:
      cursor.execute(sql,[p['value'] for p in parameters],prepare=True);rows=cursor.fetchall()
      if data:
       oids.extend(c.type_code for c in cursor.description);assert oids==[25,25]
       assert [c.name for c in cursor.description]==['name','total']
       assert all(all(type(value) is str for value in row) for row in rows)
      else:
       assert len(cursor.description)==1 and cursor.description[0].type_code in [20,25]
       assert all(len(row)==1 and type(row[0]) in [int,str] for row in rows);rows=[(str(row[0]),) for row in rows]
      return rows
    try:
     rows=execute(artifact,snapshot,query)
     assert not setup['corrupt'] and scenario=='stored-data'
     assert sorted([list(row) for row in rows])==sorted(setup['expectedRows']),(setup['homes'],setup['case'],rows,setup['expectedRows'])
     assert events[0]==events[-1]=='context';published=True
    except Refused:
     assert setup['corrupt'] or scenario!='stored-data'
     if setup['corrupt']:assert 'query' not in events
     else:assert events[-1]=='context' and 'query' in events
     published=False
    connection.execute('DROP TABLE row_home_scalar,row_home_node,row_home_state,object')
   results.append(dict(homes=setup['homes'],case=setup['case'],scenario=scenario,events=events,published=published,carrierOids=oids))
receipt=dict(scope='Original sales join across all 16 row/props cuts via psycopg native prepared placeholders; exact text results and injected prepublication views, synthetic candidate only',server=version,driver=psycopg.__version__,libpq=psycopg.pq.version(),cases=len(results),setupsSha256=hashlib.sha256(source.read_bytes()).hexdigest(),sourceHashes={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),Path(__file__).with_name('host_obligation_fixture.py'),Path(__file__).with_name('original-sales-join-native.py')]},results=results)
Path(os.environ.get('WEFT_EVIDENCE_OUTPUT',ROOT/'docs/helix/04-build/evidence/B-005-sales-join-driver-native.json')).write_text(json.dumps(receipt,indent=2)+'\n');print(f'{len(results)} sales join native driver cases passed')
