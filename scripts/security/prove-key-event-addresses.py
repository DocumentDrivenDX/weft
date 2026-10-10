"""Conditional selected-Key/member-address laws; not Rust/native refinement."""
from pathlib import Path
import hashlib
import json
import z3
ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs/helix/04-build/evidence/security-key-events"
PATHS = ["scripts/security/prove-key-event-addresses.py", "crates/weft-core/src/security_obligation_sources/key_events.rs", "crates/weft-core/src/security_obligation_sources.rs", "crates/weft-core/src/security_semantic_coverage.rs"]
FROZEN = {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in PATHS}
ref = z3.Datatype("KeyQualifiedRef")
ref.declare("Ref", ("document", z3.StringSort()), ("module", z3.StringSort()), ("element", z3.StringSort()))
Ref = ref.create()
key = z3.Datatype("SelectedKeyAddress")
key.declare("Key", ("source", z3.StringSort()), ("target", Ref), ("keyId", z3.StringSort()))
Key = key.create()
member = z3.Datatype("OrderedMemberAddress")
member.declare("Member", ("key", Key), ("position", z3.IntSort()), ("field", Ref))
Member = member.create()
s0, s1, k0, k1 = z3.Strings("original_source alternate_source selected_key_id alternate_key_id")
target, field = z3.Consts("retained_target retained_field", Ref)
i, j = z3.Ints("original_index alternate_index")
original = Key.Key(s0,target,k0)
laws = [
    ("ordered-member-position", [i>=0,j>=0,i!=j], Member.Member(original,i,field)==Member.Member(original,j,field), original==original, Member.Member(original,i,field)!=Member.Member(original,j,field)),
    ("selected-key-identity", [k0!=k1], Key.Key(s0,target,k0)==Key.Key(s0,target,k1), target==target, Key.Key(s0,target,k0)!=Key.Key(s0,target,k1)),
    ("original-key-occurrence", [s0!=s1], Key.Key(s0,target,k0)==Key.Key(s1,target,k0), k0==k0, Key.Key(s0,target,k0)!=Key.Key(s1,target,k0)),
]
records=[]
for name,premises,full,erased,witness in laws:
    for role,expression,expected in [("law",full,"unsat"),("erasure-control",erased,"sat"),("nonvacuity",witness,"sat")]:
        solver=z3.Solver();solver.set(timeout=10000);solver.add(*premises,expression)
        smt=solver.to_smt2();actual=str(solver.check())
        context=z3.Context();replay=z3.Solver(ctx=context);replay.set(timeout=10000)
        replay.add(z3.parse_smt2_string(smt,ctx=context));result=str(replay.check())
        if actual!=expected or result!=expected:raise RuntimeError((name,role,expected,actual,result))
        records.append({"law":name,"role":role,"expected":expected,"actual":actual,"replay":result,"smt2":smt})
if any(hashlib.sha256((ROOT/p).read_bytes()).hexdigest()!=d for p,d in FROZEN.items()):raise RuntimeError("Declared source changed during execution")
receipt={"version":"weft.security.key-address-laws/0.1.0","laws":3,"formulas":records,"z3Version":z3.get_version_string(),"sourcePins":FROZEN,"custody":"Sources frozen before formula construction and rechecked before publication; SMT captured before solve.","assumptions":"Typed constructors preserve exact selected Key ID, qualified target, original source occurrence and zero-based nonnegative member index. Source coordinates are opaque, not decoded.","scope":"Conditional algebraic constructor laws only. Erasure controls are projection sanity checks, not Rust mutants. No canonical-string/usize/Vec/pointer correspondence, traversal completeness, original Key selection refinement, backend/template applicability, authority or native enforcement proof."}
OUT.mkdir(parents=True,exist_ok=True);(OUT/"key-address-formal.json").write_text(json.dumps(receipt,indent=2)+"\n")
print("3 conditional laws / 9 formulas passed; fresh-context replay passed")
