# Source-owned backend metadata probe

The public Registry registered and validated Candidate metadata from exact Weft
source `362a9c6af85996a80ed4af6f68efecfe6eb22110` with Rust 1.90.0. Complete
ordered source custody agrees with the retained 1,588-file Git inventory, including
blob identity, mode and byte digest. Source, exporter, producer and inventory
custody remained unchanged through the observation. The produced backend manifest
is retained unchanged; its SHA-256 is
`c385c963bb3ba128a839b2d53a841b986aaab27680efca7c210144e44181a32d`.

This is source-probe metadata, not executable self-description, public distribution
admission or native engine support. The separately reviewed produced CLI corpus
binds the compiler realization. The retained custody record includes the exact producer command, working directory,
source inventory and effective build environment. Reproduction requires exporting
the recorded commit, supplying that inventory and Rust toolchain/cache, and choosing
a fresh output directory. Historical temporary paths describe this observation;
they are not consumer dependencies. The probe command and lockfile describe this
producer observation; consumers will use durable distribution
artifacts instead of these temporary paths. No source model or SQL was rewritten.

Sixteen focused controls exercise the actual source-custody helper, including
missing/changed/unlisted files, path traversal, symlinks, Git blob/mode drift,
duplicate JSON/paths, byte/decoded limits and payload-free configuration refusal.
The original source's total size is 213,068,735 bytes; the probe admits at most
32MiB per file and 256MiB total, with a 4MiB compressed/decoded inventory and
bounded manifest/build-log acceptance. This is a producer-resource bound, not a
graph size or query performance limit. No engine was started. Ordinary artifact
writes do not provide a crash-durable journal, and these local diagnostics do not
establish an OpenTelemetry receiver integration or a hermetic build.
