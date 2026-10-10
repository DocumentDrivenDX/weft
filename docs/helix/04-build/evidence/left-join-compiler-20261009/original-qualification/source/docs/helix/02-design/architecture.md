---
ddx:
  id: weft.architecture
  type: architecture
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.prd
      kind: informed_by
---

# Weft architecture

## Scope

Weft owns a universal UMF-powered SQL dialect, a typed logical intermediate
representation (IR), capability checks, compiled artifacts and language bindings.
It is an independent UMF consumer. First targets: Ashlar/Databricks and
Truss/PostgreSQL. A third synthetic backend is required to prove extensibility.

## System Context

| Participant | Responsibility |
| --- | --- |
| UMF | Versioned metamodel, identity/type/relationship meaning, schemas and conformance fixtures |
| Weft frontend | Source dialect, name/type resolution, typed relational meaning and diagnostics |
| Backend plugin | Physical bindings, capability/domain checks, target AST/SQL, decoder/host obligations |
| Host | Trusted plugin registration, model/binding provision, policy/publication enforcement and execution |
| Python / TypeScript callers | Thin bindings to the same compiler; no independent query semantics |

## Containers and Components

Proposed crates: `weft-core`, `weft-python`, `weft-wasm`, `weft-conformance`.
Backend-author packages implement CONTRACT-002; Weft integration fixtures/test
harnesses belong here, while storage ownership stays in the sibling repositories.
Initial backend distribution location is an open packaging decision; compiler
core must not import Truss/Ashlar storage implementations.

```mermaid
flowchart LR
  Q[Weft SQL] --> P[Parser and dialect gate]
  M[Supplied pinned UMF modules] --> R[Identity and type resolution]
  P --> R
  R --> L[Typed logical IR]
  L --> C[Capabilities and semantic obligations]
  B[Pinned physical binding] --> C
  C --> T[Registered backend]
  T --> A[SQL, parameters, result contract and diagnostics]
  A --> H[Host execution]
```

Model preparation validates exact input bytes/digest, core schema and selected
meaning; compilation resolves queryable record/field names from those supplied modules. The
logical IR contains UMF identities and scalar/presence domains, never physical
SQL names. Backends consume it and the separately pinned binding. Frontend code
has no switch over backend IDs or target dialects. Equivalent plans may have
different SQL; source AST spelling is not the conformance oracle.

## Trust, State and Resource Boundary

Core operations are deterministic for identical bytes/profiles/plugins. They
perform no network/database/filesystem I/O and retain exact source bytes plus
unknown extension trees. Plugins are trusted executable host code, not a sandbox.
WASM distribution selects linked/host-registered implementations; arbitrary
native shared libraries cannot be loaded by a browser. Manifests are inert data.

Compiler state is immutable prepared input and per-request work. No result cache,
server, connections, secrets or policy decision engine is required. Outputs
name host obligations without claiming they were enforced. Limits and cross-host
transport are defined by CONTRACT-003.

## Fidelity and Evolution

UMF version, dialect version, IR version, backend ABI/profile version, binding
revision and build version are separate. Unknown source content is retained;
unknown selected meaning blocks. No scalar family alone proves native domain
compatibility. Stored values must meet explicit backend input obligations.
Module bundles pin their owning document bytes. Combining query sources across
documents does not invent cross-document UMF reference semantics.

## External Dependencies and Risks

Rust/PyO3/maturin/WASM are proposed by ADR-001. Parser selection is spike-gated.
Use UMF schemas and shared validation corpus rather than a new semantic worldview.
A JSON schema check is not full UMF semantic validation. The reader must agree
on identity, members, facets and reference guards before language compilation.
Backend production layouts and engine profiles remain unselected; contracts
name those dependencies rather than hardcoding local experiment tables.

## Validation

TP-001 requires shared independent result fixtures, native execution for each
claimed backend profile, Python/browser parity, resource refusal and independent
plugin registration. Plans and generated SQL are observable; SQL equality alone
never establishes semantic conformance.

