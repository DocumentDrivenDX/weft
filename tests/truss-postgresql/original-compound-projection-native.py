import csv,hashlib,io,json,subprocess
from pathlib import Path
quote=lambda s:"'"+s.replace("'","''")+"'"
results=[];captures={}
for name in ['tags','address']:
 capture=Path(f'tests/truss-postgresql/fixtures/original-{name}-projection.json');raw=capture.read_bytes();e=json.loads(raw);captures[name]=hashlib.sha256(raw).hexdigest();slots=e['parameters'];assert len(slots)==3
 owner,member=slots[0]['value'],slots[1]['value'];values=','.join(quote(p['value']) for p in slots)
 valid=['é  ',''] if name=='tags' else {'slot.0':'é  '};logical=valid if name=='tags' else {'street':'é  ','zip':{'state':'absent'}}
 empty=[] if name=='tags' else {'slot.0':'','slot.1':'00123'};emptylogical=empty if name=='tags' else {'street':'','zip':{'state':'value','value':'00123'}}
 cases=[('empty-owner',[],0,[]),('exact',[{member:valid}],0,[{'state':'value','value':logical}]),('empty-value',[{member:empty}],0,[{'state':'value','value':emptylogical}]),('absent',[{}],1 if name=='tags' else 0,[] if name=='tags' else [{'state':'absent'}]),('explicit-null',[{member:None}],1,[]),('hidden-malformed',[{member:valid},{member:[1] if name=='tags' else {}}],1,[])]
 for case,props,expected,logicalrows in cases:
  sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);\n'
  for i,p in enumerate(props):sql+=f'INSERT INTO object VALUES ({i+100},{owner},{quote(json.dumps(p))}::jsonb);\n'
  sql+="INSERT INTO object VALUES (1,999,'[]');\n"
  sql+=f"PREPARE observation(int4,text,text) AS {e['check']}; EXECUTE observation({values});\n"
  if not expected:sql+=f"PREPARE projection(int4,text,text) AS {e['sql']}; EXECUTE projection({values});\n"
  sql+='ROLLBACK;\n'
  out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();rows=list(csv.reader(io.StringIO(out)));assert rows[:2]==[['violations'],[str(expected)]],(name,case,rows)
  if not expected:assert rows[2]==[name] and [json.loads(r[0]) for r in rows[3:]]==logicalrows,(name,case,rows)
  else:assert len(rows)==2
  results.append({'property':name,'case':case,'violations':expected,'queryExecuted':not expected,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
Path('docs/helix/04-build/evidence/B-005-original-compound-projection-native.json').write_text(json.dumps({'scope':'Frontend-resolved original compound props access and result projection with complete-owner prerequisite; no public Registry/embedding or recursive row-tree qualification','captureSha256':captures,'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n');print('12 original compound projection native cases passed.')
