---
ddx:
  id: CONTRACT-018
  type: contract
  activity: design
  authoring:
    home: repo
  links:
    - id: FEAT-005
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
    - id: CONTRACT-004
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
  schemas:
    - id: weft.generated-query-program
      version: "1"
      title: Generated query pre-artifact
      path: docs/helix/02-design/contracts/generated-query-program.schema.json
      schemaFormat: json-schema
    - id: weft.query-codec-instructions
      version: "1"
      title: Issued exact query codec instructions
      path: docs/helix/02-design/contracts/query-codec-instructions.schema.json
      schemaFormat: json-schema
    - id: weft.generated-ordinary-page
      version: "1"
      title: Generated ordinary Page descriptor
      path: docs/helix/02-design/contracts/generated-ordinary-page.schema.json
      schemaFormat: json-schema
    - id: weft.generated-read-algebra
      version: "6"
      title: Generated read algebra v6
      path: docs/helix/02-design/contracts/generated-read-algebra-v6.schema.json
      schemaFormat: json-schema
    - id: weft.generated-read-algebra
      version: "7"
      title: Generated read algebra v7
      path: docs/helix/02-design/contracts/generated-read-algebra-v7.schema.json
      schemaFormat: json-schema
    - id: weft.generated-read-algebra
      version: "8"
      title: Generated read algebra v8
      path: docs/helix/02-design/contracts/generated-read-algebra-v8.schema.json
      schemaFormat: json-schema
---

# CONTRACT-018: Build-generated execution programs

**Contract ID:** CONTRACT-018  
**Type:** build artifact and library boundary  
**Version:** `weft.generated-execution/1`

## Purpose

Define Weft's build-issued execution programs so an installed UMF generated ORM
can bind exact values and execute supported compositions without importing or
calling a Weft compiler. This is an additive build/export path. Existing compiler
APIs retain their authoring use.

## Scope and Boundaries

Weft Rust is the sole owner of SQL and semantic lowering. UMF owns the generated
client, final package/provenance and native execution. Batten owns host admission
and authority. Weft exports inert programs and closed composition material;
it does not execute database effects, sign a build receipt or assert native
qualification. No Python executor may reimplement Weft's query lowerer.

## Normative Surface

`PreparedQuery.export_generated_program_json() -> Result<String>` returns canonical
UTF-8 JSON `weft.generated-query-program/1`. The response is a pre-artifact, not an
admitted runtime package. Its required members are:

| Field | Shape and rule |
| --- | --- |
| `version` | Exact `weft.generated-query-program/1`. |
| `readiness` | Exact `export-only` until an executable codec instruction program and binder exist; no runtime admission claim. |
| `source` | `{modelSha256,module,operation}` from admitted query. This is the model source digest, not a final bundle receipt. |
| `target` | `{id,modelSha256}` with exact target ID and admitted model digest. The physical binding digest/profile/version are supplied and verified by the trusted UMF build wrapper. |
| `program` | Existing sealed `PreparedQuery.program_json()` result, including parameterized SQL, DB-API transport, ordered slot sources, outputs, result bounds and native obligations. Existing fields and values are preserved. |
| `parameters` | Original declared parameter ID, qualified Field reference, carrier, nullability, full original declaration and order. For an executable artifact, include generated exact codec instructions. No lossy carrier-only summary. |
| `outputs` | Original qualified output Field references, carrier/nullability/decoder obligations. |
| `programDigest` | Optional in the export-only slice; when present, SHA-256 over canonical `program`. Required before executable admission. |

The build exporter MUST use the original admitted query and original Rust
normalization/lowering owner. It MUST preserve exact decimal/integer values,
absence/null, finite domains and every current refusal. UUID/date/timestamp/
128-bit carriers must retain their original admitted or refused behavior; export
does not promote an unexecutable input carrier to supported. The first
export-only source slice may carry the full declaration and a conservative
`weft.query-input-codec/1` normalization mode. That metadata is not executable
codec coverage and cannot qualify the compiler-free binder. An executable
artifact MUST add a closed codec instruction program sufficient for a
standard-library Python binder to validate and normalize selected inputs. If
the instruction vocabulary cannot encode an original rule, executable export
blocks that declaration; it cannot omit the rule or quietly accept a smaller
value domain. The binder interprets only issued instructions and slots, never
parses UMF source or SQL or generates operators, identifiers or arbitrary query
plans from caller strings.

