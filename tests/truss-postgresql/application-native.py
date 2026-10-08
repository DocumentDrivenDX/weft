"""Execute actual candidate compiler SQL against independent two-home fixtures."""
import json,subprocess,csv,io,importlib.util,os,hashlib
from pathlib import Path
from collections import Counter
from decimal import Decimal
ROOT=Path(__file__).resolve().parents[2]
BINARY=Path(os.environ.get('WEFT_TRUSS_COMPILER',str(ROOT/'target/debug/examples/compile_probe')))
BINARY_SHA=hashlib.sha256(BINARY.read_bytes()).hexdigest()
cases=json.loads((ROOT/'tests/truss-postgresql/fixtures/application-cases.json').read_text())
data=json.loads((ROOT/'tests/application/fixtures/data.json').read_text())
seed={'Customer':[{'id':r['customer-id'],'name':r['customer-name'],'active':r['customer-active'],'tags':r['tags'], **({'address':r['address']['value']} if r['address']['state']=='value' else {}), **({'nickname':r['nickname']['value']} if r['nickname']['state']=='value' else {})} for r in data['customer']], 'Orders':[{'id':r['order-id'],'customer_id':r['order-customer'],'total':r['order-total']} for r in data['orders']]}
# Authored independently from SQL and the emitted logical plan.
expected_rows={'related-page':[('1',{'items':[['1'],['2']],'truncated':True}),('2',{'items':[['3']],'truncated':False}),('3',{'items':[],'truncated':False})], 'inverse-page':[('1',{'items':[['1']],'truncated':False}),('2',{'items':[['1']],'truncated':True}),('3',{'items':[['2']],'truncated':False})], 'has-related-scalar':[('1',)], 'inverse-has-related':[('1',),('2',)], 'global-count': [('3',)], 'grouped-count': [('Alice','1'),('Bob','1'),('isolated','1')], 'join-count':[('3',)], 'global-sum':[('9007199254740994.22',)], 'composite-cursor':[('2','Bob'),('3','isolated')], 'injection-text':[], 'case-folded-param':[('1',)], 'optional-scalar':[('1',{'state':'absent'}),('2',{'state':'value','value':''}),('3',{'state':'value','value':'I'})]}

