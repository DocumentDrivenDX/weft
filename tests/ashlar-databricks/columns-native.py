"""Independent native column/JSON home permutations on owned Ashlar projections.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 (candidate component)
"""
import hashlib,json,os,re,subprocess,sys
from pathlib import Path
from native_transport import Client
ROOT=Path(__file__).resolve().parents[2]
OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT'])
c=Client(OUT)
SCHEMA='client_dev.weft_b006_20261008_columns'
TABLE=SCHEMA+'.node_type_a'
MANIFEST=SCHEMA+'.publication_manifest'
SQL='SELECT t.name, SUM(t.rank) AS total FROM Thing t WHERE t.active = TRUE GROUP BY t.name'
layout=(ROOT/'spec/upstream/ashlar-delta-v03.sql').read_text()
layout_sha=hashlib.sha256(layout.encode()).hexdigest()
assert layout_sha=='ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e'
cases=[]; rows=[]
for bits in [8,64]:
    minimum=-(1 << (bits-1)); maximum=(1 << (bits-1))-1
    base=[('same',minimum),('same',maximum),('same',1),('é',3),('e\u0301',4),('x',5),('x ',6)]
    for label in ['exact','empty','hidden-rank-domain','hidden-name-missing','hidden-rank-type','hidden-name-nul']:
        source='weft-columns-'+str(bits)+'-'+label
        case=dict(id=str(bits)+'-'+label,bits=bits,source=source,values=[] if label=='empty' else base,corrupt=label not in ['exact','empty'])
        cases.append(case)
        incoming=[dict(name=name,rank=rank,active=True) for name,rank in case['values']]
        if case['corrupt']:
            bad=dict(name='hidden',rank=0,active=False)
            if label=='hidden-rank-domain':bad['rank']=maximum+1
            if label=='hidden-name-missing':bad.pop('name')
            if label=='hidden-rank-type':bad['rank']='17'
            if label=='hidden-name-nul':bad['name']='hidden\0name'
            incoming.append(bad)
        for i,value in enumerate(incoming,1):
            props={'23':value['rank'],'25':value['active']}
            if 'name' in value:props['24']=value['name']
            rows.append(dict(node_key='N'+str(len(source))+':'+source+':1:'+str(i),source_system=source,type_id=1,id=i,
                group_value=value.get('name'),group_present='name' in value,
                rank_value=value['rank'] if type(value['rank']) is int else None,rank_present=True,
                props_json=json.dumps(props,ensure_ascii=False,separators=(',',':')),retained_json='{}'))
payload=json.dumps(rows,ensure_ascii=False,separators=(',',':'))
payload_sha=hashlib.sha256(payload.encode()).hexdigest()
state_path=OUT/'fixture-state.json'
def p(name,value):return dict(name=name,type='STRING',value=value)
def detail(table):
    data=c.sql('identity-'+table.rsplit('.',1)[-1],'DESCRIBE DETAIL '+table)
    names=[x['name'] for x in c.records[-1]['response']['manifest']['schema']['columns']]
    assert len(data)==1
    return dict(zip(names,data[0]))
if '--resume-pinned' in sys.argv:
    state=json.loads(state_path.read_text());assert state['phase']=='ready' and state['payloadSha256']==payload_sha
else:
    assert not state_path.exists(),'Inspect previous fixture custody before any replay'
    assert c.sql('schema-custody',"SHOW SCHEMAS IN client_dev LIKE 'weft_b006_20261008_columns'")==[]
    c.sql('create-owned-schema','CREATE SCHEMA '+SCHEMA)
    for table in ['node_type_a','publication_manifest']:
        ddl=re.search(r'CREATE TABLE '+table+r' \(.*?;',layout,re.S).group(0)
        c.sql('create-'+table,ddl.replace('CREATE TABLE '+table,'CREATE TABLE '+SCHEMA+'.'+table,1))
    c.sql('insert-independent-projections','INSERT INTO '+TABLE+" SELECT r.* FROM (SELECT explode(from_json(:rows,'ARRAY<STRUCT<node_key:STRING,source_system:STRING,type_id:BIGINT,id:BIGINT,group_value:STRING,group_present:BOOLEAN,rank_value:BIGINT,rank_present:BOOLEAN,props_json:STRING,retained_json:STRING>>')) r)",[p('rows',payload)])
    identity=detail(TABLE);version=int(c.sql('table-history','DESCRIBE HISTORY '+TABLE+' LIMIT 1')[0][0])
    publication='weft-column-fixtures'
    c.sql('publish-fixture-vector','INSERT INTO '+MANIFEST+" VALUES (:id,'ashlar-delta/0.3',:vector,'{}',:revisions,:report,TIMESTAMP '2026-10-08 00:00:00')",[p('id',publication),p('vector',json.dumps({TABLE:version})),p('revisions',json.dumps({'weft-projection-fixture':'fixture-schema-1'})),p('report',json.dumps({'qualification':'synthetic compiler controls; corrupt projections intentional','payloadSha256':payload_sha}))])
    state=dict(phase='ready',payloadSha256=payload_sha,tableUuid=identity['id'],version=version,manifestUuid=detail(MANIFEST)['id'],publicationId=publication)
    state_path.write_text(json.dumps(state,indent=2)+'\n')
