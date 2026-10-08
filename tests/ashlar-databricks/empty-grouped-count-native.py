"""@covers US-007-AC3: empty global/grouped COUNT on actual candidate snapshots."""
import hashlib,json,os,subprocess
from pathlib import Path
from native_transport import Client
ROOT=Path(__file__).resolve().parents[2]
OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT']);client=Client(OUT)
binary=Path(os.environ['WEFT_ASHLAR_COMPILER']);sha=hashlib.sha256(binary.read_bytes()).hexdigest()
artifacts=[];outcomes=[]
inputs={a['id']:a for a in (json.loads(line) for line in (ROOT/'docs/helix/04-build/evidence/B-006-columns-native/compile-artifacts.jsonl').read_text().splitlines())}
for bits in [8,64]:
 for name_home in ['props','column']:
  for rank_home in ['props','column']:
   name=f'{bits}-empty-{name_home}-{rank_home}'
   request=inputs[name]['request']
   request['interfaceVersion']='weft-compile/0.2.0';request['dialect']='weft-sql/0.2.0'
   request['sql']='SELECT t.name, COUNT(*) AS total FROM Thing t GROUP BY t.name'
   response=json.loads(subprocess.check_output([str(binary)],input=json.dumps(request).encode()))
   assert response['status']=='compiled',response
   params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
   checks=0
   for obligation in response['obligations']:
    if obligation['id'] in ['ashlar.candidate.scalarIntegrity','ashlar.candidate.keyIntegrity']:
     for i,check in enumerate(obligation['parameters']['checks']):
      assert client.sql(name+'-'+obligation['id']+'-'+str(i),check['sql'],params)==[['0']];checks+=1
   assert checks>0
   actual=client.sql(name+'-user-query',response['sql'],params);assert actual==[],(name,actual)
   artifacts.append({'id':name,'request':request,'response':response})
   outcomes.append({'id':name,'rows':actual,'expected':[],'guards':checks})
assert hashlib.sha256(binary.read_bytes()).hexdigest()==sha
(OUT/'compile-artifacts.jsonl').write_text('\n'.join(json.dumps(a) for a in artifacts)+'\n')
summary={'status':'passed','cases':8,'nativeStatements':len(client.records),'outcomes':outcomes,'compilerSha256':sha,'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'scope':'Actual read-only retained synthetic Databricks snapshots: empty grouped COUNT across signed8/64 and four explicit home combinations; no production publication qualification.'}
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
