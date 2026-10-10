# Weft specification index

The active flow is `helix` under `docs/helix/`. Current catalog binding: installed HELIX **0.15.4**, full plugin `workflows/graph.yml`; templates and methodology are resolved from that plugin and are not vendored here.

Public contracts and requirements remain drafts. ADR-001 selects Rust after the bounded B-001 spike; ADR-002 accepts the structural registered backend boundary. The public compiler has native Python and browser WASM component evidence. No production storage binding is implemented.

| Activity | Read first |
|---|---|
| Discover | [Vision](00-discover/product-vision.md), [research](00-discover/research.md) |
| Frame | [PRD](01-frame/prd.md), [concerns](01-frame/concerns.md); six features and nine stories in adjacent directories |
| Design | [Architecture](02-design/architecture.md), [ADR-001](02-design/adr/ADR-001-rust-and-embedding.md), [ADR-002](02-design/adr/ADR-002-dialect-and-plugin-boundary.md) |
| Contracts | [Language/IR](02-design/contracts/CONTRACT-001-weft-sql-and-logical-plan.md), [backend interface](02-design/contracts/CONTRACT-002-backend-interface.md), [compile/host boundary](02-design/contracts/CONTRACT-003-compile-and-host-boundary.md), [application reads](02-design/contracts/CONTRACT-004-application-reads.md), [blocked compatibility security foundation](02-design/contracts/CONTRACT-007-security-compilation.md), [reliability runner](02-design/contracts/CONTRACT-008-reliability-runner.md) |
| Test | [Project plan](03-test/TP-001-compiler-conformance.md), nine story plans, [fixture corpus](03-test/fixtures/README.md) |
| Build | [Implementation plan](04-build/implementation-plan.md), [embedding spike](02-design/spikes/SPIKE-001-native-python-browser.md) |

The versioned JSON schemas and EBNF live beside their governing contracts. Stable `ddx.id` and `ddx.links` record local traceability. Sibling project evidence is cited as sources; this bootstrap does not invent a cross-flow catalog.

B-001 through B-004 complete their recorded component scopes; see the
[implementation plan and evidence](04-build/implementation-plan.md). This includes
PR #2's versioned application-read frontend, registered backend boundary and
public Rust/Python/browser compiler parity. B-005 completes its owner-authorized
candidate compiler scope; the [acceptance audit](04-build/evidence/B-005-acceptance.md)
records original mappings, native PostgreSQL results and exact host boundaries. B-006 candidate work now has registered scalar lowering and native Databricks
[preparation/corpus evidence](04-build/evidence/B-006-native-preparation.md).
B-006 completes its owner-authorized candidate compiler scope; its
[acceptance audit](04-build/evidence/B-006-acceptance.md) records native typed homes,
optional/recursive/relationship values, numeric/collation/resource boundaries and
actual buffered host custody checks. Fresh Python/browser/CLI builds match 463
artifacts with a documented declaration-only correction. Synthetic fixture bindings do not select either
production profile or qualify an engine version. Historical B-007 records [passing acceptance for all 30 P0 criteria](04-build/evidence/B-007-acceptance-matrix.json), [exact supported native/compiler and host profiles](04-build/evidence/B-007-support-inventory.json), and both historical checkpoint workspace compositions. PR review/CI/merge remain pending. The [closure audit](04-build/evidence/B-007-closure-audit.md) separates those gates from distribution and production execution.


Shared security integration is B-008 (open), framed as FR-19–22 and
[CONTRACT-007](02-design/contracts/CONTRACT-007-security-compilation.md).
Main’s separate security0.1/0.2 protocols are governed by [CONTRACT-006](02-design/contracts/CONTRACT-006-security-compilation.md). Compile0.5 remains a separately validated blocked compatibility envelope underCONTRACT-007. Trusted security0.2 registration may run once after owner admission; physical lowering remains closed. The initial0.5 transport/core08 custody foundation refuses activation;
security logical/physical lowering and Rust/Python/browser/native qualification
remain unfinished. Prior B-007 ordinary compiler claims remain qualified to their original checkpoint.

B-009 reliability remediation is governed by [US-008](01-frame/user-stories/US-008-reliability.md), [TD-008](02-design/technical-designs/TD-008-reliability.md) and [STP-008](03-test/test-plans/STP-008-reliability.md). [US-009](01-frame/user-stories/US-009-security-compilation.md) separately allocates the four open security outcomes. The 30-criterion B-007 acceptance matrix is historical and does not qualify changed source or close these additional criteria.

## B-009 current reliability qualification

R0–R5 are reviewed prerequisite, bounded-input, module, configuration, diagnostics
and finite formal assurance chunks. R6 distinguishes the historical B-007
checkpoint from current input closure and freshly loaded host artifacts. Current
evidence is under `04-build/evidence/reliability/r6-*.json`; the final hosted CI
run must succeed at the pushed branch head. Ordinary qualification remains
scoped to retained native-tested fixtures and versions. Native databases are
not re-executed, released packages are not claimed, and FR19–22 security
acceptance stays open with public activation blocked.

## Next security work

The [B-008 continuation plan](04-build/implementation-plan.md#b-008-continuation-protected-read-security-completion-plan--2026-10-10)
sequences S00–S12 from upstream contract/version reconciliation through graph
query integration, owner-derived coverage, physical lowering, native custody and
guarded release, binding parity and per-profile activation. [TD-009](02-design/technical-designs/TD-009-security-compilation.md)
and [STP-009](03-test/test-plans/STP-009-security-compilation.md) retain design and
planned AC coverage. This is planning only. Native resources/version decisions
remain explicit prerequisites; FR19–22 full acceptance and public activation stay
open. B-009 completed at `2b88a9a`; its qualification receipts retain that exact
checkpoint and are not refreshed by these documentation edits.


The 2026-10-10 main integration preserves ordinary main versions and assigns
blocked security compile 0.5 / CONTRACT-007. Its independently built main
reference covers all 2,181 unchanged retained fixture requests: 2,128 complete response objects
are unchanged from historical native receipts; 53 changed outputs require native
requalification. [Baseline and exact differences](04-build/evidence/main-integration-20261010/baseline.json)
record that distinction. Fresh integrated host and CI qualification are separate
merge gates; historical R6 receipts remain pinned to 2b88a9a.
