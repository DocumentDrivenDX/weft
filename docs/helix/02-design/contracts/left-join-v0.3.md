---
ddx:
  id: weft.left-join-0.3
  type: contract
  activity: design
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
    - id: CONTRACT-004
      kind: informed_by
    - id: weft.exact-arithmetic-0.3
      kind: informed_by
---

# LEFT JOIN reads

**Type:** language, logical-plan and result boundary. **Version:** explicit
`weft-sql/0.3.0`, `weft-ir/0.3.0`, `weft-compile/0.3.0`. **Status:** draft.

## Scope and precedence

An explicit candidate backend/profile MUST opt in to ordered LEFT JOIN stages
with scalar String equality ON predicates and direct scalar String projection.
Existing profiles and 0.1/0.2 acceptance, refusals and artifact bytes MUST remain
unchanged. Aggregates, arithmetic, relationship inference, RIGHT/FULL joins,
unmatched-row filtering and nested structured values are outside this initial
subset. The explicit initial profile requires at least one LEFT stage; existing
INNER JOIN stages may compose with it and retain their bag meaning.

UMF owns the authored Field domain and availability. Outer-join availability
belongs to the relational plan and MUST NOT rewrite an authored required Field
as absent-allowed or change its ideal scalar type. Explicit native null remains
separately selected by an exact per-Field binding encoding; source missing and
native null MUST NOT be conflated with an unmatched join row.

## Normative language and plan surface

The 0.3 AST MUST represent join kind explicitly. The public 0.3 join object MAY
add `kind: "left"`; omission retains existing inner meaning and serialization.
The public schema MUST permit only this literal when present. Old inner plans
MUST omit the member. The resolver MUST preserve source-order stages and exact
scan occurrence plus original Record/Field identities. Each LEFT stage requires
`join.left` and `value.outerJoinPresence` capabilities before binding callbacks.
Optional String ON equality additionally requires the existing native-null and
null-aware equality capabilities and exact per-Field opt-in.

The public plan MAY add `outerJoinScans`, an ordered array of distinct scan
occurrence strings, omitted when empty. It MUST equal the right-scan occurrences
of LEFT stages in source order; later INNER stages may preserve this conservative
provenance even if they remove unmatched rows. The plan MUST retain the set of
potentially unmatched scan occurrences. It MUST
retain source type descriptors unchanged. Result representation MUST distinguish
source availability from relational unmatched availability through the optional closed Value representation member
`outerJoin: {scan: string, record: Identity}`, naming the exact scan occurrence
and its original Record identity. This member is admitted only in 0.3, omitted
on all old/base/inner-only representations, and MUST agree with the projected
Field scan, original Field identity, `outerJoinScans` and mandatory obligation.
The existing original `descriptor` and exact Boolean `nativeNull` flag retain
their independent meanings. Unknown object keys, forged identities and
outerJoin metadata on a base/inner-only scan MUST refuse before callbacks.
The carrier is the existing tagged Value shape: unmatched yields
`{state:"absent"}`, matched explicit native null yields `{state:"null"}`, and
matched valid scalar yields `{state:"value",value:exactString}`. A nullable SQL
cell or property value alone MUST NOT establish whether a row matched.

A backend MUST carry a genuine match sentinel from the right scan's proven
non-null pinned physical row identity. The profile MUST declare that original
identity column and actual native type; sentinel nullness MUST NOT require a
String conversion or TRY_CAST. Ashlar native BIGINT identities remain BIGINT. It MUST validate that identity under its complete
source-integrity/publication obligations. Synthetic sentinel constants that can
survive null extension, nullable properties, fabricated String constants and authored keys
without proven non-null membership MUST NOT substitute for that identity. A
legitimate empty String identity remains valid when the selected canonical
profile declares String identities and admits that original value.
Unknown sentinel provenance or an unfulfilled guard blocks execution.

The emitted mandatory `outerJoin.matchIntegrity` obligation MUST name every
LEFT stage, including stages with no selected right output. Its closed parameter
object contains `phase: "before-user-query"`, `samePublicationRequired: true`,
`noPartialPublication: true`, and ordered `scans`. Each scan entry contains exact
`scan`, original `record: Identity`, pinned `table` identity/version reference,
physical `identityColumn`, `nativeType`, and an unchanged emitted violation-count
`sql`. The host MUST prove original binding/profile correspondence and the same
whole publication vector, execute full-source native-type/non-null identity
checks before user SQL, and retain observations alongside other source checks.
A zero count alone cannot prove schema/type or binding correspondence. The
obligation MUST agree exactly with join stages and outerJoin result metadata;
unknown or missing maps refuse, regardless of an apparently valid result bag.

