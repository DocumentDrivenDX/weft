import csv,hashlib,io,json,subprocess,sys
prefix=sys.argv[1] if len(sys.argv)>1 else "public"
from pathlib import Path
quote=lambda s:"'"+s.replace("'","''")+"'"
results=[];captures={}
for name in ['nested-sequence','cyclic']:
 capture=Path(f'tests/truss-postgresql/fixtures/{prefix}-{name}-select.json');raw=capture.read_bytes();e=json.loads(raw);captures[name]=hashlib.sha256(raw).hexdigest();slots=e['parameters'];assert len(slots)==4
 owner,member=slots[0]['value'],slots[1]['value'];values=','.join(quote(p['value']) for p in slots)
 if name=='nested-sequence':
  valid=[['9007199254740993','18446744073709551615'],[],['0']];logical=valid;empty=[];emptylogical=[];bad=[[1]];label='tags';absent_count=1
 else:
  valid={'slot.0':'outer','slot.2':{'slot.0':'inner'}};logical={'street':'outer','zip':{'state':'absent'},'next':{'state':'value','value':{'street':'inner','zip':{'state':'absent'},'next':{'state':'absent'}}}};empty={'slot.0':''};emptylogical={'street':'','zip':{'state':'absent'},'next':{'state':'absent'}};bad={'slot.0':'outer','slot.2':{}};label='address';absent_count=0
 cases=[('empty-owner',[],0,[]),('exact-nested',[{member:valid}],0,[{'state':'value','value':logical}]),('empty-value',[{member:empty}],0,[{'state':'value','value':emptylogical}]),('absent',[{}],absent_count,[] if absent_count else [{'state':'absent'}]),('explicit-null',[{member:None}],1,[]),('hidden-malformed',[{member:valid},{member:bad}],1,[]),('wrong-root-kind',[{member:False}],1,[])]
 if name=='nested-sequence':
  for token in ['-1','1.5','18446744073709551616','NaN','Infinity','not-a-number']:
   cases.append(('numeric-domain-'+token,[{member:valid},{member:[[token]]}],1,[]))
 for case,props,expected,logicalrows in cases:
  sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);\n'
  for i,p in enumerate(props):sql+=f'INSERT INTO object VALUES ({i+100},{owner},{quote(json.dumps(p))}::jsonb);\n'
  sql+="INSERT INTO object VALUES (1,999,'[]');\n"
  for i,check in enumerate(e['checks']):sql+=f"PREPARE observation_{i}(int4,text,text,text) AS {check}; EXECUTE observation_{i}({values});\n"
  if not expected:sql+=f"PREPARE projection(int4,text,text,text) AS {e['sql']}; EXECUTE projection({values});\n"
  sql+='ROLLBACK;\n'
  out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();rows=list(csv.reader(io.StringIO(out)));assert all(rows[i*2]==['violations'] for i in range(len(e['checks']))),(name,case,rows)
  counts=[int(rows[i*2+1][0]) for i in range(len(e['checks']))];assert any(counts) if expected else not any(counts)
  rest=rows[2*len(counts):]
  if not expected:assert rest[0]==[label] and [json.loads(r[0]) for r in rest[1:]]==logicalrows,(name,case,rows)
  else:assert not rest
  results.append({'property':name,'case':case,'violations':counts,'queryExecuted':not expected,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
Path(f'docs/helix/04-build/evidence/B-005-{prefix}-recursive-domain-native.json').write_text(json.dumps({'scope':'Frontend-resolved original compound props one-call SELECT/Registry emissions with complete-owner prerequisites; no embedding, row-tree or production qualification','captureSha256':captures,'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n');print('20 original recursive domain native cases passed.')
