"""Finite authored rules vs bounded implementation-fold SMT; no native authority."""
import hashlib,itertools,json,pathlib,importlib.metadata
import z3
class FormalError(Exception):pass
TRUE,FALSE,UNKNOWN=0,1,2
PERMIT,DENY,INDETERMINATE,CONFLICT=0,1,2,3

def solve(formula,want,variables):
 solver=z3.Solver();solver.set(timeout=2000);solver.add(formula);result=solver.check()
 if result!=want:raise FormalError()
 return {'result':str(result),'witness':{str(v):str(solver.model().eval(v,model_completion=True)) for v in variables} if result==z3.sat else None}

def check(root:pathlib.Path):
 if z3.get_version_string()!='4.15.4' or importlib.metadata.version('z3-solver')!='4.15.4.0':raise FormalError()
 p,r,f=z3.Ints('permit_truth require_truth forbid_truth');truth_domain=z3.And(*[z3.And(v>=0,v<=2) for v in (p,r,f)])
 unknown=z3.Or(p==UNKNOWN,r==UNKNOWN,f==UNKNOWN)
 allowed=z3.And(p==TRUE,r==TRUE,f==FALSE)
 # Authored specification is the independent complete table of27 input tuples.
 spec=z3.IntVal(DENY)
 for a,b,c in itertools.product(range(3),repeat=3):
  expected=INDETERMINATE if UNKNOWN in (a,b,c) else PERMIT if (a,b,c)==(TRUE,TRUE,FALSE) else DENY
  spec=z3.If(z3.And(p==a,r==b,f==c),expected,spec)
 # Implementation abstraction follows unknown precedence then ordered guards.
 fold=z3.If(unknown,INDETERMINATE,z3.If(z3.Or(p!=TRUE,r!=TRUE,f==TRUE),DENY,PERMIT));disclosure=z3.If(fold==PERMIT,1,0)
 properties={
  'WFT-FM-002':solve(z3.And(truth_domain,unknown,z3.Or(fold!=INDETERMINATE,disclosure!=0)),z3.unsat,(p,r,f)),
  'WFT-FM-003':solve(z3.And(truth_domain,z3.Or((fold==PERMIT)!=allowed,z3.And(fold!=PERMIT,disclosure!=0))),z3.unsat,(p,r,f)),
  'bounded_specification_correspondence':solve(z3.And(truth_domain,fold!=spec),z3.unsat,(p,r,f)),
 }
 witnesses={'allowed':solve(z3.And(truth_domain,fold==PERMIT),z3.sat,(p,r,f)),'denied':solve(z3.And(truth_domain,fold==DENY),z3.sat,(p,r,f)),'unknown':solve(z3.And(truth_domain,fold==INDETERMINATE),z3.sat,(p,r,f))}
 broken=z3.If(unknown,INDETERMINATE,z3.If(z3.Or(r!=TRUE,f==TRUE),DENY,PERMIT))
 witnesses['broken_permit_gate']=solve(z3.And(truth_domain,broken==PERMIT,p==FALSE),z3.sat,(p,r,f))
 # Three slots and three transform equivalence classes cover up to three identities.
 d=z3.Ints('disposition_0 disposition_1 disposition_2');protected=z3.Bool('protected');domain=z3.And(*[z3.And(v>=0,v<=5) for v in d]);withheld=z3.Or(*[v==2 for v in d]);empty=z3.And(*[v==0 for v in d])
 def disposition_fold(values):
  w=z3.Or(*[v==2 for v in values]);identities=[z3.Or(*[v==n for v in values]) for n in (3,4,5)];none=z3.And(*[v==0 for v in values]);conflict=z3.Sum(*[z3.If(i,1,0) for i in identities])>1
  decision=z3.If(z3.And(protected,none),INDETERMINATE,z3.If(w,PERMIT,z3.If(conflict,CONFLICT,PERMIT)))
  kind=z3.If(decision!=PERMIT,0,z3.If(w,2,z3.If(identities[0],3,z3.If(identities[1],4,z3.If(identities[2],5,1)))))
  return decision,kind
 decision,kind=disposition_fold(d)
 # Independent authored finite table, avoiding implementation guard reuse.
 spec_decision=z3.IntVal(-1);spec_kind=z3.IntVal(-1)
 for values in itertools.product(range(6),repeat=3):
  for is_protected in (False,True):
   transforms=set(values)&{3,4,5}
   expected=(INDETERMINATE,0) if is_protected and values==(0,0,0) else (PERMIT,2) if 2 in values else (CONFLICT,0) if len(transforms)>1 else (PERMIT,next(iter(transforms)) if transforms else 1)
   match=z3.And(*[v==n for v,n in zip(d,values)],protected==is_protected)
   spec_decision=z3.If(match,expected[0],spec_decision);spec_kind=z3.If(match,expected[1],spec_kind)
 properties['WFT-FM-004']=solve(z3.And(domain,z3.Or(decision!=spec_decision,kind!=spec_kind)),z3.unsat,(*d,protected))
 for index,permutation in enumerate(itertools.permutations(d)):
  other_decision,other_kind=disposition_fold(permutation)
  properties[f'disposition_order_{index}']=solve(z3.And(domain,z3.Or(other_decision!=decision,other_kind!=kind)),z3.unsat,(*d,protected))
 witnesses['withheld_dominates_conflict']=solve(z3.And(domain,d[0]==2,d[1]==3,d[2]==4,decision==PERMIT,kind==2),z3.sat,(*d,protected))
 witnesses['transform_conflict']=solve(z3.And(domain,d[0]==3,d[1]==4,d[2]==5,decision==CONFLICT,kind==0),z3.sat,(*d,protected))
 broken_kind=z3.If(z3.Or(*[v==3 for v in d]),3,kind)
 witnesses['broken_withheld_precedence']=solve(z3.And(domain,withheld,z3.Or(*[v==3 for v in d]),broken_kind!=2),z3.sat,(*d,protected))
 witnesses['missing_protected_disposition']=solve(z3.And(domain,protected,empty,decision==INDETERMINATE,kind==0),z3.sat,(*d,protected))
 witnesses['unprotected_original']=solve(z3.And(domain,z3.Not(protected),empty,decision==PERMIT,kind==1),z3.sat,(*d,protected))
 witnesses['compatible_transform']=solve(z3.And(domain,d[0]==3,d[1]==3,d[2]==3,decision==PERMIT,kind==3),z3.sat,(*d,protected))
 disposition_path=root/'crates/weft-core/tests/security-disposition-oracle.json';disposition_oracle=json.loads(disposition_path.read_bytes());seen=set()
 for vector in disposition_oracle['cases']:
  values=vector['slots'];flag=vector['protected']
  if type(flag)!=bool or len(values)!=3 or any(type(v)!=int or not 0<=v<=5 for v in values) or (tuple(values),flag) in seen:raise FormalError()
  expected={'permit':PERMIT,'deny':DENY,'indeterminate':INDETERMINATE,'conflict':CONFLICT}[vector['decision']]
  solve(z3.And(domain,*[v==n for v,n in zip(d,values)],protected==flag,z3.Or(decision!=expected,kind!=vector['disposition'])),z3.unsat,(*d,protected));seen.add((tuple(values),flag))
 if len(seen)!=432:raise FormalError()
 oracle_path=root/'crates/weft-core/tests/security-composition-oracle.json';oracle=json.loads(oracle_path.read_bytes());observed=[]
 for vector in oracle['decisions']:
  decode={'T':TRUE,'F':FALSE,'U':UNKNOWN};inputs=[decode[vector[k]] for k in ('permit','require','forbid')];expect={'permit':PERMIT,'deny':DENY,'indeterminate':INDETERMINATE,'conflict':CONFLICT}[vector['decision']]
  solve(z3.And(truth_domain,*[v==value for v,value in zip((p,r,f),inputs)],fold!=expect),z3.unsat,(p,r,f));observed.append(inputs)
 if len(observed)!=27 or len(set(map(tuple,observed)))!=27:raise FormalError()
 return {'version':'weft-formal/1','solver':'z3-solver4.15.4.0','status':'passed','properties':properties,'witnesses':witnesses,'oracleSha256':hashlib.sha256(oracle_path.read_bytes()).hexdigest(),'oracleTuples':27,'dispositionOracleCases':432,'dispositionOracleSha256':hashlib.sha256(disposition_path.read_bytes()).hexdigest(),'queryCounts':{'unsat':len(properties)+len(observed)+len(seen),'sat':len(witnesses)},'dispositionPrecondition':'Truth gate already permits; slot0 is no selected-field disclosure from an active true permit, not an absent grant. Rust oracle uses3true permit rules and1true require.', 'limits':'One permit/require/forbid,27truth tuples; one selected field,three abstract disposition slots/three distinct exact normalized transform equivalence classes. Actual Rust correspondence is separately mandatory. No arbitrary AST/hash/arithmetic/native/liveness proof.'}
