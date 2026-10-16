---
ddx:
  id: CONTRACT-017
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: US-009
      kind: informed_by
    - id: CONTRACT-012
      kind: informed_by
    - id: CONTRACT-015
      kind: informed_by
---

# CONTRACT-017: Original association and endpoint requirements

**Type:** proposed library / candidate data protocol. **Version:** 0.1-candidate.
**State:** frozen interface candidate for review and isolated Core implementation; no registration or native admission.

## Purpose

Resolve original association, relationship, endpoint and witness identities from
an actual issued presence context. Native owners join these requirements to
actual accepted/native evidence without adding a Python UMF, Key, domain or
policy interpreter. Original132 B-008 and shared158 obligations remain required;
this component does not close or narrow them.

## Scope and Boundaries

The proposed Core feature is `current-presence-graph-requirements-candidate`.
It MUST explicitly enable the model-requirements feature. Earlier/default
features MUST NOT expose the new module or change their refusals. Runtime,
Python, WASM and native registrations require separate explicit wiring and
qualification. Existing SQL graph traversal guards MUST NOT be relaxed.

This surface emits original metadata requirements, not graph instances, inferred
relationships, a query result, policy evaluation or a native permit. Original UMF
whole-source semantics, backend support, complete graph occurrence populations,
context/codec correspondence and held authority remain required. Retaining
metadata for a form does not qualify its logical/native execution.

Canonical UMF0.8 source custody is the upstream schema whose SHA256 is
`79e72f2ebef434c7d1e09d372c74d330b24109bb8af9778745a8d41275acee6e`.
Its `relationshipTarget` requires module, element and key. Its source endpoint
requires module and element, and defines no source Key. Missing/null authored
target Keys MUST refuse graph requirement resolution; a target Key default MUST
NOT be invented. Key existence and meaning require original compiler/UMF
resolution in addition to schema shape.

## Normative Surface

Proposed `current_presence_graph_requirements::derive(&CurrentPresencePreparation)`
MUST return a distinct `CurrentPresenceGraphRequirements` borrowing that same
actual context. Its fields MUST be private; no constructor, Deserialize, report
importer, copy-based issuance or callback escape is exposed. It MUST own the
actual same-context typed `CurrentPresenceModelRequirements` value;
`model_requirements()` returns a borrow of that value. `context()` MUST delegate
to that actual value and return the same context object; `json()` returns the
issued data report. It MUST NOT
outlive the actual context or its original parent.

The factory MUST derive CONTRACT015 requirements from that same actual context,
not accept a caller-supplied requirements report or reconstruct a replacement
parent. All ontology associations MUST be accounted for in original order,
including associations unused by the selected SQL projection. Each selection
MUST retain its nominal Record or Relationship namespace and full original
model revision/pins. Query names and equal numeric/native IDs are not identity.

### Report

The proposed closed report has these required fields:

| Field | Required meaning |
| --- | --- |
| `version` | `weft.current-presence.graph-requirements/0.1-candidate` |
| `scope` | `original-association-endpoint-requirements-only` |
| `complete`, `authority`, `securityReady` | Always false |
| `nativeAdmission` | Always `required` |
| `sourceRequestJson`, `sourceRequestSha256` | Exact complete actual original request UTF8 string and hash |
| `preparationSha256` | Hash of the actual context report |
| `modelRequirementsSha256` | Hash of actual same-context CONTRACT015 report |
| `modelPins` | Complete original catalog pins in original order |
| `associations` | Original ordered selection entries described below |
| `unresolved` | Original UMF admission, graph interpretation/support, native homes/codecs/populations, context, held authority and guarded release |

Original model and opaque-source bytes MUST remain in the complete request.
Parsed metadata MUST NOT replace them. Unknown annotation content and exact
numeric/Unicode source content MUST survive. Unknown annotation presence alone
MUST NOT be labeled a semantic failure. The original owner verdict distinguishes
annotations from required unsupported operational meaning; whole-source/native
admission remains unresolved here. No metadata may be silently dropped to obtain
admission, and no older closure guard is relaxed.

Each Key requirement contains original containing Record Identity, authored
`id` (the logical Key ID, never a native number) and ordered complete Field Identities. These MUST come from existing
original compiler Key resolution, with no native number/name equivalence.
Member descriptors MUST retain separate requested Field, nested descriptor and
containing Record identities. Logical availability MUST NOT come from native
column nullability.

### Record-member associations

Each entry MUST retain the nominal original Record association reference,
complete association Record Identity, original selected Key and original
selection metadata. Each endpoint MUST retain its literal role, target Record,
that entity's selected Key and the ordered association-member Field identities
and original descriptors. The containing association Record MUST actually own
those member Fields. No endpoint may be rebound by equal names or field values.

