"""Execute original decimal SUM in native row/props homes and every prerequisite.
@covers US-003-AC1: exact independently calculated decimal results.
@covers US-003-AC3: precision/scale/codec/absence integrity refusal.
"""
import csv, hashlib, io, json, subprocess, os
from decimal import Decimal, getcontext
getcontext().prec=100
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
quote=lambda v: "'"+v.replace("'", "''")+"'"
CAPTURES=Path(os.environ['WEFT_ORIGINAL_DECIMAL_CAPTURE_DIR'])
results=[]
for precision,scale,home in [(p,s,h) for p in range(1,29) for s in range(p+1) for h in ['row','props']]:
    fixture=f'original-decimal-{precision}-{scale}-{home}-sum';family='decimal'
    path=CAPTURES/f'{fixture}.json';raw=path.read_bytes();e=json.loads(raw)
    if home=='props':e['codecHex']=json.loads((CAPTURES/f'original-decimal-{precision}-{scale}-row-sum.json').read_text())['codecHex']
    maximum=(Decimal(10)**precision-1)/(Decimal(10)**scale)
    bound=Decimal(10)**(precision-scale)
    token=lambda x:format(x,'f')
    samples=[('empty',[],False),('exact',[token(maximum),token(-maximum),token(maximum)],False),('below-minimum',[token(-bound)],True),('above-maximum',[token(bound)],True),('excess-scale',[token(Decimal(1)/(Decimal(10)**(scale+1)))],True),('required-absent',['0'],True)]
    if home=='row':samples.append(('wrong-codec',['0'],True))
    else:samples.append(('wrong-carrier',['0'],True))
    for case,values,corrupt in samples:
        sql='''BEGIN; SET standard_conforming_strings=on;
CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,codec_definition_bytes bytea,original_source_bytes bytea,binary_value bytea,temporal_text text,temporal_instant timestamptz,opaque_bytes bytea);
'''
        oid=e['parameters'][0]['value'];pid=e['parameters'][2]['value'] if home=='row' else e['parameters'][1]['value']
        for i,value in enumerate(values):
            props={} if case=='required-absent' or home=='row' else {pid: int(value) if case=='wrong-carrier' else value}
            sql+=f"INSERT INTO object VALUES ({i+1},{oid},'{json.dumps(props)}'::jsonb);\n"
            if home=='props':continue
            if case=='required-absent':continue
            sql+=f"INSERT INTO row_home_state VALUES ({i+10},'object',{i+1},{oid},NULL,NULL,{oid},{pid},{i+20});\nINSERT INTO row_home_node VALUES ({i+10},{i+20},NULL);\n"
            codec='00' if case=='wrong-codec' else e['codecHex']
            sql+=f"INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES ({i+10},{i+20},'{family}',{quote(value)}::numeric,{quote(value)},decode('{codec}','hex'),decode('fe00','hex'));\n"
        sql+="INSERT INTO object VALUES (100,-999,'{}'::jsonb);\n"
        types=','.join('int4' for p in e['parameters']);args=','.join(quote(p['value']) for p in e['parameters'])
        for i,check in enumerate(e['checks']):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
        if not corrupt:sql+=f"PREPARE query({types}) AS {e['sql']}; EXECUTE query({args});\n"
        sql+='ROLLBACK;\n'
        out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-P','null=__null__','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode(); rows=list(csv.reader(io.StringIO(out)))
        counts=[]
        for i in range(len(e['checks'])):
            assert rows[2*i]==['violations'];counts.append(int(rows[2*i+1][0]))
        assert any(counts) if corrupt else not any(counts),(fixture,case,counts)
        remaining=rows[len(counts)*2:]
        if not corrupt:
            assert remaining[0]==[c['outputName'] for c in e['columns']]
            expected='__null__' if not values else format(sum(map(Decimal,values)), 'f')
            assert remaining[1:]==[[expected]],remaining
        else:assert not remaining
        results.append(dict(fixture=fixture,case=case,violations=counts,queryExecuted=not corrupt,sqlSha256=hashlib.sha256(sql.encode()).hexdigest(),captureSha256=hashlib.sha256(raw).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
report=dict(server=server,scope='Original decimal properties in row/props homes for every admitted precision 1..28 and scale 0..precision (434 pairs); exact sums and integrity refusal, no broader domain/production/embedding qualification',harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-original-decimal-domain-native.json').write_text(json.dumps(report,indent=2)+'\n')
assert len(results)==6076
print(f'{len(results)} original decimal domain native cases passed')
