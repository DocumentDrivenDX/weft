"""Independent small bag/Decimal interpreter. Does not emit SQL or call Rust semantics."""
from collections import Counter
from decimal import Decimal, localcontext
import copy,json,pathlib,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[2]
OUT=ROOT/'target/b002';OUT.mkdir(parents=True,exist_ok=True)
corpus=json.loads((ROOT/'docs/helix/03-test/fixtures/cases.json').read_text())
seed=json.loads((ROOT/'docs/helix/03-test/fixtures/sales.rows.json').read_text())
field_to_name={'customer-id':'id','customer-name':'name','customer-active':'active','order-customer':'customer_id','order-total':'total'}
def exact(value,family):
 if value is None:return None
 if family=='integer':return int(value)
 if family=='decimal':return Decimal(value)
 if family=='boolean':return value if isinstance(value,bool) else value=='true'
 return value

def expr(e,row):
 op=e['op']
 if op=='field':return exact(row[e['scan']][field_to_name[e['identity']['element']]],e['type']['family'])
 if op=='literal':return exact(e['value'] if e['type']['family']!='boolean' else e['value']=='true',e['type']['family'])
 if op=='equal':return expr(e['left'],row)==expr(e['right'],row)
 if op=='and':return expr(e['left'],row) and expr(e['right'],row)
 if op=='sum':
  rows=row['__group__']
  if not rows:return None
  return sum((expr(e['argument'],r) for r in rows),Decimal(0) if e['type']['family']=='decimal' else 0)
 raise AssertionError(op)

def node(n):
 op=n['op']
 if op=='scan':
  key={'customer':'Customer','orders':'Orders'}[n['record']['element']]
  return [{n['occurrence']:r} for r in seed[key]]
 if op=='innerJoin':
  return [r for l in node(n['left']) for right in node(n['right']) if expr(n['on'],r:={**l,**right})]
 if op=='filter':return [r for r in node(n['input']) if expr(n['predicate'],r)]
 if op=='aggregate':
  rows=node(n['input']);groups={}
  if not n['groups']:groups[()]=rows
  else:
   for r in rows:groups.setdefault(tuple(expr(g,r) for g in n['groups']),[]).append(r)
  return [{**(rows[0] if rows else {}),'__group__':rows} for rows in groups.values()]
 if op=='project':return [tuple(expr(o['expression'],r) for o in n['outputs']) for r in node(n['input'])]
 raise AssertionError(op)
reports=[];compared=0
with localcontext() as context:
 context.prec=100
 for case in corpus:
  raw=subprocess.run([str(ROOT/'target/debug/frontend')],input=json.dumps(case['request'],ensure_ascii=False),capture_output=True,text=True,check=True).stdout.strip()
  report=json.loads(raw)
  expected=case['expected'];want='resolved' if expected['status']=='compiled' else 'blocked'
  assert report['status']==want,(case['id'],report,expected)
  if want=='blocked':assert report['diagnostics'][0]['code']==expected['code'],(case['id'],report)
  else:
   assert report['retainedModules']==case['request']['modules']
   assert report['logicalPlan']['requiredCapabilities']==sorted(set(report['logicalPlan']['requiredCapabilities']))
   if 'rows' in expected:
    expected_rows=[tuple(exact(v.get('value'),v['kind']) for v in row) for row in expected['rows']]
    assert Counter(node(report['logicalPlan']['root']))==Counter(expected_rows),(case['id'],node(report['logicalPlan']['root']),expected_rows)
    compared+=1
  reports.append({'id':case['id'],'response':report,'raw':raw})
(OUT/'reports.json').write_text(json.dumps(reports,ensure_ascii=False))
print(json.dumps({'corpusCases':len(reports),'independentRelationalBags':compared,'arithmetic':'Python int and Decimal precision 100','targetExecution':False}))
