---
ddx:
  id: weft.typed-sdk-engine-order-pages-plan
  type: implementation-plan
  activity: build
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.implementation
      kind: informed_by
    - id: weft.typed-sdk-raw-collections
      kind: informed_by
---

# Proposed original engine ordered-page increment

This is a review proposal, not an implemented profile or a support claim.
It implements the remaining WR-A6/S5–S7 meaning from UMF CONTRACT-070/071/072.
CONTRACT-069's existing authored-primary-Key page profile remains unchanged.
CONTRACT-075 temporal qualification stays HELD. The installed90-case proof is
pinned separately to3f640567bc34b0ca57d871729b485c685f54dc6c.

## Current boundary

Core `resolve_page_inner` chooses the sole authored primary Key, replaces caller
ordering with KeyASC, appends only hidden Key outputs and accepts only a Key
continuation. `application_ir::Plan.order` has Fields without direction/null
placement. Exact redundant complete-primary-KeyASC normalization is qualified;
it does not implement general selected-Key or mixed ordering. Truss's owner confirms its current native accessor only decodes the required
Key vector: page state/token has no null/missing/direction inventory. Its native
accessor and cursor admission must change for general ordering. Ashlar's compiler and host both need additive
cursor metadata; the original native storage accessors already exist.

## Proposed owner contract, pending review

A distinct closed compiler/page profile must retain the complete original engine
request plus explicit canonical selected-Key identity. The SDK compares that
identity with the original generated RecordView selection; Core independently
resolves the exact authored Key and component order. No primary-Key inference or
relationship-Key substitution occurs. Before implementation, record the exact
shared request/IR/owner decoder ABI in a separate draft contract and review it
with the original engine owners.

The original SQL AST owns user ordering. Core resolves every ordered Field and
retains ASC/DESC and explicit FIRST/LAST (defaultLAST). It appends complete
selected-KeyASC unless that complete Key already terminates the order. Only
original admitted non-temporal scalar domains enter the profile; native target
capability admission determines supported families. Unestablished temporal,
collation, compound and scalar domains refuse rather than coerce.

Each sort slot carries its original Field identity, original carrier/domain,
direction, null placement, hidden output position and presence-state position.
Cursor values preserve present/null/missing explicitly; missing and null share
the S5 rank. The selected-Key DTO remains distinct from the complete sort tuple.
Every emitted hidden value must be authorized by the original engine path.
Duplicate ordering slots are retained or refused by an explicit owner rule;
metadata and nativeSQL cannot disagree through host normalization.

Core builds a lexicographic strictly-after predicate: each branch uses equality
of all preceding sort slots and the direction/null-rank comparison of its next
slot. Null/missing equality is one ordering class; present comparisons preserve
exact numeric equality/order and binary Unicode String order. Boolean, when
admitted as an ordered Key by the native owner, needs an explicit exact order
profile; S4 comparison support alone grants none. The final complete selected
Key establishes uniqueness. A single tuple`>` is insufficient for mixed order.

## Original native custody

Truss needs a reviewed native page profile/accessor that admits the chosen Key,
all hidden sort/presence outputs and their original type domains, current actor,
accepted binding/catalog context and exact cursor agreement before userSQL.
The native token must retain the full effective tuple and complete request
context. The existing fixed insensitive export cursor can only adopt the new
profile through its original owner and matching lifecycle evidence.

Ashlar needs a separate original compiler registration and host page decoder for
full marker/value tuples. Source selection and native publication/version vector
stay pinned. The original emitter lowers nativeAST and native key-integrity
obligations; neither SDK nor host builds a sort predicate from strings. Existing
publication, scalar/relationship integrity, obligations and retention checks run
before and after each bounded page. Hidden tuple decoding belongs to Ashlar.

Both owners must validate selected-Key uniqueness using original native engine
custody, including alternate-Key integrity. Merely finding a Core Key declaration
or a caller assertion is insufficient. Live mutable Truss pages retain live
semantics; Ashlar pages retain their exact immutable publication. No cross-page
Truss snapshot claim is added. Grouped/aggregate entity identity and joined
one-to-many occurrence cursors remain separate semantics and cannot masquerade
as this entity-source profile.

## Component changes and sequence

1. Review the original engine ABI with Truss via the coordinator and Ashlar's
   current owner. Add the closed contract before shared APIs. Update module
   ownership for any new Core page IR; historical plans serialize identically.
2. Add private Core construction of selected-Key/effective-order/cursor IR from
   original requests, retaining canonical identities and all source parameters.
   Old authored pages continue through their existing entrypoint and IR.
3. Extend each original emitter and host decoder under its own version/profile;
   preserve original digest owner and exact source/native artifacts. Separate
   source admission from target delivery bounds for Related target-set pages.
4. Extend runtime wrapper/checker and SDK catalog as exact owner packet forwarding.
   No SQL rewriting, display-value cursor reconstruction or paging-Key swap.
5. Qualify installed wheel, actual native owners and browser parity on one source
   cut before enrolling the profile in a stable SDK compiler release.

## Meaningful tests

Use a genuine alternate selected Key distinct from primary/relationship Keys;
composite keys and duplicate user sort values; ASC/DESC mixed order; explicit
FIRST/LAST/defaultLAST; null and missing equivalence with distinct markers;
empty/all-null pages; hidden omitted projections; binary Unicode including
normalization differences; exact numeric extrema and Decimal domains;1-row and
maximum pages; forward/inverse Related target sets; no repeats on unchanged data.
Compare the complete traversal with independent native ordering oracles.

Refuse foreign Key/Field/source, wrong sort arity/carrier/state/order, forged
hidden metadata/token, changed filters/arguments/actor/catalog/publication,
missing alternate-Key integrity, denied hidden fields, native collation drift,
stale Ashlar versions and unsupported temporal/compound domains. Exercise both
sync/async pages and stream cleanup/cancellation; retain all old page refusals.
Run default-feature negative, module-boundary controls, fresh native-wheel bytes
and actual Chromium full-program equality. Native evidence must name its exact
owner version, source, wheel, native settings and complete cleanup.

## Open owner decisions and release boundary

Truss confirms a new original cursor/native accessor profile is required; the
current Key-only accessor cannot implement arbitrary ordering. Ashlar confirms
new full cursor DTO/compiler/host profile is needed and can reuse original native
storage accessors. No implementation starts until the contract/profile proposal
is reviewed. Legacy profiles remain usable and byte-preserved; rollback is profile
deselection, with continuations from another profile refusing before dispatch.
Native least-privilege closure, managed atomic mutation and broader aggregate
profiles remain independent requirements and are not satisfied by this plan.
