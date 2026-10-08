# B-005 candidate compiler acceptance — 2026-10-08

Decision: B-005 passes its owner-authorized compiler slice. US-003-AC1–AC4
have executable evidence for the pinned candidate profiles below. The decision
closes implementation of the Truss backend component; it does not promote
candidate capabilities to supported or establish release/production conformance.
B-006 may start after PR #7 is merged. B-007 retains the release matrix gates.

## Governing scope

The build plan's B-005 gate is native PostgreSQL exact results, storage boundaries
and emitted host obligations. Its 2026-10-07 ownership correction explicitly
permits pinned draft realization descriptions and independent native fixtures;
Truss runtime/adoption and installed authorization procedures are separate from
compiler completion. US-003, TD-003, STP-003 and Contracts 001–004 require
original model interpretation, explicit physical mappings, whole-operation
refusal and exact result/host boundaries. FEAT-002 permits explicitly requested
candidates. TP-001's production support, both-engine and expanded release gates
remain B-007 requirements.

Selected profiles: UMF 0.7.0; Weft SQL/IR 0.1 and application-read 0.2;
backend interface 0.2; Truss candidate/original-definition registrations 0.1.0;
PostgreSQL 17.9, UTF8 and explicit C comparisons. Storage realization source
hashes are retained in `tests/truss-postgresql/upstream/storage-realization-pins.json`.
The layout and codecs are pinned drafts with synthetic owned fixtures, not an
installed Truss database. Original graph/presence/codec/comparator/relationship
bytes are admitted against the supplied logical module identities. Physical
interpretation is explicitly registered host code/metadata; it is not inferred
from source SQL or opaque model content.

## Acceptance evidence

| Criterion | Required observation | Executable evidence and determination |
| --- | --- | --- |
| US-003-AC1 | Same logical query preserves rows, exact types and bag multiplicity under the pinned binding/profile. | `original-sales-join-native.py`: 128 cases across all 16 homes for the original Customer/Orders name/customer_id/total query; independent nested-loop/Decimal oracle. `sales-join-driver-native.py`: independently specified string/Decimal(scale 2) descriptors, widened SUM precision, column names, text OIDs and exact lexical outputs. Recursive entities, sequences/maps/structured values, COUNT, relationships and complete-key paging have separate native receipts in the execution record. Passed for candidate compiler scope. |
| US-003-AC2 | Authored field/catalog mappings and discriminator filters isolate each scan and join; storage IDs are not business keys. | Original sales fixtures independently choose four property homes, distinct Customer/Orders IDs and unrelated storage IDs; unrelated types and unmatched rows cannot enter results. Composite-key/relationship matrices retain complete authored tuple order and mixed homes, including inverse directions and numeric/C-string cursor distinctions. Original binding/admission and `compiler.rs` tests reject wrong-owner/key/component mappings. Passed for candidate compiler scope. |
| US-003-AC3 | Unknown selected homes/meanings refuse; known absence, null limitations and exact values are not silently collapsed or coerced. | `compiler.rs` and original admission tests cover unknown binding/home/codec/join/execution profiles, malformed or stale original bytes and no-SQL refusals. Native optional/recursive and numeric corpora distinguish absent/empty/present and reject corruption independent of filters/LIMIT. Native null is expressly unsupported by the pinned profile, rather than treated as absence. The sales corpus's previously failing hidden UInt64 overflow now produces an owner-wide domain prerequisite. Passed for candidate compiler scope. |
| US-003-AC4 | Exact transport, preparation, pins/visibility/authority and publication obligations are enforced by a host or execution is refused. | Actual psycopg 3.2.10/libpq 17.5 sales driver: 224 cases, 32 publications, 96 pre-query refusals and 96 prepublication revocations; explicit parameter OID 25 and result OIDs [25,25], no float path. Separate entity driver passes 252 cases; 588 injected host tests cover malformed/unknown obligation semantics and preparation/publication failures. Hosts buffer results and recheck pins/context before publication. Passed for candidate compiler scope; authority callbacks are injected, not production authorization certification. |

## Current implementation and integration checks

The public Rust `original_admission::Configuration` owns definitions and registers
an `OriginalBackend` through the shared compiler factory; it is not a fixed
fixture digest allowlist. Conformance Python/WASM configuration exports exercise
this boundary on unlisted binding bytes. They remain `test-original`
instrumentation: released package registration/composition is B-007 work.

The sales checkpoint passed 160 core/PostgreSQL/runtime Rust tests, 80 new
Python/Chromium cases per runtime and 342 existing embedding cases per runtime.
Full responses and refusals match, with deterministic Python repeats. Fresh
native/WASM binaries are pinned in `B-005-original-sales-join.json`; the real
browser is Chromium 148.0.7778.96 / Playwright 1.62.1. The unchanged exhaustive
Decimal SUM composition was not rerun in this checkpoint and retains its earlier
434-domain evidence. The broadening of numeric comparison prerequisites was
reviewed against prior response fixtures: only new obligations changed; SQL,
parameters, columns, other metadata and every previous obligation were retained.
Affected optional/entity native and driver regressions passed again.

Independent original byte custody passes for 38 fixed conformance configurations
and 84 responses. New sales configurations additionally pass fresh original
admission for every compilation, Rust/full Python response parity and the native
corruption corpus. Source/harness/setups hashes and all driver outcomes are
retained in `B-005-sales-join-driver-native.json`; the preparation/native logs are
`/private/tmp/weft-sales-join-driver-preparation.log` and
`/private/tmp/weft-sales-join-driver-acceptance.log`. Run the native sales harness
with `WEFT_SALES_DRIVER_SETUPS` to author the driver's exact independent inputs,
then execute the driver against the owned PostgreSQL fixture using
`WEFT_DRIVER_PORT` and `WEFT_EVIDENCE_OUTPUT`.

## Claim boundaries and next slice

The declared v0.1 scalar families are Boolean, Unicode string, integers with
known widths/signedness, and exact decimal domains; v0.2 adds bounded original
type graphs, optional presence, whole entities, counts, relationships and authored
key paging. Every selected property home is explicit. Unsupported SQL/type/domain,
unrecognized native-null semantics and unknown physical meanings remain refusals.
The evidence record qualifies exact tested widths/domains/graphs and versions;
it is not a claim that every possible model graph or data value was enumerated.

No production binding, policy authority, deployment, package publication,
performance target or complete all-platform support is accepted by this slice.
All assessments remain candidate and require opt-in. No Databricks result is
claimed. Ashlar's accepted layout/binding and native Databricks evidence belong
to B-006; property/fuzz/mutation, all 30 release ACs, package ownership/license
and the full support inventory belong to B-007. These do not constitute an
unmet Truss compiler dependency.
