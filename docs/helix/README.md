# Weft specification index

What remains under `docs/helix/` is the specification Weft is built against. It is
reference material, not a statement of what is supported: the README says what
CI tests.

| Directory | Contents |
|---|---|
| [01-frame](01-frame/) | [PRD](01-frame/prd.md), [concerns](01-frame/concerns.md), features and user stories |
| [02-design](02-design/) | [Architecture](02-design/architecture.md), [ADR-001](02-design/adr/ADR-001-rust-and-embedding.md), [ADR-002](02-design/adr/ADR-002-dialect-and-plugin-boundary.md), solution and technical designs, [module boundaries](02-design/module-boundaries.json) |
| [02-design/contracts](02-design/contracts/) | The request and response contracts (markdown plus versioned JSON schemas); these are the public API reference |
| [03-test/fixtures](03-test/fixtures/README.md) | The conformance fixture corpus read by the Rust tests and `bun run specs:check` |

`bun run specs:check` compiles every contract schema and validates the fixture
scenarios against them. Release notes live in [CHANGELOG.md](../../CHANGELOG.md);
open work is in [ROADMAP.md](../../ROADMAP.md).
