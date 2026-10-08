"""Actual psycopg execution of original entities with host publication gates.
@covers US-003-AC4: native prepared transport and guarded publication.
Synthetic owned fixtures; not production authorization or Truss qualification.
"""
import copy, hashlib, json, os
from pathlib import Path
import psycopg
from psycopg import RawCursor
from psycopg.types.json import set_json_loads
from psycopg.types.string import StrDumper
class TextSlotDumper(StrDumper):
 oid=25
from host_obligation_fixture import Refused, execute
ROOT=Path(__file__).resolve().parents[2];F=ROOT/'tests/truss-postgresql/fixtures'
setups=json.loads(Path(os.environ['WEFT_ENTITY_DRIVER_SETUPS']).read_text())
transports=json.loads((F/'original-entity-public-transport.json').read_text());results=[]
def forbid_float(token):raise ValueError('unqualified JSON numeric carrier: '+token)
with psycopg.connect(host='127.0.0.1',port=int(os.environ['WEFT_DRIVER_PORT']),dbname='postgres',user='postgres',autocommit=True) as connection:
 version=connection.execute('SHOW server_version').fetchone()[0];assert version.split()[0]=='17.9',version
 assert connection.execute('SHOW server_encoding').fetchone()[0]=='UTF8'
 connection.adapters.register_dumper(str,TextSlotDumper)
 set_json_loads(lambda s:json.loads(s,parse_float=forbid_float),connection)
 for setup in setups:
  transport=next(x for x in transports if x['fixture']==setup['fixture'] and x['noteHome']==setup['noteHome'] and x['request']['sql'].endswith(f"LIMIT {setup['bound']}"))
  artifact=transport['response'];base=dict(bindingSha256=artifact['bindingSha256'],modelPins=artifact['modelPins'],targetContext=artifact['targetContext'],completeVisibility=True,authority='fixture-authority')
  for scenario in ['stored-data']+(['authority-revoked','binding-revoked','visibility-revoked'] if setup['case']=='valid' else []):
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
      cursor.execute(sql,[p['value'] for p in parameters],prepare=True)
      rows=cursor.fetchall()
      if not data:
       assert len(cursor.description)==1 and cursor.description[0].type_code in [20,25]
       assert all(len(row)==1 and type(row[0]) in [int,str] for row in rows)
       rows=[(str(row[0]),) for row in rows]
      if data:oids.extend(c.type_code for c in cursor.description)
      return rows
    try:
     rows=execute(artifact,snapshot,query)
     assert not setup['corrupt'] and scenario=='stored-data'
     tree=json.loads((F/f"original-{setup['fixture']}-native-tree.json").read_text())
     expected=[]
     for owner,id,part,note in [(1,'9007199254740993','A',None),(2,'18446744073709551615','B','')][:setup['bound']]:
      value=dict(state='absent') if setup['case']=='absent-compound' and owner==2 else dict(state='value',value=tree['logical'])
      expected.append((value,dict(state='absent') if note is None else dict(state='value',value=note),id,part))
     decoded=[tuple(json.loads(v,parse_float=forbid_float) if i<2 and isinstance(v,str) else v for i,v in enumerate(row)) for row in rows]
     assert decoded==expected,(setup['fixture'],setup['case'],setup['bound'],decoded,expected)
     assert oids[2:]==[25,25] and all(isinstance(row[2],str) for row in rows)
     published=True
    except Refused:
     assert setup['corrupt'] or scenario!='stored-data',(setup['fixture'],setup['case'],events)
     if setup['corrupt']:assert 'query' not in events
     else:assert events[-1]=='context' and 'query' in events
     published=False
    connection.execute('DROP TABLE row_home_scalar,row_home_node,row_home_state,object')
   results.append(dict(fixture=setup['fixture'],noteHome=setup['noteHome'],bound=setup['bound'],case=setup['case'],scenario=scenario,events=events,published=published,carrierOids=oids))
 receipt=dict(scope='Synthetic PostgreSQL 17.9 original whole-entity fixture execution through psycopg native prepared placeholders; injected authority/pin/visibility callbacks, not production policy enforcement',server=version,driver=psycopg.__version__,libpq=psycopg.pq.version(),cases=len(results),sourceHashes={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),Path(__file__).with_name('host_obligation_fixture.py'),Path(__file__).with_name('recursive-entity-native.py')]},results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-entity-driver-native.json').write_text(json.dumps(receipt,indent=2)+'\n');print(f'{len(results)} original entity native driver cases passed')
