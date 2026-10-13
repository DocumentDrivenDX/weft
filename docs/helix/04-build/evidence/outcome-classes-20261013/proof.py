"""Three normalized transform classes; equality supplied by original composer."""
import json,z3
ids=z3.Ints('id0 id1 id2');classes=[z3.IntVal(0)]
for i in range(1,3):
 c=z3.IntVal(i)
 for j in range(i):c=z3.If(ids[i]==ids[j],z3.If(classes[j]<c,classes[j],c),c)
 classes.append(c)
s=z3.Solver();assert s.check()==z3.sat
s.add(z3.Or(*[(classes[i]==classes[j])!=(ids[i]==ids[j]) for i in range(3) for j in range(i)]));assert s.check()==z3.unsat
s=z3.Solver();s.add(ids[0]==ids[1]);s.add(z3.IntVal(0)!=z3.IntVal(1));assert s.check()==z3.sat
print(json.dumps({'z3':z3.get_version_string(),'scope':'three abstract normalized transform identities only; no SQL/literal normalization/result-contract/authentication implementation proof','classCorrespondenceCounterexample':'unsat','perRuleClassCounterexample':str(s.model())},indent=2))
