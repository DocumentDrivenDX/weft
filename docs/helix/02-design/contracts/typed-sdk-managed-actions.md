---
ddx:
  id: weft.typed-sdk-managed-actions
  type: contract
  activity: design
  status: draft
  links:
    - id: weft.typed-sdk-delta-actions
      kind: informed_by
    - id: weft.typed-sdk-managed-build-20261016
      kind: informed_by
  authoring:
    home: repo
---

# Managed Delta action transaction prerequisite

Candidate request `weft.managed-action/0.1` wraps the exact original native action
request and an explicit catalog. Its action target is
`databricks-managed-delta`; no PostgreSQL/local-Spark fallback is permitted.
The original source, role mappings, security closure, binding and rule/Key/Field
admission remain mandatory. The output profile is
`umf.actions.managed-delta/0.1` with the existing source-preserving action program
shape. All physical SQL, including direct managed DML, is emitted by the Rust
backend. Hosts do not rewrite local candidate-frame SELECTs into writes.

The first managed transaction profile uses one native SQL-warehouse connection
with explicit interactive transaction boundaries, UC managed Delta catalog
commits and Serializable isolation. It is distinct from the earlier BEGIN ATOMIC
prerequisite witness and local Spark/JVM atomic transaction profile. Original
pre/post images, Key checks, rule markers and all effects execute within that
same native transaction. Unsupported domains retain their existing refusals.

A qualified host must own the authenticated principal, current source/schema and
native table identity interval, complete original program enrollment and role
fence. No default admitted authority exists. Data, exact verification/change
markers and the durable original request receipt must share the native commit.
The returned receipt must identify the exact retained native commit, never a
latest-history guess. Unknown commit/response loss quarantines the original
operation and retains the original request, program, attribution and recovery
context; it cannot authorize retry or replacement. Original reconciliation only
reads matching native receipt/commit custody and leaves absence uncertain.

Compiler implementation alone qualifies no native admission, transactional
semantics or managed backend. Native commit, rollback, conflict, response-loss,
identity/role drift and cleanup witnesses are required before support promotion.
Temporal SDK qualification remains HELD. The governing native behavior is the
[Databricks transaction documentation](https://docs.databricks.com/aws/en/transactions/)
and [interactive SQL Connector contract](https://docs.databricks.com/aws/en/transactions/transaction-modes),
read 2026-10-16; exact managed build pins still require native evidence.

## Unresolved native prerequisites (2026-10-16)

`umf.actions.roles/1` retains independently authenticated human semantics. A
service coordinator may own the dedicated schema and its deployment serialization,
but it cannot be labeled a human action session. A constrained human's native
ALTER, DROP and lease-modification denials must be witnessed against the actual
owner-enrolled fixture; workspace administration labels are not that evidence.

The proposed external deployment boundary uses one native Serializable persistent
lease shared by every source-owned deployment and action interval. The private
coordinator credential is excluded from the action connection. Uncertain original
leases remain occupied; no elapsed-time takeover or force-unlock is offered.
This is a candidate host prerequisite, not compiler-provided authority.

The Delta receipt must retain the complete sorted table-UUID store vector and
exact per-table native versions of this commit. A receipt-table version alone is
insufficient. Neither latest history nor matching timestamps establish that
mapping. The initial ledger-CDF idea is not admitted: Databricks documents
`WRITE_TABLE_WITH_CDF` as unsupported in multi-statement transactions. The exact
COMMIT/version metadata prerequisite therefore remains unresolved. See the
[native transaction restrictions](https://docs.databricks.com/aws/en/error-messages/multi-statement-transaction-not-supported-error-class),
read 2026-10-16. No managed action or bulk support follows from the candidate SQL
emitter or host transport controls.

The attempted dedicated native qualification is separately retained as failed:
fresh temporary service-principal creation returned native SCIM HTTP 403, and the
approved warehouse could not launch because its owner lacked cluster-creation
permission. The warehouse was stopped. No original managed SDK transaction or
native permission-fence witness ran; these failures are not acceptance evidence.

The separate candidate bulk wrapper is `weft.managed-bulk/0.1`, containing an
explicit catalog and the original `weft.native-bulk/0.1` request. Its source-owned
selection and direct set-based UPDATE use the same native transaction snapshot;
no historical snapshots or local Spark views are admitted. The compiler emits
whole-population before/after reads for the host's exact original Key/Field-frame
verification. This remains the existing bounded single scalar SET subset, without
row rules, and does not introduce Truss manifests, seals or durable submit
semantics. Runtime/Python/WASM exports require the explicit
`managed-delta-mutation-candidate` build feature; default exports remain gated.
Native original-result, concurrency, response-loss and exact receipt witnesses
remain open for both action and bulk.
