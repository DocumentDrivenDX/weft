"""Independent entity/count/keyset oracles over pinned native typed-home fixtures.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 (candidate component)
"""
import copy,hashlib,json,os,subprocess
from pathlib import Path
from native_transport import Client
from warehouse_capture import capture
OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT'])
FIXTURE=Path(os.environ['WEFT_ASHLAR_COLUMNS_FIXTURE'])
c=Client(OUT);outcomes=[]
warehouse_counts=os.environ.get('WEFT_ASHLAR_WAREHOUSE_COUNTS')=='1'
warehouse_identities=set()
binary=Path(os.environ['WEFT_ASHLAR_COMPILER']);binary_sha=hashlib.sha256(binary.read_bytes()).hexdigest()
def execute(identifier,request,expected,key_check=False):
    process=subprocess.run([str(binary)],input=json.dumps(request),text=True,capture_output=True,check=True)
    artifact=json.loads(process.stdout);assert artifact['status']=='compiled',artifact
    parameters=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in artifact['parameters']]
    integrity=[o for o in artifact['obligations'] if o['id'] in ['ashlar.candidate.scalarIntegrity','ashlar.candidate.keyIntegrity']]
    assert any(o['id']=='ashlar.candidate.keyIntegrity' for o in integrity)==key_check
    for obligation in integrity:
        for index,check in enumerate(obligation['parameters']['checks']):
            assert c.sql(identifier+'-'+obligation['id']+'-'+str(index),check['sql'],parameters)==[['0']]
    statement=capture(artifact['sql']) if warehouse_counts else artifact['sql']
    actual=c.sql(identifier+'-user-query',statement,parameters)
    metadata=c.records[-1]['response']['manifest']['schema']['columns']
    if warehouse_counts:
        assert actual and metadata[0]['name']=='__weft_warehouse' and metadata[0]['type_name']=='STRING'
        for row in actual:
            identity=json.loads(row[0])
            assert set(identity)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and identity['dbr_version'] is None
            assert all(isinstance(identity[k],str) and identity[k] for k in ['dbsql_version','u_build_hash','r_build_hash'])
            warehouse_identities.add(json.dumps(identity,sort_keys=True))
        actual=[row[1:] for row in actual];metadata=metadata[1:]
    assert actual==expected,(identifier,actual,expected)
    assert [(x['name'],x['type_name']) for x in metadata]==[(x['outputName'],'STRING') for x in artifact['columns']]
    (OUT/(identifier+'-compile.json')).write_text(json.dumps(dict(request=request,response=artifact),indent=2,ensure_ascii=False)+'\n')
    outcomes.append(dict(id=identifier,rows=actual))
    return actual
for bits in [8,64]:
    minimum=-(1 << (bits-1)); maximum=(1 << (bits-1))-1
    values=[('same',minimum),('same',maximum),('same',1),('é',3),('e\u0301',4),('x',5),('x ',6)]
    for name_home in ['props','column']:
        for rank_home in ['props','column']:
            prefix=str(bits)+'-'+name_home+'-'+rank_home
            base=json.loads((FIXTURE/(str(bits)+'-exact-'+name_home+'-'+rank_home+'-compile.json')).read_text())['request']
            base['interfaceVersion']='weft-compile/0.2.0';base['dialect']='weft-sql/0.2.0'
            request=copy.deepcopy(base);request['sql']='SELECT COUNT(*) AS total FROM Thing t'
            execute(prefix+'-count',request,[['7']])
            request=copy.deepcopy(base);request['sql']='SELECT COUNT(*) AS total FROM Thing t JOIN Thing u ON t.name = u.name'
            expected=sum(a==b for a,_ in values for b,_ in values)
            execute(prefix+'-join-count',request,[[str(expected)]])
            request=copy.deepcopy(base);request['sql']='SELECT t.name, COUNT(*) AS total FROM Thing t GROUP BY t.name ORDER BY t.name ASC'
            counts={}
            for name,_ in values:counts[name]=counts.get(name,0)+1
            execute(prefix+'-group-count',request,[[name,str(count)] for name,count in sorted(counts.items())])
            empty=json.loads((FIXTURE/(str(bits)+'-empty-'+name_home+'-'+rank_home+'-compile.json')).read_text())['request']
            empty['interfaceVersion']='weft-compile/0.2.0';empty['dialect']='weft-sql/0.2.0';empty['sql']='SELECT COUNT(*) AS total FROM Thing t'
            execute(prefix+'-empty-count',empty,[['0']])
            if warehouse_counts:continue
            for composite in [False,True]:
                request=copy.deepcopy(base)
                document=json.loads(request['modules'][0]['documentJson'])
                fields=['name','rank'] if composite else ['rank']
                document['modules'][0]['elements'][0]['keys']=[dict(id='primary',name='primary',primary=True,fields=[dict(module='main',element=f) for f in fields])]
                raw=json.dumps(document);digest=hashlib.sha256(raw.encode()).hexdigest()
                request['modules'][0]['documentJson']=raw;request['modules'][0]['pin']['sha256']=digest
                binding=json.loads(request['target']['bindingJson']);binding['modelPins'][0]['sha256']=digest
                raw=json.dumps(binding);request['target']['bindingJson']=raw;request['target']['bindingSha256']=hashlib.sha256(raw.encode()).hexdigest()
                request['readProfile']=dict(version='weft-application-read/0.2.0',subset='entity-page')
                ordered=sorted(values,key=(lambda row:row) if composite else (lambda row:row[1]))
                accumulated=[];after=None;page=0
                while True:
                    query='SELECT t.* FROM Thing t'
                    if after is not None:
                        query+=' WHERE (t.name, t.rank) > (:name, :rank)' if composite else ' WHERE t.rank > :rank'
                        request['parameters']={'rank':dict(family='integer',value=str(after[1]))}
                        if composite:request['parameters']['name']=dict(family='string',value=after[0])
                    query+=' ORDER BY t.name ASC, t.rank ASC LIMIT 2' if composite else ' ORDER BY t.rank ASC LIMIT 2'
                    request['sql']=query
                    expected=[[name,str(rank),'true'] for name,rank in ordered[len(accumulated):len(accumulated)+2]]
                    actual=execute(prefix+('-composite' if composite else '-single')+'-page-'+str(page),request,expected,True)
                    accumulated.extend(actual)
                    if not actual:break
                    after=(actual[-1][0],int(actual[-1][1]));page+=1
                    assert page<=5
                assert accumulated==[[name,str(rank),'true'] for name,rank in ordered]
assert hashlib.sha256(binary.read_bytes()).hexdigest()==binary_sha,'Compiler changed during native execution'
if warehouse_counts:assert len(outcomes)==32 and len(warehouse_identities)==1
summary=dict(warehouseIdentity=json.loads(next(iter(warehouse_identities))) if warehouse_counts else None,warehouseLinkedCountCases=len(outcomes) if warehouse_counts else 0,state='passed',cases=len(outcomes),outcomes=outcomes,compilerBinarySha256=binary_sha,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),transportSha256=hashlib.sha256((Path(__file__).parent/'native_transport.py').read_bytes()).hexdigest(),qualification=('Warehouse/build-linked count-only corpus; keyset/entity pages excluded from this run. ' if warehouse_counts else '')+'Registered 0.2 compiler and native required-scalar entity projection, exact bag/group/global counts and single/composite authored-key pages across four native/JSON homes and signed8/64. Same retained synthetic snapshots; no live policy/pin authority, optional/compound/related values, embedding or production qualification.')
(OUT/'summary.json').write_text(json.dumps(summary,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({k:v for k,v in summary.items() if k!='outcomes'},indent=2))
