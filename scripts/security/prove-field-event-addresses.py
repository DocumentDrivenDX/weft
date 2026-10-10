"""Conditional algebraic field-address laws; not Rust/native refinement."""
from pathlib import Path
import hashlib
import json
import z3

ROOT = Path(__file__).resolve().parents[2]
OUT = ROOT / "docs/helix/04-build/evidence/security-field-events"
PATHS = ["scripts/security/prove-field-event-addresses.py", "crates/weft-core/src/security_obligation_sources.rs", "crates/weft-core/src/security_semantic_coverage.rs"]
FROZEN = {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in PATHS}

ref = z3.Datatype("QualifiedFieldRef")
ref.declare("Ref", ("document", z3.StringSort()), ("module", z3.StringSort()), ("element", z3.StringSort()))
Ref = ref.create()
channel = z3.Datatype("FieldChannel")
channel.declare("Context")
channel.declare("Stored", ("owner", Ref))
Channel = channel.create()
address = z3.Datatype("FieldAddress")
address.declare("Address", ("source", z3.StringSort()), ("channel", Channel), ("field", Ref))
Address = address.create()

source, other_source = z3.Strings("occurrence_source other_occurrence_source")
field, owner, other_owner = z3.Consts("declared_field stored_owner other_stored_owner", Ref)

# Full typed constructor, an explicitly erased countermodel, and a nonvacuity
# witness. Opaque source coordinates represent independently retained occurrences.
laws = [
    ("stored-context-separation", [],
     Address.Address(source, Channel.Stored(owner), field) == Address.Address(source, Channel.Context, field),
     field == field,
     Address.Address(source, Channel.Stored(owner), field) != Address.Address(source, Channel.Context, field)),
    ("stored-owner-separation", [owner != other_owner],
     Address.Address(source, Channel.Stored(owner), field) == Address.Address(source, Channel.Stored(other_owner), field),
     field == field,
     Address.Address(source, Channel.Stored(owner), field) != Address.Address(source, Channel.Stored(other_owner), field)),
    ("original-occurrence-separation", [source != other_source],
     Address.Address(source, Channel.Stored(owner), field) == Address.Address(other_source, Channel.Stored(owner), field),
     Channel.Stored(owner) == Channel.Stored(owner),
     Address.Address(source, Channel.Stored(owner), field) != Address.Address(other_source, Channel.Stored(owner), field)),
]
records = []
for name, premises, full, erased, witness in laws:
    for role, expression, expected in [("law", full, "unsat"), ("erasure-control", erased, "sat"), ("nonvacuity", witness, "sat")]:
        solver = z3.Solver()
        solver.set(timeout=10000)
        solver.add(*premises, expression)
        smt = solver.to_smt2()
        actual = str(solver.check())
        # Independent parser/solver context, never reuse the producing solver.
        context = z3.Context()
        replay = z3.Solver(ctx=context)
        replay.set(timeout=10000)
        replay.add(z3.parse_smt2_string(smt, ctx=context))
        replay_result = str(replay.check())
        if actual != expected or replay_result != expected:
            raise RuntimeError((name, role, expected, actual, replay_result))
        records.append({"law": name, "role": role, "expected": expected, "actual": actual, "replay": replay_result, "smt2": smt})
if any(hashlib.sha256((ROOT / p).read_bytes()).hexdigest() != digest for p, digest in FROZEN.items()):
    raise RuntimeError("Declared source input changed during formal execution")
receipt = {
    "version": "weft.security.field-address-laws/0.1.0",
    "scope": "Conditional finite algebraic constructor laws. No Rust string/Vec/pointer correspondence, traversal completeness, template applicability, authority, evidence admission or native refinement proof.",
    "assumptions": "Typed address constructor preserves original source occurrence, qualified field identity and Stored(owner)/Context channel. Rust retains canonical opaque source IDs separately; this model does not decode or prove their representation.",
    "z3Version": z3.get_version_string(), "laws": 3, "formulas": records,
    "sourcePins": FROZEN,
    "custody": "Declared source hashes frozen before formula construction and rechecked before publication; SMT2 captured before solve.",
}
OUT.mkdir(parents=True, exist_ok=True)
(OUT / "field-address-formal.json").write_text(json.dumps(receipt, indent=2) + "\n")
print("3 conditional laws / 9 formulas passed; independent fresh-context replay passed")
