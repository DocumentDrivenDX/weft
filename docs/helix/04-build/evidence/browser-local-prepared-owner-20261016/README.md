# Browser local-action compatibility — 2026-10-16

This ordinary compiler update admits exactly the original browser layout0.1 /
`c203d09418fb4eddda829d791dda11c20341227c2d1d82e86d4c24afda5e7294`
and fresh layout0.2 /
`1ffa72c51518e809c773e7f9207c1de1598f331c15de61e02dda0abe0fc475e0`
pairs. Crossed and unknown pairs refuse. The prepared local-action owner is
explicitly pinned to UMF commit
`c1e316c42ebe2a3a87420a5edc7e1ae751137b5b`, producer
`sha256:6b331c0b5118435a79eb19b3c66464b4ea1ed3e70134fc3b71e07913487a408b`.
The original three schema bytes are unchanged. The old local preparation is
retained as a negative fixture and refused by this new cohort. The native/general
producer pin and all unrelated mapping checks remain unchanged.

## Source and artifact identity

Canonical compatibility source:
`ab94595f176cc67ed213843285dda4d401033959`, parent
`4f5b73df8b751b4291b84af3b59169c1a1b63ed0`, branch
`codex/browser-layout02-prepared-c1`. The newly built
[web assets](canonical-wasm.zip) use Rust1.90.0, wasm-bindgen0.2.105 targetweb and
only `truss-browser-local`. All3235 captured source input hashes match before /
after compilation and the exact canonical Git archive. This is endpoint source
correspondence, not a hermetic or held-file build claim.

The embedded owner revision is **UMF c1e316c4**, not the Weft source commit. The
Weft commit is external artifact provenance, assigned after verifying identical
source; it is not embedded by a Git-version build hook.

| Asset | SHA256 |
| --- | --- |
| JS wrapper | `2711256b057a90fa8a898f1d616b871a083f993177b6bbffcd6cbd17fcaa6119` |
| WASM | `4434aad3ba9ce064f37f21146943a74d18b26d20c9caa255bdd5e56f503b2fd2` |
| JS gzip | `0e3b4f1e08de78f9753855b14ecf698570e526b1fa62420384c5b8e52d75052a` |
| WASM gzip | `9f19d150e41488dc2e842646bff79882fef1cab90f49808196395aa52b71258a` |

This fresh WASM is not the earlier `f7976103...` candidate relabeled, and is not
a binary built from current B-008 main. The compatibility source deliberately
retains its original browser constructor ABI. Main receives the same five-file
additive compiler/resource/test change while preserving its concurrent features.
All1072 other relevant main source/resource/test files compare exactly to
`b8c34c84f3c8cae8e07680e4960942d685f9bb63`; no dirty checkout edits are borrowed.

## Validation

Both the frozen compatibility source and additive main integration pass all10
mutation compiler tests and both actual browser mapping admission tests. Main
also passes both boundary checkers,16 boundary controls,73 governed-artifact /
96-schema checks. No database execution occurs in these commands. Original
scoped source reviews are retained alongside parent-executed build/test logs.
Eight reviewed owner source files independently match the actual UMF c1 Git
commit; all3235 candidate input hashes and lossless artifact gzip pairs verify.

Initial main-check attempts lacked `yaml` dependency resolution and Cargo on
PATH; both failures are retained. Reusing existing installed Node dependencies
and specifying the pinned Cargo executable made the unchanged checks pass.
No test or compiler guard was relaxed.

## Remaining qualification

Fresh actual browser lifecycle,1024-update/replay/reopen/readback tests against
these exact new4434 assets remain downstream work. Earlier Chromium results
belong to the original6b/f797 candidate and are not transferred to this artifact.
Browser performance and publication are not qualified by compiler/build checks.
The newer UMF main producer71fa is a separate cohort, not an implicit replacement.

This update does not change or activate B-008, its current presence/authority
registrations, Truss native bindings, or any original132/shared158 security case.
No security/native/publication/performance success follows from this evidence.
