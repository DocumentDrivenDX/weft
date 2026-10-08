"""Independent native vectors for the scalar carrier used by row projection."""
import csv,hashlib,io,json,subprocess,sys
from pathlib import Path
capture,receipt=map(Path,sys.argv[1:3]);raw=capture.read_bytes();definitions=json.loads(raw)
vectors=[
 ('string-empty','string','',None,None,None,''),
 ('string-unicode','string','é😀  ',None,None,None,'é😀  '),
 ('boolean-false','boolean',None,False,None,None,'false'),
 ('boolean-true','boolean',None,True,None,None,'true'),
 ('integer-uint64','integer',None,None,'18446744073709551615','18446744073709551615','18446744073709551615'),
 ('integer-int64-min','integer',None,None,'-9223372036854775808','-9223372036854775808','-9223372036854775808'),
 ('decimal-negative-zero','decimal',None,None,'0.00','-0.00','-0.00'),
 ('decimal-exact-scale','decimal',None,None,'12345678901234567890.00100','12345678901234567890.00100','12345678901234567890.00100'),
]
columns=['node_id','scalar_kind','text_value','boolean_value','numeric_value','numeric_token','binary_value','temporal_text','temporal_instant','opaque_bytes','codec_definition_bytes','original_source_bytes']
types=['bigint','text','text','bool','numeric','text','bytea','text','timestamptz','bytea','bytea','bytea']
def literal(value,datatype):
 if value is None:return 'NULL::'+datatype
 if datatype=='bytea':return "pg_catalog.decode('"+value+"','hex')"
 if datatype=='bool':return ('true' if value else 'false')+'::bool'
 return "'"+str(value).replace("'","''")+"'::"+datatype
results=[]
for definition in definitions:
 for name,family,text,boolean,numeric,token,expected in vectors:
  selected=next(p for p in definition['payloads'] if p['family']==family)
  values=[20,family,text,boolean,numeric,token,None,None,None,None,'00ff','fe00']
  sql='SELECT '+selected['integrity']+' AS integrity, ('+selected['carrier']+')::pg_catalog.text AS value FROM (VALUES ('+','.join(literal(v,t) for v,t in zip(values,types))+')) AS "weft_scalar_3"('+','.join('"'+c+'"' for c in columns)+');'
  output=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
  rows=list(csv.reader(io.StringIO(output)));assert rows==[['integrity','value'],['t',expected]],(definition['kind'],name,rows)
  results.append({'ownerKind':definition['kind'],'id':name,'value':rows[1][1],'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
version=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt.write_text(json.dumps({'scope':'Actual row scalar carrier expression and physical payload predicate for four UMF families under object/edge captured aliases; independent valid synthetic values, not complete original property/decoder/host qualification','server':version,'captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},ensure_ascii=False,indent=2)+'\n')
print('16 native row scalar result carriers passed.')