The [issued codec schema](query-codec-instructions.schema.json) fixes a
four-instruction tuple under `weft.query-codec-instructions/1`:
`require-exact-wrapper`, `parse-carrier`, `check-refinements`, then
`normalize-native-query-value`. Rust emits admitted carrier/nullability,
normalized facets, numeric coefficient bounds and allowed-value identities.
Runtime checks each instruction and cross-step carrier/profile consistency
before it processes a value. Boolean/string identity is exact; string length is
Unicode scalar count and lone surrogates refuse. Numeric identities are exact
scaled coefficients with the original lexical and capacity bounds. Float token
parsing requires finite binary64 and original token bounds; allowed-value
identity treats positive and negative zero alike. Null is allowed only when
issued wrapper nullability permits it. Decimal native normalization preserves
the original Weft coefficient/trailing-zero/exponent result, without binary64.
Unknown opcodes, operand fields, carrier/profile combinations and issued rules
that the installed interpreter cannot enforce refuse before native execution.

For complete ORM delivery Weft also emits schema-specific fragment IDs and legal
composition transitions for every currently supported `umf_access` query method:
filter/predicate, join/related, selection, ordering, grouping/having/aggregates,
search and page forms, including exact refusal boundaries. An authored fixed
query is a first vertical slice, not acceptance of this complete algebra.
Declared native reads, writes, bulk, connection pages and recovery remain separate
generated program families under the same ABI and independent native obligations.
The generated fragment catalogue is finite per schema but may compose at runtime;
an enumerated list of manually captured whole queries cannot replace it.

UMF build tooling wraps one or more Weft pre-artifacts into the final canonical
`weft.generated-execution/1` manifest. Required top-level members are
`version`, `artifactId`, `artifactDigest`, `source`, `generator`, `executor`,
`target`, `algebra`, `programs`, `inventory`, `provenance`. `source` contains
`{documentId,coreVersion,sha256,moduleIds,bundleInputDigest}`; `generator`
contains `{umfDistribution,umfVersion,umfSourceDigest,weftDistribution,
weftVersion,weftSourceDigest,abi}`; `executor` contains
`{abiMajor,minMinor,maxMinor,packageId}`; `target` contains
`{id,profile,version,bindingDigest}`; and `provenance` contains
`{receiptDigest,issuer,buildId}`. The first executor tuple is
`{abiMajor:1,minMinor:0,maxMinor:0,packageId:"umf-access-generated-executor"}`.
All digests are lowercase 64-character SHA-256 hex strings. `inventory` maps
unique normalized relative paths to exact file SHA-256 values. The external
trusted build receipt binds a payload digest of the manifest without
`artifactDigest`/`provenance` and the same inventory; final `artifactDigest`
hashes the canonical manifest without `artifactDigest`. A host trust reference
is supplied separately, never trusted from manifest content.

Canonical JSON uses sorted Unicode scalar object keys, compact UTF-8, ordered
arrays and exact string tokens, with duplicate members/non-finite numbers refused.
Missing/unknown members, source or target drift and incompatible ABI block before
native effects. Build output must be deterministic for the same exact inputs.

## Candidate generated connection family

The Rust issuer now emits the closed candidate
`weft.generated-read-algebra/5` with executor ABI 1.5, alongside the unchanged
`/4` batch profile. The initial eligible subset has one required text/varchar
primary Key in fixed C-ascending order; unsupported roots retain explicit
per-root refusals. Other authored orders and the legacy typed `Page`
continuation remain separate preservation work.

The [issuer SQL evidence](../../04-build/evidence/generated-connection-v5-20261104/README.md)
qualifies the forward/prior templates on PostgreSQL 18.6. The closed schema,
native cursor contract and consumer qualification are maintained with UMF's
CONTRACT-907. Existing `/4` receipts retain their exact original cohorts.

The issuer must prove a total order with authored order semantics and a stable
Key tie break; emit closed keyset-after and prior-presence fragments, exact
position/codecs and native outputs; and retain the server-side byte guard for
all fetched rows, including `first+1` lookahead. `first=0` still requires
bounded next/prior probes and returns no edges with truthful flags. The native
UMF owner authenticates a cursor for **each** returned edge, scoped to
app/API, actor authority, operation, arguments, order, source and exact edge
position. Both probes run in the current selected request snapshot; a later
GraphQL operation opens a fresh snapshot. Offset paging, a Python semantic
SQL compiler, unguarded fetch-all and conversion of a legacy page-next token
are not fallbacks. Advertising Relay support for a selected cohort requires
the reviewed schema and native/Host/Relay evidence; issuer export alone does
not qualify that support.

