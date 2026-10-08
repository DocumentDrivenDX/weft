"""@covers US-006-AC1: independently authored relational bag expectations.
Frontend only. No SQL-engine execution or public embedding claim.
"""
import copy, hashlib, importlib.util, json, os, pathlib, random, subprocess
from collections import Counter
from decimal import Decimal, localcontext
ROOT=pathlib.Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('oracle',ROOT/'tests/frontend/oracle.py')
oracle=importlib.util.module_from_spec(spec);spec.loader.exec_module(oracle)
SEED=95955044271901
rng=random.Random(SEED)
base=oracle.corpus[0]['request']
reports=[];digests=set();mutations=Counter()
def cents(n):return f'{n//100}.{n%100:02d}'
def check(sql,rows,expected,case):
 request=copy.deepcopy(base);request['sql']=sql
 raw=subprocess.run([str(oracle.BINARY)],input=json.dumps(request,ensure_ascii=False),capture_output=True,text=True,check=True).stdout
 response=json.loads(raw)
 assert response['status']=='resolved',(case,response)
 oracle.seed=rows
 actual=Counter(oracle.node(response['logicalPlan']['root']))
 expected=Counter(expected)
 assert actual==expected,(case,actual,expected)
 assert response['retainedModules']==request['modules']
 digest=hashlib.sha256(json.dumps({'request':request,'rows':rows},sort_keys=True,ensure_ascii=False).encode()).hexdigest()
 assert digest not in digests;digests.add(digest)
 # Independent output-corruption controls, not compiler-source mutation coverage.
 if any(n>1 for n in expected.values()):
  assert Counter({row:1 for row in actual})!=expected;mutations['duplicate-collapse']+=1
 rounded=Counter()
 for row,n in actual.items():
  rounded[tuple(Decimal(float(v)) if isinstance(v,Decimal) else v for v in row)]+=n
 if rounded!=actual:
  assert rounded!=expected;mutations['decimal-through-double']+=1
 reports.append({'id':case,'inputSha256':digest,'rows':sum(actual.values()),'distinctRows':len(actual)})
with localcontext() as ctx:
 ctx.prec=100
 for i in range(300):
  names=['é','e\u0301','trail ','trail','😀','same',f'unique-{i}']
  customers=[{'id':str(i*100+j+1),'name':rng.choice(names),'active':rng.choice(['true','false'])} for j in range(8)]
  # Guarantee projected duplicates without violating unique entity keys.
  customers[1]['name']=customers[0]['name']
  orders=[{'customer_id':rng.choice(customers)['id'],'total':cents(rng.randrange(1,10**20))} for _ in range(18)]
  orders.append(copy.deepcopy(orders[0]))
  rows={'Customer':customers,'Orders':orders}
  check('SELECT c.name FROM Customer c',rows,[(c['name'],) for c in customers],f'projection-{i}')
  check('SELECT c.id FROM Customer c WHERE c.active = true',rows,[(int(c['id']),) for c in customers if c['active']=='true'],f'filter-{i}')
  joined=[(c['name'],Decimal(o['total'])) for c in customers for o in orders if c['id']==o['customer_id']]
  check('SELECT c.name, o.total FROM Customer c JOIN Orders o ON o.customer_id = c.id',rows,joined,f'join-{i}')
  groups={}
  for name,total in joined:groups[name]=groups.get(name,Decimal(0))+total
  check('SELECT c.name, SUM(o.total) AS total FROM Customer c JOIN Orders o ON o.customer_id = c.id GROUP BY c.name',rows,list(groups.items()),f'group-sum-{i}')
assert len(reports)==1200 and len(digests)==1200
assert mutations['duplicate-collapse']>=600 and mutations['decimal-through-double']>=600
assert hashlib.sha256(oracle.BINARY.read_bytes()).hexdigest()==oracle.BINARY_SHA
out=pathlib.Path(os.environ.get('WEFT_RELATIONAL_OUTPUT',str(ROOT/'target/b007-relational')));out.mkdir(parents=True,exist_ok=True)
summary={'status':'passed','layer':'frontend','generator':'weft-relational/0.1.0','seed':SEED,'rng':'Python random.Random','python':__import__('platform').python_version(),'distinctAssertions':len(reports),'datasets':300,'mutationControls':dict(mutations),'mutationScope':'output corruption controls, not compiler-source mutation coverage','frontendBinarySha256':oracle.BINARY_SHA,'harnessSha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'oracleSha256':hashlib.sha256((ROOT/'tests/frontend/oracle.py').read_bytes()).hexdigest(),'targetExecution':False}
(out/'cases.json').write_text(json.dumps(reports,indent=2)+'\n')
(out/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary))
