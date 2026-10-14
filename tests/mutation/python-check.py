"""Native Rust extension proof; independent owner golden and exact predicate oracle."""
import copy, hashlib, json, pathlib, sys
import weft
root=pathlib.Path(__file__).resolve().parents[2]
fixtures=root/'tests/mutation/fixtures'
config=json.loads((fixtures/'registry.json').read_text())
request=json.loads((fixtures/'request.json').read_text())
compiler=weft.MutationCompiler(json.dumps(config))
cases=[]
def check(name, changes, expected):
    value=copy.deepcopy(request);value.update(changes)
    text=json.dumps(value,ensure_ascii=False)
    response=compiler.compile_json(text);decoded=json.loads(response)
    assert decoded['status']==expected,(name,decoded)
    assert ('plan' in decoded)==(expected=='compiled'),name
    if expected=='compiled':
        plan=decoded['plan'];assert hashlib.sha256(plan['utf8'].encode()).hexdigest()==plan['sha256']
        selection=json.loads(plan['utf8'])['selection']['artifact'];assert hashlib.sha256(selection['utf8'].encode()).hexdigest()==selection['sha256']
    cases.append(dict(id=name,request=text,response=response))
    return decoded
base=check('owner-golden',{},'compiled')
assert base==json.loads((fixtures/'golden.json').read_text())
check('case-sensitive-marker',dict(sql='uPdAtE CUSTOMER SeT ACTIVE=fAlSe WhErE LAST_SEEN<:Cutoff',parameters={'Cutoff':{'integerToken':'100'}}),'compiled')
check('wrong-marker-case',dict(sql='UPDATE Customer SET active=FALSE WHERE last_seen<:Cutoff'),'blocked')
for depth in [128,129]:check('grouping-'+str(depth),dict(sql='UPDATE Customer SET active=FALSE WHERE '+'('*depth+'last_seen<:cutoff'+')'*depth),'compiled' if depth==128 else 'blocked')
atom='('*32+'last_seen<:cutoff'+')'*32
check('many-grouped-operands',dict(sql='UPDATE Customer SET active=FALSE WHERE '+' AND '.join([atom]*64)),'compiled')
for count in [64,65]:check('operand-'+str(count),dict(sql='UPDATE Customer SET active=FALSE WHERE '+' AND '.join(['last_seen<:cutoff']*count)),'compiled' if count==64 else 'blocked')
for depth in [24,25,128]:
    predicate='last_seen<:cutoff'
    for level in range(depth-1):predicate='last_seen<:cutoff '+('AND' if level%2==0 else 'OR')+' ('+predicate+')'
    check('decoded-depth-'+str(depth),dict(sql='UPDATE Customer SET active=FALSE WHERE '+predicate),'compiled' if depth==24 else 'blocked')
for count in [256,257]:
    groups=[' AND '.join(['last_seen<:cutoff']*(5 if count==257 and i==0 else 4)) for i in range(64)]
    check('comparison-'+str(count),dict(sql='UPDATE Customer SET active=FALSE WHERE '+' OR '.join('('+g+')' for g in groups)),'compiled' if count==256 else 'blocked')
for length in [65536,65537]:
    sql=request['sql'];check('sql-bytes-'+str(length),dict(sql=sql+' '*(length-len(sql))),'compiled' if length==65536 else 'blocked')
check('unicode-string',dict(sql="UPDATE Customer SET active=FALSE WHERE id='Foo😀''é'",parameters={}), 'compiled')
for key in ['interfaceVersion','dialect']:
    check('unsupported-'+key,{key:'unknown/99'},'blocked')
for token in ['-0','1e2','9007199254740993']:
    check('exact-'+token,dict(parameters={'cutoff':{'integerToken':token}}),'compiled')
for name,sql in [('no-where','UPDATE Customer SET active=FALSE'),('null','UPDATE Customer SET active=NULL WHERE active=TRUE'),('key-write',"UPDATE Customer SET id='other' WHERE active=TRUE"),('bool-order','UPDATE Customer SET active=FALSE WHERE active<TRUE'),('comment','UPDATE Customer SET active=FALSE WHERE last_seen<:cutoff -- nope'),('extra','UPDATE Customer SET active=FALSE WHERE last_seen<:cutoff;SELECT x')]:check(name,dict(sql=sql),'blocked')
for key in ['credentials','preparation','registry']:check('unexpected-'+key,{key:{}},'blocked')
check('unused-marker',dict(parameters={'cutoff':{'integerToken':'100'},'unused':{'boolean':True}}),'blocked')
# Compare typed AST evaluation with independently authored expected row identities.
r=check('predicate-oracle',dict(sql='UPDATE Customer SET active=FALSE WHERE last_seen<:cutoff OR active=TRUE AND last_seen=100'),'compiled')
ast=json.loads(json.loads(r['plan']['utf8'])['selection']['artifact']['utf8'])['predicate']
def evaluate(p,row):
    if 'and' in p:return all(evaluate(x,row) for x in p['and'])
    if 'or' in p:return any(evaluate(x,row) for x in p['or'])
    c=p['compare'];left=row[c['field']['element']];v=c['value'];right=v.get('boolean') if 'boolean' in v else int(v['integerToken'])
    return {'eq':lambda:left==right,'ne':lambda:left!=right,'lt':lambda:left<right,'le':lambda:left<=right,'gt':lambda:left>right,'ge':lambda:left>=right}[c['op']]()
rows=[dict(id='below',**{'last-seen':99,'active':False}),dict(id='edge-active',**{'last-seen':100,'active':True}),dict(id='edge-inactive',**{'last-seen':100,'active':False}),dict(id='above',**{'last-seen':101,'active':True})]
assert [row['id'] for row in rows if evaluate(ast,row)]==['below','edge-active']
out=root/'target/mutation';out.mkdir(parents=True,exist_ok=True)
(out/'parity-cases.json').write_text(json.dumps(cases,ensure_ascii=False))
report=dict(scope='native Python wrapper of Rust compiler; no executor',cases=len(cases),passed=True,ownerGolden=True,predicateOracle=True,nativeModule=weft.__file__)
(out/'python-results.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
