# Weft specification index

The active flow is `helix` under `docs/helix/`. Catalog binding for this bootstrap: installed HELIX **0.15.0**, full plugin `workflows/graph.yml`; templates and methodology are resolved from that plugin and are not vendored here.

Public contracts and requirements remain drafts. ADR-001 selects Rust after the bounded B-001 spike; ADR-002 accepts the structural registered backend boundary. The public compiler has native Python and browser WASM component evidence. No production storage binding is implemented.

| Activity | Read first |
|---|---|
| Discover | [Vision](00-discover/product-vision.md), [research](00-discover/research.md) |
| Frame | [PRD](01-frame/prd.md), [concerns](01-frame/concerns.md); five features and seven stories in adjacent directories |
| Design | [Architecture](02-design/architecture.md), [ADR-001](02-design/adr/ADR-001-rust-and-embedding.md), [ADR-002](02-design/adr/ADR-002-dialect-and-plugin-boundary.md) |
| Contracts | [Language/IR](02-design/contracts/CONTRACT-001-weft-sql-and-logical-plan.md), [backend interface](02-design/contracts/CONTRACT-002-backend-interface.md), [compile/host boundary](02-design/contracts/CONTRACT-003-compile-and-host-boundary.md), [application reads](02-design/contracts/CONTRACT-004-application-reads.md) |
| Test | [Project plan](03-test/TP-001-compiler-conformance.md), six story plans, [fixture corpus](03-test/fixtures/README.md) |
| Build | [Implementation plan](04-build/implementation-plan.md), [embedding spike](02-design/spikes/SPIKE-001-native-python-browser.md) |

The versioned JSON schemas and EBNF live beside their governing contracts. Stable `ddx.id` and `ddx.links` record local traceability. Sibling project evidence is cited as sources; this bootstrap does not invent a cross-flow catalog.

B-001 through B-004 complete their recorded component scopes; see the
[implementation plan and evidence](04-build/implementation-plan.md). This includes
PR #2's versioned application-read frontend, registered backend boundary and
public Rust/Python/browser compiler parity. B-005 is in progress: Truss's approved
mapping and native PostgreSQL qualification remain gates. Ashlar's approved layout
and Databricks access gate B-006. Synthetic fixture bindings do not select either
production profile or qualify an engine version. All 30 story criteria and release
qualification remain planned until B-005 through B-007 establish their evidence.
