"""Independent typed fixture writer; not a Truss producer qualification."""
import json
from decimal import Decimal

def literal(value):
    return "'"+str(value).replace("'","''")+"'"
def wire(value):
    if isinstance(value,Decimal):return format(value,'f')
    if isinstance(value,list):return '['+','.join(wire(v) for v in value)+']'
    if isinstance(value,dict):return '{'+','.join(json.dumps(k,ensure_ascii=False)+':'+wire(v) for k,v in value.items())+'}'
    return json.dumps(value,ensure_ascii=False)

def tree(modules,logical,value,state):
    module_input=next(m for m in modules if m['pin']['documentId']==logical['documentId'])
    doc=json.loads(module_input['documentJson']);revision=module_input['pin']['revision']
    elements={(m['id'],e['id']):e for m in doc['modules'] for e in m['elements']}
    statements=[];next_node=100000
    def emit(ref,cell,parent=None,slot='root',ordinal=None,key=None,node=None):
        nonlocal next_node
        field=elements[(ref['module'],ref['element'])]
        if isinstance(cell,dict) and cell.get('state') in ['absent','value','null'] and set(cell)<=set(['state','value']):
            assert cell['state']!='null','Fixture native-null interpretation is not qualified'
            if cell['state']=='absent':
                assert field['nullability']=='absent-allowed' and slot=='record'
                return None
            cell=cell['value']
        if node is None:node=next_node;next_node+=1
        cardinality=field['cardinality'];family=field.get('scalarType')
        kind='sequence' if cardinality=='array' else 'map' if cardinality=='map' else 'scalar' if family else 'structured'
        identity=dict(documentId=doc['id'],revision=revision,module=ref['module'],element=ref['element'])
        identity_sql="pg_catalog.decode('"+json.dumps(identity,sort_keys=True,ensure_ascii=False,separators=(',',':')).encode().hex()+"','hex')" if slot=='record' else 'NULL'
        statements.append(f"INSERT INTO row_home_node VALUES ({state},{node},{parent if parent is not None else 'NULL'},{literal(kind)},{literal(slot)},{ordinal if ordinal is not None else 'NULL'},{literal(key) if key is not None else 'NULL'},{identity_sql});\n")
        if family:
            if family=='integer':cell=int(cell)
            elif family=='decimal':cell=Decimal(str(cell))
            elif family=='boolean':cell=(cell=='true') if isinstance(cell,str) else cell
            text=literal(cell) if family=='string' else 'NULL'
            boolean=str(cell).lower() if family=='boolean' else 'NULL'
            numeric=literal(cell) if family in ['integer','decimal'] else 'NULL'
            statements.append(f"INSERT INTO row_home_scalar VALUES ({state},{node},{literal(family)},{text},{boolean},{numeric});\n")
            return cell
        if cardinality=='array':return [emit(field['itemType'],v,node,'sequence',i) for i,v in enumerate(cell)]
        if cardinality=='map':return {k:emit(field['itemType'],v,node,'map',key=k) for k,v in cell.items()}
        record_ref=next(r for r in field['references'] if r['role']=='record-type')
        record=elements[(record_ref['module'],record_ref['element'])];result={}
        for member in record['members']:
            definition=elements[(member['module'],member['element'])];name=definition['name']
            if name not in cell:
                assert definition['nullability']=='absent-allowed';continue
            decoded=emit(member,cell[name],node,'record')
            if decoded is not None:result[name]=decoded
        return result
    raw=emit(logical,value,node=state)
    return wire(raw),''.join(statements)
