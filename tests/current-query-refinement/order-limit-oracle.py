# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent finite SQL ordering/limit oracle for CONTRACT-010 design.
Enumerated permitted full row orders versus the proposed key-class predicate.
No Rust implementation, native result, current cut or authority is verified.
@covers US-009-AC3 @covers US-009-AC4
"""
from collections import Counter
from itertools import permutations,product
import argparse,hashlib,json,pathlib
p=argparse.ArgumentParser();p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()

def permitted_orders(rows,keys,descending):
 for order in permutations(rows):
  if descending is None or all((keys[x]>=keys[y]if descending else keys[x]<=keys[y])for x,y in zip(order,order[1:])):
   yield order

def key_class_predicate(rows,keys,observed,limit,descending):
 size=len(rows)if limit is None else min(limit,len(rows))
 if len(observed)!=size or Counter(observed)-Counter(rows):return False
 if descending is None or size==0:return True
 if any((keys[x]<keys[y]if descending else keys[x]>keys[y])for x,y in zip(observed,observed[1:])):return False
 boundary=sorted((keys[r]for r in rows),reverse=descending)[size-1]
 before=Counter(r for r in rows if (keys[r]>boundary if descending else keys[r]<boundary))
 if before-Counter(observed):return False
 return not any((keys[r]<boundary if descending else keys[r]>boundary)for r in observed)

count=0;worlds=0;witnesses={}
for n in range(4):
 rows=tuple('k'+str(i)for i in range(n));foreign='foreign'
 for values in product((0,1),repeat=n):
  worlds+=1;keys=dict(zip(rows,values))
  for descending in (None,False,True):
   orders=list(permitted_orders(rows,keys,descending));assert orders
   for limit in (None,*range(n+2)):
    size=n if limit is None else min(limit,n);allowed={o[:size]for o in orders}
    for width in range(n+2):
     for observed in product((*rows,foreign),repeat=width):
      oracle=observed in allowed;actual=key_class_predicate(rows,keys,observed,limit,descending)
      assert actual==oracle,(rows,keys,observed,limit,descending,oracle,actual);count+=1
      detail={'rows':rows,'sortKeys':values,'observed':observed,'limit':limit,'descending':descending,'allowed':oracle}
      if len(observed)==size and not oracle:witnesses.setdefault('cardinality-alone',detail)
      if descending is not None:
       fixed=tuple(sorted(rows,key=lambda r:(-keys[r]if descending else keys[r],r)))[:size]
       if oracle and observed!=fixed:witnesses.setdefault('invented-total-tie-order',detail)
       weak=(len(observed)==size and not(Counter(observed)-Counter(rows))and all((keys[x]>=keys[y]if descending else keys[x]<=keys[y])for x,y in zip(observed,observed[1:])))
       if weak and not oracle:witnesses.setdefault('drop-required-earlier-key',detail)
assert set(witnesses)=={'cardinality-alone','invented-total-tie-order','drop-required-earlier-key'}
source=pathlib.Path(__file__).read_bytes();result={'scope':'Finite design oracle only; no checker implementation/native authority proof','worlds':worlds,'comparisons':count,'matched':True,'mutantCounterexamples':witnesses,'sourceSha256':hashlib.sha256(source).hexdigest(),'nativeSecurityAcceptance':False,'casesPromoted':[]}
a.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'comparisons':count,'worlds':worlds,'mutationWitnesses':len(witnesses)}))
