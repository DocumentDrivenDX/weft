"""Execute captured V02 join assembly and all physical preflight checks."""
import csv, hashlib, io, json, subprocess, sys
from collections import Counter
from pathlib import Path
capture, report_path = map(Path, sys.argv[1:3])
raw = capture.read_bytes(); emission = json.loads(raw)
slots = emission['parameters']
assert len(slots)==6
types = ','.join('int4' for p in slots)
quote = lambda value: "'" + value.replace("'", "''") + "'"
values = ','.join(quote(p['value']) for p in slots)
results = []
for case, names, corrupt in [('empty', [], False), ('duplicate-unicode-empty', ['A','A','','é ','é'], False), ('wrong-codec', ['A'], True), ('required-absent', ['A'], True)]:
    sql = """BEGIN; SET standard_conforming_strings=on;
CREATE TEMP TABLE object(id bigint,type_id int);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,codec_definition_bytes bytea,original_source_bytes bytea,binary_value bytea,temporal_text text,temporal_instant timestamptz,opaque_bytes bytea);
"""
    oid=emission['ownerTypeId'];pid=emission['propertyId']
    for index,name in enumerate(names):
        state=index+10; node=index+20; owner=index+1
        sql+=f"INSERT INTO object VALUES ({owner},{oid});\n"
        if case=='required-absent': continue
        sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{owner},{oid},NULL,NULL,{oid},{pid},{node});\n"
        sql+=f"INSERT INTO row_home_node VALUES ({state},{node},NULL);\n"
        codec='00' if case=='wrong-codec' else emission['codecHex']
        sql+=f"INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,text_value,codec_definition_bytes,original_source_bytes) VALUES ({state},{node},'string',{quote(name)},pg_catalog.decode('{codec}','hex'),pg_catalog.decode('fe00','hex'));\n"
    sql+="INSERT INTO object VALUES (100,-2);\n"
    for index,check in enumerate(emission['checks']):
        sql += f'PREPARE check_{index}({types}) AS {check}; EXECUTE check_{index}({values});\n'
    if not corrupt: sql += f"PREPARE query({types}) AS {emission['sql']}; EXECUTE query({values});\n"
    sql += 'ROLLBACK;\n'
    output = subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
    rows = list(csv.reader(io.StringIO(output)))
    counts = []
    for index in range(len(emission['checks'])):
        assert rows[index*2] == ['violations']; counts.append(int(rows[index*2+1][0]))
    assert (any(counts) if corrupt else not any(counts)), (case,counts)
    remaining = rows[len(counts)*2:]
    if not corrupt:
        assert remaining[0] == [column['outputName'] for column in emission['columns']]
        assert Counter(map(tuple,remaining[1:])) == Counter((a,b) for a in names for b in names if a==b)
    else: assert not remaining
    results.append({'id':case,'violations':counts,'queryExecuted':not corrupt,'executedSqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
version = subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
report_path.write_text(json.dumps({'scope':'Captured original-definition V02 self-join and physical preflight over synthetic string typed row-home rows; callback fixture only, no production qualification, recursive results or host qualification','server':version,'captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print('4 V02 native-row join cases passed.')
