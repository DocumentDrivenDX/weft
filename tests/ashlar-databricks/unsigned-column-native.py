"""Explicit unsigned scalar view over positive BIGINT projection IDs.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 (candidate component)
"""
import hashlib,json,os,subprocess
from pathlib import Path
from native_transport import Client
OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT']);FIXTURE=Path(os.environ['WEFT_ASHLAR_COLUMNS_FIXTURE'])
c=Client(OUT);outcomes=[]
base=json.loads((FIXTURE/'64-exact-column-column-compile.json').read_text())['request']
old_binding=json.loads(base['target']['bindingJson'])
for bits in [1,2,3,8,32,63,64]:
    label='unsigned-'+str(bits)
    field=dict(id='local-id',name='local_id',kind='field',cardinality='one',nullability='required',scalarType='integer',facets={'integerWidth':{'bits':bits,'signed':False}},extensions={})
    document=json.dumps(dict(umf='0.7.0',id='unsigned-view',vocabularies={},modules=[dict(id='view',namespace='view',elements=[dict(id='rows',name='ProjectionRows',kind='record',members=[dict(module='view',element='local-id')],extensions={}),field])],extensions={}))
    pin=dict(documentId='unsigned-view',revision='1',umfVersion='0.7.0',sha256=hashlib.sha256(document.encode()).hexdigest())
    def identity(element):return dict(documentId='unsigned-view',revision='1',module='view',element=element)
    binding=dict(profile=old_binding['profile'],layoutRevision=old_binding['layoutRevision'],layoutSha256=old_binding['layoutSha256'],modelPins=[pin],publication=old_binding['publication'],records=[dict(logical=identity('rows'),table=0,kind='nodeProjection',sourceSystem='weft-columns-64-exact',typeId='1',schemaRevision='fixture-schema-1',properties=[dict(logical=identity('local-id'),home=dict(kind='column',value='id',present=None,nativeType='BIGINT'))])])
    raw=json.dumps(binding)
    request=dict(interfaceVersion='weft-compile/0.1.0',dialect='weft-sql/0.1.0',sql='SELECT SUM(t.local_id) AS total FROM ProjectionRows t',modules=[dict(documentJson=document,pin=pin,selectedModuleIds=['view'])],target=dict(backendId='ashlar.databricks',backendVersion='0.1.0-candidate',targetProfile='dbsql-candidate',bindingJson=raw,bindingSha256=hashlib.sha256(raw.encode()).hexdigest()),options=dict(allowCandidate=True))
    process=subprocess.run([os.environ['WEFT_ASHLAR_COMPILER']],input=json.dumps(request),text=True,capture_output=True,check=True)
    artifact=json.loads(process.stdout)
    (OUT/(label+'-compile.json')).write_text(json.dumps(dict(request=request,response=artifact),indent=2)+'\n')
    before=len(c.records)
    if bits==64:
        assert artifact['status']=='blocked' and artifact['diagnostics'][0]['code']=='WFT-BINDING'
        assert 'sql' not in artifact and len(c.records)==before
        outcome=dict(id=label,outcome='compiler-refusal-no-sql')
    else:
        assert artifact['status']=='compiled',artifact
        params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in artifact['parameters']]
        checks=next(o for o in artifact['obligations'] if o['id']=='ashlar.candidate.scalarIntegrity')['parameters']['checks'];assert len(checks)==1
        count=c.sql(label+'-integrity',checks[0]['sql'],params)
        expected=sum(value>(1<<bits)-1 for value in range(1,8))
        assert count==[[str(expected)]]
        if expected:outcome=dict(id=label,outcome='refused-before-user-query',violations=expected)
        else:
            result=c.sql(label+'-query',artifact['sql'],params)
            assert result==[[str(sum(range(1,8)))]]
            assert artifact['columns'][0]['decoder']=='exact-integer'
            outcome=dict(id=label,outcome='published-fixture-result',rows=result)
    outcomes.append(outcome)
summary=dict(state='passed',cases=len(outcomes),outcomes=outcomes,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),compilerBinarySha256=hashlib.sha256(Path(os.environ['WEFT_ASHLAR_COMPILER']).read_bytes()).hexdigest(),qualification='Explicit synthetic physical-ID scalar view, native BIGINT unsigned subdomains. Unsigned1/2 data violations refuse, widths3/8/32/63 have exact finite sums; UInt64/BIGINT mapping refuses before SQL. Values1..7 do not qualify every boundary in these widths. No implicit business-key inference or production binding claim.')
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary,indent=2))
