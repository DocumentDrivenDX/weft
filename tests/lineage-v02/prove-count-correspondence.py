"""Bounded conditional count correspondence; no Rust/SQL/native refinement theorem."""
import importlib.metadata,json
import z3
assert importlib.metadata.version('z3-solver')=='4.15.4.0'
checks=[]
def cardinality(values,nulls,present):
 return z3.Sum([z3.If(z3.And(present[i],z3.Not(nulls[i]),*[z3.Or(z3.Not(present[j]),nulls[j],values[j]!=values[i]) for j in range(i)]),1,0) for i in range(len(values))]) if values else z3.IntVal(0)
def run(name,n,*,equal=True,nullity=True,population=True,nonnull=False,native_nonnull=True,want='unsat'):
 l=[z3.Int(f'{name}_l{i}') for i in range(n)];v=[z3.Int(f'{name}_v{i}') for i in range(n)]
 ln=[z3.Bool(f'{name}_ln{i}') for i in range(n)];vn=[z3.Bool(f'{name}_vn{i}') for i in range(n)]
 lp=[z3.Bool(f'{name}_lp{i}') for i in range(n)];vp=[z3.Bool(f'{name}_vp{i}') for i in range(n)]
 s=z3.Solver();s.set(timeout=5000)
 if population:s.add(*[lp[i]==vp[i] for i in range(n)])
 if nullity:s.add(*[z3.Implies(z3.And(lp[i],vp[i]),ln[i]==vn[i]) for i in range(n)])
 if nonnull:
  s.add(*[z3.Implies(lp[i],z3.Not(ln[i])) for i in range(n)])
  if native_nonnull:s.add(*[z3.Implies(vp[i],z3.Not(vn[i])) for i in range(n)])
 if equal:
  for i in range(n):
   for j in range(n):s.add(z3.Implies(z3.And(lp[i],lp[j],vp[i],vp[j],z3.Not(ln[i]),z3.Not(ln[j]),z3.Not(vn[i]),z3.Not(vn[j])),(l[i]==l[j])==(v[i]==v[j])))
 a=cardinality(l,ln,lp);b=cardinality(v,vn,vp);s.add(a!=b);result=str(s.check());assert result==want,(name,result)
 record={'id':name,'rows':n,'expected':want,'actual':result,'premises':{'pairedCompletePopulation':population,'equalityAgreementBothDirections':equal,'nullityAgreement':nullity,'actualIdealNonnull':nonnull,'nativeNonnullGuard':native_nonnull if nonnull else None}}
 if result=='sat':
  m=s.model();record['counterexample']={'logicalCount':str(m.eval(a,model_completion=True)),'nativeCount':str(m.eval(b,model_completion=True)),'logical':[{'present':str(m.eval(lp[i],model_completion=True)),'null':str(m.eval(ln[i],model_completion=True)),'value':str(m.eval(l[i],model_completion=True))} for i in range(n)],'native':[{'present':str(m.eval(vp[i],model_completion=True)),'null':str(m.eval(vn[i],model_completion=True)),'value':str(m.eval(v[i],model_completion=True))} for i in range(n)]}
 checks.append(record)
for n in range(5):run('nullable_correspondence_'+str(n),n)
run('missing_equality_counterexample',2,equal=False,want='sat')
run('missing_nullity_counterexample',1,nullity=False,want='sat')
run('missing_population_counterexample',2,population=False,want='sat')
run('required_nonnull_specialization',4,nullity=False,nonnull=True)
run('missing_native_nonnull_guard',1,nullity=False,nonnull=True,native_nonnull=False,want='sat')
print(json.dumps({'scope':'Bounded0..4 paired rows over mathematical equality classes and explicit null/presence. Conditional encoding/nullity/population correspondence only; no pre-storage plaintext equality, native codec, current authority, compiler/SQL refinement or nullable/absent source admission proof.','z3':importlib.metadata.version('z3-solver'),'checks':checks},indent=2))