## Candidate generated ordinary Page family

The proposed `weft.generated-read-algebra/6` adds a distinct per-root
`weft.generated-ordinary-page/1` descriptor and executor ABI 1.6. The closed
root and response schema is
[`generated-read-algebra-v6.schema.json`](generated-read-algebra-v6.schema.json).
It leaves the
closed `/5` Relay family unchanged. The initial PostgreSQL subset is one
required string primary Key in C ascending order, with no join or authored
alternate order. `/6` uses the `/4` root algebra and issues this page family
independently; Relay eligibility is not a prerequisite. The descriptor schema is
[`generated-ordinary-page.schema.json`](generated-ordinary-page.schema.json).
The bounded source/schema test record is
[`generated ordinary Page `/6` review evidence`](../../04-build/evidence/generated-ordinary-page-v6-source-review-20261105/README.md);
it does not qualify downstream consumers or release artifacts. The
[installed-wheel varchar schema correction](../../04-build/evidence/generated-ordinary-page-v6-varchar-schema-20261105/README.md)
requires the exact positive-integer varchar length in both Key outputs and
pairs each text/varchar native type with its PostgreSQL OID. This is schema
validation of emitted compiler data, not native execution qualification.

The descriptor issues the hidden Key projection, typed key output and codec,
keyset predicate, total order, guarded SQL template, one-row lookahead, and a
size slot bounded to 1–1000. The exact requested size is an identity input and
the SQL fetches `size + 1` rows for lookahead. PostgreSQL `snapshot=True` is
unsupported. Continuations retain `owner_cursor: null` and an empty snapshot
vector for this profile. Materializers preserve the legacy owner rule: a
continuation is emitted whenever exactly `size` items are returned, even when
lookahead found no further item. `/6` does not reuse Relay cursor tokens,
signature context, or edge envelopes.

The raw page SELECT emits `__weft_ordinal` from a window over the issued C
order, then the hidden Key and an explicit public-projection slot. That slot is
either empty or begins with `, ` followed by issued projection fragments. The
guard's outer result is ordered by `f.__weft_ordinal`. A required native
precondition query checks the mapped Key column's qualified PostgreSQL
collation identity and deterministic flag against `pg_catalog.C`; an explicit
`COLLATE` clause in page SQL does not substitute for this check. The issuer
requires qualified schema binding for this cohort. The executor binds the
precondition's parameters from the emitted `binding.schema`, native table name,
and native Key-column name, in that order, then requires exactly one row equal
to `(pg_catalog, C, true)` before page SQL.

Continuation identity is `weft.generated-ordinary-page-continuation-identity/1`.
The fixed identity binds source model digest, target ID/model digest/binding
digest, module and root Record, authored primary-Key ID and Field ID, issued
program digest, and exact page-template digest. Per call it binds ordered
selected Field IDs and the canonical exact predicate tree. The predicate tree
uses leaves `{op:"predicate",id}`, variadic `{op:"and"|"or",children:[...]}`
with at least two children, or unary `{op:"not",child:...}`. Every leaf ID
must be present in the issued predicate catalogue. The ordered `predicateIds`
are the tree's depth-first, left-to-right leaf IDs; normalized values bind in
that same traversal order for arity-one leaves only. Predicate codecs come
from the closed issued predicate catalogue; values must pass those exact codecs
before identity calculation. The exact bounded `size` is also included. No
other caller-controlled input is omitted from the identity.
An absent filter uses the canonical empty-expression sentinel `null` and emits
the literal `TRUE` into the filter slot. Nonempty filter trees are serialized
using this contract's canonical JSON rules before they enter continuation
identity.
`issuedProgramDigest` is SHA-256 of the canonical complete root algebra after
`ordinaryPage` is inserted and with only that digest member omitted. It therefore
binds inherited `fields.*.selectSql`, predicates, output metadata, and every
ordinary Page fragment without a recursive digest. Consumers recompute it from
the admitted root before executing any Page SQL. The Page descriptor also emits
`fieldOutputs`, a closed ordered inventory joining each selectable Field ID to
its exact source Field reference, bound native column, carrier, nullability,
native type, and PostgreSQL OID. The closed output relation includes PostgreSQL
`char`/1042 and `bytea`/17 alongside their exact Weft carriers. `bytea` can be
issued as a selectable Bits128 Field where source admission allows it. Although
the output helper and schema preserve `char`/1042, the current native table
lifecycle admission refuses authored `char` string storage with
`native column type/mapping/availability differs`; this is not a public `/6`
source support claim. A `char` primary Key is unsupported in all cases. Empty
selection remains valid.

