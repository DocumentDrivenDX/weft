import csv,hashlib,io,json,subprocess,sys
from pathlib import Path
capture,receipt=map(Path,sys.argv[1:3]);raw=capture.read_bytes();e=json.loads(raw)
quote=lambda s:"'"+s.replace("'","''")+"'"
slots=e['parameters'];owner=next(p['value'] for p in slots if p['origin']['use']=='admitted-owner-scan');member=next(p['value'] for p in slots if p['origin']['use']=='admitted-jsonb-member')
types=','.join('int4' if p['origin']['use']=='admitted-owner-scan' else 'text' for p in slots);values=','.join(quote(p['value']) for p in slots)
results=[]
for case,tokens,corrupt,expected in [('empty',[],False,'__null__'),('exact',['18446744073709551615','18446744073709551615'],False,'36893488147419103230'),('wrong-kind',['1'],True,None),('fraction',['0.1'],True,None),('negative',['-1'],True,None),('overflow',['18446744073709551616'],True,None),('nonfinite',['Infinity'],True,None),('invalid-native',['not-a-number'],True,None)]:
 sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);\n'
 for i,token in enumerate(tokens):sql+=f"INSERT INTO object VALUES ({i+1},{owner},{quote(json.dumps({member:token}))});\n"
 sql+="INSERT INTO object VALUES (100,999,'[]');\n"
 if case=='wrong-kind':sql+=f"INSERT INTO object VALUES (101,{owner},{quote(json.dumps({member:False}))});\n"
 for i,check in enumerate(e['checks']):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({values});\n'
 if not corrupt:sql+=f"PREPARE query({types}) AS {e['sql']}; EXECUTE query({values});\n"
 sql+='ROLLBACK;\n'
 out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-P','null=__null__','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();rows=list(csv.reader(io.StringIO(out)))
 counts=[int(rows[i*2+1][0]) for i in range(len(e['checks']))];assert any(counts) if corrupt else not any(counts)
 if corrupt and case!='wrong-kind':assert counts[-1]>0 and not any(counts[:-1])
 assert rows[2*len(counts):]==([] if corrupt else [['total'],[expected]])
 results.append({'case':case,'violations':counts,'result':expected,'queryExecuted':not corrupt,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
receipt.write_text(json.dumps({'scope':'Original uint64 UMF/property/codec/comparator V01 and V02 identical SUM SQL on synthetic JSONB rows; no complete domain/production qualification','captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n');print('8 original uint64 SUM native cases passed.')
