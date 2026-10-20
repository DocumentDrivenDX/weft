# Pinned UMF schema dependency

The runtime schema adapter accepts UMF core `0.10.0` only. Its owner-pinned
upstream core schema is `umf-0.10.0.schema.json`; source provenance and the
semantic-type reference resource are pinned under
`actions-plan-010/owner-pins.json`.

`umf-0.10.0-bundled.schema.json` is the compile-time validator input. Recreate
it with `python3 scripts/upstream/bundle-umf010-schema.py`. The generator checks
the owner-pinned source hashes and writes provenance to
`umf-0.10.0-bundled.source.json`. The bundle resolves the pinned semantic-type
reference schema locally without changing its assertions or recursive
references.

The `umf-0.7.0.schema.json`, `umf-0.8.0.schema.json`, and related source records
remain only for frozen historical evidence or explicit negative controls; they
do not enable runtime support for core formats below `0.10.0`. Structural
validation supplements Weft's selected-meaning and identity guards and does
not claim a complete Rust reimplementation of UMF.
