import copy,hashlib,json,pathlib,subprocess
R=pathlib.Path('/private/tmp/ashlar-weft-distribution-d2d')
O=pathlib.Path('/private/tmp/astra-weft-paths-keys-artifacts-20261010-b');O.mkdir(exist_ok=False)
B=R/'target/debug/weft-paths-keys'
def h(b):return hashlib.sha256(b).hexdigest()
def encoded(x):return json.dumps(x,ensure_ascii=False,separators=(',',':')).encode()
base=json.loads((R/'tests/ashlar-databricks/fixtures/original-commerce-path-request.json').read_bytes())
base['target'].update(backendId='ashlar.databricks.paths-keys',backendVersion='0.4.0-paths-keys-candidate',targetProfile='spark4-delta4-paths-keys-candidate')
def req(sql):
 r=copy.deepcopy(base);r['sql']=sql;return r
def variant(r,fn):
 d=json.loads(r['modules'][0]['documentJson']);fn(d);raw=encoded(d).decode();sha=h(raw.encode());r['modules'][0]['documentJson']=raw;r['modules'][0]['pin']['sha256']=sha
 b=json.loads(r['target']['bindingJson']);b['modelPins'][0]['sha256']=sha
 for p in b['relationships']:
  m=next(m for m in d['modules'] if m['id']==p['logical']['module']);p['acceptedDefinition']=next(x for x in m['relationships'] if x['id']==p['logical']['relationship'])
 raw=encoded(b).decode();r['target']['bindingJson']=raw;r['target']['bindingSha256']=h(raw.encode())
def each_element(d,name,fn):
 for m in d['modules']:
  for e in m['elements']:
   if e['id']==name:fn(e)
def relation(d):return next(x for m in d['modules'] for x in m['relationships'] if x['id']=='products.supplier_id')
def check(a,r):
 assert a['status']=='compiled',a
 assert a['backend']=={'backendId':'ashlar.databricks.paths-keys','backendVersion':'0.4.0-paths-keys-candidate','targetProfile':'spark4-delta4-paths-keys-candidate','interfaceVersion':'weft-backend/0.3.0'}
 assert 'ROW_NUMBER' not in a['sql']
 obs=a['obligations'];by={x['id']:x for x in obs};assert len(by)==len(obs)
 ir=a['logicalPlan'];outputs=ir['outputs'];keys=[];ordinal=[];binding=json.loads(r['target']['bindingJson'])
 for pos,o in enumerate(outputs,1):
  e=o['expression'];op=e['op']
  if op=='relatedKeys':
   keys.append((pos,e));ordinal.append({'outputPosition':pos,'kind':'relatedKeys'})
   col=a['columns'][pos-1];rep=col['representation'];assert rep=={'kind':'relatedKeys','relationship':e['relationship']['identity'],'key':e['relationship']['targetKey'],'bound':e['bound']}
   assert col['nullable'] is False
   assert set(json.dumps(x,sort_keys=True) for x in col['sourceIdentities'])==set(json.dumps(x,sort_keys=True) for x in [e['relationship']['from'],e['relationship']['to']]+e['relationship']['sourceKey']['fields']+e['relationship']['targetKey']['fields'])
  elif op=='relatedPaths':ordinal.append({'outputPosition':pos,'kind':'relatedPaths'})
 if keys:
  o=by['ashlar.relatedKeys.collectionIntegrity'];assert o['owner']=='host' and o['failureCode']=='WFT-BINDING'
  p=o['parameters'];assert set(p)=={'phase','samePublicationRequired','noPartialPublication','collections','edgeSchemas','checks','success'}
  assert p['phase']=='before-user-query' and p['samePublicationRequired'] is True and p['noPartialPublication'] is True
  assert p['collections']==[{'outputPosition':i,'startScan':e['scan'],'relationship':e['relationship'],'bound':e['bound']} for i,e in keys]
  assert len(p['edgeSchemas'])==len(p['checks'])==len(keys)
  for (i,e),s,g in zip(keys,p['edgeSchemas'],p['checks']):
   phys=next(x for x in binding['relationships'] if x['logical']==e['relationship']['identity'])
   table=binding['publication']['tables'][phys['table']]
   assert s=={'outputPosition':i,'relationship':e['relationship']['identity'],'table':table,'identityColumn':'id','nativeType':'BIGINT'}
   assert set(g)=={'outputPosition','kind','sql','failureCode'} and g['outputPosition']==i and g['kind']=='collectionEncoding' and g['failureCode']=='WFT-BINDING'
   sql=g['sql'];assert 'WHERE id IS NULL' in sql and 'GROUP BY id HAVING' in sql
   assert "from_json(to_json(__item),'ARRAY<STRING>') <=> __item" in sql
   assert 'size(c.__items)<>LEAST(totals.total' in sql and 'c.__truncated<>(totals.total>' in sql
   assert 'WHERE __ordinal<=' in sql and 'TRY_SUM(CAST(1 AS DECIMAL(38,0))) OVER' in sql
 else:assert 'ashlar.relatedKeys.collectionIntegrity' not in by
 if ordinal:
  o=by['ashlar.relatedKeys.ordinalCapacity'];assert o['owner']=='host' and o['failureCode']=='WFT-CAPABILITY'
  p=o['parameters'];assert set(p)=={'phase','samePublicationRequired','noPartialPublication','nativeRepresentation','maximum','collections','checks','success'}
  assert p['collections']==ordinal and p['nativeRepresentation']=='decimal38' and p['maximum']=='9'*38
  assert p['phase']=='before-user-query' and p['samePublicationRequired'] is True and p['noPartialPublication'] is True
  assert len(p['checks'])==len(ordinal)
  for e,g in zip(ordinal,p['checks']):
   assert set(g)=={'outputPosition','kind','sql','failureCode'} and g['outputPosition']==e['outputPosition'] and g['kind']=='fullOccurrencePrefix' and g['failureCode']=='WFT-CAPABILITY'
   assert 'WHERE __ordinal IS NULL' in g['sql'] and 'TRY_SUM(CAST(1 AS DECIMAL(38,0))) OVER' in g['sql']
 else:assert 'ashlar.relatedKeys.ordinalCapacity' not in by
 return {'oneHopCollections':len(keys),'wideCollections':len(ordinal),'guards':{x['id']:len(x['parameters'].get('checks',[])) for x in obs}}