Original component order MUST survive. Domain compatibility and endpoint
semantics remain original UMF/compiler obligations; the Python native join MUST
NOT invent casts, reorder components, infer missing Fields or implement a second
domain interpreter.

### Core relationships

Each entry MUST retain the nominal `SecurityRelationshipRef` plus its original
model revision, original selection metadata and complete authored relationship
metadata. Relationship and property-owner Record domains MUST remain distinct.

All authored source and target declarations MUST survive in original order with
their original ordinals and qualified resolved Record identities. A source has
no authored Key requirement. Every target MUST retain its explicit authored Key
and ordered original Key Fields. Multiple branches, direction, multiplicities,
lifecycle, inverse and associationRecord MUST NOT be silently converted into a
single directed traversal or a query helper's defaults.

Each ontology-selected endpoint MUST retain its role, authored side, selected
Record and ontology-selected Key separately from the authored declaration. Its
matching declaration ordinals MUST account for all matches without collapsing
authored occurrences. No match refuses. For a target, the selected Key MUST
correspond to the explicit authored target Key. For a source, its selected Key
comes from the original ontology/entity selection, never a default source Key.
Ambiguous or unsupported original semantics MUST refuse subsequent admission;
metadata retention cannot select one branch or change direction implicitly.

The witness discriminator MUST retain the original `opaque-existential` or
`record-key` meaning. A record-key witness MUST retain its complete owner Record
and selected ordered Key and correspond to the original associationRecord.
An opaque witness MUST NOT stand in for an authored association Record. No edge
ID, relationship type ID, owner type ID or Key byte may be equated across domains.

This report contains no native edge instances. Later capture MUST independently
retain actual occurrence/version/source/target provenance, including parallel
edges. A declaration inventory cannot prove population completeness, empty-set
coverage or existential truth.

### Bounds and construction order

All inherited source/parser/model/descriptor limits remain. Proposed association
inventory is bounded to4096 selections and4096 declarations per side; reports
MUST fit16,000,000 encoded UTF8 bytes. Overflow MUST refuse the whole report,
never truncate or omit dependencies. These are proposed component limits, not
whole-process memory or host capacity guarantees. Their positive/first-failing
controls MUST be independently reviewed before implementation qualification.

The implementation MUST retain the actual already-resolved ModelRequirements
map only under the graph feature. It MUST NOT reparse the issued16M ModelQ
through an input parser with a smaller limit. Model-only/default feature
footprint and the existing ModelQ wire bytes remain unchanged.

Logical encoded-payload accounting has separate16M limits for cached original
Keys, cumulative retained declaration/member/matching-ordinal data, and complete
entries plus original-source/pin payload. Charge before cache/vector retention;
borrow large raw metadata instead of cloning it per occurrence. Final bounded
serialization is independently16M. These are logical accounting phases, not a
heap/process bound. Every charged item is an emitted payload component; no
shared cache/entry double-charge narrows a report that fits the encoded limit.

Produced-report schema validation uses an explicit issued-output profile of
16Mbytes, wrapper depth256 and a value-node ceiling derived from encoded bytes
(at most16Mvalues). It accepts no external report and changes no input scanner
or native decoder guard. Original inputs retain their existing limits.

A persistent binding descriptor precedes the context. This report's hash MUST
NOT be fed back into that same descriptor. A separate actual held-operation
witness MAY bind descriptor and requirements hashes after original preparation;
it MUST retain actual operation/context custody rather than import metadata as
permission. Placeholder preparation/re-preparation cannot establish continuity.

## Precedence and Compatibility

CONTRACT012 owns actual presence context issuance. CONTRACT015 owns original
Record/member/ordered-Key requirements. This proposal changes neither report's
version/meaning nor any existing closure or backend registration. A later graph
native join must qualify its exact source/codec/occurrence/witness protocol;
metadata possession never activates it. All required native and shared cases
remain open until their own actual end-to-end evidence passes.

## Exact entry wire shape

Every association contains required `ordinal` (its zero-based source-array
position), `kind`, `reference` (the existing nominal SecurityAssociationRef
shape), `revision` and `selection` (complete parsed original selector metadata).
Record references use elementId; relationship references use relationshipId.
No reference is converted into the other namespace. Metadata views preserve
parsed values; exact original UTF8 remains in sourceRequestJson.

A `record-members` entry additionally contains exactly `owner`, `key` and
`endpoints`. Each endpoint contains exactly `ordinal`, `role`, `target`, `key`,
`members`. Each member contains exactly `field`, `descriptors` in original
component order; descriptors delegate unchanged to CONTRACT015. Endpoint count
is1..256; component count is1..256, matching the selected ontology shape.

