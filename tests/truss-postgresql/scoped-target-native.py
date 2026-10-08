"""Execute independently captured scoped native target accesses."""
import csv,hashlib,io,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];results=[]
for scope in [0,1]:
    path=ROOT/'tests/truss-postgresql/fixtures'/f'original-integer-scoped-{scope}.json';raw=path.read_bytes();e=json.loads(raw)
    for case in ['empty','exact']:
        sql='''BEGIN;
CREATE TEMP TABLE object(id bigint,type_id int);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,codec_definition_bytes bytea,original_source_bytes bytea,binary_value bytea,temporal_text text,temporal_instant timestamptz,opaque_bytes bytea);
'''
        if case=='exact':
            sql+="INSERT INTO object VALUES (1,-1),(1,-999),(2,-1); INSERT INTO row_home_state VALUES (10,'object',1,-1,NULL,NULL,-1,0,20),(11,'object',2,-1,NULL,NULL,-1,0,21); INSERT INTO row_home_node VALUES (10,20,NULL),(11,21,NULL);\n"
            for state,node,value in [(10,20,'9007199254740993'),(11,21,'18446744073709551615')]:
                sql+=f"INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES ({state},{node},'integer',{value},'{value}',decode('{e['codecHex']}','hex'),decode('fe00','hex'));\n"
        args=','.join("'"+p['value']+"'" for p in e['parameters']);types=','.join('int4' for p in e['parameters'])
        for i,check in enumerate(e['checks']):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
        sql+=f"PREPARE q({types}) AS {e['sql']}; EXECUTE q({args}); ROLLBACK;\n"
        rows=list(csv.reader(io.StringIO(subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode())))
        for i in range(len(e['checks'])):assert rows[i*2:i*2+2]==[['violations'],['0']],rows
        assert rows[len(e['checks'])*2:]==[['owner','value']]+([] if case=='empty' else [['1','9007199254740993'],['2','18446744073709551615']]),rows
        results.append(dict(scope=scope,case=case,captureSha256=hashlib.sha256(raw).hexdigest(),executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
receipt=dict(scope='Distinct scoped target native row access and outer correlation only; full relationship lowering and publication remain unfinished',harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-scoped-target-native.json').write_text(json.dumps(receipt,indent=2)+'\n')
print('4 scoped target native cases passed')
