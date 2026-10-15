import json,collections,copy
from pathlib import Path

def verify(r):
 cases=r['cases'];assert len(cases)==6
 by={c['id']:c for c in cases};assert len(by)==6
 ids=[]
 def rows(name):
  p=by[name]['response'];assert p['status']['state']=='SUCCEEDED'
  assert p['manifest']['total_row_count']==1
  assert p['result']['row_count']==1
  ids.append(p['statement_id']);return p['result']['data_array']
 assert rows('exact-signed64-sum')==[['decimal(38,0)','18446744073709551614']]
 assert rows('empty-aggregate-carrier')==[[None,'[]','0']]
 c=rows('complete-duplicate-bag');assert c[0][0]=='1'
 tuples=json.loads(c[0][1]);assert len(tuples)==3
 assert collections.Counter(json.dumps(t,sort_keys=True) for t in tuples)==collections.Counter({json.dumps([{'scan':'r','key':'one'}],sort_keys=True):1,json.dumps([{'scan':'r','key':'two'}],sort_keys=True):2})
 assert rows('binary-collation')==[['false','false','false']]
 assert rows('exact-bound-parameter')==[['9007199254740993']]
 carrier=rows('qualified-builtins');assert json.loads(carrier[0][0])==[{'scan':'r','key':'one'}]
 assert len(set(ids))==6
 return True
if __name__=='__main__':
 import sys
 r=json.loads(Path(sys.argv[1]).read_text());verify(r)
 rejected=[]
 for label,case,column,value in [('rounded-sum',0,1,'18446744073709552000'),('dropped-duplicate',2,1,'[[{"scan":"r","key":"one"}],[{"scan":"r","key":"two"}]]'),('wrong-empty-nullity',1,0,'0'),('case-folded',3,0,'true'),('rounded-parameter',4,0,'9007199254740992'),('wrong-private-key',5,0,'[{"scan":"r","key":"other"}]')]:
  mutant=copy.deepcopy(r);mutant['cases'][case]['response']['result']['data_array'][0][column]=value
  try:verify(mutant)
  except AssertionError:rejected.append(label)
  else:raise AssertionError('Undetected corruption: '+label)
 print(json.dumps({'observedNativeCases':6,'rejectedReceiptCorruptions':rejected,'scope':'Finite value/shape oracle over actual constant-only managed SQL responses. Not backend/caller authority proof.'},indent=2))