A `core-relationship` entry additionally contains exactly `authored`, `sources`,
`targets`, `associationRecord`, `endpoints`, `witness`. Each source declaration
contains exactly `ordinal`, `record`, `metadata`; each target adds exactly
`key`. declaration metadata retains its complete parsed original object.
associationRecord is a complete Identity or null for an absent declaration;
an explicit invalid null in the authored declaration is not absence.
Each selected endpoint contains exactly `ordinal`, `role`, `side`, `target`,
`key`, `declarationOrdinals`. Exactly two endpoints must identify source and
target once each; matching declaration ordinals preserve every match, including
repeated authored declarations. Opaque witness contains exactly `kind`;
record-key witness contains exactly `kind`, `record`, `key`.

Every Key object contains exactly `record`, `id`, `fields`, using complete
compiler Identities and original component order. All other typed objects are
closed. Complete raw selection/authored/declaration metadata objects retain
unknown content. No parsed metadata view replaces original byte custody.

Associations and declared branch occurrences MUST NOT be deduplicated. Native
semantic admission remains the original owner's responsibility even when a
metadata inventory retains duplicate or otherwise semantically invalid inputs.
Known mandatory structural mismatches that prevent correspondence refuse the
whole factory. No domain cast/comparator is inferred by this report.

## Error Semantics

Proposed structural failures use `WFT-CURRENT-PRESENCE-GRAPH-REQUIREMENTS` /
`Original graph requirements refused`. Existing original compiler refusal codes
remain for original model/Key/domain resolution. Bad namespace/revision, missing
or ambiguous declaration, foreign membership, missing target Key, mismatched
selection or witness and budget failures MUST NOT
return a partial inventory or choose an older registration/default. No SQL,
external effects, diagnostics containing private facts or automatic retry occur.
Original-owner semantic/required-meaning refusal remains an independent admission
obligation; this data factory MUST NOT invent that verdict from annotation presence.

## Examples

For `AssignmentProject`, preserve Relationship identity independently from an
`Assignment` owner Record. A source selection using `Assignment.pk` is ontology
selected; a target declaration `Project.key=pk` is authored and mandatory.
The target endpoint also selects `Project.pk`; keep both evidence identities,
verify correspondence and preserve component order. Equal native IDs do not
establish any of these relationships. Removing the authored target `key` is an
invalid canonical0.8 declaration, not a request for a default.

A record-key graph witness with an association Record remains different from
an opaque existential edge. Two equal endpoint pairs may still be two distinct
native occurrences; this metadata report cannot collapse or enumerate them.

## Refusal precedence and validation

The factory first derives the actual typed ModelRequirements; its existing
component refusal codes/limits remain unchanged and do not imply a whole-source
owner semantic verdict. Next it parses the actual ontology and checks count/
required structural shape. It visits associations in original order, resolves
nominal reference/revision and original compiler Keys/membership, then endpoint
side/match and witness correspondence. Cached typed-Key and retained-entry
payload accounting occurs before retention; final bounded encoding is last.
No partial inventory returns on any refusal. Unknown raw annotations are retained,
not classified by an invented semantic interpreter.

Freeze the accompanying closed report schema and independently review exact
entry fields, declaration occurrence projection, required structural refusal
precedence and capacity controls. Isolated implementation is authorized by the
ongoing B-008 handoff; no public registration or native activation follows from
that implementation. Actual original UMF interpretation and native correspondence are separate
required checks; schema shape or a data report alone is insufficient.

## Explicit runtime and Python candidate bridge

The separately selected runtime/Python feature
`current-presence-graph-requirements-candidate` MUST enable that layer's
model-requirements feature and the lower layer's graph-requirements feature.
Default, old presence and model-only selections MUST NOT expose
`graph_requirements_json()`.

The existing issued runtime `CurrentPresencePreparation` MAY expose that
zero-argument method only under the graph feature. It MUST use its retained
immutable original Arc, re-derive only the scoped Core borrow, compare the
issued context report and invoke the actual Core graph factory. Returning its
exact JSON bytes is data production, not capability import or native admission.
The existing frozen Python class MUST check its actual original parent before
calling the runtime method. No new constructor, caller-supplied report, public
callback, Python interpretation or replacement parent is permitted. Wrong
receivers, extra arguments, copying, pickling and parent mutation MUST refuse
through the existing type/custody boundary without coercion callbacks.

CONTRACT-014 and CONTRACT-016 custody/error rules continue to apply. A returned
Graph string MUST NOT replace the actual issued preparation in a native
operation, qualify a closed operation or become part of the same persistent
binding whose hash contributes to that preparation. Equal report bytes from a
second original parent do not equate those parents. Host/native owners remain
responsible for held-operation correspondence, admission and guarded release.
Existing release, wheel defaults and consumer pins MUST remain unchanged.
