---
ddx:
  id: TD-003
  type: technical-design
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: ADR-001
      kind: informed_by
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# TD-003: truss-postgresql

**User Story:** [[US-003-truss-postgresql]]. **Feature:** FEAT-002.
**Parent Solution:** [[SD-002]].

## Technical Approach

Inherit SD-002's separation of source meaning and physical lowering. Realize
US-003-AC1–AC4 through the shared pure Rust core and its registered backend/host
boundaries. Contracts 001–003 own exact interfaces and failure semantics.

## Component Changes

Target `crates/weft-core/`, backend integration packages and thin
`crates/weft-python/` / `crates/weft-wasm/` wrappers according to story scope.
Use one resolved plan, explicit capability evidence and exact value carriers.
Before implementation, pin dependency versions and finalize the IR/trait schema
listed in CONTRACT-001/002. Do not expand this story to invent database layouts.

## Interfaces and Security

Apply CONTRACT-003 input/pin/limit guards, CONTRACT-001 meaning and CONTRACT-002
plugin registration. Core has no I/O; plugins are trusted host code. No credentials,
policy decisions or untrusted SQL fragments enter the model-driven frontend.

## Testing

STP-003 names planned failing tests and assertions for every AC. Native profiles,
Python/browser behavior and independent expected values must receive evidence
at their actual layer; mocks/generated SQL never replace native semantic checks.

## Migration, Rollback and Implementation Sequence

No database migrations. Add fixtures/refusal tests first, implement the smallest
story components, execute the mapped evidence, then review conformance. A rejected
new profile/version rolls back by unregistering it; retained models are untouched.
Keep source-language/IR/backend versions separate and refuse stale caches.

## B-005 storage-owner reconciliation (2026-10-05)

The owner checkout `/Users/erik/Projects/truss` still contains the earlier
partitioned-layout decision; its active design worktree
`/private/tmp/claude-501/truss-spec-wt` proposes the flat `truss-layout 0.2`
CONTRACT-001 and checked DDL. These are different physical profiles. B-005 must
pin exact owner source hashes and version/status; it cannot silently use a spike's names
or a comment/header version as evidence of inventory compatibility.

The owner architecture assigns Rust lowering/emission to Weft and storage-profile,
catalog export, domain verification and execution to Truss. Its design queue D-04,
D-05 and D-08 still gates document identity, exact codecs and the versioned mapping.
The following are binding requirements proposed for joint review, not approved
Truss storage changes:

- Map every selected owning document/revision/module/element identity explicitly
  to catalog type/property/relationship IDs. Pin the complete supplied model bundle,
  catalog revision, layout version, checked-DDL hash and exporter version. The
  current `(module,element)` uniqueness cannot represent arbitrary independent
  documents sharing names. A restrictive profile must reject collisions explicitly;
  a document-qualified layout requires a separately versioned owner change.
- Include deployment namespace as a validated PostgreSQL identifier; fixed table
  inventory comes from the profile, never user-authored SQL fragments. Catalog
  discriminator/property IDs are typed prepared parameters, not interpolated values.
  A property's home and encoding are explicit. `home=row` has no complete access
  table in the supplied checked layout, so it refuses until an owner contract
  provides one; it cannot fall back to `object.props`.
- Map logical business key components through their property homes and authored
  scalar types. `object.id` is only storage identity. `object_key.k` is canonical
  identity text; comparing it lexically is wrong for integer/decimal pagination
  and composite JSON-array keys. Cursor comparison and ORDER BY must use the
  individual typed components in authored order.
- Required scalar JSON values need explicit stored-domain obligations, numeric
  casts with exact-or-error behavior and text result carriers. C-collated text
  requires UTF8/version/native evidence and retains trailing spaces. PostgreSQL
  numeric limits must be recorded; mathematical SUM is not silently unlimited.
- Whole entity/structured/sequence/map values require recursive type-directed
  exact numeric string encoding before host JSON decoding. Return a typed presence
  envelope distinguishing missing, explicit native null, empty list and present
  values. Native null needs its own assessed capability and recognized binding
  semantics. A raw JSONB-to-host JSON-number path is forbidden.
- Relationship mappings retain authored source/target type IDs, relationship ID,
  target key and complete ordered key-property mappings; inverse traversal swaps
  access direction, not authored cardinality/lifecycle. EXISTS preserves entity
  multiplicity. RELATED bounded key lists need deterministic authored-key order,
  not storage-ID order. Unique-edge storage does not license a universal frontend
  duplicate-edge assumption; qualification identifies the actual stored subset.
- Hosts establish effective role and snapshot/catalog context, validate mapping
  pins and accept exact transport/runtime-domain obligations before executing SQL.
  Separate page transactions do not freeze data; the application cursor cannot
  claim snapshot continuity. Truss CONTRACT-007 owns execution enforcement.

`tests/truss-postgresql/native-boundaries.sql` supplies independent engine probes
before lowering code. No Truss adapter/native support is claimed until these and
STP-003's full model/corpus/context gates execute against the approved profile.

## Candidate implementation authorization

The owner explicitly directed continued development against the versioned
candidate and confirmed non-JSONB property access is a required compiler benefit.
Production binding adoption is a qualification gate, not a prerequisite for
candidate code or fixtures. Implement props and typed row-store access through
explicit mapping/codec assessment; exercise the same resolved queries across
both homes without changing logical UMF identities. Keep incomplete guarantees
candidate or unsupported, never silently advertise the draft as production.

## UMF storage-realization boundary (2026-10-07)

UMF supplies schema meaning. The Truss backend consumes the documented UMF physical layout and logical-to-physical mapping, including catalog identifiers, property homes, value encodings, presence and join paths. Weft derives logical result descriptors and emits SQL/parameters/decoding instructions. Execution and authorization remain host responsibilities. Truss's runtime or decoder implementation is not a dependency for Weft's compiler; exact draft layout pins and native fixtures suffice for candidate implementation. Production claims require separate evidence.

For required non-null native scalar roots, lower string to text_value, boolean to boolean_value text, and integer/decimal result carriers to original numeric_token. Do not substitute numeric_value's rendered spelling for exact token custody. Retain owner-wide structural/payload checks and exact original codec-byte correspondence separately from logical query filters. Full source/domain correctness is a stored-data/host obligation; compiler tests independently arrange conforming and corrupt storage. Optional/null/recursive roots require their documented representation paths and remain required work, not JSONB fallback. Current physical source pins are recorded in tests/truss-postgresql/upstream/storage-realization-pins.json.
