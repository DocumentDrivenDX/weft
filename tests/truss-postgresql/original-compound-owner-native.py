import csv,hashlib,io,json,subprocess
from pathlib import Path
quote=lambda s:"'"+s.replace("'","''")+"'"
results=[];captures={}
for name in ['tags','address']:
 capture=Path(f'tests/truss-postgresql/fixtures/original-{name}-owner-observation.json');raw=capture.read_bytes();e=json.loads(raw);captures[name]=hashlib.sha256(raw).hexdigest();slots=e['parameters'];assert len(slots)==3
 owner,member=slots[0]['value'],slots[1]['value'];values=','.join(quote(p['value']) for p in slots)
 valid=['é  ',''] if name=='tags' else {'slot.0':'é  '}
 bad=[1] if name=='tags' else {'slot.0':'x','unknown':'x'}
 for case,props,expected in [('empty',[],0),('exact',[{member:valid}],0),('root-array',[[]],1),('hidden-malformed',[{member:valid},{member:bad}],1),('absent',[{}],1 if name=='tags' else 0),('explicit-null',[{member:None}],1)]:
  sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);\n'
  for i,p in enumerate(props):sql+=f'INSERT INTO object VALUES ({i+100},{owner},{quote(json.dumps(p))}::jsonb);\n'
  sql+="INSERT INTO object VALUES (1,999,'[]');\n"
  sql+=f"PREPARE observation(int4,text,text) AS {e['sql']}; EXECUTE observation({values}); ROLLBACK;\n"
  out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();rows=list(csv.reader(io.StringIO(out)));assert rows==[['violations'],[str(expected)]],(name,case,rows)
  results.append({'property':name,'case':case,'violations':expected,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
Path('docs/helix/04-build/evidence/B-005-original-compound-owner-native.json').write_text(json.dumps({'scope':'Original-admitted compound props home complete-owner physical prerequisites; no public projection, recursive row-tree or production qualification','captureSha256':captures,'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n');print('12 original compound owner native cases passed.')
