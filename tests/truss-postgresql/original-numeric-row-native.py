"""Execute complete original-registry numeric row SUM/page and every prerequisite."""
import csv, hashlib, io, json, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
quote=lambda v: "'"+v.replace("'", "''")+"'"
results=[]
for fixture,family in [('original-decimal-row-sum','decimal'),('original-integer-row-sum','integer'),('original-integer-row-page','integer')]:
    path=ROOT/'tests/truss-postgresql/fixtures'/f'{fixture}.json'; raw=path.read_bytes(); e=json.loads(raw)
    page=fixture.endswith('page')
    samples=[('empty',[],False),('exact',['18446744073709551615','9007199254740993','0'] if family=='integer' else ['99999999999999999999999999.12','-0.01'],False),('overflow',['18446744073709551616'] if family=='integer' else ['100000000000000000000000000.00'],True),('wrong-codec',['1'],True),('required-absent',['1'],True)]
    if family=='integer':samples.append(('fraction',['1.5'],True))
    else:samples.append(('excess-scale',['1.001'],True))
    if page:samples.append(('duplicate-key',['1','1'],True))
    for case,values,corrupt in samples:
        sql='''BEGIN; SET standard_conforming_strings=on;
CREATE TEMP TABLE object(id bigint,type_id int);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,codec_definition_bytes bytea,original_source_bytes bytea,binary_value bytea,temporal_text text,temporal_instant timestamptz,opaque_bytes bytea);
'''
        oid=e['parameters'][0]['value'];pid=e['parameters'][2]['value']
        for i,value in enumerate(values):
            sql+=f'INSERT INTO object VALUES ({i+1},{oid});\n'
            if case=='required-absent':continue
            sql+=f"INSERT INTO row_home_state VALUES ({i+10},'object',{i+1},{oid},NULL,NULL,{oid},{pid},{i+20});\nINSERT INTO row_home_node VALUES ({i+10},{i+20},NULL);\n"
            codec='00' if case=='wrong-codec' else e['codecHex']
            sql+=f"INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES ({i+10},{i+20},'{family}',{quote(value)}::numeric,{quote(value)},decode('{codec}','hex'),decode('fe00','hex'));\n"
        sql+='INSERT INTO object VALUES (100,-999);\n'
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
            if page:
                logical=[row[0] for row in remaining[1:]]
                assert logical==(['0','9007199254740993'] if values else []),logical
            else:
                expected='__null__' if not values else ('18455751272964292608' if family=='integer' else '99999999999999999999999999.11')
                assert remaining[1:]==[[expected]],remaining
        else:assert not remaining
        results.append(dict(fixture=fixture,case=case,violations=counts,queryExecuted=not corrupt,sqlSha256=hashlib.sha256(sql.encode()).hexdigest(),captureSha256=hashlib.sha256(raw).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
report=dict(server=server,scope='Original-registry native numeric row SUM/page over synthetic PostgreSQL tables; fixture source semantics, no production codec or host qualification',harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-original-numeric-row-native.json').write_text(json.dumps(report,indent=2)+'\n')
print(f'{len(results)} original numeric row native cases passed')
