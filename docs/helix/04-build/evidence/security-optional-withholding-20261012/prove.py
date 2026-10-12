"""Conditional finite disposition observability; not native noninterference."""
from pathlib import Path
import json,z3
root=Path(__file__).parent
a,b=z3.Ints('a b');masked=z3.Bool('masked')
valid=z3.And(a>=0,a<=2,b>=0,b<=2)
publication=lambda state:z3.If(masked,3,state) # value/null/absence -> identical withheld tag
queries=[('active-mask-presence-equality',z3.And(valid,masked,publication(a)!=publication(b)),'unsat'),
 ('false-mask-retains-presence',z3.And(valid,z3.Not(masked),a!=b,publication(a)==publication(b)),'unsat'),
 ('absence-first-leaks-under-mask',z3.And(valid,masked,a!=b,z3.If(a==2,2,3)!=z3.If(b==2,2,3)),'sat')]
out={}
for name,query,expected in queries:
 s=z3.Solver();s.add(query);p=root/(name+'.smt2');p.write_text(s.to_smt2());assert str(s.check())==expected
 r=z3.Solver(ctx=z3.Context());r.add(z3.parse_smt2_file(str(p),ctx=r.ctx));assert str(r.check())==expected
 out[name]=expected
print(json.dumps({'z3':z3.get_version_string(),'results':out,'replays':3,'limits':'Finite supplied-fact state and tag model. Same authorized resource/policy/context, valid complete cuts and exact withheld cell are premises. No native execution, errors, timing, private-fact or SQL noninterference proof.'},indent=2))
