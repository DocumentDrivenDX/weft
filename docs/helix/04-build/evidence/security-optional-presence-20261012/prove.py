"""Conditional supplied-fact presence model; no native/issuer authentication."""
from pathlib import Path
import json,z3
root=Path(__file__).parent
fact,cell=z3.Ints('fact cell') # 0=value, 1=null, 2=explicit absence, 3=missing fact
original,optional,covered,equal=z3.Bools('original optional covered equal_value')
valid=z3.And(fact>=0,fact<=3,cell>=0,cell<=2)
selected=z3.And(original,covered,z3.Or(z3.And(cell==2,optional,fact==2),z3.And(cell<2,fact==cell,z3.Or(cell==1,equal))))
queries=[('presence-correspondence',z3.And(valid,selected,fact!=cell),'unsat'),
 ('missing-fact-refusal',z3.And(valid,selected,fact==3),'unsat'),
 ('truth-only-countermodel',z3.And(valid,original,covered,optional,fact==1,cell==2),'sat')]
results={}
for name,query,expected in queries:
 s=z3.Solver();s.add(query);(root/(name+'.smt2')).write_text(s.to_smt2());observed=str(s.check());assert observed==expected
 replay=z3.Solver(ctx=z3.Context());replay.add(z3.parse_smt2_file(str(root/(name+'.smt2')),ctx=replay.ctx));assert str(replay.check())==expected
 results[name]={'observed':observed,'fresh_context_replay':expected}
print(json.dumps({'z3':z3.get_version_string(),'queries':results,'scope':'Finite value/null/explicit-absence/missing-fact model conditional on exact original disposition and complete covered supplied facts. Truth-only selection cannot establish presence. No Rust/SQL refinement, trusted fact issuance, native execution or publication proof.'},indent=2))
