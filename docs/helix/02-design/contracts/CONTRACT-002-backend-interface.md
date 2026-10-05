---
ddx:
  id: CONTRACT-002
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-002
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
---

# CONTRACT-002: Registered backend interface

**Type:** plugin/library. **Version:** `weft-backend/0.1.0`. **Status:** draft.

## Purpose and Scope and Boundaries

Lower typed Weft IR into a backend's SQL without changing language meaning.
Initial identities are `ashlar.databricks` and `truss.postgresql`. They are registry
entries, never special cases in the frontend. Host execution is separate.

## Normative Surface

A trusted host registers a backend object implementing these pure operations:

| Operation | Input | Result |
| --- | --- | --- |
| describe | None | Immutable versioned manifest |
| validateBinding | Supplied pinned binding/module bundle | Validated mapping plus diagnostics/obligations, or rejection |
| assess | Typed IR, mapping, target profile | Capability/domain disposition for each required operation |
| lower | Accepted IR, mapping, target profile | Typed target plan or explicit refusal |
| emit | Typed target plan | SQL, ordered parameter slots, result carriers/decoders and obligations |

No operation may fetch modules, access databases, choose a fallback or reinterpret
Weft names. Backends must resolve every selected record/field to physical access
through supplied mappings and verify UMF/model/binding pins. Implementation code
may be linked or explicitly registered; manifest/binding content MUST NOT select
an executable path/import URL. Dynamic native ABI loading is outside v0.1.
The Rust trait and its exact IR/target-plan types must be finalized together with
this contract before backend implementation; the five operations above fix the
behavior boundary, not a released binary ABI.

[backend-manifest.schema.json](backend-manifest.schema.json) defines its structural
shape; semantic guards bind capabilities to target profiles and require evidence
for supported status. A manifest contains backendId, backendVersion, interfaceVersion, dialectProfile,
irVersion, bindingProfile, targetProfiles, capabilities and evidence. A target
profile names engine/version, relevant session settings, collation/numeric rules
and storage-layout/publication revisions. A capability names operation ID, logical
domain, result domain, constraints, required obligations, conformance status and
evidence reference. A boolean such as `supportsSQL:true` is insufficient.
Evidence identities are reproducible records, never permission to execute code.

A mapping is backend-owned, revision-pinned data keyed by exact logical identity
and scan kind. It describes physical source, field home/type/encoding, presence,
comparison and row-membership predicates. SQL fragments in untrusted bindings
are forbidden in v0.1. Unknown mapping members affecting selected meaning block;
unselected content remains retained. Mapping validity is not a stored-data proof.

Assessment outcomes are supported, candidate or unsupported. Supported means the
registered profile has evidence for the selected domain; it does not mean this
query or database was executed. Candidate requires explicit `allowCandidate:true`
and retains unverified assumptions. Unsupported always blocks. Default compilation
requires supported dispositions for every required operation; emitted artifacts
must carry qualifications, including every bounded native result domain.

The backend allocates parameter slots once from typed literals and discriminator
values. PostgreSQL `$n` and Databricks `:pn` are initial emission conventions;
values are never interpolated into SQL text. Identifier escaping, identifier
length/version limits and physical discriminator filters are target-owned and
must be covered by native evidence. Result numeric carriers must remain exact text.

## Initial Backend Profiles

Truss/PostgreSQL binds generic catalog storage per Truss's owned contract.
Objects and property homes must be explicitly mapped; filter type identities
before joins. Missing props keys, explicit JSON null and SQL NULL cannot be
collapsed when selected semantics distinguish them. Core record keys never
silently become storage IDs. Prepared statements, exact text transport, catalog
revision and policy obligations belong in the host contract. Nested/row homes
not explicitly supported by the profile block. No spike's table names are defaults.

Ashlar/Databricks binds an accepted gold-layout/publication contract, with explicit
column/relationship/identity mapping. No PostgreSQL JSONB assumptions transfer.
Require exact numeric carriers, verified string comparison, selected runtime/
warehouse behavior and consistent publication read obligations. Bounded native
SUM must be proved within its input/group domain or disclosed as an exact-or-error
runtime obligation accepted by the host; overflow-to-null/wrapping is forbidden.
Its physical layout remains an integration gate, not a definition in this repo.

Both backends preserve duplicate rows, isolated sources and selected identity.
No backend derives authorization from an untrusted query. Unsupported delegation
or publication guarantees cannot silently fall back to broader privileges/newer data.

## Precedence and Compatibility

CONTRACT-001 governs language meaning; UMF/owning storage contracts govern source
and layout. Exact manifest/interface/IR/binding versions must agree. Added
backend implementations do not change the dialect version. Every semantic change
requires new versioned capabilities and refreshed qualified evidence.

## Error Semantics

WFT-BACKEND-MISSING, WFT-BACKEND-VERSION, WFT-BINDING, WFT-CAPABILITY,
WFT-OBLIGATION and WFT-EMIT block without partial SQL. Lowering/emit failure
invalidates the entire artifact. Required plugin code is trusted; panics/throws
are caught at host boundaries and normalized to WFT-BACKEND-FAILURE. No recovery
chooses another backend implicitly.

## Examples and Validation Checklist

The same resolved Sales plan registers with both initial backends and a third
synthetic backend. Changing a Truss property home changes its mapping/lowering,
not its FieldRef or source SQL. A Databricks profile lacking proven exact SUM
blocks by default and may emit only an explicitly requested candidate.
Require registration independence, malformed/unknown manifest guards, injected
identifier/SQL-fragment refusal and independent native result comparison.
