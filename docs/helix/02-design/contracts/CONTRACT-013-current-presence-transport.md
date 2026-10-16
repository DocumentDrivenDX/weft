---
ddx:
  id: CONTRACT-013
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: US-009
      kind: informed_by
    - id: CONTRACT-010
      kind: informed_by
    - id: CONTRACT-012
      kind: informed_by
---

# CONTRACT-013: Conditional tagged presence transport

**Type:** library / candidate data protocol. **Version:** 0.1-candidate.

## Purpose

Compile the original direct-projection plan over a held PostgreSQL materialization
of complete UMF disclosure cells, then compare its complete native result bag
against independently captured eligible rows. Preserve explicit original null,
absence, withholding, transformed null and transformed values as distinct cells.
US-009 and all original132 B-008 cases remain required.

## Scope and Boundaries

This draft defines the optional `weft-postgresql/current-presence-candidate`
feature and separate `current_presence::lower` and `current_presence::check`
functions. Both MUST receive an actual issued CONTRACT-012 context, not imported
JSON. `CurrentPresenceSql` MUST borrow that context and MUST NOT deserialize from
JSON. This increment covers one-scan direct scalar Field projections, repeated
projection positions and unordered row bags. Joins, predicates, groups, order,
LIMIT, aggregates, wildcards, structured selected values and complete entities
remain refused by the original .4 preparation. Other B-008 operators remain
required subsequent work; this subset does not redefine acceptance.

Weft owns source/binding/artifact correspondence, SQL projection order and exact
cell-byte/row multiplicity comparison. UMF owns whole V2 batch/domain/value and
selected-policy-occurrence validation and actual atomic recipe execution. Hosts
MUST establish genuine authenticated source/caller/Subject/current authority;
actual producer invocation with original operation budget; complete eligible
population and independent counts; held native statement/result custody; closing
authority revalidation; guarded release; cancellation and cleanup. Obligations
apply on denied and empty results. Neither declaration hashes nor a matching
transport frame establish these facts.

## Normative Surface

### Materialization binding

The closed UTF8 JSON object, at most4,000,000 bytes, has fields:

| Field | Required meaning |
| --- | --- |
| version | `weft.postgresql.current-presence/0.1-candidate` |
| capabilities | Exactly `["tagged-current-projection"]` |
| sourceRequestSha256 | Exact original request bytes |
| preparationSha256 | Exact issued presence report bytes |
| sourceBindingSha256 | Exact original native-binding source pin |
| relation | Closed schema/table/eligibleColumn/rowKeyColumn identifiers |
| header | Actual V2 batch without rows: version/policy/protection/domains/columns |
| cells | Ordered mappings `{field,column}` corresponding bijectively to header columns |

Digests MUST be64 lowercase hexadecimal characters. Field identities retain
`documentId`, `revision`, `module`, `element`. Header fields retain UMF's exact
`documentId`, `moduleId`, `elementId` reference shape, with revisions supplied by
header domains. Policy/catalog pins MUST equal the held source context. Header
core pins MUST correspond to held model pins; a host MUST additionally validate
UMF's full closure bijection. Columns retain storageEffect, optional
storageTransform and optional explicit Gregorian valueProfile. Unknown members,
unknown versions, duplicate mappings, unused extra cells and ambiguous domain
revisions MUST refuse. No source policy or transform is inferred from column
names, values or matching transform output.

Identifiers MUST be nonempty, NUL-free and at most63 UTF8 bytes. They MUST be
quoted with embedded double quotes doubled. Physical cell columns MUST be unique
and separate from eligibility and private-key columns. Eligibility MUST be native
BOOL; keys MUST be unique nonnull native TEXT, nonempty, NUL-free and at most256
UTF8 bytes. Each cell MUST be nonnull native TEXT containing the complete JSON
cell issued by the validated actual batch. SQL MUST project that text without
JSONB coercion, scalar extraction, COALESCE or SQL NULL collapse. Header/cells
MUST retain actual correspondence to the same held producer batch and current cut.

### Compiled artifact and native result

Artifact version is `weft.postgresql.current-presence.sql/0.1-candidate`.
`authority:false`, `securityReady:false`, `nativeAdmission:required` and the exact
capability MUST be retained. It includes original/preparation/materialization
hashes, SQL, empty parameters, disclosureHeader, resultColumns,
privateResultColumns and mandatory host obligations.

Each public column retains position, original output name, sourceField,
headerColumn, nativeType `text`, encoding
`weft.umf-disclosure-cell-text/0.1-candidate`, and a tagged type descriptor with
originalDomain exactly from its original read requirement. Allowed alternatives
are original/withheld/transformed, plus absent only for absent-allowed original
Fields. Transformed domain is the individual cell's exact outputDomain resolved
against the held header/closure. It MUST NOT be mislabeled as the original scalar
or collapsed into a uniform logicalType. Repeated projections retain distinct
positions and reuse their one header/cell mapping.

