"""False authored uniqueness assertions must refuse before a native page query.
@covers US-004-AC2 @covers US-004-AC4 (candidate native integrity, not live policy)
"""
import hashlib,json,os,subprocess
from pathlib import Path
from native_transport import Client
OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT'])
FIXTURE=Path(os.environ['WEFT_ASHLAR_COLUMNS_FIXTURE'])
c=Client(OUT);outcomes=[]
for bits in [8,64]:
    for name_home in ['props','column']:
        for rank_home in ['props','column']:
            label=str(bits)+'-'+name_home+'-'+rank_home
            request=json.loads((FIXTURE/(str(bits)+'-exact-'+name_home+'-'+rank_home+'-compile.json')).read_text())['request']
            request['interfaceVersion']='weft-compile/0.2.0';request['dialect']='weft-sql/0.2.0'
            request['sql']='SELECT t.* FROM Thing t ORDER BY t.name ASC LIMIT 2'
            request['readProfile']=dict(version='weft-application-read/0.2.0',subset='entity-page')
            document=json.loads(request['modules'][0]['documentJson'])
            document['modules'][0]['elements'][0]['keys']=[dict(id='invalid-unique-name',name='invalid-unique-name',fields=[dict(module='main',element='name')])]
            raw=json.dumps(document);digest=hashlib.sha256(raw.encode()).hexdigest()
            request['modules'][0]['documentJson']=raw;request['modules'][0]['pin']['sha256']=digest
            binding=json.loads(request['target']['bindingJson']);binding['modelPins'][0]['sha256']=digest
            raw=json.dumps(binding);request['target']['bindingJson']=raw;request['target']['bindingSha256']=hashlib.sha256(raw.encode()).hexdigest()
            run=subprocess.run([os.environ['WEFT_ASHLAR_COMPILER']],input=json.dumps(request),text=True,capture_output=True,check=True)
            artifact=json.loads(run.stdout);assert artifact['status']=='compiled',artifact
            params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in artifact['parameters']]
            scalar=next(o for o in artifact['obligations'] if o['id']=='ashlar.candidate.scalarIntegrity')
            for index,check in enumerate(scalar['parameters']['checks']):assert c.sql(label+'-scalar-'+str(index),check['sql'],params)==[['0']]
            key=next(o for o in artifact['obligations'] if o['id']=='ashlar.candidate.keyIntegrity')
            assert key['parameters']['key']['id']=='invalid-unique-name'
            checks=key['parameters']['checks'];assert len(checks)==1
            assert checks[0]['failureCode']=='WFT-BINDING'
            # Independently authored rows contain exactly one duplicated name:
            # 'same' occurs three times; every other name occurs once.
            assert c.sql(label+'-key-refusal',checks[0]['sql'],params)==[['1']]
            (OUT/(label+'-compile.json')).write_text(json.dumps(dict(request=request,response=artifact),indent=2,ensure_ascii=False)+'\n')
            outcomes.append(dict(id=label,outcome='refused-before-user-query',duplicateKeyGroups=1))
summary=dict(state='passed',cases=len(outcomes),outcomes=outcomes,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),qualification='Native authored-key uniqueness refusal across explicit JSON/column homes and signed8/64. Only integrity SELECTs run; user page query is never submitted. No live delegated authorization, producer schema acceptance or production policy qualification.')
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
