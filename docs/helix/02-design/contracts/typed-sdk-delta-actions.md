---
ddx:
  id: weft.typed-sdk-delta-actions
  type: contract
  activity: design
  status: draft
  links:
    - weft.typed-sdk-engine-catalog
  authoring:
    home: repo
---

# Typed SDK local Delta action lowering

**Version:** `weft.native-action/0.1` with explicit `spark-delta4` target and
`umf.actions.native-delta/0.1` profile; bulk uses `weft.native-bulk/0.1` and
`umf.actions.native-delta-bulk/0.1`. **Status:** implementation candidate.

## Scope and boundary

Reuse original core0.8 action/source, complete role/security source, Field, Key,
read/write frame and rule admission. Weft owns all native SQL. Ashlar owns the
local Delta transaction, original program enrollment, native actor permission,
complete source/deployment fence, bounded buffered results, deadlines and native
receipts. No Truss protected action authority, distributed multi-store commit,
managed Unity Catalog authorization or temporal qualification is implied.
PostgreSQL lowering MUST refuse the Delta target and vice versa.

The initial Delta profile admits one complete physical table, string/boolean
and explicitly signed native integer Fields of at most64bits. Decimal and wider
integer action domains refuse. Bulk selection currently admits string/boolean
source columns. Unknown or unsupported selected assertions refuse through the
original core admission; their meaning is never dropped to gain support.

## Action program

The program retains the complete admitted source/action/binding/security context,
role mappings, entities, frames, exact scalar slot sources and conditions. It
contains native pre/post images, complete Key integrity SQL, first-false rule
markers, effect change guards and complete candidate SELECT statements. Integer
slots are explicitly BIGINT to avoid native32bit expression narrowing. Native
Spark ANSI behavior and binary string comparison are mandatory.

The fixed private `__umf_delta_stage` view MUST be the engine's version-pinned
original transaction frame, then its materialized complete candidates. The
engine may bind views and native parameters but MUST NOT rewrite emitted SQL.
Create, SET and delete candidates remain uncommitted until every postcondition
and complete Key check passes. Key mutation and cross-table effects refuse.
The native transaction registers the whole-table predicate/files under
Serializable isolation and commits data, original verification/effect/change
markers and the request's SetTransaction in one native commit. Receipt values
are actual native table UUID, enrolled native epoch and returned Delta version.

Validation evaluates all supported preconditions without a native commit.
Invocation stops at the first false condition; a later missing-frame evaluation
error remains distinct when validation reaches it. Native no-op means no logical
change; a native marker-only or data-equivalent commit may still record it.

## Bulk program

Only one admitted scalar SET without row pre/post rules is selected. The complete
original selection AST must project the complete Key and changed Fields and order
by the complete ascending Key. `maxItems`1..999 uses a max+1 sentinel. Preview is
advisory. Execute MUST reselect inside its own original native transaction.

The transaction-only selection program has version `weft.delta-selection/0.1`;
it cannot be extracted or advertised as an immutable public raw read. It refers
to the original transaction's stage view and never substitutes a fabricated
snapshot version. Candidate SQL performs one set-based complete-frame transform.
The engine freezes the exact original bounded selection as
`__umf_delta_selected`, retains the original frame as `__umf_delta_before`, and
executes the emitted complete pre/post verification SQL before native commit.
This proves unchanged Key census, selected assignments and unchanged nonselected
rows/Fields. Counts and exact selected/changed Keys share the native data commit.

## Failure semantics and evidence

Blocked compilation retains diagnostics. No unsupported target falls back to
PostgreSQL. Native authority denial is distinct from an empty selection. Original
Key/cardinality, source/program/schema/epoch/profile drift, row-bound overflow,
condition error or failed candidate verification refuses before commit. Native
optimistic conflicts do not retry. Uncertain submission or final release must
quarantine the executor and retain original recovery context; it never means
rollback or permission to resubmit. Exact commit reconciliation is a separate
required host operation before any retry/replay support claim.

Native qualification must cover installed artifacts, generated schema/instances,
original compiler parity, validate/first-false/postcondition behavior, create and
no-op Keys, independent native role denial, held native closure/role changes,
real concurrent Delta conflicts, complete bulk reselect/bounds, candidate
verification and same-commit marker/data custody. Local evidence remains scoped
by native versions, source domains and authority. Managed and temporal gates
remain separate.