cases=[]
def add(name,sql,ok=True,mut=None):
 r=req(sql)
 if mut:mut(r)
 cases.append((name,r,ok))
sql='SELECT p.id, RELATED_KEYS(p."products.supplier_id", 2) AS suppliers FROM products p'
add('original',sql)
add('empty_outer',sql+" WHERE p.id = 'missing-owner' ORDER BY p.id LIMIT 1")
add('repeated_expressions','SELECT RELATED_KEYS(p."products.supplier_id", 1) AS x, RELATED_KEYS(p."products.supplier_id", 1000) AS y FROM products p')
add('mixed_reverse_order','SELECT RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 2) AS a, l.id, RELATED_KEYS(l."order_lines.product_id", 1) AS b, RELATED_PATHS(l."order_lines.product_id", "products.supplier_id", 1) AS c FROM order_lines l')
add('two_roots','SELECT RELATED_KEYS(p."products.supplier_id", 2) AS a, RELATED_KEYS(q."products.supplier_id", 1) AS b FROM products p JOIN products q ON p.id = q.id')
add('required_root_with_left_other','SELECT RELATED_KEYS(l."order_lines.product_id", 2) AS a, p.id FROM order_lines l LEFT JOIN products p ON l.product_id = p.id')
add('arithmetic_and_collection','SELECT l.quantity + 1 AS quantity, RELATED_KEYS(l."order_lines.product_id", 2) AS products FROM order_lines l')
add('inverse','SELECT RELATED_KEYS(s.products, 2) AS products FROM suppliers s',mut=lambda r:variant(r,lambda d:relation(d).update(inverse='products')))
add('composite',sql,mut=lambda r:variant(r,lambda d:each_element(d,'suppliers',lambda e:e['keys'][0]['fields'].append({'module':'domain','element':'suppliers.name'}))))
add('scalar_only','SELECT p.id FROM products p')
for name,q in [('bound0',sql.replace(', 2)',', 0)')),('bound1001',sql.replace(', 2)',', 1001)')),('left_root','SELECT RELATED_KEYS(p."products.supplier_id", 2) AS suppliers FROM order_lines l LEFT JOIN products p ON l.product_id = p.id'),('aggregate','SELECT COUNT(*), RELATED_KEYS(p."products.supplier_id", 2) AS suppliers FROM products p'),('group',sql+' GROUP BY p.id'),('expansion_mix','SELECT RELATED_KEYS(l."order_lines.product_id", 2) AS products FROM order_lines l CROSS JOIN EXPAND_PATHS(l."order_lines.product_id", "products.supplier_id") AS p')]:add(name,q,False)
add('optional_key',sql,False,lambda r:variant(r,lambda d:each_element(d,'suppliers.id',lambda e:e.update(nullability='absent-allowed'))))
add('numeric_key',sql,False,lambda r:variant(r,lambda d:each_element(d,'suppliers.id',lambda e:e.update(scalarType='integer'))))
add('candidate_off',sql,False,lambda r:r['options'].update(allowCandidate=False))
add('wrong_profile',sql,False,lambda r:r['target'].update(targetProfile='spark4-delta4-paths-candidate'))
rows=[]
for name,r,ok in cases:
 raw=encoded(r);(O/(name+'.request.json')).write_bytes(raw)
 p=subprocess.run([str(B)],input=raw,stdout=subprocess.PIPE,stderr=subprocess.PIPE,env={'PATH':'/usr/bin:/bin'},cwd='/private/tmp',timeout=10)
 assert p.returncode==0 and p.stderr==b'' and len(p.stdout)<4*1024*1024,(name,p.returncode,p.stderr)
 (O/(name+'.response.json')).write_bytes(p.stdout);a=json.loads(p.stdout)
 if ok:
  detail=check(a,r)
  if name=='empty_outer':
   for o in a['obligations']:
    if o['id'].startswith('ashlar.relatedKeys.'):
     assert all('LIMIT 1' not in g['sql'] and 'missing-owner' not in g['sql'] for g in o['parameters']['checks'])
  if name=='inverse':assert 'e.target_id AS __root' in a['sql']
  if name=='composite':assert 'p.`k0` COLLATE UTF8_BINARY ASC, p.`k1` COLLATE UTF8_BINARY ASC, p.__edge ASC' in a['sql']
 else:
  assert a['status']=='blocked' and 'sql' not in a,(name,a);detail={'diagnosticCodes':[d['code'] for d in a['diagnostics']]}
 rows.append({'name':name,'compiled':ok,'requestSha256':h(raw),'responseSha256':h(p.stdout),'responseBytes':len(p.stdout),'detail':detail})
print(json.dumps({'status':'passed','cases':len(rows),'compiled':sum(x['compiled'] for x in rows),'blocked':sum(not x['compiled'] for x in rows)},sort_keys=True))
(O/'results.json').write_text(json.dumps({'binary':str(B),'binarySha256':h(B.read_bytes()),'scope':'Actual bounded debug compiler artifacts only, no SQL execution/native qualification','cases':rows},indent=2)+'\n')
