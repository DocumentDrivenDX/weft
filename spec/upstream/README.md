# Pinned UMF schema dependency

UMF experimental core 0.8.0 schema-properties envelope is pinned separately from
core 0.7.0. Its exact revision, source path and SHA-256 are retained in
`umf-0.8.0.source.json`; `umf-0.8.0.schema.json` is the unchanged upstream schema.
Owning versions dispatch independently without rewriting source declarations.

UMF experimental core 0.7.0 relationship envelope, from commit `202b32ad89dfb8eb67c0c05393ef400b51d02c2c`, path `spec/core/relationship-document.schema.json`. SHA-256: `a95512729481417f1a3a094cc337d7264a575bd6b54046b05c037db6c33b4813`.

[Upstream source](https://github.com/DocumentDrivenDX/umf/blob/202b32ad89dfb8eb67c0c05393ef400b51d02c2c/spec/core/relationship-document.schema.json). This runtime schema dependency is unrelated to the HELIX methodology catalog. Structural validation supplements Weft selected-meaning and identity guards; it does not claim a complete Rust reimplementation of UMF.
