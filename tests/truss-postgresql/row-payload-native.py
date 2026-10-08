"""Independent physical-slot oracle for Rust-captured row payload predicates."""
import csv, hashlib, io, json, subprocess, sys
from pathlib import Path
capture = Path(sys.argv[1]); definitions = json.loads(capture.read_text())
columns = ['node_id','scalar_kind','text_value','boolean_value','numeric_value','numeric_token','binary_value','temporal_text','temporal_instant','opaque_bytes','codec_definition_bytes','original_source_bytes']
types = ['bigint','text','text','bool','numeric','text','bytea','text','timestamptz','bytea','bytea','bytea']
def literal(value, datatype):
    if value is None: return 'NULL::'+datatype
    if datatype == 'bytea': return "pg_catalog.decode('"+value+"','hex')"
    if datatype == 'bool': return ('true' if value else 'false')+'::bool'
    return "'"+str(value).replace("'","''")+"'::"+datatype
queries = []; expected = []
for definition in definitions:
    for payload in definition['payloads']:
        family = payload['family']
        base = dict.fromkeys(columns); base.update(node_id=20,scalar_kind=family,codec_definition_bytes='00ff',original_source_bytes='fe00')
        if family == 'string': base['text_value']='é😀  '
        elif family == 'boolean': base['boolean_value']=False
        else:
            base['numeric_value']='18446744073709551615' if family=='integer' else '0.00'
            base['numeric_token']='18446744073709551615' if family=='integer' else '-0.00'
        scenarios = ['valid','missing-selected','wrong-token-slot','wrong-kind','mixed','missing-codec','missing-source','other-slot']
        if family in ('integer','decimal'): scenarios.append('domain-mismatch')
        for scenario in scenarios:
            row = base.copy(); valid = scenario in ('valid','domain-mismatch')
            if scenario == 'missing-selected': row[{'string':'text_value','boolean':'boolean_value'}.get(family,'numeric_value')]=None
            if scenario == 'wrong-token-slot': row['numeric_token']=None if family in ('integer','decimal') else '1'
            if scenario == 'wrong-kind': row['scalar_kind']='other'
            if scenario == 'mixed': row['text_value']='foreign' if family!='string' else row['text_value']; row['boolean_value']=True if family!='boolean' else row['boolean_value']
            if scenario == 'missing-codec': row['codec_definition_bytes']=None
            if scenario == 'missing-source': row['original_source_bytes']=None
            if scenario == 'other-slot': row['binary_value']=''
            if scenario == 'domain-mismatch': row['numeric_value']='2'; row['numeric_token']='1'
            tag = definition['kind']+':'+family+':'+scenario
            queries.append("SELECT '"+tag+"' AS scenario, "+payload['integrity']+' AS integrity, '+definition['numeric']+' AS numeric_text, '+definition['token']+' AS token, '+definition['codec']+' AS codec, '+definition['source']+' AS source FROM (VALUES ('+','.join(literal(row[c],t) for c,t in zip(columns,types))+')) AS "weft_scalar_3"('+','.join('"'+c+'"' for c in columns)+');')
            expected.append([tag,'t' if valid else 'f',row['numeric_value'] or '',row['numeric_token'] or '',row['codec_definition_bytes'] or '',row['original_source_bytes'] or ''])
sql='\n'.join(queries)
result=subprocess.run(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql,text=True,capture_output=True,check=True)
observed=[r for r in csv.reader(io.StringIO(result.stdout)) if r and r[0]!='scenario']
assert observed==expected, (observed,expected)
version=subprocess.run(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-At','-c','SELECT version()'],text=True,capture_output=True,check=True).stdout.strip()
report={'scope':'Physical slot and custody expressions only; mismatched numeric value/token deliberately passes for later semantic procedure refusal','server':version,'scenarios':len(expected),'captureSha256':hashlib.sha256(capture.read_bytes()).hexdigest(),'sqlSha256':hashlib.sha256(sql.encode()).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'observations':observed}
Path(sys.argv[2]).write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'scenarios':len(expected),'passed':True}))
