---
ddx:
  id: STP-006
  type: story-test-plan
  activity: test
  status: draft
  authoring:
    home: repo
  links:
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# STP-006: qualify-and-evolve

## Story Reference

[[US-006-qualify-and-evolve]], [[TD-006-qualify-and-evolve]], [[SD-004]], TP-001.

## Scope and Objective

Prove this P0 journey under its selected dialect/model/backend/host versions.
Every covering test must cite its AC as `@covers US-006-ACm`. B-007 now has
executed component evidence below. Component passes do not close the complete
release corpus or qualify untested platforms/native domains.

## Acceptance Criteria Test Mapping

| AC | Planned test | Observable assertion | Layer | State |
| --- | --- | --- | --- | --- |
| US-006-AC1 | `support_evidence_audit` | Missing/failed/skipped evidence cannot count as supported. | Rust + host/contract | executed components; release closure open |
| US-006-AC2 | `pin_and_version_guards` | SHA-256 bytes and identity guards; no implicit migrations. | Rust + host/contract | executed components; release closure open |
| US-006-AC3 | `unknown_content_retention` | Original bytes/content recoverable; no opaque-to-known inference. | Rust + host/contract | executed components; release closure open |
| US-006-AC4 | `resource_and_fuzz_guards` | Duplicate keys, bounds, nesting, malformed text and plugin failures. | Rust + host/contract | executed components; release closure open |

## Executable Proof and Data

Implementation tests live in `tests/qualify-and-evolve/`; shared language-neutral
fixtures remain in `docs/helix/03-test/fixtures/`. Commands and environment
requirements are in `tests/qualify-and-evolve/README.md`. `bun run specs:check`
validates spec/fixture integrity only; it does not execute this story or certify
coverage.

| Criterion | Inspected component evidence | Remaining proof |
| --- | --- | --- |
| AC1 | B-007-support-audit: 41 synthetic controls; retained replay: twenty-two components, including 76 independent Truss row comparisons across 21 artifact scopes, 65 ordered comparisons, eleven native receipt/custody controls and sixteen host receipt controls | Every applicable case on each advertised native profile must have exact version/domain evidence. Candidate inventory remains unqualified. |
| AC2 | B-007-properties: stale module SHA refusals; B-007-backend-branches: original-byte/binding/profile guards; public corpus native Python/Chromium byte parity | Final release matrix must identify applicable pin/interface cases and exact feature builds. |
| AC3 | B-007-properties: Unicode/opaque retention and selected unknown refusal; model/backend branch receipts preserve unselected content and refuse selected meaning | Reconcile original-source retention assertions to the final shared host corpus; no opaque-to-known promotion. |
| AC4 | B-007-resources and parser receipts: generated malformed/duplicate text and exact limits; B-007-plugin-boundary: 26 atomic failure combinations; B-007-host-resources: seven raw cases through CLI/Python/Chromium | Review the complete critical-path boundary matrix. Seven host cases do not prove every resource or plugin path. |

Receipts are under `docs/helix/04-build/evidence/`. The 30-criterion acceptance
matrix records hashes and remaining scope; its in-progress status is deliberate.
The real Truss support-report audit joins all 76 retained compiler artifacts to
actual Python and Chromium receipts (152 joins), with request/response hashes and
retained runtime identities. These joins establish compiler transport parity;
they do not establish database execution by each host or promote a candidate
profile. The terminal Linux CI checkpoint executes 214 Rust tests across 35
suites with no ignored or filtered tests. Its retained replay checks twelve
components at that revision; the current local replay adds session reconciliation
and six session-custody corruption controls, plus Ashlar same-statement engine
reconciliation and six corruption controls, plus warehouse/build-linked unsigned
SUM reconciliation and six further controls, plus seven-scope real Ashlar report
consistency across 22 native outcomes, plus 32 warehouse-linked native COUNT
cases, nine projection-annotation controls and ten count-custody corruption
controls, for twenty-two components; it does not execute native engines or Chromium.

Actual backend engine checks are separate from Python/browser compiler transport
checks. The latest raw host resource run uses CPython 3.12.14, Chromium
148.0.7778.96, the recorded candidate CLI and test-third Python/WASM artifacts.
Their early common-boundary parity does not assert identical backend features.


## Edge Cases and Build Handoff

100% of applicable positive/refusal/boundary cases must pass. Do not count a
skipped platform test as a pass; missing native/host access blocks qualification.
Expected results are authored independently of emitted SQL/compiler code.
Require tests-before-code, retained logs/input hashes, and zero phantom claims.
