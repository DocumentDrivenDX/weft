"""Execute actual candidate compiler SQL against independent two-home fixtures."""
import json,subprocess,csv,io
from pathlib import Path
from collections import Counter
from decimal import Decimal
ROOT=Path(__file__).resolve().parents[2]
cases=json.loads((ROOT/'tests/truss-postgresql/fixtures/compiler-cases.json').read_text())
seed=json.loads((ROOT/'docs/helix/03-test/fixtures/sales.rows.json').read_text())
quote=lambda s:"'"+s.replace("'","''")+"'"
def setup(mapping):
    sql='''CREATE TEMP TABLE object(id bigint PRIMARY KEY,type_id int NOT NULL,props jsonb NOT NULL);
CREATE TEMP TABLE row_home_state(state_id bigint PRIMARY KEY,owner_kind text,object_id bigint,object_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,PRIMARY KEY(state_id,node_id));
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value boolean,numeric_value numeric,PRIMARY KEY(state_id,node_id));
'''
    records={};oid=0;state=0
    for m in mapping:records.setdefault((m['record'],m['typeId']),[]).append(m)
    for (name,typeid),members in records.items():
        for row in seed[name]:
            oid+=1;props=[]
            for m in members:
                v=row[m['name']];fam=m['family'];state+=1
                leaf=json.dumps(v,ensure_ascii=False) if fam=='string' else v
                props.append(json.dumps(m['propertyId'])+':'+leaf)
                text=quote(v) if fam=='string' else 'NULL'
                boolean=v if fam=='boolean' else 'NULL'
                numeric=quote(v) if fam in ['integer','decimal'] else 'NULL'
                sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{oid},{typeid},{typeid},{m['propertyId']},{state});\n"
                sql+=f"INSERT INTO row_home_node VALUES ({state},{state},NULL,'scalar');\n"
                sql+=f"INSERT INTO row_home_scalar VALUES ({state},{state},{quote(fam)},{text},{boolean},{numeric});\n"
            sql+=f"INSERT INTO object VALUES ({oid},{typeid},{quote('{'+','.join(props)+'}')});\n"
    # An unrelated type has deliberately invalid selected-name lookalike content.
    sql+="INSERT INTO object VALUES (999999,-99,'{\"0\":\"not-an-integer\",\"1\":\"unrelated\"}');\n"
    return sql
def canonical(v,kind):
    if kind=='null':return None
    if kind in ['integer','decimal']:return Decimal(v)
    if kind=='boolean':return v in ['true','t']
    return v
reports=[]
for c in cases:
    raw=subprocess.check_output([ROOT/'target/debug/examples/compile_probe'],input=json.dumps(c['request'],ensure_ascii=False).encode()).decode().strip()
    r=json.loads(raw);assert r['status']=='compiled',(c['id'],r)
    assert r['qualification']['status']=='candidate',r['qualification']
    params=r['parameters'];types=','.join({'string':'text','boolean':'bool','integer':'numeric','decimal':'numeric'}[p['logicalType']['family']] for p in params)
    values=','.join(quote(p['value']) for p in params)
    integrity=next(o for o in r['obligations'] if o['id']=='truss.candidate.scalarIntegrity')['parameters']['checks']
    sql='BEGIN ISOLATION LEVEL REPEATABLE READ;\nSET standard_conforming_strings=on;\n'+setup(c['mapping'])
    for i,g in enumerate(integrity):sql+=f"PREPARE check_{i}({types}) AS {g['sql']};\nEXECUTE check_{i}({values});\n"
    sql+=f"PREPARE query({types}) AS {r['sql']};\nEXECUTE query({values});\nROLLBACK;\n"
    result=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-P','null=__WEFT_FIXTURE_NULL__','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
    rows=list(csv.reader(io.StringIO(result)))
    for i in range(len(integrity)):assert rows[2*i:2*i+2]==[['count'],['0']],(c['id'],rows)
    rows=rows[2*len(integrity):]
    if len(r['columns'])==1:rows=[row if row else [''] for row in rows]
    assert rows[0]==[col['outputName'] for col in r['columns']],(c['id'],rows)
    assert all(len(row)==len(r['columns']) for row in rows[1:]),(c['id'],rows)
    actual=[]
    for row in rows[1:]:
        actual.append(tuple(None if value=='__WEFT_FIXTURE_NULL__' and col['nullable'] else canonical(value,col['logicalType']['family']) for value,col in zip(row,r['columns'],strict=True)))
    expected=[tuple(canonical(cell.get('value'),cell['kind']) for cell in row) for row in c['expected']['rows']]
    assert Counter(actual)==Counter(expected),(c['id'],actual,expected)
    reports.append(dict(id=c['id'],raw=raw,response=r,rows=rows[1:]))
OUT=ROOT/'target/b005';OUT.mkdir(exist_ok=True)
(OUT/'compiler-reports.json').write_text(json.dumps(reports,ensure_ascii=False))
print(f'{len(reports)} actual candidate compiler/native cases passed across both property homes.')
