"""Abstract K3 lowering lemmas; not a proof of source extraction or SQL code."""
import json,z3
F,U,T=0,1,2
checks=[]
def prove(name,premises,discrepancy):
 s=z3.Solver();s.add(*premises);assert s.check()==z3.sat
 s.add(discrepancy);r=s.check();assert r==z3.unsat,(name,r)
 checks.append({'name':name,'satisfiablePremises':True,'counterexample':str(r)})
a,b=z3.Ints('a b');domain=[a>=F,a<=T,b>=F,b<=T]
# K3 truth order yields min/max; SQL CASE specifications are separate forms.
logical_and=z3.If(a<b,a,b);logical_or=z3.If(a>b,a,b)
sql_and=z3.If(z3.Or(a==F,b==F),F,z3.If(z3.Or(a==U,b==U),U,T))
sql_or=z3.If(z3.Or(a==T,b==T),T,z3.If(z3.Or(a==U,b==U),U,F))
prove('and',domain,logical_and!=sql_and);prove('or',domain,logical_or!=sql_or)
prove('not',domain,(T-a)!=z3.If(a==T,F,z3.If(a==F,T,U)))
for size in range(9):
 xs=[z3.Int('x'+str(i)) for i in range(size)];ds=[z3.And(x>=F,x<=T) for x in xs]
 logical=z3.IntVal(F)
 for x in xs:logical=z3.If(x>logical,x,logical)
 sql=z3.If(z3.Or(*[x==T for x in xs]),T,z3.If(z3.Or(*[x==U for x in xs]),U,F))
 prove('exists_'+str(size),ds,logical!=sql)
# Counterexample to SQL EXISTS WHERE condition (unknown witnesses discarded).
s=z3.Solver();s.add(a==U);s.add(z3.If(a==T,T,F)!=a);assert s.check()==z3.sat
print(json.dumps({'z3':z3.get_version_string(),'scope':'abstract K3 operators; existential witness counts 0..8 only; no SQL/parser/authentication/source-completeness implementation theorem','checks':checks,'weakenedExistsCounterexample':str(s.model())},indent=2))
