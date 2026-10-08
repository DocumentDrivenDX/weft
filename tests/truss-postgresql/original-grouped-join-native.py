"""Original binding grouped equality join through actual Python and PostgreSQL.
@covers US-003-AC1: independent grouped sums preserve join bag multiplicity.
@covers US-003-AC2: authored keys/fields and repeated source type filtering.
@covers US-003-AC3: owner corruption refuses before query execution.
Synthetic candidate self/cross-record joins; not original name/total qualification.
"""
import copy,csv,hashlib,io,json,os,subprocess,sys
from pathlib import Path
import weft
ROOT=Path(__file__).resolve().parents[2];F=ROOT/'tests/truss-postgresql/fixtures'
source=F/'original-multi-recursive-entity-inputs.json';cut=json.loads(source.read_text())
request=copy.deepcopy(cut['requests'][0]['request'])
cross='--cross-record' in sys.argv
request['sql']='SELECT c.part, SUM(o.id) AS total FROM Customer c JOIN Customer o ON o.part = c.part GROUP BY c.part'
if cross:request['sql']=request['sql'].replace('JOIN Customer o','JOIN Orders o')
configuration=json.dumps(cut['composition']);raw=weft.compile_json_with_conformance_configuration(json.dumps(request),configuration)
assert raw==weft.compile_json_with_conformance_configuration(json.dumps(request),configuration)
r=json.loads(raw);assert r['status']=='compiled',r
assert 'INNER JOIN' in r['sql'] and 'GROUP BY' in r['sql']
checks=[o['parameters']['sql'] for o in r['obligations'] if 'sql' in o['parameters']]
binding=json.loads(request['target']['bindingJson']);codecs={int(binding['properties'][p['index']]['propertyId']):p['leafCodecs']['root']['originalJson'].encode().hex() for p in cut['composition']['properties'] if not p.get('nativeTree',True)}
quote=lambda v:"'"+str(v).replace("'","''")+"'"
results=[]
cases=['empty','bag-multiplicity','unicode-and-spaces','wrong-codec','out-of-domain','repeated-numeric-values']
if cross:cases += ['wrong-right-codec','right-out-of-domain']
for case in cases:
    corrupt=case in ['wrong-codec','out-of-domain','wrong-right-codec','right-out-of-domain']
    owners=[] if case=='empty' else [(1,'9007199254740993','A'),(2,'18446744073709551615','A'),(3,'7','A '),(4,'11','é'),(5,'13','e\u0301')]
    if case=='bag-multiplicity':owners=owners[:2]
    if case=='repeated-numeric-values':owners[1]=(2,owners[0][1],'B')
    if case=='out-of-domain':owners[1]=(2,'18446744073709551616','A')
    left_owners=owners+([(7,'23','unmatched-left')] if cross and owners else [])
    right_owners=[(owner+100,value,part) for owner,value,part in owners]+([(106,'19','unmatched')] if owners else []) if cross else owners
    if case=='right-out-of-domain':right_owners[1]=(102,'18446744073709551616','A')
    storage=[(owner,value,part,-1,0,20) for owner,value,part in left_owners]
    if cross:storage += [(owner,value,part,-2,6,21) for owner,value,part in right_owners]
    sql='''BEGIN; SET standard_conforming_strings=on;
CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text,record_field_identity_bytes bytea,definition_bytes bytea,source_bytes bytea);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,temporal_text text,temporal_instant timestamptz,binary_value bytea,opaque_bytes bytea,codec_definition_bytes bytea,original_source_bytes bytea);
'''
    for owner,value,part,type_id,id_pid,part_pid in storage:
        sql+=f"INSERT INTO object VALUES ({owner},{type_id},'{{}}'::jsonb);\n"
        for pid,family,v in [(id_pid,'integer',value),(part_pid,'string',part)]:
            state=owner*100+pid;node=state+10000;codec='00' if (case=='wrong-codec' and owner==2 and pid==(part_pid if cross else id_pid)) or (case=='wrong-right-codec' and owner==102 and pid==id_pid) else codecs[pid]
            text=quote(v) if family=='string' else 'NULL';numeric=v if family=='integer' else 'NULL';token=quote(v) if family=='integer' else 'NULL'
            sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{owner},{type_id},NULL,NULL,{type_id},{pid},{node}); INSERT INTO row_home_node(state_id,node_id,parent_node_id,value_kind) VALUES ({state},{node},NULL,'scalar'); INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,text_value,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES ({state},{node},'{family}',{text},{numeric},{token},decode('{codec}','hex'),decode('fe00','hex'));\n"
    # Matching property spelling on an unrelated type must enter neither side.
    sql+="INSERT INTO object VALUES (999,-999,'{\"0\":\"18446744073709551615\",\"20\":\"A\"}'::jsonb);\n"
    types=','.join('text' for _ in r['parameters']);args=','.join(quote(p['value']) for p in r['parameters'])
    for i,check in enumerate(checks):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
    if not corrupt:sql+=f"PREPARE q({types}) AS {r['sql']}; EXECUTE q({args});\n"
    sql+='ROLLBACK;\n'
    out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();rows=list(csv.reader(io.StringIO(out)));counts=[]
    for i in range(len(checks)):assert rows[2*i]==['violations'];counts.append(int(rows[2*i+1][0]))
    assert any(counts) if corrupt else not any(counts),(case,counts)
    remaining=rows[len(counts)*2:]
    if corrupt:assert not remaining
    else:
        assert remaining[0]==['part','total']
        # Independent relational nested loops, exact Python integers, no SQL oracle.
        expected={}
        for _,_,left in left_owners:
            for _,value,right in right_owners:
                if left==right:expected[left]=expected.get(left,0)+int(value)
        assert sorted(remaining[1:])==sorted([[key,str(value)] for key,value in expected.items()]),(case,remaining,expected)
    results.append(dict(case=case,violations=counts,queryExecuted=not corrupt,sqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original candidate grouped equality '+('Customer/Orders join' if cross else 'self-join')+' over native UInt64 and Unicode C roots; no original name/total-field or production qualification',cases=len(results),server=server,binaryProvenance='B-005-numeric-sequence native extension reused; compiler runtime unchanged',deterministicPythonRepeat=True,sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),responseSha256=hashlib.sha256(raw.encode()).hexdigest(),results=results)
(ROOT/('docs/helix/04-build/evidence/B-005-original-grouped-'+('cross-record-' if cross else '')+'join-native.json')).write_text(json.dumps(receipt,indent=2)+'\n')
print(f'{len(results)} original grouped join native cases passed')
