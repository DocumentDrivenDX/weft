# B-006 candidate compiler acceptance — 2026-10-08

Decision: B-006 passes its owner-authorized compiler slice. US-004-AC1–AC4
have executable evidence for the pinned candidate below. This closes the Ashlar
backend component; capabilities remain candidate and require explicit opt-in.
B-007 may start after PR #8 merges. No production compatibility, deployment or
released-package conformance is established.

## Governing scope

The build gate is native Databricks exact results, numeric/collation/publication
boundaries and the application-read extension. US-004, TD-004, STP-004 and
Contracts 001–004 govern logical interpretation, original UMF identity, explicit
storage homes, atomic refusals, exact carriers and host custody. The owner
permits pinned draft storage realizations and independent native fixtures for
compiler work; storage-runtime adoption and production policy/resolver deployment
are separate. TP-001's expanded property/mutation/fuzz, all-criterion and release
support inventory remain B-007 work.

Pins: UMF core 0.7.0; Weft SQL/IR and compile 0.1/0.2; backend interface 0.2;
backend `ashlar.databricks/0.1.0-candidate`; target `dbsql-candidate`; layout
`ashlar-delta/0.3`, DDL SHA-256
`ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e`.
Native evidence uses existing warehouse `2439e1f2e37ac563`. Its reported
`4.2.0` zero-build version is retained without inventing a warehouse release.
The compiler emits explicit UTF8_BINARY comparisons and exact-or-error numeric
prerequisites; fixed-base-ten VARIANT carriers are admitted, exponent/DOUBLE
representations refuse. Evidence is scoped to this observed warehouse and
synthetic private fixture publications, not all Databricks versions.

## Acceptance evidence

| Criterion | Required observation | Evidence and determination |
| --- | --- | --- |
| US-004-AC1 | Logical rows, exact types and multiplicity match independent expected values on native Databricks. | The original Customer/Orders query retains UInt64 values, widened exact Decimal SUM, duplicate fanout and unmatched/isolated inputs. Native scalar, typed-column, global/empty aggregate, 112 application/count/keyset, 36 optional and 52 relationship cases pass. The 133 compound cases add independently authored nested/structured/cyclic/list/map numeric and presence oracles; 48 entity/keyset combinations include props and native BIGINT ID homes. Rust/Python/Chromium match 463 saved native-backed artifacts, with the declaration-only correction below. Passed for candidate compiler scope. |
| US-004-AC2 | Owned physical layout and authored mappings preserve identity and bags without inferred tables or JSONB conventions. | Binding admission uses original supplied document/module/Field identities, original accepted relationships, exact layout/model pins and immutable table UUID/version vectors. Canonical and serving relationships retain parallel edges and derive logical key tuples from mapped Fields rather than physical IDs. Typed STRING/BIGINT homes and presence flags are lowered directly; props remain Ashlar STRING/VARIANT carriers. Native whole-input endpoint/key/multiplicity/duplicate-ID guards precede predicates, including hidden corrupt rows. Passed for candidate compiler scope. |
| US-004-AC3 | Unsupported meanings/domains refuse or require exact-or-error execution; no rounding, wrapping, null substitution or collation loss. | Native scalar/optional/compound corruptions refuse before the user query; finite DECIMAL38 overflow and empty SUM/COUNT controls retain exact semantics. Unicode normalization variants, case, trailing spaces, quotes and backslashes remain distinct. Six recursive controls prove exact member names, unknown-member refusal, 100000-node acceptance/100001-node refusal and depth-127 acceptance/depth-128 refusal. The host refuses partial/truncated/wrong-native-type transport. Native null is explicitly unsupported by this profile; absence is separately enveloped. Unknown selected extension/encoding/home/relationship meaning blocks compilation atomically. Passed for candidate compiler scope. |
| US-004-AC4 | Missing or changed publication/delegation custody prevents a valid read under inconsistent or broader authority. | A 49-case host fixture checks pre-query visibility/caller/policy/pins, unknown obligations, guard violations and exact transport. The actual native host driver passes 30 cases: six exact buffered reads and 24 discarded buffers after caller, policy, publication or retained-file changes. Native callbacks inspect actual caller, table/manifest UUIDs, manifest vectors, owner DDL schemas and retained-version reads; all guards run before data and custody is rechecked before returning rows. Synthetic complete-admin authority/lifecycle/projection attestations and injected changes prove fixture orchestration, not a production policy service or delegated credential system. Passed for candidate compiler scope. |

## Exact receipts and integration

The chronological record is [B-006-native-preparation.md](B-006-native-preparation.md).
Compound receipts and their independently authored expectations are indexed by
[B-006-compound-preparation.md](B-006-compound-preparation.md). The final native
host and compound application directories retain summaries, full statements,
accepted final statement ranges and custody hashes. Earlier setup/harness errors
and the concurrent-build hash rejection are controls, not accepted executions.
The successful final native runs use a frozen CLI outside Cargo's target directory,
SHA-256 `02f6db7df59ce6c075995083a2ff4750545d89765df4a7af0d101864f74d562c`.

The workspace test checkpoint passes 182 tests. The final Ashlar package passes
18 tests, including the additional presence declaration assertion. Bun's integrity
check passes 43 artifacts, 10 schemas, 30 planned allocations and 636 fixtures;
it does not certify compiler/native/story execution.

Final Python extension SHA-256:
`8e04e6812141e895c8002624d0d81e9288dcc645d918855a2fecc5fd3546d831`.
Final browser WASM SHA-256:
`73cf013ac6b14e07222f81201820cc049975c04fa93e3db8f37813e0712bc046`.
Chromium 148.0.7778.96 / Playwright 1.62.1 proves full byte parity for 463 artifacts,
without runtime network IO or Node globals, and exercises actual trap retirement.
The native Python extension runs with PATH empty and subprocesses forbidden;
it has no JavaScript sidecar. The final CLI independently matches all 463 expected
artifacts, SHA-256 `3b1ab2973c2323f34d827698c8bfee28ecbf67c3e74086184f880b11149140c5`.

The presence capability declaration originally said scalar envelopes; it now
explicitly includes compounds. Historical native receipts remain unchanged.
The parity harness applies one independently specified declaration-field amendment
to 172 cases and requires every other field—including native SQL, parameters,
result descriptors, obligations and diagnostics—to match exactly. This is a
metadata correction, not a replacement native oracle or a lowering change.
Its failing baseline, Rust assertion and per-case canonical hashes are retained.

## Claim boundaries and next slice

The registered adapter is optional and pure: no database IO, model fetching or
policy decision enters compilation. Hosts own connections and fulfill emitted
obligations. Selected scalar/type/relationship meanings and compound encodings
are admitted explicitly; unknown meanings and unsupported forms refuse.
Compound depth/node/global native resource errors refuse atomically rather than
publish shortened values. The observed candidate subset is not an enumeration
of every possible graph, numeric domain, platform or warehouse version.

B-007 must retain these exact qualifications while completing the 30-criterion
matrix, initial/expanded corpus, deterministic property cases, mutation/fuzz
checks, versioned support inventory and packaging/release decisions. No production
profile, performance target, package publication or installed storage runtime is
accepted by this component audit.
