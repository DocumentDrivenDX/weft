---
ddx:
  id: SD-001
  type: solution-design
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-001
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
    - id: ADR-001
      kind: informed_by
    - id: ADR-002
      kind: informed_by
---

# SD-001: logical-dialect

## Scope and Approach

Feature FEAT-001 inherits the system architecture. Sequence: core model participation -> Weft AST validation -> name/type resolution -> typed relational plan.
Use the same core meanings and data across language bindings and backends.

## Domain and Component Responsibilities

UMF snapshots/modules carry identity and retained semantics. Logical plans carry
scan occurrences, typed expressions and relational operations. Binding/config
and target plans carry physical names/capabilities only after resolution.
Wrappers transport artifacts; conformance consumers inspect independent results.

## Interfaces and Data Flow

CONTRACT-001 defines language/IR, CONTRACT-002 backend operations,
CONTRACT-003 module/host transport. Trust executable plugin code only through
explicit registration. Unsupported semantics or profile mismatches produce
whole-operation refusals; no partial executable plan is returned.

## Security and Performance

No model-driven code loading or execution credentials. Apply declared limits
and immutable copies. Cache only by verified module/binding/profile/build pins;
no cache or cost optimizer is required in v0.1. Performance claims need measured
workloads; no latency target is invented at bootstrap.

## Testing and Risks

TP-001 and associated STPs allocate positive/negative/boundary, native and host
layers. Unknown selected content, exact arithmetic and physical layout drift
require independent evidence. A missing owning backend contract blocks its
integration; a parser accepting syntax does not establish language support.

## Delivery and Rollback

Finalize versioned schemas/traits, add failing tests, implement small story slices,
run appropriate real hosts/engines, then qualify the capability. Unregister a
failing backend/profile or decline the release; never rewrite retained UMF meaning.