## Bags, order and predicates

For each incoming left-prefix occurrence, retain every right occurrence whose
ON conjunction is TRUE. If none match, emit exactly one null-extended occurrence.
Parallel equal rows and repeated matches MUST remain separate occurrences;
uniqueness, existential selection and relationship meaning MUST NOT be inferred.
Apply each later join to the complete prior bag, including unmatched rows.
An ON equality involving an unmatched or explicitly null operand is UNKNOWN and
cannot match, including null/null. JOIN ON name resolution remains prefix-only.

Every consumed original source occurrence MUST pass complete source/type/null
and pinned-snapshot guards before user SQL, including right rows that never match.
A malformed scalar MUST NOT become an unmatched row through TRY_CAST. A missing
source property remains a representation refusal or source-validity failure
according to its original public semantics, rather than join-generated absence.
ORDER BY retains exact scan/Field identity and UTF8_BINARY String behavior.
The initial subset permits ordering only by required non-null fields of the
non-null-extended left source; other order domains refuse explicitly.

## Errors and acceptance requirements

Unselected/unknown LEFT capability MUST refuse before all backend callbacks.
An unsupported output presence profile or unknown sentinel MUST refuse before
runtime acquisition. Invalid source carriers remain integrity/source failures;
unsupported representation remains WFT-CAPABILITY. Failure withholds the entire
buffered result until all publication/source/ACK holds and cleanup close.

Conformance MUST cover the unchanged original archaeology two-LEFT query; all
matches; no match; duplicate matches; repeated left occurrences; sequential joins;
matched empty String; matched explicit null; null/null nonmatch; required-null,
missing property and malformed nonnull refusal; exact source revision and sentinel
mismatch; wrong capability/profile zero callbacks; complete ordered tagged bags;
and native SQL unchanged from the compiler. Public AST/IR/result/request/response
schemas, native/Python/browser parity and every retained historical artifact MUST
be verified. A native pass requires independent original-source expected bags,
actual sentinel/schema/cell observations, unchanged native files and complete
opening/closing publication and ordinary ACK custody. Compile-only coverage MUST
NOT be reported as native qualification.


## Construction, diagnostics and assurance

The profile identity is `ashlar.databricks.left-join`, version
`0.3.0-left-join-candidate`, target `spark4-delta4-left-join-candidate`.
Runtime feature `ashlar-databricks-left-join` MUST register this profile and the
unchanged earlier profiles explicitly; mutually exclusive feature compositions
MUST refuse at build time. The request MUST select the exact backend/version,
target profile, candidate permission and authenticated binding. No SQL-driven
routing, environment-derived compiler settings, plugin loading or fallback is
permitted. Bindings retain the original profile and model/publication hashes.

Core owns join syntax, scan/Field provenance, logical availability and emission
validation. The Databricks adapter owns native match identity/type, target SQL,
exact carriers and mandatory host obligations. Thin Python/WASM APIs share the
same compiler; hosts alone own credentials, connections, native schema checks,
ordinary ACKs and result release. The Cargo boundary map/checker MUST remain
closed with no new workspace edge or storage client. Native source guards and
publication admission MUST NOT be moved into a shadow compiler validator.

Diagnostics MUST retain named capability, source span where available, original
scan/Field identity and a supported public phase. They MUST NOT report an
unmatched row as invalid original UMF or a source-invalid matched row as a valid
absence. Library compilation remains pure and deterministic; no telemetry
exporter or live engine is implied. Execution hosts MUST retain emitted-check
order, match-sentinel schema observations, exact original and native carriers,
complete multiplicity/order bags and successful closing context before release.

Assurance is precise-only for this finite subset: closed public schemas,
scan-qualified invariants, zero-callback capability controls, independent bag
oracles and native observations are required. Unit or compiler parity checks
MUST NOT imply exhaustive proof, model checking, arbitrary outer-join arithmetic,
production authority or all native engine versions.
