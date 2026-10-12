# Python release preparation — 2026-10-11

Target: `weft-sql` 0.4.2, native module `weft`. Owner decisions in this session:
MIT OR Apache-2.0 dual license; include newer candidate APIs; use the regular 0.4.2
release version. Intended PyPI owner/release maintainer is the user; account
identity and project rights are unverified. The owner authorizes a regular GitHub release with wheel assets; PyPI upload is not yet authorized.

Governing sources: [implementation plan](../../implementation-plan.md),
[TD-006](../../../02-design/technical-designs/TD-006-qualify-and-evolve.md),
[ADR-001](../../../02-design/adr/ADR-001-rust-and-embedding.md),
[compile boundary](../../../02-design/contracts/CONTRACT-003-compile-and-host-boundary.md),
[historical readiness](../B-007-release-readiness.md), current main integration
baseline and retained 0.4.1/0.4.2 distribution records. Catalog: installed HELIX
0.16.0 `workflows/graph.yml`, runtime-handoff route, product lane, `docs/helix/`.

The baseline is the clean remote commit
`cdd8fa698683cd87bb126916a75200e80e19d831`. The existing local checkout is older
and has unrelated unfinished security edits. Preparation uses an isolated source
snapshot and a reviewable patch; those edits remain untouched.

## Release composition

Python-only version `0.4.2` maps to Rust binding package `0.4.2`;
core/compiler versions remain unchanged. Wheel features are
`truss-postgresql-qualified`, `ashlar-databricks-qualified`,
`ashlar-databricks-paths-keys`, `ashlar-databricks-implicit-arithmetic`.
`compile_json` retains its old composition; the two explicit candidate functions
are versioned additive transports. Incompatible generic registrations are not
combined. Candidate labels, version/profile refusal and opt-in remain enforced.
No test-original/test-third backend enters the wheel.

The historical claim "license decision pending" is superseded for this release.
Both project licenses and third-party notices are packaged. Registry ownership,
final clean integration commit, hosted cross-platform evidence and authorization
to upload remain separate gates. PyPI project lookup returned HTTP404; name
availability is not an ownership claim.

## Evidence and limitations

See `summary.json` for exact wheels, digests, interpreters and passing validations.
Every installed-wheel replay checks 2,181 ordinary and 174 candidate/control
response objects, repeat determinism, package contents and native payload custody,
with subprocesses disabled during compilation. These are compiler/transport tests.

The current ordinary baseline preserves 2,128 historical native response objects;
53 changed outputs still lack native requalification. Candidate semantics retain
their original evidence scope. Neither ABI compatibility nor this release
preparation closes that gap or expands production storage/engine support.

Platform labels specify wheel compatibility, not engine support. macOS Intel and
Windows x64 require the prepared hosted matrix before being advertised. Linux
x86_64 local evidence uses Docker emulation. Python 3.14 from the pinned 2025
manylinux image is an RC build; hosted stable 3.14 validation remains required.
Supported minimum OS versions come from wheel tags, not direct execution on every
historical OS version. PyPy, free-threaded CPython, musl and other architectures
remain excluded.

## Operator handoff

[Build, validation and Trusted Publishing instructions](../../../../../scripts/python-release/README.md)
name exact account/repository/workflow/environment configuration. Publication is
manual and requires the protected `pypi` environment; all platform and interpreter
jobs must pass. No sdist is offered without extracted-source validation.

The user still supplies their PyPI username and establishes the pending publisher;
integration/push approval is needed for the clean release patch and hosted runs.
Final upload authorization follows artifact review. A narrower platform release
requires a reviewed matrix change, rather than ignoring a failed platform.
