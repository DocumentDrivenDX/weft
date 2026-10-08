"""Independent native equality oracle for every resolved parameter boundary.
One stored value equals the authored boundary; SQL must return it exactly. Logical
business keys deliberately differ from storage IDs. No emitted expression or
compiler decoder is used to construct fixture values or expected numbers.
"""
import csv,io,json,subprocess
from decimal import Decimal
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
reports=json.loads((ROOT/'target/b005/full-corpus-reports.json').read_text())
quote=lambda s:"'"+s.replace("'","''")+"'"
selected=[r for r in reports if r['id'].startswith('parameter-') and r['response']['status']=='compiled']
chunks=[];expected=[]
for case in selected:
    request=case['request'];response=case['response'];binding=json.loads(request['target']['bindingJson'])
    parameter=request['parameters']['boundary'];value=parameter['value'];family=parameter['family']
    name='customer-id' if family=='integer' else 'order-total'
    prop=next(p for p in binding['properties'] if p['logical']['element']==name)
    ty=prop['ownerTypeId'];pid=prop['propertyId']
    # Independent producer fixture; model-defined number goes into both homes.
    sql='BEGIN ISOLATION LEVEL REPEATABLE READ;\n'
    sql+='CREATE TEMP TABLE object(id bigint PRIMARY KEY,type_id int NOT NULL,props jsonb NOT NULL) ON COMMIT DROP;\n'
    sql+='CREATE TEMP TABLE row_home_state(state_id bigint PRIMARY KEY,owner_kind text,object_id bigint,object_type_id int,property_owner_type_id int,property_id int,root_node_id bigint) ON COMMIT DROP;\n'
    sql+='CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,PRIMARY KEY(state_id,node_id)) ON COMMIT DROP;\n'
    sql+='CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value boolean,numeric_value numeric,PRIMARY KEY(state_id,node_id)) ON COMMIT DROP;\n'
    sql+=f"INSERT INTO object VALUES (987654321,{ty},pg_catalog.jsonb_build_object({quote(pid)}, {quote(value)}::numeric));\n"
    sql+=f"INSERT INTO row_home_state VALUES (1,'object',987654321,{ty},{ty},{pid},1);\nINSERT INTO row_home_node VALUES (1,1,NULL,'scalar');\nINSERT INTO row_home_scalar VALUES (1,1,{quote(family)},NULL,NULL,{quote(value)}::numeric);\n"
    parameters=response['parameters']
    types=','.join({'string':'text','boolean':'bool','integer':'numeric','decimal':'numeric'}[p['logicalType']['family']] for p in parameters)
    values=','.join(quote(p['value']) for p in parameters)
    guards=[g for o in response['obligations'] if o['id']=='truss.candidate.scalarIntegrity' for g in o['parameters']['checks']]
    for i,g in enumerate(guards):
        sql+=f"PREPARE boundary_guard_{i}({types}) AS {g['sql']};\nEXECUTE boundary_guard_{i}({values});\n"
    sql+=f"PREPARE boundary_result({types}) AS {response['sql']};\nEXECUTE boundary_result({values});\nDEALLOCATE ALL;\nCOMMIT;\n"
    chunks.append(sql);expected.append((case['id'],len(guards),Decimal(value)))
observed=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=''.join(chunks).encode()).decode()
rows=list(csv.reader(io.StringIO(observed)));offset=0;receipts=[]
for identity,guards,value in expected:
    for _ in range(guards):
        assert rows[offset:offset+2]==[['count'],['0']],(identity,rows[offset:offset+2]);offset+=2
    assert len(rows[offset])==1 and len(rows[offset+1])==1,(identity,rows[offset:offset+2])
    assert Decimal(rows[offset+1][0])==value,(identity,value,rows[offset+1]);offset+=2
    receipts.append(dict(id=identity,expected=str(value),actual=rows[offset-1][0],integrityViolations=0))
assert offset==len(rows),(offset,len(rows))
(ROOT/'target/b005/parameter-native-reports.json').write_text(json.dumps(receipts,indent=2))
print(f'{len(receipts)} actual native parameter boundary executions passed.')