assert detail(TABLE)['id']==state['tableUuid']
assert detail(MANIFEST)['id']==state['manifestUuid']
assert c.sql('resolve-fixture-vector','SELECT table_versions_json FROM '+MANIFEST+' WHERE publication_id=:id',[p('id',state['publicationId'])])==[[json.dumps({TABLE:state['version']})]]
outcomes=[]
for case in cases:
    elements=[dict(id='thing',name='Thing',kind='record',members=[dict(module='main',element=e) for e in ['name','rank','active']],extensions={})]
    for field,family in [('name','string'),('rank','integer'),('active','boolean')]:
        value=dict(id=field,name=field,kind='field',scalarType=family,nullability='required',cardinality='one',extensions={})
        if family=='integer':value['facets']={'integerWidth':{'bits':case['bits'],'signed':True}}
        elements.append(value)
    document=json.dumps(dict(umf='0.7.0',id='column-fixture',vocabularies={},modules=[dict(id='main',namespace='columns',elements=elements)],extensions={}))
    pin=dict(documentId='column-fixture',revision='1',umfVersion='0.7.0',sha256=hashlib.sha256(document.encode()).hexdigest())
    def logical(element):return dict(documentId=pin['documentId'],revision='1',module='main',element=element)
    for name_home in ['props','column']:
        for rank_home in ['props','column']:
            identifier=case['id']+'-'+name_home+'-'+rank_home
            properties=[]
            for field,property_id,home in [('name','24',name_home),('rank','23',rank_home),('active','25','props')]:
                physical=dict(kind='props',propertyId=property_id) if home=='props' else dict(kind='column',value='group_value' if field=='name' else 'rank_value',present='group_present' if field=='name' else 'rank_present',nativeType='STRING' if field=='name' else 'BIGINT')
                properties.append(dict(logical=logical(field),home=physical))
            binding=json.dumps(dict(profile='ashlar-databricks-candidate/0.1.0',layoutRevision='ashlar-delta/0.3',layoutSha256=layout_sha,modelPins=[pin],publication=dict(id=state['publicationId'],manifestUuid=state['manifestUuid'],tables=[dict(name=TABLE.split('.'),uuid=state['tableUuid'],version=state['version'])]),records=[dict(logical=logical('thing'),table=0,kind='nodeProjection',sourceSystem=case['source'],typeId='1',schemaRevision='fixture-schema-1',properties=properties)]))
            request=dict(interfaceVersion='weft-compile/0.1.0',dialect='weft-sql/0.1.0',sql=SQL,modules=[dict(documentJson=document,pin=pin,selectedModuleIds=['main'])],target=dict(backendId='ashlar.databricks',backendVersion='0.1.0-candidate',targetProfile='dbsql-candidate',bindingJson=binding,bindingSha256=hashlib.sha256(binding.encode()).hexdigest()),options=dict(allowCandidate=True))
            run=subprocess.run([os.environ['WEFT_ASHLAR_COMPILER']],input=json.dumps(request),text=True,capture_output=True,check=True)
            artifact=json.loads(run.stdout);assert artifact['status']=='compiled',artifact
            (OUT/(identifier+'-compile.json')).write_text(json.dumps(dict(request=request,response=artifact),ensure_ascii=False,indent=2)+'\n')
            parameters=[p('p'+str(slot['position']),slot['value']) for slot in artifact['parameters']]
            checks=next(o for o in artifact['obligations'] if o['id']=='ashlar.candidate.scalarIntegrity')['parameters']['checks']
            assert len(checks)==3
            counts=[c.sql(identifier+'-integrity-'+str(i),check['sql'],parameters) for i,check in enumerate(checks)]
            if case['corrupt']:
                assert any(int(n[0][0])>0 for n in counts),(identifier,counts)
                outcome=dict(id=identifier,outcome='refused-before-user-query',counts=counts)
            else:
                assert all(n==[['0']] for n in counts),(identifier,counts)
                actual=c.sql(identifier+'-user-query',artifact['sql'],parameters)
                grouped={}
                for name,rank in case['values']:grouped[name]=grouped.get(name,0)+rank
                expected=sorted([[name,str(total)] for name,total in grouped.items()])
                assert sorted(actual)==expected,(identifier,actual,expected)
                columns=c.records[-1]['response']['manifest']['schema']['columns']
                assert [(x['name'],x['type_name']) for x in columns]==[('name','STRING'),('total','STRING')]
                assert artifact['columns'][1]['decoder']=='exact-integer'
                outcome=dict(id=identifier,outcome='published-fixture-result',rows=actual)
            outcomes.append(outcome)
summary=dict(state='passed',cases=len(outcomes),positive=sum(o['outcome']=='published-fixture-result' for o in outcomes),refusals=sum(o['outcome']=='refused-before-user-query' for o in outcomes),outcomes=outcomes,fixture=state,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),transportSha256=hashlib.sha256((Path(__file__).parent/'native_transport.py').read_bytes()).hexdigest(),qualification='Actual registered compiler and native node_type_a STRING/BIGINT column and JSON-text home permutations, signed8/64. Synthetic admin projections, intentional residual/corrupt controls. No accepted projection correspondence, delegated policy, embedding, application-read or production admission.')
(OUT/'summary.json').write_text(json.dumps(summary,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({k:v for k,v in summary.items() if k!='outcomes'},indent=2))
