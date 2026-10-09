"""Host result check for the explicit experimental mathematical integer profile.

Caller admits the original compiler artifact and all source obligations first.
These exact host values are not canonical UMF wire values or publication authority.
"""
import re
def decode_rows(artifact,rows):
    backend=artifact.get('backend',{})
    if artifact.get('status')!='compiled'or backend.get('backendId')!='ashlar.databricks.mathematical-integer'or backend.get('backendVersion')!='0.1.0-candidate'or backend.get('targetProfile')!='dbsql-mathematical-integer-candidate':raise ValueError('Exact experimental backend profile required')
    columns=artifact.get('columns');names=[]
    if not isinstance(columns,list)or not columns:raise ValueError('Original integer columns required')
    for i,c in enumerate(columns,1):
        t=c.get('representation',{});logical=t.get('logicalType',{})
        if type(c.get('position'))is not int or c['position']!=i or set(t)!={'kind','logicalType','carrier','decoder'}or set(logical)!={'family','facets','nullable'}or type(c.get('nullable'))is not bool or type(logical.get('nullable'))is not bool or c['nullable']is not logical['nullable']or t.get('kind')!='scalar'or t.get('carrier')!='text'or t.get('decoder')!='exact-integer'or logical.get('family')!='integer'or logical.get('facets')!={}:raise ValueError('Original mathematical integer representation required')
        name=c.get('outputName')
        if not isinstance(name,str)or name in names:raise ValueError('Original ordered unique output names required')
        names.append(name)
    if not isinstance(rows,list):raise ValueError('Buffered result bag required')
    decoded=[]
    for row in rows:
        if not isinstance(row,dict)or set(row)!=set(names):raise ValueError('Complete original result columns required')
        values={}
        for c,name in zip(columns,names):
            value=row[name]
            if value is None:
                if c['nullable']is not True:raise ValueError('Non-null result required')
                values[name]=None;continue
            if not isinstance(value,str)or len(value)>128 or re.fullmatch(r'-?[0-9]+',value)is None:raise ValueError('Exact original integer text required')
            if len(value.lstrip('-').lstrip('0'))>38:raise ValueError('Backend representation capability exceeded')
            values[name]={'exactInteger':str(int(value)),'originalCarrier':value}
        decoded.append(values)
    return decoded
