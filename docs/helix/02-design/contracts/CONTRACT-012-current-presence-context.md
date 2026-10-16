---
ddx:
  id: CONTRACT-012
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
    - id: CONTRACT-007
      kind: informed_by
---

# CONTRACT-012: Separate current-presence source context

**Type:** library / candidate data protocol. **Version:** 0.1-candidate.

## Purpose

Bind an actual immutable .4 projection-lineage preparation to explicitly selected
source pins and a required canonical disclosure-selection producer. The context
is a compiler data object. It authenticates no caller, source, cut or disposition.
US-009 and every original132 shared/native case remain required.

## Scope and Boundaries

The optional Core feature is `current-presence-context-candidate`. It explicitly
enables `tagged-current-projection-candidate`; the latter alone MUST NOT expose
the new context. The initial subset inherits CONTRACT-010's .4 one-scan direct
scalar projections and admits only original UMF0.8 model pins. No value operators,
SQL lowering, native result codec, Python/WASM registration or database execution
is provided by this increment. Those downstream requirements remain unfinished.

UMF owns policy evaluation and the canonical winning source occurrences. The
required producer is the private candidate API
`evaluateCandidateCatalogReadDisclosureBatchWithinOperation`, returning protocol
`umf.security.logical-disclosure-batch/0.1-candidate`. This atomic producer contract is a planned requirement while its implementation
and review are in progress. The earlier selection-only protocol remains separate
and MUST NOT be relabeled as a rendered batch. No lowering may use this context
until the exact producer is implemented, reviewed and qualified for its declared
logical subset. A producer declaration is not a verified invocation; `complete:false` and `authority:false` remain data
qualifications. Hosts MUST establish actual producer/source/caller/current-cut
and final release custody independently, including empty results.

## Normative Surface

`current_presence::prepare(&ApplicationLineage)` returns a distinct borrowed
`CurrentPresencePreparation`. The actual parent MUST have .4 typed IR and require
`tagged-current-projection`. The new type MUST NOT expose an old
`CurrentQueryDesignations` or `CurrentQueryOperands`, be deserializable from JSON,
or outlive its original parent.

The exact original request MUST contain one `presence-designation` opaque source
with JSON media type. Its UTF8, identity and SHA256 MUST match the supplied source
and the closed
[profile schema](current-presence-profile-v0.1-candidate.schema.json).

| Profile field | Required meaning |
| --- | --- |
| `version` | `weft.current-presence.designation-profile/0.1-candidate` |
| `id`, `revision` | Exact opaque-source identity; nonempty, NUL-free, at most4096 UTF8 bytes |
| `modelPins` | Exact complete original model pins in original order |
| `sources` | Exactly one pin each for policy, ontology, catalog and owner; exact identities and raw UTF8 SHA256 |
| `binding` | Explicit backend ID/version/target profile and exact native-binding source identity/SHA256; identifiers obey the same text bounds |
| `targets` | Exactly the original selected Record reference |
| `primaryAction` | Explicit ontology action whose declared purposes include read |
| `capabilities` | Exactly `["tagged-current-projection"]` |
| `producer` | Exactly the protocol and entrypoint above |

The selected native-binding source MUST occur once, be JSON object data and match
its exact pin. This validates source correspondence, not physical binding meaning
or native acceptance. All four policy/ontology/catalog/owner roles MUST occur
once and match their pins. Unrelated opaque sources remain in the original
request; no role is selected by fallback. Unknown profile members refuse.
The action-purpose check inherits the existing bounded ontology0.2 subset; it
does not establish whole-source UMF semantic admission or grant the action.

`preparation()`, `json()`, `primary_action()`, `binding_scope()` and
`producer_protocol()` expose immutable original source context or data. The
[preparation schema](current-presence-preparation-v0.1-candidate.schema.json)
defines a closed report with exact source/profile/lineage hashes, original model
pins and read requirements, explicit producer and action, and mandatory unresolved
obligations. `authority` and `securityReady` MUST be false; `nativeAdmission` MUST
be `required`. No cells or authenticated caller/cut are supplied by this API.

`native_request_json()` returns the closed
[native-input schema](current-presence-native-input-v0.1-candidate.schema.json):
exactly the preparation fields with version
`weft.current-presence.native-input/0.1-candidate`, plus `preparationSha256` over
the actual preparation JSON bytes and `sourceRequestJson` containing the complete
original request UTF8. Reports and native inputs MUST serialize within16,000,000
UTF8 bytes using the bounded writer; failure returns no partial output.

## Precedence and Compatibility

Existing current-query designation .1/.2/.3 versions MUST remain closed to the
.4 parent. The separate presence profile MUST NOT be inferred from older uniform
profile booleans or package versions. Old scalar lowerers cannot consume the new
Rust type. Future Python/WASM registrations MUST use separate classes/brands and
reject cross-profile objects, old versions and feature-disabled calls before
binding or SQL handling. No success JSON may be imported as an issued context.
There is no native result ABI allocation or automatic backend registration here.

## Error Semantics

Wrong parent/version, capability, producer, purpose, source pin, duplicate role,
target, unknown member or size refuses atomically with
`WFT-CURRENT-PRESENCE-CONTEXT` / model / `Source-bound presence preparation refused`.
The caller MUST correct its explicitly authored inputs, not coerce the original
model, reuse an old registration or treat a report as authority. Existing parent
lineage preparation errors retain their original diagnostics.

## Examples

A .4 `SELECT r.token FROM Resource r` lineage plus a separately authored
presence-designation profile may produce source-bound data retaining Token's
absent-allowed descriptor. The fixture's existing policy does not establish Token
disclosure permission. A test-only unadmitted native-binding object can exercise
pin correspondence, but MUST NOT be described as an accepted Truss binding.
Calling the old current-query designation constructor on that same parent still
refuses; neither constructor produces SQL in this profile.

## Non-Normative Notes

Canonical owner-fold/source occurrences and per-row values remain separate from
representable alternatives. Native producer invocation, transform computation,
crypto/key custody, complete eligible populations, current native authority,
materialization/result correspondence and guarded release remain downstream
obligations under the full B-008 acceptance scope.