spec=importlib.util.spec_from_file_location('fixture_tree',ROOT/'tests/truss-postgresql/fixture-tree.py')
fixture_tree=importlib.util.module_from_spec(spec);spec.loader.exec_module(fixture_tree)
quote=lambda s:"'"+s.replace("'","''")+"'"
def setup(mapping, seed=seed, edges=data["edges"], modules=None):
    sql='''CREATE TEMP TABLE object(id bigint PRIMARY KEY,type_id int NOT NULL,props jsonb NOT NULL);
CREATE TEMP TABLE edge(id bigint,rel_type_id int,source_id bigint,source_type int,target_id bigint,target_type int);
CREATE TEMP TABLE row_home_state(state_id bigint PRIMARY KEY,owner_kind text,object_id bigint,object_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text COLLATE "C",record_field_identity_bytes bytea,PRIMARY KEY(state_id,node_id));
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value boolean,numeric_value numeric,PRIMARY KEY(state_id,node_id));
'''
    records={};oid=0;state=0;ids={};types={}
    for m in mapping:records.setdefault((m['record'],m['typeId']),[]).append(m)
    for (name,typeid),members in records.items():
        for row in seed.get(name,[]):
            oid+=1;props=[]
            ids[(name,row['id'])]=oid;types[name]=typeid
            for m in members:
                if m['name'] not in row:continue
                v=row[m['name']];fam=m['family'];state+=1
                if fam=='compound':
                    raw,nodes=fixture_tree.tree(modules,m['logical'],v,state)
                    props.append(json.dumps(m['propertyId'])+':'+raw)
                    sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{oid},{typeid},{typeid},{m['propertyId']},{state});\n"+nodes
                    continue
                if m.get('structuredMembers'):
                    raw={}
                    for member in m['structuredMembers']:
                        item=v.get(member['name'])
                        if isinstance(item,dict):
                            if item['state']=='absent':continue
                            item=item['value']
                        if member['family']=='integer':item=int(item)
                        raw[member['name']]=item
                    props.append(json.dumps(m['propertyId'])+':'+json.dumps(raw,ensure_ascii=False))
                    if any(member['family'] is None for member in m['structuredMembers']):
                        # Recursive props probes do not assert an alternate row codec.
                        continue
                    sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{oid},{typeid},{typeid},{m['propertyId']},{state});\n"
                    sql+=f"INSERT INTO row_home_node(state_id,node_id,parent_node_id,value_kind,slot_kind) VALUES ({state},{state},NULL,'structured','root');\n"
                    for ordinal,member in enumerate(m['structuredMembers']):
                        if member['name'] not in raw:continue
                        item=raw[member['name']];child=100000+ordinal;family=member['family']
                        identity=json.dumps(member['identity'],sort_keys=True,ensure_ascii=False,separators=(',',':')).encode().hex()
                        sql+=f"INSERT INTO row_home_node VALUES ({state},{child},{state},'scalar','record',NULL,NULL,pg_catalog.decode('{identity}','hex'));\n"
                        text=quote(item) if family=='string' else 'NULL'
                        boolean=str(item).lower() if family=='boolean' else 'NULL'
                        numeric=quote(str(item)) if family in ['integer','decimal'] else 'NULL'
                        sql+=f"INSERT INTO row_home_scalar VALUES ({state},{child},{quote(family)},{text},{boolean},{numeric});\n"
                    continue
                if isinstance(v,dict):
                    props.append(json.dumps(m['propertyId'])+':'+json.dumps(v,ensure_ascii=False))
                    if m['itemFamily'] is None:continue
                    sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{oid},{typeid},{typeid},{m['propertyId']},{state});\n"
                    sql+=f"INSERT INTO row_home_node(state_id,node_id,parent_node_id,value_kind,slot_kind) VALUES ({state},{state},NULL,'map','root');\n"
                    for ordinal,(key,item) in enumerate(v.items()):
                        child=100000+ordinal
                        sql+=f"INSERT INTO row_home_node VALUES ({state},{child},{state},'scalar','map',NULL,{quote(key)},NULL);\n"
                        if m['itemFamily']=='string':
                            sql+=f"INSERT INTO row_home_scalar VALUES ({state},{child},'string',{quote(item)},NULL,NULL);\n"
                        else:
                            sql+=f"INSERT INTO row_home_scalar VALUES ({state},{child},{quote(m['itemFamily'])},NULL,NULL,{quote(str(item))});\n"
                    continue
                if isinstance(v,list):
                    props.append(json.dumps(m['propertyId'])+':'+json.dumps(v,ensure_ascii=False))
                    if m['itemFamily'] is None:continue
                    sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{oid},{typeid},{typeid},{m['propertyId']},{state});\n"
                    sql+=f"INSERT INTO row_home_node(state_id,node_id,parent_node_id,value_kind,slot_kind) VALUES ({state},{state},NULL,'sequence','root');\n"
                    for ordinal,item in enumerate(v):
                        child=100000+ordinal
                        sql+=f"INSERT INTO row_home_node VALUES ({state},{child},{state},'scalar','sequence',{ordinal},NULL,NULL);\n"
                        if m['itemFamily']=='string':
                            sql+=f"INSERT INTO row_home_scalar VALUES ({state},{child},'string',{quote(item)},NULL,NULL);\n"
                        else:
                            sql+=f"INSERT INTO row_home_scalar VALUES ({state},{child},{quote(m['itemFamily'])},NULL,NULL,{quote(str(item))});\n"
                    continue
                leaf=json.dumps(v,ensure_ascii=False) if fam=='string' else v
                props.append(json.dumps(m['propertyId'])+':'+leaf)
                text=quote(v) if fam=='string' else 'NULL'
                boolean=v if fam=='boolean' else 'NULL'
                numeric=quote(v) if fam in ['integer','decimal'] else 'NULL'
                sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{oid},{typeid},{typeid},{m['propertyId']},{state});\n"
                sql+=f"INSERT INTO row_home_node(state_id,node_id,parent_node_id,value_kind,slot_kind) VALUES ({state},{state},NULL,'scalar','root');\n"
                sql+=f"INSERT INTO row_home_scalar VALUES ({state},{state},{quote(fam)},{text},{boolean},{numeric});\n"
            sql+=f"INSERT INTO object VALUES ({oid},{typeid},{quote('{'+','.join(props)+'}')});\n"
    for eid,(source,target) in enumerate(edges,1):
        sql+=f"INSERT INTO edge VALUES ({eid},0,{ids[('Customer',source)]},{types['Customer']},{ids[('Orders',target)]},{types['Orders']});\n"
    # An unrelated type has deliberately invalid selected-name lookalike content.
    sql+="INSERT INTO object VALUES (999999,-99,'{\"0\":\"not-an-integer\",\"1\":\"unrelated\"}');\n"
    return sql
