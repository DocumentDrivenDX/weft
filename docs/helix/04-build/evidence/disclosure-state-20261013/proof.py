"""Conditional original/withheld state lemma, not SQL refinement or authority."""
import z3,json
original,withheld=z3.Ints('original withheld');constraints=[original>=0,original<=2,withheld>=0,withheld<=2]
# 0=false,1=unknown,2=true. State0=n/a,1=original,2=withheld.
applicable=z3.And(original!=1,withheld!=1,z3.Or(original==2,withheld==2))
logical=z3.If(applicable,z3.If(withheld==2,2,1),0)
emitted=z3.If(z3.Or(original==1,withheld==1),0,z3.If(z3.And(original!=2,withheld!=2),0,z3.If(withheld==2,2,z3.If(original==2,1,0))))
s=z3.Solver();s.add(*constraints);assert s.check()==z3.sat;s.add(logical!=emitted);assert s.check()==z3.unsat
weak=z3.If(withheld==2,2,1)
s=z3.Solver();s.add(*constraints,logical!=weak);assert s.check()==z3.sat
print(json.dumps({'z3':z3.get_version_string(),'scope':'two-permit abstract original/withheld states, protected field; no transformation/value/null/absence/authentication or SQL implementation proof','equivalenceCounterexample':'unsat','ungatedWitness':str(s.model())},indent=2))
