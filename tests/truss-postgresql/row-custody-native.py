"""Exact private scalar custody columns, independently authored native vectors."""
import csv, hashlib, io, json, subprocess, sys
from pathlib import Path
capture, receipt = map(Path, sys.argv[1:3]); raw = capture.read_bytes()
definitions = json.loads(raw)
columns = ['node_id','scalar_kind','text_value','boolean_value','numeric_value','numeric_token','codec_definition_bytes','original_source_bytes']
types = ['bigint','text','text','bool','numeric','text','bytea','bytea']
headers = ['scalar_present','scalar_kind','text_value','boolean_value','native_numeric_text','original_numeric_token','codec_bytes_hex','source_bytes_hex']
vectors = [
 ('unicode',[1,'string','é😀  ',None,None,None,'00ff','fe00'],['true','string','é😀  ',None,None,None,'00ff','fe00']),
 ('empty',[1,'string','',None,None,None,'',''],['true','string','',None,None,None,'','']),
 ('false',[1,'boolean',None,False,None,None,'00','ff'],['true','boolean',None,'false',None,None,'00','ff']),
 ('true',[1,'boolean',None,True,None,None,'00','ff'],['true','boolean',None,'true',None,None,'00','ff']),
 ('uint64',[1,'integer',None,None,'18446744073709551615','18446744073709551615','00','ff'],['true','integer',None,None,'18446744073709551615','18446744073709551615','00','ff']),
 ('negative-zero',[1,'decimal',None,None,'0.00','-0.00','00','ff'],['true','decimal',None,None,'0.00','-0.00','00','ff']),
 ('mismatch',[1,'decimal',None,None,'2','1','00','ff'],['true','decimal',None,None,'2','1','00','ff']),
 ('absent',[None]*8,['false']+[None]*7),
]
def literal(value, datatype):
 if value is None: return 'NULL::'+datatype
 if datatype=='bytea': return "pg_catalog.decode('"+value+"','hex')"
 if datatype=='bool': return ('true' if value else 'false')+'::bool'
 return "'"+str(value).replace("'","''")+"'::"+datatype
results=[]
for definition in definitions:
 for name,values,expected in vectors:
  sql='SELECT '+','.join(definition['custody'])+' FROM (VALUES ('+','.join(literal(v,t) for v,t in zip(values,types))+')) AS "weft_scalar_3"('+','.join('"'+c+'"' for c in columns)+');'
  output=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-P','null=__native_null__','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
  rows=list(csv.reader(io.StringIO(output))); assert rows[0]==headers
  actual=[None if v=='__native_null__' else v for v in rows[1]]
  assert actual==expected,(definition['kind'],name,actual,expected)
  results.append({'kind':definition['kind'],'case':name,'values':actual,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
version=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt.write_text(json.dumps({'scope':'Private scalar custody projection over synthetic native values, preserving NULL/empty, native numeric versus original token and exact binary bytes; not codec/domain correspondence, logical result decoding, installed layout or host qualification','server':version,'captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},ensure_ascii=False,indent=2)+'\n')
print('16 native scalar custody vectors passed.')
