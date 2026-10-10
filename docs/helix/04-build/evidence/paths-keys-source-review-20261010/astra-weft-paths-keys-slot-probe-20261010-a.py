import pathlib,json,re
p=pathlib.Path('/private/tmp/astra-weft-paths-keys-artifacts-20261010-b');r=json.loads((p/'results.json').read_bytes());total=0
for case in r['cases']:
 if not case['compiled']:continue
 a=json.loads((p/(case['name']+'.response.json')).read_bytes());n=len(a['parameters']);assert [x['position'] for x in a['parameters']]==list(range(1,n+1))
 def sqls(v):
  if isinstance(v,dict):
   for k,x in v.items():
    if k=='sql':yield x
    else:yield from sqls(x)
  elif isinstance(v,list):
   for x in v:yield from sqls(x)
 for sql in sqls(a):
  assert all(1<=int(i)<=n for i in re.findall(r':p(\d+)\b',sql));total+=1
print(json.dumps({'compiledArtifacts':10,'sqlStatements':total,'orderedSlotsAndEveryPlaceholderInRange':True}))