For `/6` admission, consumers validate the exact top-level keys
`version/readiness/executorAbi/source/target/roots`, and reject unknown keys.
Each root allows only the inherited `/4` root fields
`module/record/fromSql/defaultSelection/fields/predicates/orders/keyTieOrder/
maxRows/obligations/nativeReadGuard/primaryKey/primaryKeyId/terminals/
frameOrdinalSql/frameScalarOrdinalSql/getMany` plus the required
`ordinaryPage` descriptor. The `ordinaryPage` is validated against the closed
schema above, including its exclusive eligible/unsupported branch; unknown
descriptor members and missing eligible members refuse admission. Inherited
root members are validated with the exact `/4` base validator before the `/6`
extension is considered. This is a required UMF verifier rule; the Weft
descriptor schema alone is not an admission claim.

Unsupported roots receive a closed refusal descriptor. Mixed ordering, null or
missing markers, multi-component Keys, joins, target-specific snapshots and
owner continuations remain unsupported until each has its own issued semantics
and evidence. This artifact is a transport descriptor, not a new UMF model
meaning; it retains original UMF Core 0.10 and `umf.tables` 0.1 identities.

## Proposed same-artifact ordinary Page and Relay connection profile

The additive `/7` issuer candidate and its closed schema
[`generated-read-algebra-v7.schema.json`](generated-read-algebra-v7.schema.json)
carry the existing `/5` forward connection and `/6` ordinary Page semantic
payloads from one admitted `/4` root under executor ABI 1.7. Their per-root
support/refusal branches remain independent, including a root where both
extensions refuse while its inherited `/4` read algebra remains usable. Such a
root advertises neither extension. Standalone `/5` and `/6` retain their
existing global eligible-root requirements. Shared source, target, Record, Key,
Field and native mapping identities must agree. This candidate does not broaden
either profile's eligible subset or reinterpret ordinary continuations as Relay
edge cursors. `/5` and `/6` artifact bytes, schemas and evidence remain unchanged.

The `/5` forward descriptor has hashes for its SQL templates, but no digest of
the complete root. In `/6`, `ordinaryPage.continuation.fixed.issuedProgramDigest`
is the current root-digest feedback slot. For the combined profile, compute
SHA-256 of canonical JSON for the complete combined root with only this slot
omitted. The preimage includes the entire forward descriptor and its template
hashes. Both Rust issuer and UMF verifier must recompute that exact preimage;
the verifier rejects disagreement before native acquisition. A future second
root-digest feedback slot would need a further versioned normalization that
omits both slots. The new digest changes the ordinary descriptor bytes, even
though the issued page and forward SQL, codecs, outputs and native obligations
retain their semantics. The complete combined algebra bytes are covered by
the generated file inventory, algebra SHA-256, final manifest and independently
trusted receipt.

Qualification requires one installed compiler-free package and one admitted
source/artifact/runtime/Host over PostgreSQL 18 to run typed ordinary SDK
initial/next/exhausted Pages and Lintel API3 `first=0`/interior connections,
including a mixed Note/Task/Node GraphQL operation on the Relay path,
token-crossing refusal, live policy/head checks,
byte guards and lease closure. Browser and Workerd support require separate
real-host evidence. This is coexistence only; the next distinct issuer slice
may cover one authored required to-one inner join using the existing
`Query.join(relationship, *, how=None)` shape and typed joined selection.
Rust must derive that join's closed transition, SQL fragment, exact output
metadata and native relationship correspondence from original admitted UMF
and physical binding. Optional/left and multi-hop joins remain unsupported.
The Page call uses a separate standalone statement operation through Batten's
public Host; the Relay path alone owns a held request snapshot. UMF/Batten must
add a closed Page call/position mapping and disjoint Host route for that
consumer. Weft's issuer does not alter either lifecycle or compile at runtime.

## Precedence and Compatibility