The final private result column is `__weft_source_rows`, native TEXT containing
`weft.stage-row-tuple/0.1-candidate`, with exactly one original scan/key tuple.
It is internal custody data and MUST NOT be released as a public result column.

### Independent transport input and check

`check(context, original_binding_json, transport_input_json)` MUST invoke the
actual selected lowerer on the original binding before comparing rows. The closed
input version is `weft.current-presence.transport-input/0.1-candidate` with:
sourceRequestSha256, preparationSha256, materializationBindingSha256,
compiledArtifactSha256, expectedRowCount, header, rows and resultRows.
`expectedRowCount` MUST be canonical unsigned decimal text matching complete
independently captured eligible rows. It MUST NOT be derived from result rows.
Each expected row is exactly `{key,cells}`; cells are complete cell JSON strings
in original header order. Each native result row contains public cell strings in
projection order followed by the private tuple text. Native null refuses.

The checker MUST validate every expected row before comparing result rows,
including an empty result. Keys MUST be unique. Expected and result populations
MUST have exactly equal multiplicity. Unknown/duplicate/missing result keys,
wrong scan identity, width, private version or extra members MUST refuse.
Corresponding public cell strings MUST be byte-identical to independent capture;
JSON-equivalent reserialization is not accepted in this transport profile.
Result row ordering is immaterial; cell order and repeated projection positions
are not. Header MUST exactly match the lowerer's held header.

The input MUST fit16,000,000 UTF8 bytes. Whole UMF V2 admission bounds also apply:
at most256 header columns,4096 expected/result rows and16384 expected cells.
These are independent ceilings, not a guarantee that every combined maximum
fits the original operation budget; the shared UMF producer budget may refuse
earlier.
Success output MUST fit16,000,000 bytes and have version
`weft.current-presence.transport-result/0.1-candidate`, matched:true,
authority:false, securityReady:false, nativeAdmission:required, the four exact
correspondence hashes and transportInputSha256 over original input bytes, plus
mandatory unresolved host obligations. It MUST contain no row keys, values or
counts. A success is transport correspondence only and MUST NOT be imported or
reused as native admission, even when hashes match another operation.

Structural cell checking MUST preserve V2's closed alternatives and refuse
wrong-payload absence/withholding, unknown alternatives, malformed selection or
transform/outputDomain shapes. Whole UMF CoreLiteral/domain/value-profile and
actual selected-occurrence validation remain mandatory before native staging;
structural object shape alone MUST NOT establish valid original/transformed
values. The complete actual producer output, not reconstructed equal values,
MUST supply transform identity, provenance and selected indices.

## Precedence and Compatibility

Old uniform current-value bindings, contexts, registrations and scalar decoders
MUST reject these versions and types. No implicit backend registration, fallback,
public security compile activation or reinterpretation of frozen ABI1 is allowed.
Future Python/WASM classes MUST be separate issued types with exact feature and
profile dispatch. Databricks needs its own dialect lowering/native custody proof;
PostgreSQL evidence MUST NOT qualify it.

## Error Semantics

Wrong binding, context correspondence, header, mapping, unsupported operation,
bounds or native result MUST refuse atomically at the failing stage. A lowering
failure MUST return no SQL artifact. A transport-check failure MUST return no
success report or public rows; it cannot erase a previously issued SQL artifact.
Hosts MUST release nothing and clean up the owned stage on either failure.
Lowering diagnostic is `WFT-CURRENT-PRESENCE-SQL` / capability /
`Tagged current projection lowering refused`. Transport diagnostic is `WFT-CURRENT-PRESENCE-TRANSPORT` / capability /
`Tagged current projection transport refused`. It MUST expose no private
values, keys, policy census or native counts. Hosts MUST release nothing on any
refusal and clean up the complete owned stage under the same held operation.

## Examples

For `SELECT r.token AS first, r.token AS second FROM Resource r`, both public
positions project the same complete TEXT cell and refer to one header column.
`{"disposition":"original","value":null}` is distinct from
`{"disposition":"absent"}`, `{"disposition":"withheld"}` and a transformed
cell with `value:null`. A complete withheld cell retains its eligible row.
A SQL-null scalar in any of those positions refuses; five declared cells do not
establish that an actual policy selected any of them.

## Validation and Unresolved Evidence

Derive independent positive/negative controls for exact source pins, all five
states, repeated projections, empty/denied populations, input budgets, identifier
quoting, old registrations, full whole-UMF V2 validation and weakened transport
barriers. Actual atomic producer snapshots are being integrated, not yet qualified
as native execution. Native PostgreSQL/Databricks authenticated caller/source/cut,
end-to-end guarded release and original132 acceptance remain unfinished.
