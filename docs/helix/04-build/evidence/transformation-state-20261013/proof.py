"""Conditional three-transform conflict fold; not native SQL/literal refinement."""
import z3,json
active=[z3.Bool('active'+str(i)) for i in range(3)]
identity=[z3.Int('identity'+str(i)) for i in range(3)]
w=z3.Bool('withheld');eligible=z3.Bool('eligible')
conflict=z3.Or(*[z3.And(active[i],active[j],identity[i]!=identity[j]) for i in range(3) for j in range(i)])
# IDs abstract original normalized transform/version/output-field/literal tuples.
representative=z3.If(active[0],identity[0],z3.If(active[1],identity[1],identity[2]))
uniform=z3.And(*[z3.Implies(active[i],identity[i]==representative) for i in range(3)])
logical=z3.If(z3.Not(eligible),0,z3.If(w,1,z3.If(z3.And(z3.Or(*active),z3.Not(uniform)),2,z3.If(z3.Or(*active),3,4))))
emitted=z3.If(z3.Not(eligible),0,z3.If(w,1,z3.If(conflict,2,z3.If(z3.Or(*active),3,4))))
s=z3.Solver();assert s.check()==z3.sat;s.add(logical!=emitted);assert s.check()==z3.unsat
s=z3.Solver();s.add(eligible,w,conflict);assert s.check()==z3.sat
print(json.dumps({'z3':z3.get_version_string(),'scope':'three abstract normalized transform identities and selected row eligibility; tuple equality supplied by original composer; no SQL/native projection/literal/authentication implementation theorem','equivalenceCounterexample':'unsat','withheldOverConflictWitness':str(s.model())},indent=2))
