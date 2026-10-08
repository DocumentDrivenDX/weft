# B-007 preparation — 2026-10-08

State: in progress. B-006 merged as PR #8, commit
`1ed1c24dce356e1dfe34ae049959f4af52ea0bc5`, before this slice started.

The first component is a local native-backend support-claim verifier. It reads
hash-pinned reports, requires exact compiler/dialect/IR/backend/target/engine/
layout/model/binding/settings context, and compares independently supplied
expected rows. Explicit bag comparisons retain duplicate counts; ordered cases
require exact row order. A passing summary alone cannot replace semantic results.
Missing/failed/skipped evidence, duplicate/unknown report fields, missing layers,
invalid/stale hashes, rounding, normalization changes and absent/null substitution
refuse. Unqualified engines and candidate claims cannot be silently promoted.

`tests/qualify-and-evolve/evidence-check.py` passes 41 independently authored
controls. Missing implementation, ordering and malformed-digest failures precede
those implementations; retained logs and summary are in B-007-support-audit/.
These are synthetic audit-algorithm tests, not native qualification or a completed
support inventory. The verifier checks consistency/custody of trusted local test
reports; it cannot manufacture producer provenance or prove oracle independence.

The complete B-007 gates remain intact:

- Audit all 30 P0 criteria against actual layer-appropriate evidence.
- Evaluate all 636 initial fixtures and at least 1000 distinct expanded assertions.
- Execute at least 10000 deterministic property cases with pinned generator/library
  versions, recorded seeds and persisted minimized regressions.
- Exercise mutation/fuzz and critical supported/refused semantic branches.
- Qualify every claimed native operation/type/domain on its exact engine profile;
  retain candidate/unknown status where evidence does not prove support.
- Verify native Python and real browser behavior, interfaces, pins, unknown-content
  retention and resource/refusal boundaries.
- Publish a versioned support inventory and review packaging/release procedures,
  package ownership and license before distribution. The owner license choice is
  pending; this does not stop independent conformance work.

The first component closes none of those broader gates by itself. No package
publication, production profile or broader platform claim is made.
