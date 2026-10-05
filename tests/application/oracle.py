"""Independent bag/Decimal interpreter plus authored result expectations.

No parser/model resolver/emitter code is reused. The plan is the subject under
validation. Fixture edge duplicates deliberately remain observable.
"""
import json
from decimal import Decimal,getcontext
from pathlib import Path
getcontext().prec=100
ROOT=Path(__file__).resolve().parents[2]
data=json.loads((ROOT/'tests/application/fixtures/data.json').read_text())
reports=json.loads((ROOT/'target/b002a/reports.json').read_text())
def exact(value,t):
    if t['family']=='integer':return int(value)
    if t['family']=='decimal':return Decimal(value)
    if t['family']=='boolean':return value=='true'
    return value
def field(f,row):return exact(row[f['scan']][f['identity']['element']],f['type'])
def value(v,row):return field(v['field'],row) if v['kind']=='field' else exact(v['value'],v['type'])
def related(rel,scan,row):
    source=row[scan]; key=source['order-id' if rel['inverse'] else 'customer-id']
    ids=[a if rel['inverse'] else b for a,b in data['edges'] if (b if rel['inverse'] else a)==key]
    target=data[rel['to']['element']];id_name='customer-id' if rel['inverse'] else 'order-id'
    tuples=[[r[f['element']] for f in rel['targetKey']['fields']] for id in ids for r in target if r[id_name]==id]
    return sorted(tuples,key=lambda tup:tuple(exact(v,t) for v,t in zip(tup,rel['targetKey']['types'])))
def predicate(p,row):
    if p['op']=='equal':return field(p['left'],row)==value(p['right'],row)
    if p['op']=='lexicographicGreater':return tuple(field(f,row) for f in p['columns'])>tuple(value(v,row) for v in p['values'])
    desired=tuple(value(v,row) for v in p['key'])
    return any(tuple(exact(v,t) for v,t in zip(k,p['relationship']['targetKey']['types']))==desired for k in related(p['relationship'],p['scan'],row))
def execute(plan):
    scan=plan['source'];rows=[{scan['occurrence']:r} for r in data[scan['record']['element']]]
    for join in plan['joins']:
        s=join['right'];rows=[merged for left in rows for right in data[s['record']['element']] if all(predicate(p,merged:={**left,s['occurrence']:right}) for p in join['on'])]
    rows=[r for r in rows if all(predicate(p,r) for p in plan['filters'])]
    if plan['aggregate']:
        groups={}
        for row in rows:groups.setdefault(tuple(field(f,row) for f in plan['groups']),[]).append(row)
        if not plan['groups']:groups={():rows}
        buckets=list(groups.values())
    else:buckets=[[r] for r in rows]
    if plan['order']:buckets.sort(key=lambda b:tuple(field(f,b[0]) for f in plan['order']))
    if plan['limit'] is not None:buckets=buckets[:plan['limit']]
    result=[]
    for bucket in buckets:
        output=[]
        for o in plan['outputs']:
            e=o['expression']
            if e['op']=='field':v=bucket[0][e['scan']][e['identity']['element']]
            elif e['op']=='count':v=str(len(bucket))
            elif e['op']=='sum':
                v=None if not bucket else sum((field(e['argument'],r) for r in bucket),Decimal(0) if e['type']['family']=='decimal' else 0)
                if v is not None:v=format(v,f".{e['type']['facets']['scale']}f") if e['type']['family']=='decimal' else str(v)
            else:
                items=related(e['relationship'],e['scan'],bucket[0]);v={'items':items[:e['bound']],'truncated':len(items)>e['bound']}
            output.append(v)
        result.append(output)
    return result
alice=data['customer'][0];bob=data['customer'][1];isolated=data['customer'][2]
entity=lambda r:[r[f] for f in ['customer-id','customer-name','customer-active','nickname','tags','address']]
expected={
 'whole-entity':[entity(alice),entity(bob),entity(isolated)],
 'scalar-page':[[r['customer-id'],r['nickname'],r['tags'],r['address']] for r in data['customer']],
 'single-cursor':[], 'injection-text':[], 'global-count':[['3']],
 'grouped-count':[['Alice','1'],['Bob','1'],['isolated','1']],
 'join-count':[['3']], 'global-sum':[['9007199254740994.22']],
 'related-page':[['1',{'items':[['1'],['2']],'truncated':True}],['2',{'items':[['3']],'truncated':False}],['3',{'items':[],'truncated':False}]],
 'related-filter':[entity(alice)],
 'inverse-page':[['1',{'items':[['1']],'truncated':False}],['2',{'items':[['1']],'truncated':True}],['3',{'items':[['2']],'truncated':False}]],
 'unicode-key-order':[['','6'],['é','2'],['trail','4'],['trail ','3'],['é','1'],['😀','5']],
 'unicode-key-cursor':[['trail ','3'],['é','1'],['😀','5']],
 'exact-large-key-order':[['9007199254740992'],['9007199254740993'],['18446744073709551615']],
 'case-folded-param':[['1']], 'composite-cursor':[['2','Bob'],['3','isolated']],
}
count=0
for r in reports:
    if r['id'] in expected:
        original_customer=data['customer']
        if r['id'].startswith('unicode-key-'):
            data['customer']=[{'customer-name':n,'customer-id':str(i)} for i,n in enumerate(['é','é','trail ','trail','😀',''],1)]
        elif r['id']=='exact-large-key-order':
            data['customer']=[{'customer-id':n} for n in ['9007199254740993','18446744073709551615','9007199254740992']]
        actual=execute(r['response']['logicalPlan']);data['customer']=original_customer;assert actual==expected[r['id']],(r['id'],actual,expected[r['id']]);count+=1
# Empty global aggregates still return a row; grouped counts return no rows.
original=data['customer'];data['customer']=[]
for id,want in [('global-count',[['0']]),('grouped-count',[])]:
    p=next(r['response']['logicalPlan'] for r in reports if r['id']==id);assert execute(p)==want;count+=1
data['customer']=original
summary={'authoredResults':count,'exactDecimals':True,'bagCounts':True,'duplicateEdges':True,'inverseTraversal':True,'absenceAndEmptyList':True,'nativeExecution':False}
(ROOT/'target/b002a/oracle-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print('Independent application read results:',json.dumps(summary))
