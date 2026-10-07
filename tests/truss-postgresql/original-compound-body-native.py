import hashlib,json,subprocess
from pathlib import Path
quote=lambda s:"'"+s.replace("'","''")+"'"
results=[];captures={}
for name,cases in {
 'tags': [('exact',['é  ','1.a[0]',''],True),('empty',[],True),('number',[1],False),('null-leaf',[None],False),('object',{},False),('native-null',None,False),('absent',None,False)],
 'address':[('required',{'slot.0':'é  '},True),('optional-present',{'slot.0':'','slot.1':'00123'},True),('missing-required',{},False),('null-leaf',{'slot.0':None},False),('unknown',{'slot.0':'x','unknown':'x'},False),('wrong-name',{'street':'x'},False),('array',[],False),('native-null',None,False),('absent',None,True)]
}.items():
 capture=Path(f'tests/truss-postgresql/fixtures/original-{name}-observation.json');raw=capture.read_bytes();e=json.loads(raw);captures[name]=hashlib.sha256(raw).hexdigest()
 for case,value,expected in cases:
  carrier='NULL' if case=='absent' else quote(json.dumps(value))+'::jsonb'
  sql=f"PREPARE observation(text) AS SELECT {e['sql']} AS integrity FROM (VALUES ({carrier})) fixture(v); EXECUTE observation({quote(e['parameters'][0]['value'])});\n"
  out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();assert out.strip().splitlines()==['integrity','t' if expected else 'f'],(name,case,out)
  if expected and case!='absent':
   body_sql=f"PREPARE body(text) AS SELECT {e['body']} AS body FROM (VALUES ({carrier})) fixture(v); EXECUTE body({quote(e['parameters'][0]['value'])});\n"
   body_out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=body_sql.encode()).decode()
   import csv,io
   rows=list(csv.reader(io.StringIO(body_out)));assert rows[0]==['body'];actual=json.loads(rows[1][0])
   logical=value if name=='tags' else {'street':value['slot.0'],'zip':{'state':'value','value':value['slot.1']} if 'slot.1' in value else {'state':'absent'}}
   assert actual==logical,(name,case,actual,logical)
  results.append({'property':name,'case':case,'integrity':expected,'bodyChecked':expected and case!='absent','sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
Path('docs/helix/04-build/evidence/B-005-original-compound-body-native.json').write_text(json.dumps({'scope':'Original-topology recursive JSONB observation and logical result SQL on admitted sequence/structured props; no public projection or production qualification','captureSha256':captures,'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print(f'{len(results)} original compound observation native cases passed.')
