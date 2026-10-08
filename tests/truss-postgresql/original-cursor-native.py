import csv,hashlib,io,json,subprocess,sys
from pathlib import Path
capture,receipt=map(Path,sys.argv[1:3]);raw=capture.read_bytes();e=json.loads(raw);slots=e['parameters']
quote=lambda s:"'"+s.replace("'","''")+"'"
owner=next(p['value'] for p in slots if p['origin'].get('use')=='admitted-owner-scan');member=next(p['value'] for p in slots if p['origin'].get('use')=='admitted-jsonb-member')
types=','.join('int4' if p['origin'].get('use')=='admitted-owner-scan' else 'text' for p in slots);values=','.join(quote(p['value']) for p in slots);results=[]
cursor=next(int(p['value']) for p in slots if p['origin'].get('parameter')=='cursor');assert cursor==2
for case,tokens,corrupt in [('empty',[],False),('numeric-order',['10','2','18446744073709551615','3'],False),('exact-adjacent',['9007199254740993','9007199254740992'],False),('equal-and-below',['1','2'],False),('maximum',['18446744073709551615'],False),('hidden-by-limit',['2','3'],True),('duplicate-key',['1','1','3','4'],True),('invalid-key',['-1','3','4'],True)]:
 sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);\n'
 for i,token in enumerate(tokens):sql+=f"INSERT INTO object VALUES ({987654321+i},{owner},{quote(json.dumps({member:token}))});\n"
 sql+="INSERT INTO object VALUES (1,999,'[]');\n"
 if case=='hidden-by-limit':sql+=f"INSERT INTO object VALUES (999,{owner},{quote(json.dumps({member:False}))});\n"
 for i,check in enumerate(e['checks']):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({values});\n'
 if not corrupt:sql+=f"PREPARE query({types}) AS {e['sql']}; EXECUTE query({values});\n"
 sql+='ROLLBACK;\n'
 out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();rows=list(csv.reader(io.StringIO(out)))
 counts=[int(rows[i*2+1][0]) for i in range(len(e['checks']))];assert any(counts) if corrupt else not any(counts)
 expected=[] if corrupt else [['id']]+[[token] for token in sorted((t for t in tokens if int(t)>cursor),key=int)[:2]]
 assert rows[2*len(counts):]==expected,(case,rows,expected)
 results.append({'case':case,'violations':counts,'queryExecuted':not corrupt,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
receipt.write_text(json.dumps({'scope':'Public original UMF uint64 named-cursor page, exact numeric comparison and complete-owner prerequisites over synthetic props storage; no production or embedding qualification','captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n');print('8 public numeric cursor native cases passed.')
