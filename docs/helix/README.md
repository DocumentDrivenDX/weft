# Weft specification index

The active flow is `helix` under `docs/helix/`. Catalog binding for this bootstrap: installed HELIX **0.15.0**, full plugin `workflows/graph.yml`; templates and methodology are resolved from that plugin and are not vendored here.

All governed documents are drafts; ADRs contain proposed decisions. No target SQL execution, Python wheel, WASM compiler or production storage binding is implemented by this bootstrap.

| Activity | Read first |
|---|---|
| Discover | [Vision](00-discover/product-vision.md), [research](00-discover/research.md) |
| Frame | [PRD](01-frame/prd.md), [concerns](01-frame/concerns.md); four features and six stories in adjacent directories |
| Design | [Architecture](02-design/architecture.md), [ADR-001](02-design/adr/ADR-001-rust-and-embedding.md), [ADR-002](02-design/adr/ADR-002-dialect-and-plugin-boundary.md) |
| Contracts | [Language/IR](02-design/contracts/CONTRACT-001-weft-sql-and-logical-plan.md), [backend interface](02-design/contracts/CONTRACT-002-backend-interface.md), [compile/host boundary](02-design/contracts/CONTRACT-003-compile-and-host-boundary.md) |
| Test | [Project plan](03-test/TP-001-compiler-conformance.md), six story plans, [fixture corpus](03-test/fixtures/README.md) |
| Build | [Implementation plan](04-build/implementation-plan.md), [embedding spike](02-design/spikes/SPIKE-001-native-python-browser.md) |

The five JSON schemas and EBNF live beside their governing contracts. Stable `ddx.id` and `ddx.links` record local traceability. Sibling project evidence is cited as sources; this bootstrap does not invent a cross-flow catalog.

First implementation gate: the native Python/browser spike. Parallel independent progress can prepare the frontend oracle and obtain owner-approved Truss binding and Ashlar layout profiles. Open questions are recorded in the PRD and contracts. In particular, synthetic fixture bindings do not select Ashlar's production layout or declare a supported engine version.
