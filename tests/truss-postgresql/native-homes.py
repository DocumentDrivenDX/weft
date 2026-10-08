"""Independently authored native location witnesses; no full compiler claim."""
import subprocess,json,csv,io
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
artifacts=json.loads(subprocess.check_output([ROOT/'target/debug/examples/home_probe']))
values=['2','10','0','18446744073709551615','2']
expected=['0','2','2','10','18446744073709551615']
setup='''CREATE TEMP TABLE object(id bigint PRIMARY KEY,type_id int NOT NULL,props jsonb NOT NULL);
CREATE TEMP TABLE row_home_state(state_id bigint PRIMARY KEY,owner_kind text,object_id bigint,object_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,PRIMARY KEY(state_id,node_id));
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,numeric_value numeric,PRIMARY KEY(state_id,node_id));
'''
for i,v in enumerate(values,1):
    setup+=f"INSERT INTO object VALUES ({i},-1,'{{\"0\":{v}}}');\n"
    setup+=f"INSERT INTO row_home_state VALUES ({i},'object',{i},-1,-1,0,{i});\n"
    setup+=f"INSERT INTO row_home_node VALUES ({i},{i},NULL,'scalar');\n"
    setup+=f"INSERT INTO row_home_scalar VALUES ({i},{i},'integer',{v});\n"
def execute(sql):
    raw=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
    return list(csv.reader(io.StringIO(raw)))
observations=[]
for artifact in artifacts:
    parameters=artifact['parameters']
    types=','.join('text' if p['logicalType']['family']=='string' else 'int' for p in parameters)
    literals=','.join("'"+p['value'].replace("'","''")+"'" for p in parameters)
    queries=f"PREPARE integrity({types}) AS {artifact['integritySql']};\nEXECUTE integrity({literals});\nPREPARE projection({types}) AS {artifact['sql']};\nEXECUTE projection({literals});\n"
    rows=execute('BEGIN ISOLATION LEVEL REPEATABLE READ;\n'+setup+queries+'ROLLBACK;\n')
    assert rows==[['count'],['0'],['exact_value']]+[[v] for v in expected],(artifact['home'],rows)
    observations.append(dict(home=artifact['home'],values=expected,integrityFailures='0',duplicatesPreserved=True))
# Broken row state is observed as integrity failure, not an absent/dropped owner.
a=artifacts[1];parameters=a['parameters'];literals=','.join("'"+p['value']+"'" for p in parameters)
rows=execute(setup+f"DELETE FROM row_home_scalar WHERE node_id=1;\nPREPARE integrity(int,int) AS {a['integritySql']};\nEXECUTE integrity({literals});\n")
assert rows==[['count'],['1']],rows
OUT=ROOT/'target/b005';OUT.mkdir(exist_ok=True)
(OUT/'home-summary.json').write_text(json.dumps(dict(observations=observations,brokenPayloadDetected=True,registeredCompilerQualified=False),indent=2)+'\n')
print('Both native property homes preserve exact values and duplicate owners; broken payload detected.')
