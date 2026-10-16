---
ddx:
  id: CONTRACT-015
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
    - id: CONTRACT-014
      kind: informed_by
---

# CONTRACT-015: Original model requirements for native correspondence

**Type:** library / candidate data protocol. **Version:** 0.1-candidate.

## Purpose

Issue original Record, Field-membership and ordered selected-Key requirements
from the actual current-presence preparation. A native owner can join these
requirements to its accepted catalog and observed storage without implementing a
second UMF model or key interpreter. This report is incomplete binding data.
All original132 B-008 shared/native requirements remain required.

## Scope and Boundaries

The separate Core feature `current-presence-model-requirements-candidate` MUST
enable `current-presence-context-candidate`. The old context feature alone MUST
NOT expose this module. Default compilation, published Python wheels, runtime,
WASM, backend registrations and existing consumer computation pins are unchanged.

The helper owns compiler model resolution only. It MUST NOT choose a physical
property home, native ID/type, key codec, comparison meaning, graph direction,
endpoint default, authority generation or disclosure disposition. The original
UMF semantic program and the host's accepted-source/native/current-authority
verification remain required. Unimplemented Truss row homes MUST NOT become JSON
fallbacks or empty populations through this report.

## Normative Surface

`current_presence_model_requirements::derive(&CurrentPresencePreparation)` MUST
return `CurrentPresenceModelRequirements` borrowing that same actual context.
The issued object has private fields, no public constructor or Deserialize,
and only `context()` and `json()` accessors. `context()` MUST return the same
context object. It MUST NOT outlive that context or its original parent.
JSON equality, a copied report or equal source bytes MUST NOT issue this type.

The factory MUST resolve every ontology `entities` entry by exact original
Record identity and selected model revision. Each declared Field reference MUST
be distinct, owned by that Record and present in its exact original members set;
missing, extra or foreign members refuse. Original compiler member descriptors
MUST provide availability and value-shape requirements; native NOT NULL flags
MUST NOT replace availability. Original compiler authored-Key validation MUST
resolve the explicit `keyId` and preserve its component order. Unsupported
required model/key meaning refuses rather than being approximated.

Record-member associations MUST refer to an entity in this report and use its
same explicit selected `keyId`. Other graph forms MUST remain retained original
source obligations. This version MUST NOT claim graph closure or infer a
relationship's declared target Key from an ontology-selected endpoint Key.

The closed emitted report has exactly these fields:

| Field | Meaning |
| --- | --- |
| `version` | `weft.current-presence.model-requirements/0.1-candidate` |
| `scope` | `original-record-field-key-requirements-only` |
| `complete`, `authority`, `securityReady` | Always false |
| `nativeAdmission` | Always `required` |
| `sourceRequestJson` | Complete exact original request string, including opaque sources and all authored module strings |
| `sourceRequestSha256` | SHA256 of that exact UTF8 string |
| `preparationSha256` | SHA256 of the actual context's exact report UTF8 |
| `modelPins` | Original catalog pins, in original order |
| `ontologySha256` | SHA256 of the original opaque ontology UTF8 |
| `records` | One entry per original entity, sorted by document/module/element reference |
| `unresolved` | Required outstanding semantic, graph, context, physical/codec, native witness and closing-release obligations |

A record entry contains `record` (complete compiler Identity with documentId,
revision, module, element), `fields` (ontology order), and `selectedKey`.
A field entry contains `field` (complete Identity) and `descriptors` (the original
compiler's descriptor graph, in its original traversal order). A selected Key
contains `id` and ordered complete `fields` identities. Key field order MUST NOT
be replaced by set order, native attnum order or sorted Field names.

Entities MUST number 1..4096 and associations 0..4096, subject to all earlier
original request and parser guards. Encoded report UTF8 MUST fit16,000,000 bytes;
capacity failures refuse the entire report. No guard is raised by this contract.
Unknown content remains in complete original request strings; parsed descriptor
summaries MUST NOT replace those originals. They establish no whole-source
semantic acceptance or native permission.

The actual context already includes its selected opaque native-binding source.
A persistent descriptor MUST NOT incorporate this complete report's hash or
source-request/preparation hash into that same binding: this would create a
self-hash cycle. Persistent correspondence MUST use independent original model
and accepted/native source pins. The context-bound report MAY validate/join that
selected descriptor during an actual operation; a fresh held witness can bind
its context-specific hashes. A provisional binding and later re-preparation
MUST NOT be treated as one original admitted context or continuous custody.

## Precedence and Compatibility

CONTRACT-012 governs issuance of the actual parent context. This new factory
MUST NOT change old registration acceptance, claim that a report is a native
binding, or activate the current disclosure consumer. It MUST NOT expose the old
uniform-scalar current-query type. A later complete mapping/witness protocol
requires separate versioning and conformance evidence; this version's false
completion/authority fields MUST NOT be reinterpreted as admission.

## Error Semantics

Missing or ambiguous originals, wrong Record membership, duplicate entities or
members, incomplete member census, invalid selected Keys, mismatched
record-member association Keys and capacity overflow MUST refuse the entire
operation without a partial report. Structural failures use
`WFT-CURRENT-PRESENCE-MODEL-REQUIREMENTS` / `Original model requirements refused`;
existing original compiler model/key failures retain their own diagnostics.
No failure permits fallback to guessed IDs, graph defaults, JSON storage or an
older registration. This pure factory executes no SQL and schedules no retry.

## Examples

`let requirements = current_presence_model_requirements::derive(&context)?;`
issues data from the actual context. `std::ptr::eq(requirements.context(),
&context)` MUST hold. A Resource Record whose selected Key is `[tenant, id]`
MUST retain that order even if the native columns appear `[id, tenant]`.
A token descriptor with `availability: absent-allowed` MUST retain that meaning.
A graph declaration without an authored target Key remains in original source;
this report does not manufacture an authored Key or admit that graph.

Construction order is explicit:

1. Retain descriptor `D` from independently verified accepted/model/native inputs.
   `D` has no hash of the not-yet-existing preparation or requirements report.
2. Build original request `R` containing exact opaque binding bytes `D` and its
   hash. Prepare `R` once with the actual compiler, then issue context `C`.
3. Derive requirements `Q` from that actual `C`. Join its original model
   identities to `D` without rewriting `R`, `D` or `C`.
4. Under the actual held host operation, a separate witness can bind hashes of
   both `D` and `Q`. That witness and its native/authority checks remain required.

A control changes an independently authored ontology extension while retaining
the same supplied data-fixture binding bytes. The two actual contexts have
different source/preparation hashes; their shared binding stays byte-identical,
and each requirements object borrows its own context. The fixture is explicitly
unadmitted and does not demonstrate step4 or native descriptor issuance.

## Validation and Unresolved Evidence

Run actual Core context and helper tests for complete source retention, pointer
identity, exact hashes/model pins, all Record members, optional availability,
ordered composite Keys, missing/foreign/duplicate members and association Key
mismatch. Compile-fail tests MUST reject JSON import and escaped context
lifetimes; feature-disabled builds MUST omit the module. Native inventory,
Field/property/home/codec joins, relationship/endpoint correspondence, whole-UMF
semantic admission, genuine authenticated authority and guarded release require
their own end-to-end evidence. This component promotes no acceptance case.