CONTRACT-003/004 retain compiler/host and application-read authority. This
contract adds a generated execution transport, without redefining native policy,
session, correspondence, transaction, cursor or receipt checks. ABI major changes
for incompatible shape/meaning; a minor adds closed opcodes or program families
without changing earlier meanings. An executor accepts only the manifest's
declared compatible interval. Installed source and final artifact are never
silently migrated. Explicit uploaded UMF may compile in a separate admitted
authoring workflow, followed by verification and fresh output admission.
Ordinary execution MUST NOT invoke that workflow or a compiler fallback.

## Error Semantics

| Condition | Outcome | Recovery |
| --- | --- | --- |
| Original rule cannot be emitted as closed codec | Build `blocked`, no partial program | Extend the Rust generator and version the instruction vocabulary. |
| Unsupported composition/opcode | Runtime `UNSUPPORTED`, no native call | Rebuild with qualified generated algebra or change query. |
| Missing/tampered/stale/untrusted artifact | Admission refusal, no native call | Rebuild, verify exact receipt and readmit. |
| Native current authority or source mismatch | Live denial/refusal | Restore authority or rebuild/readmit; artifact checks never waive live guards. |
| Possible write effect with lost response | Original-key unknown/uncertain outcome | Observe original key; never fresh-key automatic retry. |

## Examples

```text
Weft Rust PreparedQuery -> generated-query-program/1 (fixed declared read).
UMF build + trusted receipt -> generated-execution/1 manifest and Python files.
Stdlib executor -> bind exact values into issued parameterized SQL and decoder.
UMF native session -> current head/policy checks -> PostgreSQL 18 execution.
```

## Schema Basis

The inspected UMF core 0.10 Records, Fields and Keys plus `umf.tables` 0.1
declared operations remain source authority. This contract's artifact is an
execution/build transport, not a new model schema. Original UMF bytes and
unknown extension content remain recoverable. Selected unknown meaning blocks
generation. A machine-readable manifest schema and `ddx.schemas` reference
must be added when the G0 issuer implements it; no current source claims that
the proposed schema exists.

## Proposed required to-one inner join profile

The additive `/8` issuer request is a closed `weft.generated-read-algebra-request/8`
with required `nativeSource3`. The source projection is build-declared input; it
must be pinned to the original model, configuration and verified deployment
bundle, while the UMF verifier compares the emitted join descriptor back to
that same source and exact binding. It is not an independently trusted native
proof. The complete output uses executor ABI 1.8 and the closed schema
[`generated-read-algebra-v8.schema.json`](generated-read-algebra-v8.schema.json).
The existing request shapes, `/1` through `/7` artifact bytes and defaults remain
unchanged.

Each root carries a `requiredInnerJoins` catalog keyed by canonical escaped
original Relationship module/id. Every original Relationship is represented
as either a closed eligible descriptor or a bounded unsupported reason; an
unsupported or empty catalog does not disable the base root, Page or Relay
semantics. A caller selects one relationship explicitly. Eligibility requires
a forward directed Relationship from this root, target multiplicity exactly
`min=max=1`, `toOne`, one ordered target Key, exact ordered original Field
pairs, exact raw binding pairs and exact verified native FK/PK rows. The source
pair Fields must be non-null. Rust owns the aliases, join fragment, owner
qualified field/predicate/order catalogs, output names and global ordinals,
statement template, OIDs, source-derived result codecs and inherited server byte
guard. Standard carriers reuse `weft.query-codec-instructions/1` with nullability
rebound to the native output. UUID, 128-bit bytes, date and UTC timestamptz use
`weft.query-result-codec/1`, with exact native decoder/OID and physical type
metadata; this result-only profile does not imply predicate support. It is issued
only when those source Fields have no unsupported facets or allowed values; an
unsupported refinement refuses that relationship while preserving base reads. A caller sends
only owner-qualified issued IDs and typed values. Root fragments retain the
existing issued semantics; target fragments carry the fixed target alias.

Only the `all` terminal is issued. The statement always emits a hidden int8
ordinal. Its projection slot can be
empty, so an empty selection returns one empty public tuple per matching row
without losing row cardinality. Inner join behavior is only for visible matching
target rows under current policy; it does not guarantee root survival when
policy hides a target. Optional/left, multi-hop, self/alias, joined Page, grouped,
search and action compositions remain unsupported by this slice. The `/8` Page
feedback digest is recomputed over the complete `/8` root with only
`ordinaryPage.continuation.fixed.issuedProgramDigest` omitted; the join descriptor
does not add a feedback digest. This changes `/8` Page descriptor bytes while
preserving its `/7` meaning. No native execution qualification is implied by
issuer tests or schema validation.
