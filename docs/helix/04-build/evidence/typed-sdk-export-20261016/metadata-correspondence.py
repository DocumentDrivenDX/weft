import json,pathlib,sys,hashlib,weft
request=json.load(open('/private/tmp/weft-reporting-integration/tests/engine-catalog/native-truss-export-request.json'))
values=[]
for token in ['export',"export' OR true --",'other']:
 current=json.loads(json.dumps(request));current['parameters']['status']['value']=token
 page=weft.compile_engine_page_json(json.dumps({'version':'weft.engine-page/0.1','compiler':'truss-runtime','request':current,'pageSize':999}))
 values.append({'ordinary':json.loads(weft.compile_truss_runtime_json(json.dumps(current))),'page':json.loads(page),'export':json.loads(weft.compile_engine_export_json(page,json.dumps(current)))})
path=pathlib.Path('/private/tmp/sdk-export-pre-metadata-programs.json')
if sys.argv[1]=='before':path.write_text(json.dumps(values,sort_keys=True,separators=(',',':')))
else:
 previous=json.loads(path.read_bytes())
 for old,new in zip(previous,values,strict=True):
  assert old['ordinary']==new['ordinary'] and old['page']==new['page']
  expected=json.loads(json.dumps(old['export']))
  expected['artifact']['targetContext']['sessionSettings']['read_context']='fixed-insensitive-cursor'
  digest=expected.pop('structuralDigest')
  digest['sha256']=hashlib.sha256(json.dumps(expected,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()
  expected['structuralDigest']=digest
  assert expected==new['export']

 pathlib.Path('/private/tmp/sdk-export-metadata-correspondence.json').write_text(json.dumps({'status':'pass','ordinaryAndPageProgramsEqual':True,'exportChangeOnly':'explicit cursor targetContext read_context and covering owner structural digest; SQL/parameters/IR/columns/obligations unchanged','cases':9,'scope':'three actual native requests each ordinary/page/export; bound values unchanged by descriptive manifest correction','previousProgramsSha256':hashlib.sha256(path.read_bytes()).hexdigest()},indent=2)+'\n')
print(sys.argv[1],'complete programs',len(values)*3)
