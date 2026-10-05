# Weft specification index

The active flow is `helix` under `docs/helix/`. Catalog binding for this bootstrap: installed HELIX **0.15.0**, full plugin `workflows/graph.yml`; templates and methodology are resolved from that plugin and are not vendored here.

Public contracts and requirements remain drafts. ADR-001 selects Rust after the bounded B-001 spike; ADR-002 remains proposed. A local native Python wheel and browser WASM experiment now exist. No target database execution or production storage binding is implemented.

| Activity | Read first |
|---|---|
| Discover | [Vision](00-discover/product-vision.md), [research](00-discover/research.md) |
| Frame | [PRD](01-frame/prd.md), [concerns](01-frame/concerns.md); five features and seven stories in adjacent directories |
| Design | [Architecture](02-design/architecture.md), [ADR-001](02-design/adr/ADR-001-rust-and-embedding.md), [ADR-002](02-design/adr/ADR-002-dialect-and-plugin-boundary.md) |
| Contracts | [Language/IR](02-design/contracts/CONTRACT-001-weft-sql-and-logical-plan.md), [backend interface](02-design/contracts/CONTRACT-002-backend-interface.md), [compile/host boundary](02-design/contracts/CONTRACT-003-compile-and-host-boundary.md), [application reads](02-design/contracts/CONTRACT-004-application-reads.md) |
| Test | [Project plan](03-test/TP-001-compiler-conformance.md), six story plans, [fixture corpus](03-test/fixtures/README.md) |
| Build | [Implementation plan](04-build/implementation-plan.md), [embedding spike](02-design/spikes/SPIKE-001-native-python-browser.md) |

The five JSON schemas and EBNF live beside their governing contracts. Stable `ddx.id` and `ddx.links` record local traceability. Sibling project evidence is cited as sources; this bootstrap does not invent a cross-flow catalog.

B-001 passed on its recorded platform; see [execution evidence](04-build/evidence/B-001-native-python-browser.md). B-002 now establishes the 0.1 frontend; [evidence](04-build/evidence/B-002-frontend.md). Next is B-002A, the required versioned application-read extension from PR #2. Parallel independent progress can prepare the frontend oracle and obtain owner-approved Truss binding and Ashlar layout profiles. Open questions are recorded in the PRD and contracts. In particular, synthetic fixture bindings do not select Ashlar's production layout or declare a supported engine version.
