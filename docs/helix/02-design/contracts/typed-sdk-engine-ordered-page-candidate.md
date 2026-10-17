---
ddx:
  id: weft.typed-sdk-engine-ordered-page-candidate
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.typed-sdk-raw-collections
      kind: informed_by
---

# Proposed original engine ordered-page candidate

**Review proposal only.** This describes a new owner ABI for UMF WR-A6/S5–S7
and CONTRACT-070/071/072. It does not amend or qualify old authored-Key page
profiles, native engines or the SDK. Temporal qualification remains HELD.

## Proposed framing and source authority

New separate wrapper entrypoints compile/check an original ordered page with
an explicitly selected authored Key and full sort tuple. Proposed wrapper
version is `weft.engine-page/0.2`; digest owner remains the engine-page owner
with projection version`0.2`. Old0.1 checkers and native hosts refuse0.2 until
explicitly adopted. Exact API names/profile versions require owner review.

The closed compile envelope retains original compiler selector, complete original
engine request, pageSize1..999, canonical selectedKey(module,record,key) and
initial after:null. A Related page additionally retains the complete original
twin source/target packet and directed canonical relationship identity. The
source selection keeps its original ordering and explicitLIMIT independently of
target delivery. The selectedKey is explicitly passed, then resolved against
that exact original Core Record/Key; the SDK also checks it equals its original
RecordView.selectedKey. No source identity or Key component is inferred.

SQL AST is the sole source of user ordering. The owner derives effective order,
including explicit direction/defaultNULLSLAST and complete selected-KeyASC
suffix, then validates original Field custody and native supported domain. A
claimed duplicated order inventory cannot redirect the SQL. All requested,
effective and hidden-output metadata participate in the owner digest.

## Proposed original IR and cursor schema

Use a separate original language capability, proposed
`weft-ir/engine-ordered-page/0.1`, constructed privately by Core. Ordinary plans
omit it and retain byte-identical old serialization. Its page descriptor retains:

- exact selected authored Key identity, ordered component identities/types and
  native uniqueness/integrity obligation;
- effective sort slots: original Field identity/source occurrence, carrier/facets,
  ASC/DESC, FIRST/LAST, nullable/missing domain, native comparison profile and
  hidden value/state output positions;
- selected-Key hidden positions independently of sort positions, including when
  a user-ordered Key component is repeated with different direction;
- complete original page request/context, size/lookahead, original source and
  target obligations, native cursor/profile identity and continuation version.

The proposed continuation payload carries selectedKey encoded components and
sortValues in effective order. Each sort value is a closed union:
`{state:"present",value:<original canonical scalar encoding>}`,
`{state:"null"}` or`{state:"missing"}`. Null/missing carry no value. Every scalar
encoding is validated by the original native domain, including exact numeric
facets and finite canonical values; no display-string roundtrip defines order.
The native owner token remains bounded to65,536 bytes and retains the same tuple,
selected Key, original actor/binding/catalog/query/noncursor-argument context.
SDK retains exact bytes and values; original owner independently verifies their
agreement before dispatch. It is inert process-local navigation custody, never
authorization or a cross-page snapshot assertion.

## Exact ordering and continuation meaning

For each slot, null and missing occupy the same rank selected by FIRST/LAST;
their state marker remains explicit. Two nullish values compare equal for prefix
ordering, so changing null to missing does not create a second sort class.
Present comparisons use the slot's native exact scalar comparator. Only the
value comparison reverses for DESC; null placement stays explicit. String order
requires binary Unicode scalar ordering without normalization/case folding;
native collation and exact current declaration/setting custody are obligations.
No session default or arbitrary native operator is admitted by coincidence.

The owner builds a strictly-after lexicographic disjunction over the entire
rank/value tuple, using null-safe prefix equality and the correct next-slot
comparison. It appends the selected complete unique KeyASC unless already an
exact complete terminating suffix. Native selected-Key integrity is checked
before and after execution, including alternate-Key uniqueness. It does not
reuse relationshipKey or assume primaryKey. Key-domain order, including Boolean
components, must have an explicit original native comparison rule qualified by
the owning profile; otherwise that exact domain refuses with a named missing
requirement. Temporal, compound and unqualified collation domains remain closed.

Hidden sort/state/Key disclosure must be authorized by the same original read
path. If authorization cannot expose those values, page compilation/admission
refuses; host recovery by fetching unconstrained properties is forbidden.

## Native owner changes

Truss's current Key-only`query_page_objects` and`page_state.after` do not encode
arbitrary null/missing/direction. A new original accessor/profile must return all
original hidden values/states, check selected alternate-Key integrity, enforce
exact binary/scalar comparisons and bind full cursor context to the current
accepted publication and actor. Native token decoding/rebinding, control guards
and pre/post result release remain Truss-owned. Fixed insensitive export cursor
adoption needs its own original profile/native proof; old export is unchanged.

Ashlar's original compiler emits the rank/value comparisons from admitted IR,
versioned native source selection and shared parameter slots. Its new host
page DTO/decoder consumes every original hidden slot/state and complete Key,
matching exact immutable publication/version vector and current control custody.
Existing Key-only pages remain unchanged. Retention/snapshot loss refuses stale
continuations. Original publication/scalar/relationship obligations survive
Related pages and execute before and after buffered result release.

Both native owners must advertise the exact new interface/language/profile and
full output decoder before the SDK enrolls it. No SDK SQL construction, Key
substitution, arbitrary wrapper cast or old native-token reuse is permitted.
Schema and current authority drift, denied hidden data, wrong state/type/arity,
foreign source/key, changed effective order and cursor/native disagreement refuse
before data dispatch. Existing native resource limits remain explicit; API page
size does not imply bounded total source processing.

## Qualification gates

Require source/default/crossprofile negatives; exact installed five-platform
wheel API controls; actual Chromium full native-program parity; independent
native ordering oracles and unchanged-source traversal for mixed ASC/DESC,
FIRST/LAST, null/missing, duplicate user values, Unicode, exact numeric extrema,
composite alternate Keys, omitted hidden projections, Related target sets,
empty/maximum pages, sync/async and stream cleanup. Native drift, permissions,
publication/actor/context/cursor tampering and stale snapshot controls are
mandatory. No source or browser pass alone qualifies native support.
