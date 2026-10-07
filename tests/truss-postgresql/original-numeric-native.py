import csv,hashlib,io,json,subprocess,sys
from pathlib import Path
capture,receipt=map(Path,sys.argv[1:3]);raw=capture.read_bytes();e=json.loads(raw)
quote=lambda s:"'"+s.replace("'","''")+"'"
slots=e['parameters'];owner=next(p['value'] for p in slots if p['origin']['use']=='admitted-owner-scan');member=next(p['value'] for p in slots if p['origin']['use']=='admitted-jsonb-member')
types=','.join('int4' if p['origin']['use']=='admitted-owner-scan' else 'text' for p in slots);values=','.join(quote(p['value']) for p in slots)
results=[]
for case,tokens,corrupt,expected in [('empty',[],False,'__null__'),('exact',['9999999999999999999.12','-0.01','0.02'],False,'9999999999999999999.13'),('wrong-kind',['1.00'],True,None)]:
 sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);\n'
 for i,token in enumerate(tokens):sql+=f"INSERT INTO object VALUES ({i+1},{owner},{quote(json.dumps({member:token}))});\n"
 sql+="INSERT INTO object VALUES (100,999,'[]');\n"
 if corrupt:sql+=f"INSERT INTO object VALUES (101,{owner},{quote(json.dumps({member:False}))});\n"
 for i,check in enumerate(e['checks']):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({values});\n'
 if not corrupt:sql+=f"PREPARE query({types}) AS {e['sql']}; EXECUTE query({values});\n"
 sql+='ROLLBACK;\n'
 out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-P','null=__null__','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();rows=list(csv.reader(io.StringIO(out)))
 counts=[int(rows[i*2+1][0]) for i in range(len(e['checks']))];assert any(counts) if corrupt else not any(counts)
 assert rows[2*len(counts):]==([] if corrupt else [['total'],[expected]])
 results.append({'case':case,'violations':counts,'result':expected,'queryExecuted':not corrupt,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
receipt.write_text(json.dumps({'scope':'Original decimal(28,2) UMF/property/codec/comparator V01 and V02 identical SUM SQL on synthetic JSONB rows; no complete domain/production qualification','captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n');print('3 original decimal SUM native cases passed.')