## Module Boundaries

**Source Applicability**: source; handwritten Rust compiler, target adapters and embedding entrypoints require one owned meaning boundary. The map covers all Cargo workspace packages, including historical spike and conformance packages; it excludes no workspace member.

| Module | Responsibility / Owned Types | Public API | Allowed Dependencies | Forbidden Dependencies |
| --- | --- | --- | --- | --- |
| `crates/weft-core` | Original owning-version model interpretation, logical AST/IR, capability gates and compile protocol | Compiler/Registry, CONTRACT-001–003 | Locked external compiler dependencies; no workspace imports | Backend crates, embedding crates, storage/host implementations |
| `crates/weft-databricks` | Target binding, finite native representation, SQL/decoder obligations; `count_having` and `left_join` own their separate explicit profiles | Backend implementations of CONTRACT-002 | `weft-core`; locked serialization dependencies | Runtime/wrappers, PostgreSQL adapter, live storage clients |
| `crates/weft-postgresql` | PostgreSQL target meanings and SQL/obligations | Backend implementations of CONTRACT-002 | `weft-core`; locked target dependencies | Runtime/wrappers, Databricks adapter, live storage clients |
| `tests/register-backend/probe` | Independent conformance backend | Registered test Backend | `weft-core` | Concrete production adapters/runtime |
| `tests/frontend/wasm-probe` | Browser core probe entrypoint | WASM probe exports | `weft-core` | Target adapters/runtime |
| `crates/weft-runtime` | Explicit feature-selected compiler composition and portable entrypoint | `compile_json` | `weft-core`, Databricks/PostgreSQL adapters, backend probe | Python/WASM wrappers, storage/host implementations |
| `crates/weft-python` | Thin native Python translation | Python `compile_json` | `weft-runtime`; locked PyO3 | Direct core/adapters, browser wrapper, storage |
| `crates/weft-wasm` | Thin browser/WASM translation | WASM `compile_json` | `weft-runtime`; locked wasm-bindgen | Direct core/adapters, Python wrapper, storage |
| `spikes/b001/core` | Historical isolated foundation | Spike compiler exports | Locked external dependencies only | Active compiler/adapters/runtime |
| `spikes/b001/python` | Historical native spike wrapper | Spike Python exports | `weft-spike-core`; locked PyO3 | Active compiler/runtime and other wrappers |
| `spikes/b001/wasm` | Historical browser spike wrapper | Spike WASM exports | `weft-spike-core`; locked wasm-bindgen | Active compiler/runtime and other wrappers |

**Integration Owners**: UMF document interpretation -> `weft-core` owning-version reader; Databricks physical translation -> `weft-databricks`; PostgreSQL physical translation -> `weft-postgresql`; Python/browser translation -> their thin wrappers. External execution, original public UMF receipts, native source guards and authority/ACK admission belong to the caller host, not these compiler packages.

**Construction Policy**: `weft-runtime` registers explicitly compiled features; request backend ID/version/profile selects registered code. A SQL expression MUST NOT choose or silently switch a backend. Existing library consumers may construct their own Registry through CONTRACT-002.

**Boundary Check**: `python3 scripts/checks/check-module-boundaries.py`; its actual `cargo metadata --locked --offline --no-deps` graph MUST match every named workspace member, permitted workspace edges and exact manifest paths, refuse unknown local dependencies and cycles. Use this command locally, in `.githooks/pre-commit` (opt-in installation with `git config core.hooksPath .githooks`) and CI. `python3 tests/module-boundaries/check.py` MUST demonstrate a real allowed Cargo edge, a real forbidden core-to-adapter edge and Rust compiler refusal of private symbol access. Rust visibility owns symbol access; this checker does not claim AST-level within-crate responsibility checks. Such checks, target semantics and external dependency behavior remain explicit semantic review obligations. New workspace edges/members require reviewed map changes, not an expanded debt baseline.
