# Weft generated ordinary Page `/6` development wheel

**Candidate:** `weft-sql 0.4.65.dev1` / Cargo `weft-python 0.4.65-dev.1`  
**Class:** local build-only candidate; not published, committed, or deployed.

The wheel was built from Weft HEAD `ca86d095d4c362903a3461900dada0157efbdc63`
with a dirty worktree. `source-inventory.json` records that HEAD, every dirty
path visible after the build, and candidate source hashes. The package metadata
files and updated `Cargo.lock` show the exact version bump. The build used the
existing pyproject Maturin feature list and Rust/Python/Maturin tool versions
in `tool-versions.txt`.

The preserved `/5` wheel was
`/private/tmp/weft-generated-v5-wheelhouse/weft_sql-0.4.64.dev1-cp39-abi3-macosx_11_0_arm64.whl`,
SHA-256 `1bdb1d68d57227629ee49111f24d6e99ad1a8281f9ec562b2fae7dca145cdf18`.
Its hash was rechecked unchanged after this build. The new unique wheel is
`/private/tmp/weft-generated-v6-wheelhouse-20261105/weft_sql-0.4.65.dev1-cp39-abi3-macosx_11_0_arm64.whl`,
SHA-256 `d6650eb65226d94928526014b815e041cb7c00bf29cecb55bc828972c0f275b8`.
Its tag is `cp39-abi3-macosx_11_0_arm64`. The embedded `weft/weft.abi3.so`
SHA-256 is `f749ac20be43fa22f71b98a3804488339ac0b19480bc0c9c208f1e4c0eb5d181`.
Wheel metadata and WHEEL contents are retained here.

Maturin's first build invocation was still compiling when the command wrapper
returned its initial output; the captured `build.log` ends during compilation
and no wheel was present then. The retry completed successfully; its complete
output and built wheel path are in `build-retry.log`. Both invocations targeted
the same newly created empty wheelhouse; the final directory contains only the
`.65.dev1` wheel. No existing `.64` receipt or wheel was overwritten.

An isolated Python 3.9 venv installed the wheel with `--no-deps`. It imported
as version `0.4.65.dev1`, loaded the recorded extension hash, and exposed
`prepare_generated_read_algebra_v6_json`. `verification-retry.log` records:

- canonical `/6` output byte-equal to the retained Sol-accepted source vector
  in `generated-ordinary-page-v6-correction-20261105/issued-v6.json`;
- `/5` output byte-equal to the frozen source vector
  `generated-connection-v5-20261104/issued-v5.json`;
- refusal of an invalid `/6` request.

`schema-validation.log` validates the wheel's actual `/6` result against the
current Sol-accepted closed schema, including the required `nativeType` output.
The reconstructed v5/v6 request inputs and outputs are retained in this folder.

This is the compiler/issuer wheel. It does not establish a compiler-free
runtime, a UMF consumer, continuation resume behavior, native PostgreSQL
result decoding, generated client support, or deployment qualification. The
current authored `char` storage refusal remains explicit in the preceding
correction evidence.