def family(col):
    return col['representation']['logicalType']['family'] if col['representation']['kind']=='scalar' else 'value'
def canonical(v,kind):
    if v is None:return None
    if kind=='value':return json.dumps(json.loads(v) if isinstance(v,str) else v,sort_keys=True,separators=(',',':'))
    if kind=='null':return None
    if kind in ['integer','decimal']:return Decimal(v)
    if kind=='boolean':return v in ['true','t']
    return v
expected_rows['row-alias-collision']=expected_rows['related-page']
map_rows=[('1',{'x':'one','X':'two','x ':'three','é':'combining','é':'composed','':''}),('2',{}),('3',{'emoji😀':'I'})]
expected_rows['map-page']=map_rows
expected_rows['optional-map-page']=[('1',{'state':'absent'}),('2',{'state':'value','value':{}}),('3',{'state':'value','value':{'emoji😀':'I'}})]
expected_rows['numeric-map-page']=[('1',{'10':'9007199254740993','2':'18446744073709551615'}),('2',{}),('3',{'emoji😀':'0'})]
expected_rows['numeric-sequence-page']=[('1',['9007199254740993','9007199254740993']),('2',[]),('3',['18446744073709551615'])]
expected_rows['optional-sequence-page']=[('1',{'state':'absent'}),('2',{'state':'value','value':[]}),('3',{'state':'value','value':['']})]
expected_rows['sequence-page']=[('1',['x','x']),('2',[]),('3',[''])]
expected_rows['related-alias-collision']=expected_rows['related-page']
expected_rows['exists-alias-collision']=expected_rows['has-related-scalar']
expected_rows['related-numeric-order']=[('1',{'items':[['2'],['2']],'truncated':True}),('2',{'items':[['18446744073709551615']],'truncated':False}),('3',{'items':[],'truncated':False})]
def whole(row):return (row['customer-id'],row['customer-name'],row['customer-active'],row['nickname'],row['tags'],row['address'])
expected_rows['whole-entity']=[whole(r) for r in data['customer']]
expected_rows['single-cursor']=[]
expected_rows['whole-cursor-one']=[whole(r) for r in data['customer'][1:]]
expected_rows['related-filter']=[whole(data['customer'][0])]
expected_rows['scalar-page']=[(r['customer-id'],r['nickname'],r['tags'],r['address']) for r in data['customer']]
expected_rows['numeric-structured-page']=[('1',{'state':'value','value':{'street':'Main','zip':{'state':'value','value':'18446744073709551615'}}}),('2',{'state':'absent'}),('3',{'state':'absent'})]
cyclic_address={'state':'value','value':{'street':'Main','zip':{'state':'absent'},'next':{'state':'value','value':{'street':'Second','zip':{'state':'absent'},'next':{'state':'absent'}}}}}
expected_rows['cyclic-structured-props']=[('1',cyclic_address),('2',{'state':'absent'}),('3',{'state':'absent'})]
expected_rows['nested-sequence-props']=[('1',[['9007199254740993','18446744073709551615'],[]]),('2',[]),('3',[['0']])]
expected_rows['structured-page']=[(r['customer-id'],r['address']) for r in data['customer']]
expected_rows['global-sum-complete-tree']=expected_rows['global-sum']
expected_rows['whole-entity-complete-tree']=expected_rows['whole-entity']
expected_rows['unicode-key-order']=[('', '6'),('é','2'),('trail','4'),('trail ','3'),('é','1'),('😀','5')]
expected_rows['unicode-key-cursor']=[('trail ','3'),('é','1'),('😀','5')]
expected_rows['exact-large-key-order']=[('9007199254740992',),('9007199254740993',),('18446744073709551615',)]
expected_rows['empty-global-count']=[('0',)]
expected_rows['empty-grouped-count']=[]
expected_rows['empty-global-sum']=[(None,)]
reports=[]
for c in cases:
    case_seed=json.loads(json.dumps(seed));case_edges=data['edges']
    if c['id'].startswith('empty-'):
        case_seed={};case_edges=[]
    if c['id'].startswith('unicode-key-'):
        case_seed={'Customer':[{'name':n,'id':str(i)} for i,n in enumerate(['é','é','trail ','trail','😀',''],1)]};case_edges=[]
    if c['id'].startswith('exact-large-key-order'):
        case_seed={'Customer':[{'id':n} for n in ['9007199254740993','18446744073709551615','9007199254740992']]};case_edges=[]
    if c['id'].startswith('cyclic-structured-props'):
        case_seed['Customer'][0]['address']['next']={'state':'value','value':{'street':'Second'}}
    if c['id'].startswith('nested-sequence-props'):
        case_seed['Customer'][0]['tags']=[[9007199254740993,18446744073709551615],[]]
        case_seed['Customer'][2]['tags']=[[0]]
    if c['id'].startswith('numeric-structured-page'):
        case_seed['Customer'][0]['address']['zip']={'state':'value','value':'18446744073709551615'}
    if c['id'].startswith(('map-page','optional-map-page','numeric-map-page')):
        for row,(_,values) in zip(case_seed['Customer'],map_rows):row['tags']=values
        if c['id'].startswith('numeric-map-page'):
            case_seed['Customer'][0]['tags']={'10':9007199254740993,'2':18446744073709551615}
            case_seed['Customer'][2]['tags']={'emoji😀':0}
        if c['id'].startswith('optional-map-page'):del case_seed['Customer'][0]['tags']
    if c['id'].startswith('optional-sequence-page'):
        del case_seed['Customer'][0]['tags']
    if c['id'].startswith('numeric-sequence-page'):
        case_seed['Customer'][0]['tags']=[9007199254740993,9007199254740993]
        case_seed['Customer'][2]['tags']=[18446744073709551615]
    if c['id'].startswith('related-numeric-order'):
        for row,new in zip(case_seed['Orders'],['10','2','18446744073709551615']):row['id']=new
        case_edges=[['1','10'],['1','2'],['1','2'],['2','18446744073709551615']]
    raw=subprocess.check_output([BINARY],input=json.dumps(c['request'],ensure_ascii=False).encode()).decode().strip()
    r=json.loads(raw);assert r['status']=='compiled',(c['id'],r)
    assert r['qualification']['status']=='candidate',r['qualification']
    params=r['parameters'];types=','.join({'string':'text','boolean':'bool','integer':'numeric','decimal':'numeric'}[p['logicalType']['family']] for p in params)
    values=','.join(quote(p['value']) for p in params)
    integrity=[check for o in r['obligations'] if o['id'] in ['truss.candidate.scalarIntegrity','truss.candidate.relationshipIntegrity'] for check in o['parameters']['checks']]
    sql='BEGIN ISOLATION LEVEL REPEATABLE READ;\nSET standard_conforming_strings=on;\n'+setup(c['mapping'],case_seed,case_edges,c['request']['modules'])
    sql += "SELECT json_build_object('engine',version(),'serverVersion',current_setting('server_version'),'serverEncoding',current_setting('server_encoding'),'clientEncoding',current_setting('client_encoding'),'standardConformingStrings',current_setting('standard_conforming_strings'),'transactionIsolation',current_setting('transaction_isolation'),'lcCollate',(SELECT datcollate FROM pg_database WHERE datname=current_database()),'lcCtype',(SELECT datctype FROM pg_database WHERE datname=current_database()))::text AS weft_session;\n"
    for i,g in enumerate(integrity):sql+=f"PREPARE check_{i}({types}) AS {g['sql']};\nEXECUTE check_{i}({values});\n"
    sql+=f"PREPARE query({types}) AS {r['sql']};\nEXECUTE query({values});\nROLLBACK;\n"
    result=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-P','null=__WEFT_FIXTURE_NULL__','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
    rows=list(csv.reader(io.StringIO(result)))
    assert rows[0] == ['weft_session'] and len(rows[1]) == 1, (c['id'], rows[:2])
    session = json.loads(rows[1][0])
    assert session['serverEncoding'] == session['clientEncoding'] == 'UTF8', session
    assert session['standardConformingStrings'] == 'on', session
    assert session['transactionIsolation'] == 'repeatable read', session
    rows = rows[2:]
    for i in range(len(integrity)):assert rows[2*i:2*i+2]==[['count'],['0']],(c['id'],rows)
    rows=rows[2*len(integrity):]
    if len(r['columns'])==1:rows=[row if row else [''] for row in rows]
    assert rows[0]==[col['outputName'] for col in r['columns']],(c['id'],rows)
    assert all(len(row)==len(r['columns']) for row in rows[1:]),(c['id'],rows)
    actual=[]
    for row in rows[1:]:
        actual.append(tuple(None if value=='__WEFT_FIXTURE_NULL__' and col['nullable'] else canonical(value,family(col)) for value,col in zip(row,r['columns'],strict=True)))
    name=c['id'].rsplit('-',1)[0]
    expected=[tuple(canonical(value,family(col)) for value,col in zip(row,r['columns'],strict=True)) for row in expected_rows[name]]
    assert Counter(actual)==Counter(expected),(c['id'],actual,expected)
    if 'ORDER BY' in c['request']['sql'].upper():assert actual==expected,(c['id'],'ordered rows',actual,expected)
    edge_checks=[check for o in r['obligations'] if o['id']=='truss.candidate.relationshipIntegrity' for check in o['parameters']['checks']]
    if edge_checks:
        # A corrupt edge must be counted before any data query is published.
        bad='BEGIN ISOLATION LEVEL REPEATABLE READ;\n'+setup(c['mapping'],case_seed,case_edges,c['request']['modules'])+"INSERT INTO edge VALUES (999,0,1,-99,4,-2);\n"
        for i,g in enumerate(edge_checks):bad+=f"PREPARE bad_{i}({types}) AS {g['sql']};\nEXECUTE bad_{i}({values});\n"
        bad+='ROLLBACK;\n'
        observed=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=bad.encode()).decode()
        assert list(csv.reader(io.StringIO(observed)))==[['count'],['1']],(c['id'],observed)
    if c['id'].startswith(('sequence-page','numeric-sequence-page')):
        bad='BEGIN ISOLATION LEVEL REPEATABLE READ;\n'+setup(c['mapping'],case_seed,case_edges,c['request']['modules'])
        if c['home']=='row':
            bad+="UPDATE row_home_node SET sequence_ordinal=3 WHERE slot_kind='sequence' AND sequence_ordinal=1;\n"
        else:
            prop=next(m['propertyId'] for m in c['mapping'] if m['record']=='Customer' and m['name']=='tags')
            bad+=f"UPDATE object SET props=pg_catalog.jsonb_set(props,ARRAY[{quote(prop)},'0'],'true'::jsonb) WHERE id=1;\n"
        for i,g in enumerate(integrity):bad+=f"PREPARE broken_sequence_{i}({types}) AS {g['sql']};\nEXECUTE broken_sequence_{i}({values});\n"
        bad+='ROLLBACK;\n'
        observed=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=bad.encode()).decode()
        assert list(csv.reader(io.StringIO(observed)))==[['count'],['1']],(c['id'],observed)
    if c['id']=='global-sum-row':
        prop=next(m['propertyId'] for m in c['mapping'] if m['record']=='Orders' and m['name']=='total')
        for variant,mutation in [
            ('state', f"ALTER TABLE row_home_state DROP CONSTRAINT row_home_state_pkey; INSERT INTO row_home_state SELECT * FROM row_home_state WHERE object_id=4 AND property_id={prop};"),
            ('root', f"ALTER TABLE row_home_node DROP CONSTRAINT row_home_node_pkey; INSERT INTO row_home_node SELECT * FROM row_home_node WHERE state_id IN (SELECT state_id FROM row_home_state WHERE object_id=4 AND property_id={prop});"),
            ('payload', f"ALTER TABLE row_home_scalar DROP CONSTRAINT row_home_scalar_pkey; INSERT INTO row_home_scalar SELECT * FROM row_home_scalar WHERE state_id IN (SELECT state_id FROM row_home_state WHERE object_id=4 AND property_id={prop});"),
        ]:
            bad='BEGIN ISOLATION LEVEL REPEATABLE READ;\n'+setup(c['mapping'],case_seed,case_edges,c['request']['modules'])+mutation+'\n'
            for i,g in enumerate(integrity):bad+=f"PREPARE duplicate_scalar_{i}({types}) AS {g['sql']};\nEXECUTE duplicate_scalar_{i}({values});\n"
            bad+='ROLLBACK;\n'
            observed=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=bad.encode()).decode()
            assert list(csv.reader(io.StringIO(observed)))==[['count'],['2']],(c['id'],variant,observed)
    if c['id']=='global-sum-row':
        prop=next(m['propertyId'] for m in c['mapping'] if m['record']=='Orders' and m['name']=='total')
        bad='BEGIN ISOLATION LEVEL REPEATABLE READ;\n'+setup(c['mapping'],case_seed,case_edges,c['request']['modules'])
        bad+=f"UPDATE row_home_scalar SET text_value='competing' WHERE state_id IN (SELECT state_id FROM row_home_state WHERE object_id=4 AND property_id={prop});\n"
        for i,g in enumerate(integrity):bad+=f"PREPARE competing_scalar_{i}({types}) AS {g['sql']};\nEXECUTE competing_scalar_{i}({values});\n"
        bad+='ROLLBACK;\n'
        observed=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=bad.encode()).decode()
        assert list(csv.reader(io.StringIO(observed)))==[['count'],['1']],(c['id'],'competing scalar payload',observed)
    if c['id'].startswith('global-sum'):
        prop=next(m['propertyId'] for m in c['mapping'] if m['record']=='Orders' and m['name']=='total')
        for invalid in ['1.001','100000000000000000000000000','-100000000000000000000000000']:
            bad='BEGIN ISOLATION LEVEL REPEATABLE READ;\n'+setup(c['mapping'],case_seed,case_edges,c['request']['modules'])
            if c['home']=='row':
                bad+=f"UPDATE row_home_scalar SET numeric_value={invalid} WHERE state_id IN (SELECT state_id FROM row_home_state WHERE object_id=4 AND property_id={prop});\n"
            else:
                bad+=f"UPDATE object SET props=pg_catalog.jsonb_set(props,ARRAY[{quote(prop)}],'{invalid}'::jsonb) WHERE id=4;\n"
            for i,g in enumerate(integrity):bad+=f"PREPARE decimal_facet_{i}({types}) AS {g['sql']};\nEXECUTE decimal_facet_{i}({values});\n"
            bad+='ROLLBACK;\n'
            observed=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=bad.encode()).decode()
            assert list(csv.reader(io.StringIO(observed)))==[['count'],['1']],(c['id'],invalid,observed)
    if c['id'].startswith('numeric-sequence-page'):
        prop=next(m['propertyId'] for m in c['mapping'] if m['record']=='Customer' and m['name']=='tags')
        for invalid in ['-1','18446744073709551616','1.5']:
            bad='BEGIN ISOLATION LEVEL REPEATABLE READ;\n'+setup(c['mapping'],case_seed,case_edges,c['request']['modules'])
            if c['home']=='row':
                bad+=f"UPDATE row_home_scalar SET numeric_value={invalid} WHERE (state_id,node_id) IN (SELECT state_id,node_id FROM row_home_node WHERE state_id IN (SELECT state_id FROM row_home_state WHERE object_id=1 AND property_id={prop}) AND sequence_ordinal=0);\n"
            else:
                bad+=f"UPDATE object SET props=pg_catalog.jsonb_set(props,ARRAY[{quote(prop)},'0'],'{invalid}'::jsonb) WHERE id=1;\n"
            for i,g in enumerate(integrity):bad+=f"PREPARE facet_{i}({types}) AS {g['sql']};\nEXECUTE facet_{i}({values});\n"
            bad+='ROLLBACK;\n'
            observed=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=bad.encode()).decode()
            assert list(csv.reader(io.StringIO(observed)))==[['count'],['1']],(c['id'],invalid,observed)
    if c['id'].startswith(('optional-sequence-page','optional-map-page')):
        prop=next(m['propertyId'] for m in c['mapping'] if m['record']=='Customer' and m['name']=='tags')
        bad='BEGIN ISOLATION LEVEL REPEATABLE READ;\n'+setup(c['mapping'],case_seed,case_edges,c['request']['modules'])
        if c['home']=='row':
            bad+=f"UPDATE row_home_node SET value_kind='null' WHERE state_id IN (SELECT state_id FROM row_home_state WHERE object_id=2 AND property_id={prop}) AND parent_node_id IS NULL;\n"
        else:
            bad+=f"UPDATE object SET props=pg_catalog.jsonb_set(props,ARRAY[{quote(prop)}],'null'::jsonb) WHERE id=2;\n"
        for i,g in enumerate(integrity):bad+=f"PREPARE invalid_optional_{i}({types}) AS {g['sql']};\nEXECUTE invalid_optional_{i}({values});\n"
        bad+='ROLLBACK;\n'
        observed=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=bad.encode()).decode()
        assert list(csv.reader(io.StringIO(observed)))==[['count'],['1']],(c['id'],observed)
    if c['id'].startswith(('map-page','optional-map-page','numeric-map-page')):
        prop=next(m['propertyId'] for m in c['mapping'] if m['record']=='Customer' and m['name']=='tags')
        bad='BEGIN ISOLATION LEVEL REPEATABLE READ;\n'+setup(c['mapping'],case_seed,case_edges,c['request']['modules'])
        if c['home']=='row':
            bad+="INSERT INTO row_home_node SELECT state_id,999000,parent_node_id,value_kind,slot_kind,sequence_ordinal,map_key,record_field_identity_bytes FROM row_home_node WHERE slot_kind='map' ORDER BY state_id,node_id LIMIT 1;\n"
            bad+="INSERT INTO row_home_scalar SELECT p.state_id,999000,p.scalar_kind,p.text_value,p.boolean_value,p.numeric_value FROM row_home_scalar p JOIN row_home_node n USING(state_id,node_id) WHERE n.slot_kind='map' ORDER BY p.state_id,p.node_id LIMIT 1;\n"
        else:
            oid=3 if c['id'].startswith('optional-map-page') else 1
            key='emoji😀' if oid==3 else ('10' if c['id'].startswith('numeric-map-page') else 'x')
            bad+=f"UPDATE object SET props=pg_catalog.jsonb_set(props,ARRAY[{quote(prop)},{quote(key)}],'true'::jsonb) WHERE id={oid};\n"
        for i,g in enumerate(integrity):bad+=f"PREPARE broken_map_{i}({types}) AS {g['sql']};\nEXECUTE broken_map_{i}({values});\n"
        bad+='ROLLBACK;\n'
        observed=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=bad.encode()).decode()
        assert list(csv.reader(io.StringIO(observed)))==[['count'],['1']],(c['id'],observed)
    if c['id'].startswith('structured-page'):
        mapping=next(m for m in c['mapping'] if m['record']=='Customer' and m['name']=='address')
        prop=mapping['propertyId']
        for variant in ['missing-required','unknown-member','optional-null']:
            bad='BEGIN ISOLATION LEVEL REPEATABLE READ;\n'+setup(c['mapping'],case_seed,case_edges,c['request']['modules'])
            if c['home']=='props':
                if variant=='missing-required':bad+=f"UPDATE object SET props=props #- ARRAY[{quote(prop)},'street'] WHERE id=1;\n"
                elif variant=='unknown-member':bad+=f"UPDATE object SET props=pg_catalog.jsonb_set(props,ARRAY[{quote(prop)},'unknown'],'true'::jsonb) WHERE id=1;\n"
                else:bad+=f"UPDATE object SET props=pg_catalog.jsonb_set(props,ARRAY[{quote(prop)},'zip'],'null'::jsonb) WHERE id=1;\n"
            else:
                root=f"SELECT state_id FROM row_home_state WHERE object_id=1 AND property_id={prop}"
                if variant=='missing-required':bad+=f"DELETE FROM row_home_node WHERE state_id IN ({root}) AND slot_kind='record';\n"
                elif variant=='unknown-member':bad+=f"UPDATE row_home_node SET record_field_identity_bytes=pg_catalog.decode('7b7d','hex') WHERE state_id IN ({root}) AND slot_kind='record';\n"
                else:
                    member=next(m for m in mapping['structuredMembers'] if m['name']=='zip')
                    identity=json.dumps(member['identity'],sort_keys=True,ensure_ascii=False,separators=(',',':')).encode().hex()
                    bad+=f"INSERT INTO row_home_node SELECT state_id,999001,root_node_id,'null','record',NULL,NULL,pg_catalog.decode('{identity}','hex') FROM row_home_state WHERE state_id IN ({root});\n"
            for i,g in enumerate(integrity):bad+=f"PREPARE broken_record_{i}({types}) AS {g['sql']};\nEXECUTE broken_record_{i}({values});\n"
            bad+='ROLLBACK;\n'
            observed=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=bad.encode()).decode()
            assert list(csv.reader(io.StringIO(observed)))==[['count'],['1']],(c['id'],variant,observed)
    topology=[]
    if c['id']=='sequence-page-row':
        prop=next(m['propertyId'] for m in c['mapping'] if m['record']=='Customer' and m['name']=='tags')
        selected=f"SELECT state_id FROM row_home_state WHERE object_id=1 AND property_id={prop}"
        for variant in ['orphan-node','disconnected-cycle','orphan-payload','child-of-scalar','competing-payload','duplicate-state']:
            bad='BEGIN ISOLATION LEVEL REPEATABLE READ;\nSET statement_timeout=2000;\n'+setup(c['mapping'],case_seed,case_edges,c['request']['modules'])
            if variant=='orphan-node':bad+=f"INSERT INTO row_home_node SELECT state_id,999001,999999,'null','sequence',0,NULL,NULL FROM row_home_state WHERE state_id IN ({selected});\n"
            elif variant=='disconnected-cycle':
                for node,parent in [(999001,999002),(999002,999001)]:bad+=f"INSERT INTO row_home_node SELECT state_id,{node},{parent},'null','sequence',0,NULL,NULL FROM row_home_state WHERE state_id IN ({selected});\n"
            elif variant=='orphan-payload':bad+=f"INSERT INTO row_home_scalar SELECT state_id,999001,'string','orphan',NULL,NULL FROM row_home_state WHERE state_id IN ({selected});\n"
            elif variant=='child-of-scalar':bad+=f"INSERT INTO row_home_node SELECT state_id,999001,100000,'null','sequence',0,NULL,NULL FROM row_home_state WHERE state_id IN ({selected});\n"
            elif variant=='competing-payload':bad+=f"UPDATE row_home_scalar SET numeric_value=1 WHERE state_id IN ({selected}) AND node_id=100000;\n"
            else:
                bad+=f"INSERT INTO row_home_state SELECT 999,owner_kind,object_id,object_type_id,property_owner_type_id,property_id,root_node_id FROM row_home_state WHERE state_id IN ({selected});\n"
                bad+=f"INSERT INTO row_home_node SELECT 999,node_id,parent_node_id,value_kind,slot_kind,sequence_ordinal,map_key,record_field_identity_bytes FROM row_home_node WHERE state_id IN ({selected});\n"
                bad+=f"INSERT INTO row_home_scalar SELECT 999,node_id,scalar_kind,text_value,boolean_value,numeric_value FROM row_home_scalar WHERE state_id IN ({selected});\n"
            for i,g in enumerate(integrity):bad+=f"PREPARE topology_{i}({types}) AS {g['sql']};\nEXECUTE topology_{i}({values});\n"
            bad+='ROLLBACK;\n'
            observed=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=bad.encode()).decode()
            violations='2' if variant=='duplicate-state' else '1'
            assert list(csv.reader(io.StringIO(observed)))==[['count'],[violations]],(c['id'],variant,observed)
            topology.append(dict(variant=variant,violations=violations))
    reports.append(dict(id=c['id'],raw=raw,response=r,rows=rows[1:],topology=topology,session=session,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
assert hashlib.sha256(BINARY.read_bytes()).hexdigest()==BINARY_SHA
OUT=Path(os.environ.get('WEFT_TRUSS_OUTPUT',str(ROOT/'target/b005')));OUT.mkdir(parents=True,exist_ok=True)
(OUT/'application-native-reports.json').write_text(json.dumps(reports,ensure_ascii=False))
(OUT/'summary.json').write_text(json.dumps({'status':'passed','cases':len(reports),'binarySha256':BINARY_SHA,'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'corpusSha256':hashlib.sha256((ROOT/'tests/truss-postgresql/fixtures/application-cases.json').read_bytes()).hexdigest(),'scope':'Actual isolated PostgreSQL application fixture execution; candidate profile only.'},indent=2)+'\n')
print(f"{len(reports)} actual candidate compiler/native cases passed (props={sum(c['home']=='props' for c in cases)}, row={sum(c['home']=='row' for c in cases)}).")
