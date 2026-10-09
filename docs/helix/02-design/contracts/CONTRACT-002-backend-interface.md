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

**Type:** plugin/library. **Version:** `weft-backend/0.2.0` (implemented library boundary; retained 0.1 schema is historical). **Status:** draft.

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
an executable path/import URL. Dynamic native ABI loading is outside v0.2.
The finalized Rust trait uses associated Mapping and TargetPlan types in `crates/weft-core/src/backend.rs`; the five operations fix a typed library boundary, not a released binary ABI.

[backend-manifest-v0.2.schema.json](backend-manifest-v0.2.schema.json) defines its structural
shape; semantic guards bind capabilities to target profiles and require evidence
for supported status. A manifest contains backendId, backendVersion, interfaceVersion, languageProfiles (exact dialectProfile/irVersion pairs), bindingProfile, targetProfiles, capabilities and evidence. A target
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

### Mathematical integer representability

An owning core 0.8 Field with no integer width retains its unbounded mathematical
source domain. Existing backend profiles MUST refuse `type.integer.unbounded`
before binding dispatch unless their manifest explicitly admits that capability
for the selected language/profile and candidate opt-in. Native representation
capacity MUST NOT be added as an invented logical width or integer-to-decimal
type conversion. Source validity, resource limits and backend representation
capability are distinct dispositions.

`ashlar.databricks.mathematical-integer` / `dbsql-mathematical-integer-candidate`
is an explicit candidate representation profile for required scalar relational
reads. It may represent finite values exactly using DECIMAL(38,0), retaining
logical Integer and exact integer text results. It MUST guard every consumed
selected source occurrence over the same pinned source before user SQL. A
nonrepresentable source or literal MUST fail with WFT-CAPABILITY, without
asserting that the original UMF value is invalid. Exact lexical literal magnitude
checks MUST avoid floats, rounding and bounded host integer conversions.

The `ashlar.mathematicalInteger.publicSourceValidity` host obligation MUST
validate every consumed original numeric JSON token using the actual owning UMF
public Field validator, retaining exact model/field and pinned row identities,
raw property bytes, token spans and original receipts. Native revision, presence,
container and carrier integrity remain separate scalar integrity checks. A host
MUST NOT validate a rounded VARIANT numeric value. Original numeric token
extraction MUST match the retained raw token exactly; unsupported lexical
representations refuse as backend capability failures even if source-valid.
JSON string, boolean, null, fractional and duplicate-member corruption MUST NOT
be reported merely as finite magnitude failure. Numeric JSON property custody
is required in this candidate; direct column homes remain unsupported.

The `ashlar.mathematicalInteger.representability` host obligation MUST remain
separate from source-value integrity observations; its successful finite native
representation does not establish original source validity or global coverage.
Arithmetic/aggregate inputs being representable does not prove intermediate or
result capacity. SUM MUST retain exact integer meaning, use exact-or-error
overflow detection and refuse partial/null/rounded/wrapped overflow results.
Empty global SUM remains its independently declared nullable result. Hosts MUST
buffer results and fulfill `ashlar.mathematicalInteger.exactResult` with exact
integer text decoding and finite representation checks before publication.
Unknown obligations refuse. No general arbitrary-precision execution, native
Databricks qualification, publication authority or ACK follows from this profile.

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

## B-003 finalized library interface

The implemented application frontend adds explicit 0.2 plans. B-003 implements
`weft-backend/0.2.0` alongside the retained 0.1 schema; this does not redefine prior
0.1 evidence. Its [manifest schema](backend-manifest-v0.2.schema.json) declares
exact `languageProfiles` dialect/IR pairs and binds each capability to declared
target and language profiles. Registration snapshots and validates the manifest;
unknown members, repeated IDs, mismatched pairs, missing domains and supported
capabilities without declared evidence refuse. A declared evidence ID remains a
trusted qualification reference; the compiler does not fetch or execute it.

The Rust `Backend` trait in `crates/weft-core/src/backend.rs` uses associated
`Mapping` and `TargetPlan` types. Each registered implementation owns those types;
the heterogeneous registry erases them only around its generic adapter, which
runs validate-binding, assess, lower and emit in order. `Plan` borrows either typed
IR version. Bindings stay bounded raw JSON with exact byte digest and retained
content. Selected records, fields, type-graph identities and relationship IDs
must receive explicit coverage. Plugins validate their own physical mapping
semantics and unknown selected members; common identity coverage alone does not
prove that physical mappings or existing data are correct.

Assessment must cover every required operation exactly once, against the selected
profile. Unsupported always refuses. Candidate requires explicit opt-in and cannot
be upgraded to supported by assessment. Manifest/assessment obligations survive
emission; conflicting requirements under one obligation ID refuse. No fallback
backend is selected. Default Rust unwind panics are normalized atomically;
non-unwinding aborts and WASM traps require wrapper/host normalization, which is
part of B-004 qualification, not a capability established by native unwind tests.

This library interface passes B-003 component/native/browser gates; [evidence](../../04-build/evidence/B-003-backend-interface.md) qualifies the exact subset. ADR-002 accepts the structural boundary. Public compile wrappers, production mappings and target-native qualification remain B-004 through B-007. No production mapping is invented here.

The registry also checks emitted column count/order/name, logical type,
selected source identities, exact numeric carrier/decoder pairing and typed
parameter lexical/domain validity. Slot positions are contiguous; target SQL is
bounded to one MiB and never silently clipped. These structural checks do not
prove the SQL implements the plan; independent/native result gates remain required.

Binding validation may declare additional required capability IDs, for example
`value.nativeNull` when a recognized physical/native profile requires explicit
null-state decoding. They receive the same complete assessment, evidence and
candidate gates as frontend-required operations. An optional UMF field alone
cannot authorize that capability. Value/related representations use typed JSON
text carriers with exact string numeric leaves; their decoders follow retained
type/key descriptors, not generic JSON-number conversion. The emitted qualification
retains both the selected declaration (domains/constraints/obligations/evidence)
and its per-query assessment. Obligation data is validated and conflicting IDs
refuse before an artifact is returned.
