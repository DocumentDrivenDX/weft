---
ddx:
  id: CONTRACT-006
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.prd
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
---

# CONTRACT-006: Security compilation admission foundation

FR-19–22 govern this slice. Owner direction is the shared UMF security implementation goal. UMF owns
CONTRACT-062/063 (the security artifacts migrated from CONTRACT-052/053 on UMF main) policy/ontology/lifecycle meaning; Weft owns compiler lowering;
hosts/backends own authenticated execution and publication.

`weft-security-compile/0.1.0` retains `weft-sql/0.2.0` grammar. Its request schema requires
UMF 0.8.0 source pins and a security source object with version
`umf.security/0.1.0`, policyJson and ontologyJson. Strings preserve authored bytes;
these bytes are not credentials, understood policy, or globally current revisions.
They must eventually undergo exact bounded semantic admission under the shared
UMF contracts before a security-enabled artifact can be emitted.

This initial foundation validates the transport and exact model byte/identity/
version/schema custody through a separate Catalog::prepare_security path. The
canonical upstream schema is pinned from UMF; source correspondence is retained
in the sibling integration evidence. Existing ordinary requests and
Catalog::prepare retain main's ordinary core 0.7.0/0.8.0 admission. Security preparation requires core 0.8.0. Unknown versions never downgrade.

All well-formed security 0.1 requests currently return WFT-SECURITY-UNSUPPORTED before
backend composition/emission, with no partial SQL, parameters or logical plan.
This refusal is deliberate unfinished implementation, not security support or
native acceptance. Malformed input/model custody may refuse earlier. Source
schema custody is not full core property or security semantic interpretation.

Next implement complete typed policy/ontology admission, logical security IR,
backend physical lowering, versioned disclosure/result-domain declarations and
mandatory host obligations. Then verify Rust, native Python and real browser
parity plus independent native backend acceptance. No string source, model flag,
callback, generic obligation or Supported ordinary fixture qualifies those gates.
No protected profile may route through the older ordinary compiler as fallback.

The initial response schema admits only a blocked security 0.1 artifact. No compiled
security 0.1 result-domain is selected yet; that owner contract must be authored and
validated before removing the unsupported activation gate. It is not valid to
return an ordinary compiled artifact with a security flag appended.

## Bounded source-packet admission

Rust SecuritySourcePacket now reads exact policy/ontology JSON with duplicate-key,
JSON structural and four-million-byte bounds. Canonical shared schemas plus an
explicit selected-source overlay refuse unknown members on defined semantic
objects; policy.native remains opaque archive data. Exact original JSON strings
and immutable parsed snapshots are retained. Global expression limits match the
shared 16-depth/4,096-node profile. Rule ID duplication and disclosure on non-permit
rules refuse. Policy ontology identity/revision and the complete ontology document
pin set must match the separately prepared core08 source bundle.

This is source-shape/pin admission, not complete ontology type/domain/variable
closure or authorization. After this stage, well-formed requests still return
WFT-SECURITY-UNSUPPORTED before any backend callback. Invalid source, bounds or
pins may refuse earlier with coarse WFT-SECURITY-SOURCE/LIMIT/PIN diagnostics.
Native archives, original source strings and immutable snapshots remain data;
none can select a role, plugin or current trusted fact provider. Further semantic
IR and backend lowering gates remain open.

## Exact ontology reference closure

Rust SecurityOntologyClosure now resolves full document/module/element identities
against the pinned core08 catalog. Every declared type must be a Record with one
unambiguous declared Key. Every Record member has exactly one explicit field
classification. Optional Boolean primary metadata is preserved on source Keys;
at most one declared Key per Record may have primary=true. Security identity
continues to select the exact ontology keyId, including when another Key is
primary. Primary metadata does not establish native enforcement or choose a
default security identity. Selected key components are unique, nonempty, required members;
subject is a declared entity. Association roles are unique, target types are
declared, and ordered endpoint components belong to the association and match
the target key's declared scalar domain shape, including facets/nullability and
allowed values. Same local IDs in different documents remain distinct.

This stage understands exact scalar singleton reference/domain shapes and refuses
unknown selected core qualifiers. It is not complete literal/facet validity,
expression variable/type inference, data uniqueness, physical mapping or native
authorization. Those later semantic/native stages remain required. Existing
WFT-SECURITY-UNSUPPORTED activation closure stays in place after reference closure.

## Declared policy term checking

SecurityPolicyTypes now derives ontology closure from the same admitted source
packet/catalog snapshot. Lexical exists variables must be fresh and declared
association bindings; subject/resource/context/member references must belong to
their selected binding. Endpoint roles resolve to exact target type/key identity.
Identity and scalar operands cannot mix, distinct qualified type/key identities
cannot compare as identities, and scalar operands require equal declared domain
shapes. Rule actions/targets and disclosed fields must be declared and unique.
Basic constant/transform wrappers must match their declared scalar family.
Expanded target expression checking has a one-million-node bound.

This stage checks declared types and basic literal shape, not complete numeric/
facet/allowed-value semantics, security truth/composition IR or native lowering.
Activation remains WFT-SECURITY-UNSUPPORTED before any backend factory. Source
packets retain exact model pins; changed input bytes and rehashed inputs differing
from prepared catalog definitions refuse rather than mixing stage snapshots.


## Exact scalar literal admission

The selected singleton boolean/string/integer/decimal/binary subset now checks
constants, transform outputs and declared field refinements with exact coefficient
arithmetic. Integer widths, decimal precision/scale, Unicode scalar lengths, binary
normalization, inclusive/exclusive ranges, empty finite-domain ranges, allowed-value
identity, examples and defaults are checked without floating-point conversion.
Numeric input and expanded coefficient length are bounded at four million bytes;
selected metadata integers are bounded by the core safe-integer contract.
Thirty-four focused admission/literal/frontend/envelope tests pass under Rust
1.90.0. This is selected-source admission evidence, not full UMF validation,
security truth/composition IR, physical lowering, binding parity or native
authorization. Activation remains WFT-SECURITY-UNSUPPORTED before backend creation.


## Type-admitted logical plan foundation

SecurityLogicalPlan owns the admitted source packet and expands each rule target
into a qualified Rule. Terms retain qualified field/domain or type/key identity;
endpoint terms retain their association and role. Existential variables are lowered
to distinct lexical slot numbers with outer correlated references preserved.
Constant and transform literal wrappers remain exact retained JSON; normalization
for evaluation remains a later stage. Original, withheld and typed transformed
dispositions remain distinct. Unknown selected meaning and type errors refuse
before plan construction. Plan reuse checks the pinned input bytes against current
prepared catalog definitions, including mutation of the latter.

This plan is an internal Rust foundation, not a released serialized contract or an
authorization capability. It does not establish truth evaluation, composition,
trusted fact cuts, query-use controls, physical lowering, or native obligations.
The public compile transport remains blocked-only with no backend factory calls.


## Pure rule-composition foundation

The Rust core implements bounded strong Kleene not/and/or and scoped rule
composition against an admitted logical plan. Selected target/action rule truths
are supplied exactly: missing or extra truths refuse. Any unknown produces an
indeterminate result with empty disclosure; otherwise a true permit, every
require true and every forbid false are necessary. False permits add no masks.
Protected fields require explicit dispositions; unprotected fields default to
original. Withholding dominates; incompatible transforms conflict. Transform
identity retains its constant transform/version, qualified output field and exact
normalized literal, rather than relying on lexical numeric/binary tokens.

The TypeScript reference supplies 120 truth vectors and all 27 permit/require/
forbid combinations; Rust checks correspondence and rule reversal independently.
These are pure caller-truth simulation results. They do not admit fact cuts,
compute expression truths, authenticate a subject, authorize native effects or
release data. The public compile activation gate remains closed.

Composition takes an explicit ordered output projection; duplicate or foreign
fields refuse. Only protected selected outputs require obligations, so a read
projecting an unprotected key remains admissible when an unselected protected
field has no disposition. Query-use permissions remain a separate unfinished
stage. Exact transform checks distinguish adjacent integers above 2^53 while
recognizing equivalent numeric tokens and signed zero.


## Bounded fact-dependent logical simulation

The Rust core can evaluate its admitted logical plan over an explicit simulated
cut. Full selected-source closure, one subject, policy/ontology/generation claims,
unique private fact identities, exact ordered key components, field/key consistency,
context and coverage are checked. Qualified type/key/component structures provide
logical identity; native key-codec correspondence remains unqualified. Dependencies
are preflighted across every scoped branch and every selected target, including
empty collections. Complete association coverage and endpoint target coverage
are required. Existential evaluation retains outer slots and nested same-type
relations; relations are immutable shared data and scans consume the step budget.
Original selected values require field coverage and presence/explicit absence.
Unknown or conflicting decisions refuse the entire simulated collection; denied
rows are omitted and admitted duplicate resource rows preserve bag multiplicity.

This simulation API is explicitly conditional on caller-supplied facts and trust
claims. It provides dispositions only, not native authentication, admitted host
authority, output values or a release capability. Query-use request members, writes
and unrecognized selected meaning refuse; backend lowering and publication remain
unimplemented. Twenty independently executed TypeScript scenarios provide logical
correspondence evidence; they do not establish language-binding or database parity.
The profile bounds input JSON at four million UTF-8 bytes, normalized literal
storage at four million bytes, fact count at 10,000 and steps at one million.
These byte bounds are more conservative than the portable model's text accounting.

Runtime scalar/identity copies additionally consume a sixteen-million-byte work
budget, preventing repeated clones of large exact coefficients from escaping the
step bound. This is a conservative logical simulation resource profile, not a
native backend performance qualification.


## Query-use logical admission

All five operators (predicate, order, group, join, aggregate) now select the
ontology's disclosed/original-authorized/prohibited mode. Protected omissions
default to prohibited; unprotected omissions default to disclosed. A prohibited
selection refuses before row evaluation, including empty collections. Original
use requires a distinct declared action, original-field coverage/presence and
all dependencies of that action, including empty/false primary-action branches.
Each admitted row must separately permit that action; no grant or a forbid
refuses. Disclosed use applies the composed disposition and refuses withholding;
original disclosed values require coverage/presence even outside the output
projection. Typed constant replacements remain admitted as disclosed values.
Query-only fields do not become output projection fields.

Seventy-five TypeScript/Rust scenarios include all operators, empty prohibited
queries, empty incomplete original-action cuts, missing grants/values, forbids
and typed transforms. This qualifies logical admission simulation, not query
execution, SQL operator parity, inference resistance or native authority. The
portable model's prior row-only admission gap was tightened consistently. Native
query lowering, writes, host custody and release qualification remain open.


## Source-bound query profile foundation

`weft.security.query-profile/0.1.0` is a draft compiler admission protocol. It
binds primary action and selected targets to exact model pins, retained policy/
ontology bytes and backend ID/version/target-profile/binding bytes. Every
original-authorized field/operator tuple on the selected targets requires one
exact separate declared action; omissions, duplicates, extra bindings, unknown
meaning and primary-action substitution refuse. Qualified field/type references
are never replaced by labels. The immutable object derives request annotations
and rejects a caller's different action, including on empty collections.
Conditional logical simulation can then use those derived annotations.

Security compile 0.1 accepts optional queryProfileJson inside security, admits it before
the unchanged unsupported activation gate, and never invokes a backend factory
for admitted or stale security profiles. Prior compile transports are unchanged.
Profiles retain exact sources; same-revision policy changes, backend-version
changes and binding-byte changes require re-admission. A source digest or this
object is not native authority. Profile ownership/current authority, installed
physical mapping interpretation, SQL query-use extraction and lowering, native
actors and final output release remain unqualified. Opaque binding fixtures
exercise custody only and do not establish raw-table or graph installations.

Profile reuse additionally compares exact retained ModuleInput snapshots,
including document bytes and selected module IDs. Hash pins alone do not
establish unchanged model selection. The current profile-source implementation
still requires independent native source ownership and installed qualification;
its source-bound object is not a credential or a release capability.


## Resolved SQL query-use extraction foundation

SecurityResolvedQuery owns the application resolver's typed SQL plan rather than
accepting a caller-constructed IR. It records qualified field/operator uses from
filters (including keyset tuples), both join operands, grouping, ordering and SUM
arguments. Distinct self-join scan occurrences remain distinct. COUNT(*) retains
all scans even without field uses; profile admission checks those targets.
Projection references are deduplicated dependencies, while the original resolved
plan retains output aliases/order/multiplicity. Exact model/module-selection and
policy/ontology snapshots qualify reuse.

Security compile 0.1 with a query profile now resolves its actual SQL and derives admitted
uses before the still-closed activation gate. Protected default-prohibited SQL
uses and out-of-profile count scans refuse without a backend factory. Relationship
helper predicates/projections refuse explicitly until their typed security bridge
is implemented; no relationship meaning is inferred from endpoint labels.
This qualifies the application read 0.2 resolver's admitted scalar source subset
against core 0.8. It does not qualify transformed-output operator typing, native
SQL lowering, physical relationships, source authority or released results.

Profile admission returns a private-constructor immutable SecurityProfiledQuery
artifact borrowing the resolved query and admitted profile. Its uses are read-only
source lineage, and source reuse is rechecked explicitly. A caller-constructed
list of field/action annotations cannot stand in for that artifact. It remains
conditional compiler data, not native authority or a release capability.
SQL extraction witnesses use authored names and signed 64-bit integer fields
within the existing 0.2 resolver subset; unbounded integer selection refuses in
that frontend, though the separate security literal model can interpret it.


Profiled queries now retain immutable per-scan, per-action obligation inventories.
Every scan retains its full ordered resource and subject identity components,
including field-free COUNT; self-join occurrences remain distinct. All scoped
rule IDs and every branch are visited without truth simplification. Separate
original-value actions retain their own closures. Correlated existence retains
complete classified association fields, inventory identities and ordered endpoint
target keys. Context references remain separate from record attributes; literal
domain references add no live fact read. Projection/query fields remain separate
from authorization facts. Extraction has global one-million-work and 16-million copied identifier-text
byte bounds; every examined rule and copied key/association component is charged.
This is a conservative physical-mapping input, not trusted fact completeness,
authenticated identity, transformed query typing, SQL enforcement or release.


### Mapping handoff 0.1 component

`SecurityProfiledQuery::mapping_handoff_json` exports experimental
`weft.security.mapping-handoff/0.1.0` only after exact source reuse checks. It
retains profile identity/revision/exact-byte digest, model pins, policy/ontology
digests, backend/version/profile/binding digest, original SQL digest and the
complete original resolved application plan. The plan preserves literals,
parameter substitutions, output order/aliases/multiplicity and native-independent
types. Extracted uses retain scan/target/field/operator and derived original
action (explicit null when no separate original action applies).

Each distinct scan carries target, projection/query fields and every action's
rule IDs, typed ordered key components, typed fact fields, context references
and complete association dependencies. Typed-key maps are represented as arrays
of target/keyId/ordered-fields entries; no JSON object key string substitutes
for qualified identity. Stable source collections make repeated exports equal.
Serialized output above 16,000,000 UTF-8 bytes refuses; extraction budgets remain
independent prerequisites. This Rust component has no public compile-envelope
activation, native Python or browser export yet. No declared general external
JSON importer is admitted by this component.

The packet is mapping input, never authentication, an executable policy, a grant
or a release capability. A backend must receive it through an admitted compiler
boundary, verify its current source/native binding, prove all required private
fact mappings and enforce action/field policy before lowering. Copying or
self-signing a packet does not establish these obligations. Security compile 0.1 security
activation still refuses atomically; native backend lowering remains open.


The experimental Rust example `security_mapping_handoff` now accepts
`weft.security.mapping-input/0.1.0` inspection inputs over stdin: modules, exact
policy/ontology/profile/binding JSON strings, backend identity/version/profile,
SQL, typed parameters and optional read profile. It resolves through the same
Rust owner and exports the mapping handoff only after admission. Root members
are strict; duplicate JSON and invalid UTF-8 refuse; input exceeds 32,000,000
bytes refuses before source decoding. Refusal emits a diagnostic on stderr and
no stdout handoff. The example has no backend factory, connection or execution
path. It is a bounded native inspection component, not a released CLI, public
compile-envelope activation, Python/WASM export or trusted issuer.


### Mapping handoff 0.2: normalized policy rules

The current experimental export is `weft.security.mapping-handoff/0.2.0`. The
0.1 shape is superseded because complete admitted normalized rules are now
required under `securityLogicalPlan`, version `weft.security.logical-ir/0.1.0`.
No native consumer may ignore this added semantic layer or accept an older
packet as equivalent. Input inspection transport remains 0.1; it emits the
current handoff from the actual admitted Rust plan.

Rules retain ID, permit/require/forbid effect, action list, qualified target,
condition and ordered field/disposition pairs. Rust's externally tagged
camelCase encoding retains literal/equal/and/or/not/exists expressions; equal
is an ordered two-term array, and/or are ordered expression arrays, and exists
has slot/association/condition. Slots are lexical and local to each rule.
Identity/endpoint/field/context/constant terms retain admitted bindings,
qualified identities, keyId, endpoint role, declared domains and exact typed
literal wrappers as applicable. Bindings encode subject/resource as strings
and variable as an object containing its slot. Dispositions encode original
and withheld as strings; transformed is tagged with transform/version,
outputField, domain and exact literal. No numeric literal is coerced to a
floating point representation by this export.

This JSON is obtained from admitted private-constructor plans, not reparsed
policy syntax or caller annotations. There is no general deserializer granting
admitted-plan status. Backends still must prove native translations of truth,
identity, effects, disclosure and correlated existence, and enforce complete
fact/source/current-authority obligations. Export alone closes none of those
physical obligations; compile security activation still refuses.


## Graph source and IR migration boundary — 2026-10-09

The original source packet and logical plan still consume security 0.1 Record
associations. Graph support must adopt the shared common-Record ontology and
qualified Relationship selectors coherently; rewriting an edge into fabricated
Record endpoint fields or discarding selected core qualifiers is prohibited.
The shared 0.2 schemas are currently draft spike inputs, not an admitted public
upstream version. No request version is widened by this checkpoint.

The Rust SecurityAssociationRef foundation distinguishes qualified Record and
Relationship namespaces. Original Record JSON uses elementId and graph JSON
uses relationshipId. Both require documentId and moduleId; ambiguous mixed
selectors, empty identifiers and unknown members refuse. Original spelling is
retained, including distinct Unicode scalar sequences. This parser is a typed
reference foundation only; its result is not source closure, a logical plan,
authenticated fact selection, or a native authorization capability.

The coordinated migration requires all of the following before graph source can
produce an executable protected artifact:

1. Versioned source custody must select the exact understood policy/ontology
   schema pair and core source pins. Older request versions must retain their
   source contracts and refusal behavior. A new compiled response/result-domain
   contract must be selected explicitly; the current blocked-only security 0.1 contract
   cannot silently acquire compiled output or downgrade through an ordinary path.
2. Ontology closure must separate common Record entities from association
   selectors. Record-member selectors bind ordered member fields; Relationship
   selectors resolve the original qualified directed core Relationship, each
   exact endpoint side/role and selected target Key. Record-backed witnesses must
   retain the actual witness Record and its selected Key. Opaque witnesses expose
   incidence/existence only and must refuse identity/attribute operations.
3. Type checking and IR must use the distinct association reference throughout
   lexical variables, existential binders, endpoint terms and dependency lookup.
   Target entity identity must not switch to a different native or primary Key.
   Repeated associations have distinct occurrence slots and outer correlation
   survives normalization, serialization and backend lowering. Bare source labels
   cannot resolve an association or erase a Record/Relationship namespace.
4. The evaluator and query-use/dependency consumers must retain three-valued
   unknown behavior, complete fact inventories and one admitted source/authority
   cut. Stored fields and trusted context are separate channels even when their
   qualified field reference is identical. Incidence, witness-owner and identity
   dependencies must map to actual protected native writers and invalidation.
5. Physical lowering must establish original Truss/Ashlar endpoint, witness,
   logical Key, type, attribute and current-state correspondence. Native surrogate
   IDs alone do not replace the selected logical Key. Graph profile activation
   remains blocked until native metadata/role/source closure, false-grant controls,
   concurrent revocation, diagnostic/disclosure privacy and publication obligations
   pass the shared acceptance plan.
6. Rust, native Python and real-browser WASM must share the same compiler and
   source/IR behavior. Raw and graph policies require equivalent results under
   explicitly verified fact mappings. Exact scalar domains, source preservation,
   malformed/unknown-version refusal and positive/negative controls must retain
   evidence tied to the shared criteria. Private TypeScript transports cannot
   substitute for the original Rust/Python/compiler path.

The existing activation gate remains unchanged. The reference foundation is the
first original Rust migration increment; source 0.2 closure, IR adoption,
evaluation, physical lowering, response selection and backend acceptance remain
unfinished. The sibling shared acceptance ledger remains 26/132.


### Association namespaces in the logical IR — 2026-10-09

Existential and endpoint IR fields now carry SecurityAssociationRef. The admitted
security 0.1 source path produces only Record variants; its serialized qualified
Record reference remains unchanged. The logical type can retain Relationship
references without converting relationshipId to elementId. This does not admit
graph policy sources, witness shapes or directional endpoint interpretation.

The current Record-fact evaluator and scan-obligation collector explicitly
require the Record variant before selecting fact inventories or endpoint member
fields. Unsupported graph variants refuse at each preflight/evaluation and
inventory entry path. They must not become empty relations, Record identities,
or fabricated field dependencies. Future graph consumers require the separately
specified graph source closure and witness/incidence facts before these gates can
be replaced with graph behavior. Source 0.2 closure, opaque/Record witness typing,
directional endpoint IR, graph fact evaluation and physical lowering remain open.

Three new original-core regressions exercise exact Record/Relationship IR JSON,
selected endpoint Key retention, four evaluator refusal paths and refusal before
invented scan inventories. Existing policy/evaluation/obligation and public
unsupported-envelope controls remain required compatibility evidence. Caller
construction of public IR enums is not construction of an admitted logical plan;
SecurityLogicalPlan retains its private admitted packet and read boundary.


### Separate draft 0.2 source custody — 2026-10-09

SecurityCandidateSourcePacket now retains exact policy/ontology 0.2 source bytes
and parsed snapshots under the shared draft schemas and explicit selected-source
overlays. Its wrapper type has no conversion or dereference into an admitted
SecuritySourcePacket. Existing public compiler requests and SecuritySourcePacket
read continue to select only their original 0.1 source contract. The draft packet
cannot be supplied to the existing logical-plan read API and does not widen the
blocked-only public response or native activation.

The shared source reader applies duplicate-key/JSON/byte bounds, global expression
depth/node limits, rule uniqueness, permit-only disclosure and complete core08
source/pin identity checks to either explicitly selected profile. Draft semantic
objects reject unknown members; native archive objects remain opaque retained
JSON. The selected draft policy/ontology pair and its pin set must match exactly;
mixed versions and version fallback refuse. Original draft schemas are retained
separately from their closed overlays, and the sibling admission runner compares
both with the current authored UMF spike sources.

This is source shape and snapshot custody, not ontology interpretation. A
well-shaped Relationship selector or opaque witness attribute expression may be
retained even when semantic closure would refuse it; neither reaches an admitted
plan through this API. Actual Relationship presence, directed role/side/target Key
closure, witness ownership and opaque capability restrictions remain the next
required stages. The draft source schema is not a released UMF extension version,
backend security profile or authentication credential.

Three original-core tests cover exact authored bytes/native archive retention,
strict profile separation, unknown/ambiguous selectors, stale pins, duplicate
JSON/rule keys and graph shape/depth limits. Required type/domain, graph fact,
compiler and native acceptance tests remain open beyond this custody stage.


### Draft 0.2 selected ontology correspondence — 2026-10-09

`SecurityCandidateOntologyClosure` consumes only the separate draft source
packet and rechecks exact catalog custody. The original 0.1 closure and the
draft reuse one Record declaration/ordered Key/field-domain closure, without
rewriting draft sources into an old-version ontology. Common entities carry
association Record declarations as well as subject and resource declarations.
Context references must be classified fields; action names must be unique.

Raw member selectors require the common owner entity's selected Key, unique
roles and ordered member-domain agreement with endpoint entity Keys. Directed
core Relationship selectors resolve one actual qualified Relationship, one
source and target reference, recognized lifecycle labels, unique role/side bindings and exact endpoint entity
Key selection. Multiplicity maximum may exceed one: singular endpoint arrays
do not assert a single edge instance. Record witnesses require actual
associationRecord ownership and the common entity's selected Key. Opaque
witnesses require associationRecord to be absent. Unsupported selected nested
qualifiers, including associationRecord/key, refuse. No interpretation profile
is invented for that core-unknown qualifier.

Three original Rust integration tests cover raw correspondence and actual graph
source correspondence with Record and opaque witnesses, plus semantic mutation
refusals. Selected Fields have one global Record owner; raw endpoint Records
remain in the association owning document. An independent Astra reproduction
exposed the shared-owner ambiguity; the corrected closure rejects it under an
otherwise-valid baseline. Two-document common entities remain supported, while
cross-document raw endpoints refuse under the current candidate profile.
This is a selected draft subset: all selected Record fields currently
use the original compiler's primitive singleton domain profile. Whole-core
semantic validity/complete qualifier interpretation, broader unused field types,
draft policy typing and IR, graph fact evaluation, formal compiler refinement
and native admission remain unfinished. This type is not accepted by the
original admitted 0.1 plan constructor or public compiler activation.

The selected graph check uses exact numeric normalization for integral decimal
and exponent multiplicity spellings; retained original source bytes are not
rewritten. Inverted bounds and unknown lifecycle labels refuse. Recognized
lifecycle labels, multiplicity declarations and inverse references remain
archived source metadata: this API does not prove edge population constraints,
inverse correspondence or write/delete lifecycle enforcement.


### Selected entity alternate-Key integrity — 2026-10-09

Draft ontology closure now checks every Key on each selected common entity,
including alternate Keys not selected for security identity. Key IDs, names and
component sets must be unique; component-set comparison ignores order only for
duplicate detection. Each component must belong to that Record and have a
required primitive singleton domain. Original component order and authored Key
metadata remain unchanged. The explicitly declared ontology keyId still selects
identity when another Key is marked primary.

An original Rust regression admits a distinct salary Key marked primary while
retaining pk as the ontology selection. Duplicate ID/name/component-set, wrong
owner and nullable alternate component mutants refuse. Repeated components
refuse earlier with WFT-MODEL under the existing envelope uniqueItems check; the
regression asserts that boundary separately. This extends selected Record
integrity, not whole-document semantic validation or public security admission.


### Separate draft declared policy term checks — 2026-10-09

`SecurityCandidatePolicyTypes` consumes the draft packet through selected
ontology closure. Variables bind qualified `SecurityAssociationRef` values,
retaining the Record and Relationship namespaces. Raw and graph variables
resolve declared endpoint roles to exact entity/selected-Key identities. A
Record witness exposes its actual owner identity and classified attributes;
an opaque witness exposes only existence and endpoints. Lexical shadowing and
unbound variables refuse; nested scopes retain outer correlation. Intrinsic
subject/resource identities and classified required scalar context/constants
are checked independently of association selectors. Literal values use the
original exact scalar/facet checker, including integerToken values outside the
JavaScript safe-number range. Actions, target identity, field ownership and
disclosure declarations are checked against the selected ontology.

Three new original Rust regressions exercise raw correlated typing and intrinsic
exact constants; graph endpoint typing with same-operand opaque refusals and
matching Record-witness positives; and classified/required constant/result
declarations. Astra independently reproduced two initial scope defects: an
unclassified Field could type a constant, and absent-allowed scalar operands
could type a null equality. Both now refuse under isolated positive baselines.
The nullable regression removes disclosure so a result-domain guard cannot
mask a missing operand guard. Initial fixture errors (invented literal wrapper,
missing transform/version members) were corrected to the original shared grammar.

This is declared draft term checking, not an admitted logical plan, evaluation,
authenticated source/context custody or native authorization. The selected
primitive singleton subset and incomplete whole-core semantic closure remain
explicit. General compiler refinement, graph IR/evaluation and physical lowering
remain required; public compiler activation and the admitted 0.1 plan API are
unchanged.


### Separate selected draft logical IR — 2026-10-09

`SecurityCandidateLogicalPlan` is privately constructed after draft selected
ontology and policy type checks. It owns the exact draft source packet and
rechecks original catalog custody for reuse. It is a separate Rust type from
`SecurityLogicalPlan`: existing Record-only evaluators and physical consumers
cannot accept it. Public compiler output and activation remain unchanged.

Draft existential nodes retain qualified Record/Relationship references and
explicit RecordKey or OpaqueExistential witness descriptors. Record witnesses
retain actual owner identity, selected Key ID and original Key metadata/order.
Endpoint terms retain selected entity/Key identity and either ordered raw member
carriers or directed source/target incidence. Opaque witnesses do not acquire
Record identity or attributes. Intrinsic and Record-witness value terms retain
original exact literal/domain descriptors and bindings. Lexical slots are fresh
across nested and sibling existential nodes; outer references keep their slots.
Rule effects, actions, target expansion and disclosure output descriptors are
preserved in the separate draft rule structure.

Original Rust regressions check exact inner/outer project correlation, ordered
raw endpoint carriers, both graph directions, opaque descriptors, distinct
sibling slots, source byte custody and stale catalog refusal. These are source
transformation tests, not a general compiler refinement proof or fact execution.
Whole-core semantic closure, broader attribute domains, graph evaluation and
scan obligations, formal refinement, Python/WASM parity and native physical
lowering remain required before a protected backend profile can be admitted.


### Separate draft action dependencies — 2026-10-09

`SecurityCandidateDependencies::derive` accepts a privately constructed draft
plan, exact catalog, selected target and declared action. It rechecks source
custody and selected ontology closure, then derives dependencies for matching
rules. This is an action inventory, not an admitted query/per-scan obligation
or proof that matching rules authorize the action.

Ordered identity Keys and stored Field reads retain owner identities. Raw
associations retain complete conservative Record witness fields and endpoint
member carriers. Graph associations retain qualified role/side/target/Key
incidences; Record-backed graph witnesses retain their actual owner Key and
classified fields. Opaque graph witnesses introduce no invented Record owner,
Key or member fields. Trusted context references remain separate from stored
reads even for the same qualified Field. Constant descriptors are not live reads.
Work and retained identifier text are bounded independently.

Original regressions check raw witness inventories, two opaque graph incidences
without Assignment/WorksOn Record inventories, stored/context dual channels,
constant-only descriptors and undeclared action refusal. Graph fact evaluation,
physical mapping, authenticated complete inventories/current cuts, native guards
and acceptance remain unfinished. The existing admitted 0.1 scan obligations
are unchanged and cannot consume this separately typed draft plan.

Astra's independent dependency review identified missing explicit text charges
for retained association references, incidence target references and ordered
Key-vector component copies. The corrected collector charges those copies before
insertion/allocation, including root and variable owner references. An original
unit independently sums a graph endpoint's retained identifiers: Key map 8,
Field map 6 and incidence 9 bytes. A 22-byte budget refuses; exactly 23 succeeds
with zero remaining. Separate work/text exhaustion checks and actual
Record-backed graph owner-Key/attribute inventory positives also pass.


### Shared pure composition for separately typed draft plans — 2026-10-09

Original admitted 0.1 and separate draft composition now use one pure rule
metadata fold. Each entrypoint first checks its own source/catalog and ontology
closure, output field ownership and declared action. It supplies scoped
effect/disclosure metadata to the common fold; no draft condition is rewritten
into an old-version plan. `compose_candidate` accepts only the separate draft
plan and caller-supplied rule truths, not authenticated graph facts.

The fold requires every matching rule truth exactly once. Unknown truth yields
indeterminate with no disclosure; permit/require/forbid and mask conflict rules
remain unchanged. The original regression tests for exact transformation
identity, withheld precedence and missing protected disposition still pass.
A new draft regression matches all 27 independent TypeScript-generated decision
vectors. An opaque graph draft control verifies permit with withheld output
under supplied true membership and indeterminate/no disclosure under unknown
membership. These are composition controls, not graph fact evaluation or native
authorization. Graph simulation and physical enforcement remain required.

Astra's review found no refactor defect and requested candidate-specific
disclosure bridge coverage. The additional original regression verifies missing
protected disposition yields indeterminate/no output; masks using integerToken
9007199254740993 and 9007199254740993.0e0 agree exactly; differing values conflict
without disclosure; and withheld output dominates conflicting masks. These
controls pass through the draft source/type/IR path and shared fold.


### Draft graph incidence value normalization — 2026-10-09

`CandidateIncidenceValue::read` checks a graph fact endpoint against the separate
draft plan and exact catalog. Qualified Relationship namespace, declared role,
source/target side, endpoint entity and explicit selected Key must agree. Key
components retain order and use the original exact scalar/facet normalization.
Scalar carrier text is bounded before allocation; nested or malformed carriers
refuse. This API does not accept a same-named Record association as a graph
incidence. It does not establish witness grouping, edge population completeness,
source authentication or graph truth.

The normalized value retains each component domain and original model pins,
and rechecks exact input hashes, identities/versions and prepared document
snapshots for reuse. It deliberately has no context-free equality: normalized
numeric carriers alone cannot identify integer versus scaled decimal domains
across sources. Normalized component access is metadata, not an execution permit.

Original graph fixture controls pass valid string identity and reject wrong
role, direction, target qualifier, selected Key, arity, scalar type, nested
carrier, Record/Relationship namespace confusion and stale catalog custody.
Domain descriptors and value reuse checks are retained. General source/IR
refinement, correlated graph witness facts, complete cuts, evaluation and native
physical admission remain required.

Astra's aggregate expansion finding is addressed with a separate 4,000,000-byte
normalized scalar-storage budget, charged before retaining each component.
Original regressions use a real admitted private draft graph plan: two one-byte
integer components refuse at budget one and pass at two; the short token
1e1000 refuses at budget ten and passes at the normal bound. Independently
valid integer 1 and decimal(scale 1) 0.1 models normalize to the same numeric
carrier, retain distinct component domains, and each refuses the other's
catalog. This is bounded value normalization, not graph fact authority.

### Draft endpoint bundle custody — 2026-10-09

A separately constructed CandidateIncidenceBundle retains the qualified
Relationship and both original source digests. It requires every declared role
exactly once with matching side, target, ordered Key and exact scalar domain.
Input ordering does not carry role meaning. Aggregate retained scalar payload
is bounded across the bundle. Reuse checks both authored source byte digests
and the exact catalog; same catalog under a changed policy refuses.
This groups caller-supplied endpoint inputs only: it does not authenticate
same-edge provenance, establish a Record-backed witness identity, prove
population completeness or evaluate policy. Those obligations remain required.

### Draft same-bundle existential simulation — 2026-10-09

The original Rust simulate_incidence_exists function evaluates a conjunction
of one or two distinct endpoint constraints within each single supplied bundle.
It checks every bundle's exact plan/catalog/association before observing truth,
and never combines endpoints from different bundles. Caller-declared complete
coverage with no match yields False; incomplete coverage with no match yields
Unknown; a match yields True under either coverage. This is a simulation kernel,
not a general candidate-policy evaluator or authorization API.

Bounds cover 4096 bundles, two constraints, 4M normalized expected scalar bytes
and 16M scalar-comparison payload bytes. Catalog/source hashing and parsing and
domain-descriptor equality are outside the comparison-payload budget; catalog
checks currently repeat for each bundle/value. No runtime CPU bound is claimed.
Controls include split-witness rejection, complete/incomplete empty populations,
exact budget boundaries, later-source substitution after a valid matching
bundle, and comparison exhaustion after a first match. An independent pair-set
oracle checks all 16 populations over two Staff and two Project identities, all
four requested endpoint pairs and both coverage states (128 correspondences).
Authenticated edge grouping, actual completeness, Record-witness attributes,
general AST evaluation and native enforcement remain required.

### Original opaque graph IR simulation — 2026-10-09

The selected interpreter now consumes an actual source-admitted draft rule
condition, preserving fresh lexical slots, outer references, endpoint identities,
Boolean composition and existential quantifiers. Populations must name declared
qualified opaque Relationships, including unused empty populations. Every
bundle retains exact plan/catalog/association custody. Recursive preflight
refuses intrinsic, raw-member and Record-backed witness expressions even beneath
empty quantifiers; those profiles still require implementation.

The authored nested correlation control asks whether one Staff has two distinct
Projects: same Staff/different Projects passes; different Staff/different
Projects fails; incomplete populations yield Unknown without a witnessed match.
The interpreter bounds supplied population count (512), total bundles (4096),
expression visits (1M) and scalar comparison payload (16M bytes). Hash/parse,
domain equality and lexical environment copying are outside those counters.
This conditional simulation does not authenticate edge grouping/completeness,
evaluate the full draft profile, compose authorization or activate a backend.

An interpreter-specific ledger regression uses an authored Exists with staff
endpoint self-equality over two valid matching bundles: two expression visits
refuse, three visits/four scalar bytes pass, and three scalar bytes refuse.
These isolate both ledgers and confirm a first matching witness cannot bypass
subsequent evaluation costs.

### Shared logical entity identities in graph simulation — 2026-10-09

CandidateEntityIdentity normalizes the common ontology Record's explicit
ordered Key independently of association storage. It retains exact primitive
component domains, source model pins and policy/ontology byte digests with no
context-free equality. Input scalar text and retained normalized payload each
have a 4M-byte limit. The explicit identity-aware interpreter checks supplied
subject identity against the ontology subject and resource identity against
the selected rule target, even when unused, and preflights required identities
beneath empty quantifiers.

Authored Staff/Project identity matching, split-edge rejection, missing/swapped
identity refusals, source substitution, exact integers above 2^53, ordered
composite components and integer/decimal domain separation have original Rust
controls. Private budget controls isolate composite budget one/two and short
exponent expansion; unused valid identities pass a literal rule while wrong
types and independently valid stale-policy identities refuse. Scalar fields,
context, constants, raw associations and Record-backed witness evaluation
remain unfinished. No native authority, completeness or full policy claim is
created by this private identity-aware simulation.

### Conditional original graph source/IR fixture correspondence — 2026-10-09

The private security_candidate_ir example admits original candidate source and
serializes the actual separate draft rules. A UMF Z3 harness constructs two
source/IR fixture comparisons for nested Staff/Project correlation and its
negation. Source lexical names and emitted slots are interpreted independently
over arbitrary witness populations with complete faithful membership and exact
role-specific nominal Key projection as premises. Source/IR inequivalence is
UNSAT; joint valid populations are SAT. A changed outer-to-inner reference
produces a SAT false grant in the positive condition and SAT false denial under
negation. Exact requests, emitted IR, mutants, binary/source hashes and six
replayable formulas are retained. This does not prove arbitrary AST induction,
the executable interpreter, unknown facts, native enforcement or authority.

### Original graph ABAC field facts — 2026-10-09

CandidateEntityFact combines a source-bound common Record identity with selected
qualified nonnull scalar field values. Foreign/duplicate fields and supplied
Key-field contradictions refuse. Inventories can be partial at construction,
but recursive preflight requires every used subject/resource field even under
empty quantifiers. The explicit fact entry point evaluates those scalar fields
and typed constants using original normalization and charges scalar copies
before cloning stored values. Subject/resource nominal and source checks remain
eager, including unused bindings. Context and witness field profiles still
refuse until implemented.

The fact constructor bounds 4096 fields, 4M aggregate carrier/qualified-field
text bytes and 4M normalized field payload bytes, separately from identity
normalization bounds. Authored graph ABAC controls combine Staff membership,
subject field equality and exact Resource salary 9007199254740993. The next
integer fails, missing subject/resource inventory under empty quantifiers
refuses, and normalized field budget six/seven and interpreter payload budget
seventeen/eighteen have isolated refusal/success controls. These are explicit
simulated facts, not trusted native projections or authorization decisions.

Additional controls accumulate Resource.resourceId r1 (two normalized bytes)
and salary 9007199254740993 (seven) in both field orders: budget eight refuses
and nine passes. An independently admitted policy changing only the subject
constant to two returns False with subject key/field one, unchanged matching
graph membership and unchanged passing salary guard; every supplied fact and
bundle is reconstructed under that policy's source custody.

### Independent original simulation context channel — 2026-10-09

The selected ontology closure now retains its already-validated context
declarations. CandidateContextValues accepts only those qualified nonnull
scalar attributes, with duplicate/foreign refusal and exact model/policy/
ontology source custody. The explicit context entry point keeps these values
separate from stored subject/resource fields, including when both use the same
qualified declaration. Used context must exist in recursive preflight even
beneath an empty quantifier; supplied unused context still checks source reuse.

The context constructor bounds 256 fields, 4M aggregate qualified-reference/
carrier text and 4M retained normalized scalar payload. Stored/context salary
controls independently pass stored one/context two, fail stored two/context
two, and fail stored one/context one. Missing context, unused stale source,
duplicates and foreign declarations refuse. Aggregate payload two/three
controls cover both field orders; exponent expansion and interpreter copy
three/four-byte boundaries have real constructor/interpreter controls. These
are caller-supplied simulation values. Authentication and native context custody
remain required, as do raw associations and Record-backed witness evaluation.

### Original raw association witness projection — 2026-10-09

CandidateRecordWitness retains one source-bound common Record fact and projects
all declared raw member endpoints to common logical entity identities. It checks
the Record namespace, selected raw selector, exact owner and explicit selected
Key. Each required endpoint component comes from that same fact in declared
order; target domains and exact source custody remain attached. Its own Record
identity stays distinct from endpoint identities. This is row projection, not
raw existential interpretation or population/authentication authority.

The constructor bounds 64 declared endpoint roles and 4M aggregate copied
scalar payload, charging before cloning. Missing endpoint fields, wrong owner
and a same-named Relationship refuse. Ownership Resource/Project projections
check independent logical identities and three/four-byte aggregate boundaries.
An actual separately revised compound Resource Key uses resourceId then salary
9007199254740993. Projection matches exact target, selected Key, ordered values
and domains; reversing differently typed member fields refuses semantic admission.
Independently valid changed policy and ontology plans cannot reuse the witness.
Astra independently verified the eighteen incidence tests with no actionable
finding in this projection subset. General raw/graph interpretation and trusted
complete native facts remain required.


### Original mixed raw/opaque graph simulation — 2026-10-09

The separate simulate_candidate_rule entry point evaluates original admitted
logical IR over raw Record witnesses and opaque graph bundles in one lexical
environment. Raw own-row identity and fields remain separate from projected
endpoint identities. Subject/resource identities and facts, typed constants
and independently supplied context use the same existing typed channels.
All supplied populations are checked eagerly, including unused empty ones;
Record and Relationship namespaces remain distinct. All rows retain exact
source custody. Recursive preflight requires used raw fields even when an
inner graph population is empty. Record-backed graph witnesses remain refused.

Limits cover 512 combined populations, 4096 combined rows, one million
expression visits and 16M charged scalar copy/comparison payload. Raw own-Key
duplicate detection compares explicit prior rows, charges both component
payloads before equality and charges at least one byte per scalar, including
empty strings. These limits do not establish total memory or CPU bounds:
source validation, metadata comparisons and environment copying have separately
qualified scope. Duplicate raw own Keys refuse rather than invent bag semantics.

The actual Resource/Ownership/WorksOn source rule returns True for shared
Project membership, False for split graph assignments or split raw owners,
Unknown for incomplete graph evidence, and False for an empty complete raw
population. Missing used raw fields and duplicate own Keys refuse. Isolated
literal controls cover three-byte duplicate-check refusal/four-byte success,
later stale-policy row refusal and unused invalid namespace/selector refusal.
Nested raw identity inequality distinguishes two own IDs with identical
endpoints and returns False for one row. Astra independently verified twenty
incidence tests and 59 library/42 admission tests without remaining actionable
findings after the comparison-budget repair.

This is caller-supplied simulation, not authentication, authorization or native
completeness. Existing six graph source/IR formulas do not prove this mixed
interpreter. Arbitrary compiler/evaluator refinement, trusted native cuts,
Record-backed graph facts, query/scan integration and physical enforcement
remain required. Public unsupported compiler activation stays closed.


### Original Record-backed graph simulation — 2026-10-09

A separate CandidateGraphRecordWitness pairs one source-bound incidence bundle
with one common Record fact. Admission checks the actual core Relationship's
selected record-key witness, exact Record owner and selected Key. Fact and
bundle must belong to the same source plan. This preserves caller grouping;
it does not establish authenticated edge-to-Record correspondence or infer
endpoint field mappings absent from the authored model.

The simulate_candidate_rule_with_graph_records entry point combines raw,
opaque graph and Record-backed graph populations. Record-backed variables
expose their own Record identity and selected fields while endpoint terms retain
incidence role/side/target/Key carriers. Recursive used-field checks, eager
population/source checks, duplicate own-Key refusal and the existing combined
population/row/expression/payload limits apply. Duplicate graph Record Keys
charge both scalar payloads before equality, including minimum empty-scalar
charges. Existing opaque-only entry points continue to refuse this population.

Actual authored associationRecord Assignment controls require active=true and
the matching Staff endpoint on one edge. Joined evidence returns True; splitting
active and matching Staff across edges returns False. Complete empty coverage
is False and incomplete empty coverage is Unknown. Missing active, wrong Record
owner, duplicate edge Keys, stale fact source and a later stale edge refuse.
Literal-only duplicate comparisons refuse at three bytes and succeed at four
for distinct two-byte Keys; equal Keys refuse. Identical endpoints with distinct
Record Keys satisfy nested own-identity inequality while a singleton is False.
Unused empty unknown, Record-namespace and opaque-selector populations refuse.
Scoped admission passes 60 library and 42 security admission tests.

These are simulations of caller-supplied facts. They do not prove authenticated
fact custody, complete native cuts, arbitrary evaluator/compiler refinement,
physical lowering, backend privacy or authorization. Existing original graph
source/IR formulas remain limited to opaque fixture correspondence. Public
unsupported activation remains closed and backend acceptance is unchanged.


## Planned owner security backend boundary — 2026-10-09

This section defines the next implementation boundary for FR-20–22. It is
not delivered support. Security compile 0.1 and its blocked-only response remain
unchanged until a separately versioned compiled result and its conformance
schema are implemented. No existing ordinary backend context can discharge
this security boundary.

The owner MUST dispatch only after source/custody, logical policy, SQL lineage,
query profile and complete scan/action dependencies have all been admitted
against the same Catalog and exact binding. The dispatch context MUST expose
immutable borrowed views of the actual Catalog, SecurityLogicalPlan,
SecurityResolvedQuery, SecurityProfiledQuery and exact binding/backend/profile
selection. These are owner-constructed values; no JSON deserializer, digest,
caller-supplied handoff or backend success flag may construct their admitted
status. Model/ontology/policy/profile bytes, model pins, original SQL and typed
parameter substitution MUST remain available with their exact source identities.

A security backend MUST be explicitly registered by trusted host code under a
security-specific interface/version. Registration and composition MUST NOT
select an ordinary V01/V02 emitter as fallback. A missing security registration,
unsupported source version or unknown selected obligation MUST block before
backend emission. A selected backend MUST receive the complete per-scan/action
obligations, projection/query field lineage and normalized rule dispositions;
a query filter or read predicate alone is insufficient.

The planned compiled result MUST distinguish owner admission, physical lowering
and native qualification. Owner admission and physical lowering never supply
native qualification. Its separately versioned schema MUST retain:

| Element | Required meaning |
| --- | --- |
| Exact source identity | Original request/model/policy/ontology/profile/binding bytes or lossless pinned references with explicitly qualified custody; no omitted selected content. |
| Logical security | Actual owner-normalized policy rules, typed application plan, field/operator uses and complete per-scan/action dependency inventory. |
| Physical lowering | Backend/version/profile, complete selected mapping, fixed resolution/parameter contracts and installation/execution operations. |
| Result domain | Versioned field order, multiplicity, exact scalar/presence domains, disclosure dispositions and their transport encoding; original null, absence, withholding and replacement remain distinct. |
| Obligations | Each required obligation has a stable ID, semantic source, enforcement site, prerequisite, failure behavior and independent native evidence requirement. |
| Native admission | Explicitly required; no compiler artifact or caller boolean may mark the installed profile qualified. |

The native host MUST refuse activation, ordinary exposure and execution until
all selected obligations are independently verified under current
source/inventory and trusted authority/publication custody. Isolated
transactional staging MAY install a candidate under preserved prior protection;
the host MUST independently verify installed correspondence before atomic
commit/publication and roll back or keep access closed on verification failure. Unsupported path, transform, history,
composition or disposition-query semantics MUST return an atomic blocked
result without executable SQL, partial parameters or install instructions.
Candidate/report options MUST NOT bypass this refusal. The compiler MUST NOT
replace a failed security result with an ordinary artifact.

Conformance MUST use the same public compiler entrypoint and real installer:
disclosed/original-authorized/prohibited behavior across predicate, order,
group, join and aggregate, including empty selections; complete-dependency
refusal; owner-admitted valid source whose selected backend rejects unsupported
path/transform/history/composition, with prior protection preserved; malformed
source refusal separately; injected installation failure; concurrent dependency changes; and exact independent
installed inventory and ordinary authenticated execution. Rust, Python and
browser parity are required for the selected owner protocol. Raw PostgreSQL is
the first executable target; this does not qualify actual Truss graph storage,
its unfinished commit finalizer, Delta or Ashlar.

The compiled response version, exact schema and security registration API are
explicit open design items. Implementing dispatch or lifting activation before
those items are resolved is prohibited. This boundary is not permission to
reinterpret security compile 0.1 or publish an ordinary artifact with a security flag.

## Draft security compile 0.2 and security backend 0.1 protocol — 2026-10-09

This section resolves the preceding planned version/schema/API items for
implementation. It does not deliver their implementation or qualify a backend.
`security-compile-request-v0.2.schema.json`, `security-compile-response-v0.2.schema.json` and
`security-cells-v0.1.schema.json` govern the proposed transport, and
`security-backend-manifest-v0.1.schema.json` governs registration shape. Existing compile
0.1/0.2 behavior and blocked-only security 0.1 are unchanged. Security compile 0.2 selects
SQL/application IR 0.2, pinned core 0.8 and security policy/ontology 0.1, with
mandatory exact queryProfileJson. Unknown versions or selected semantics refuse.
Common-Record/Relationship security 0.2 still requires coordinated original
owner source/IR adoption; this protocol does not relabel its candidate plans.

### Security registration API

`Compiler::compile_json_with_security_factory(raw, factory) -> String` is the
new Rust host entrypoint. It accepts security compile 0.2 only. `compile_json(raw)` MAY
recognize security 0.2, but MUST return blocked without an explicitly registered security
backend. The ordinary `compile_json_with_factory` MUST NOT dispatch a security
request through an ordinary registry. Python/browser hosts MUST expose the same
owner protocol without a separately implemented semantic sidecar.

`SecurityRegistryFactory<'host>` is a trusted host-supplied, higher-ranked
callback: for every temporary owner-context lifetime it accepts
`&SecurityBackendContext<'context>` and returns `Result<SecurityRegistry>`.
The context has no public constructor or deserializer. Its read-only accessors
are `catalog()`, `logical_plan()`, `query()`, `profiled_query()`,
`binding_json()`, `backend_id()`, `backend_version()` and `target_profile()`.
The first four borrow the actual mutually admitted Catalog, SecurityLogicalPlan,
SecurityResolvedQuery and SecurityProfiledQuery. The remaining accessors borrow
exact selected input strings. Source/profile/mapping reuse is rechecked before
callback dispatch. The callback is invoked once only after complete owner
admission; an owner refusal invokes it zero times.

`SecurityRegistry::register(backend)` explicitly registers a trusted executable
implementing `SecurityBackend`. Duplicate backend ID/version selections refuse.
`SecurityBackend::manifest_json() -> &str` supplies a strict versioned declaration;
`SecurityBackend::lower(&SecurityBackendContext) -> Result<SecurityLowering>`
performs pure physical lowering. Model content never loads executable code.
Neither callback nor lowering may acquire native connections, execute data,
activate installations or publish output. This is an audited trusted-code
boundary, not a sandbox for arbitrary Rust callbacks.

The manifest is exactly `{interfaceVersion,backendId,backendVersion,
sourceProfiles,bindingProfile,targetProfiles,capabilities,evidence}`.
`interfaceVersion` is `weft-security-backend/0.1.0`; sourceProfiles entries are
exact `{dialect,applicationIr,policy,ontology,securityIr}` version tuples.
Target profiles use the existing typed TargetProfile declaration. Capabilities
use the existing typed Capability declaration, selected explicitly against
security source/result domains; unknown selected constraints or obligations
refuse. Manifest evidence is not installed-inventory qualification.
Registries MUST NOT contain ordinary backend emitters as security fallbacks.

The owned return type is `SecurityLowering { lowering: SecurityPhysicalPlan,
result_contract: SecurityResultContract,
obligations: Vec<SecurityAdmissionObligation>, capability_ids: Vec<String> }`.
The first three fields have exactly the closed transport shapes of `lowering`,
`resultContract` and `obligations` below. `capability_ids` contains between one
and 4096 distinct IDs from this registered manifest. It is compiler validation
input, not a backend assertion that native obligations have been discharged.
All fields own their contents; none borrow the temporary context. Lowering
cannot supply or replace the source request, owner plan, their digests,
compiler/backend identity, response status or nativeAdmission. The compiler
constructs those envelope fields from its own admitted context and registered
manifest after validating the entire return value. It serializes the checked
result contract once, retains those exact bytes and computes the contract SHA;
the backend cannot supply an independent unchecked contract hash.

Manifest registration parses at most one MiB of UTF-8 with duplicate JSON
members refused, then applies a closed schema and the following semantic rules.
Identity strings are nonempty, contain no NUL and are at most 4096 Unicode
scalar values. `sourceProfiles` contains exactly one entry in this interface
version: `{dialect:"weft-sql/0.2.0", applicationIr:"weft-ir/0.2.0",
policy:"0.1.0", ontology:"0.1.0",
securityIr:"weft.security.logical-ir/0.1.0"}`. These are exact version tokens,
not ranges or compatibility hints. A future tuple requires explicit owner
protocol adoption; an unknown or duplicate tuple refuses registration.
`targetProfiles` has one to 256 entries with distinct IDs. Each entry has
exactly the existing TargetProfile fields `id`, `engine`, `engineVersion`,
`sessionSettings`, `storageLayoutRevision` and `publicationRevision`; all
identity strings follow the same rules and sessionSettings is an object.

`capabilities` has one to 4096 entries with distinct IDs and exactly the existing
Capability fields `id`, `targetProfiles`, `languageProfiles`, `logicalDomain`,
`resultDomain`, `constraints`, `obligations`, `status` and `evidence`.
Target references are nonempty, distinct and resolve within this manifest.
Language profiles contain exactly the selected dialect/application-IR pair,
using existing keys `dialectProfile` and `irVersion`. logicalDomain and
resultDomain are nonempty objects; their contents, target session settings and
constraint meanings must be understood by the selected executable admission
profile, rather than accepted merely because they are JSON objects or strings.
Unknown selected semantics refuse lowering. Constraints and evidence lists
contain distinct identity strings, with at most 4096 entries each. Manifest
evidence is likewise a distinct list of at most 4096 identity strings.
Capability evidence must be a subset of manifest evidence; a supported
capability requires at least one declared evidence ID.

The registry MUST retain the exact bounded original manifest JSON alongside its
parsed declaration and read the callback declaration only at registration.
Private borrowed selection must retain that original entry and selected target;
callback drift cannot replace it. The private obligation custody component
retains original parameters and all selected capability origins, deduplicating
only identical ID/parameter/owner/failure-code declarations. Conflicting repeated
IDs refuse. Its exact-registration comparison is conditional on an independently
trusted host selection; labels or copied response bytes do not authenticate that
selection. This is preservation only: unknown parameter meanings, independent
source/action coverage and closed response reconstruction remain required before
admission. No generic parameter field is added to the security 0.2 transport.
 Each capability has at
most 4096 original typed Obligation entries with distinct IDs, object parameters,
owner `host` or `backend`, and a nonempty `WFT-[A-Z0-9-]+` failure code. Their
meanings and parameters must be preserved in the required admission obligation
inventory; they cannot disappear when the lowering result is assembled.

Selection matches the requested backend ID, backend version and target profile
ID exactly; neither the factory nor backend may substitute a different
selection. The registered bindingProfile governs validation of the exact
requested binding bytes; the request supplies no separate binding-profile
override. The full target declaration and binding bytes are checked, not just
the target ID. Every returned capability ID must apply to that exact
target and source tuple. Unsupported capabilities always refuse. Candidate
capabilities require explicit `options.allowCandidate:true`; omission is false.
That option permits only the declared candidate status: it does not erase a
missing capability, unsupported domain or constraint, incomplete obligations,
source mismatch, or failed semantic check. No selected status can be upgraded by
the lowering return value, evidence identifiers or a supplied success flag.

The compiler derives required coverage from the complete admitted application
and security plans, including every scan/action dependency and disposition.
It checks the returned capability set and physical/result/obligation inventory
against that requirement set and the registered declarations; an empty,
unrelated or partial capability set cannot replace this independent inventory.
Duplicate IDs, unresolved references, conflicting repeated obligations or any
uncovered requirement refuse. Registration and supported/candidate declarations
are never evidence of native installation, authentication, enforcement or final
release. The native host must check the same exact registered declaration as
part of installation admission rather than trusting a backend ID in a copied
compiled response.

#### Private closed obligation projection experiment

The draft private profile `weft.security.admission-obligation/0.1.0` interprets
original registered `Obligation.parameters` with exactly version,
semanticSources, enforcementSite, prerequisites and evidenceCaseIds. Source and
case lists are nonempty; all lists are distinct and contain at most4096 nonempty
NUL-free identities of at most4096 UTF8 bytes. Original obligation IDs also obey
that byte bound. The selected projected inventory contains1..4096 obligations.
Original ID/failureCode and all selected capability origins remain in custody.
For this first subset, host-owned declarations project only to host; backend-owned
declarations project to backend or native. Compiler sites and unknown parameter
members/versions refuse projection, without destroying their original content.

Prerequisites resolve only against the complete selected obligation inventory,
not unselected manifest entries; all components must be acyclic. The owner uses
an iterative whole-inventory traversal and independently bounded projection
work/text ledger. Projection reconstructs declared response members only. Actual
selected source identities, owner-derived required capability/inventory coverage,
independent required case identity/coverage, installed enforcement and authenticated
current authority are still required before emission. No backend declaration
or successful projection discharges those checks; public lowering stays closed.
This experimental subset does not add a public parameter member or change security 0.2.

#### Private owner-source correspondence experiment

The private context projection issues canonical compact JSON string-array IDs
from the actual compiler-owned requirements: exact source hashes/model pins,
selected modules, primary action, scan/action/rule dependencies, ordered key
components, stored/context fields, associations, query/projection fields,
operator uses/modes and output positions. IDs retain repeated scan occurrences
and output positions; concatenated names or noncanonical aliases cannot replace
them. The projected declared source union must equal this complete issued set.
Each encoded ID is NUL-free and at most4096 UTF8 bytes, and the set at most4096
members; the owner charges bytes before hashing/serialization under a separate
bounded derivation ledger. This is original security0.1 owner-context inventory
correspondence, not draft graph0.2/native enforcement qualification. COUNT keeps
requirements but still needs separate computed-result support. Complete semantic
capability, per-site source assignment, required independent native case coverage,
exact registration custody and authenticated current authority remain mandatory
before emission. Public lowering remains closed.

### Compiled result and result contract

Blocked security 0.2 responses contain exactly interfaceVersion, status and diagnostics,
including an error. They contain no executable or partial artifact. A compiled
security 0.2 response contains exactly the schema-required fields: compilerVersion,
backend, sourceRequestJson/sourceRequestSha256, ownerPlan, lowering,
resultContract/resultContractJson/resultContractSha256, obligations and
nativeAdmission, plus version/status/diagnostics.
Compiled diagnostics MUST contain no error. Native admission is always
`required`; no compiled response field can grant native qualification.

sourceRequestJson preserves exact original request bytes, including model,
policy.native and currently uninterpreted archived extension content. Its SHA
is over those UTF-8 bytes. ownerPlan.json is the actual owner-produced mapping
handoff 0.2; its explicit version and SHA MUST match parsed contents and exact
UTF-8 bytes. This serialization is for inspection/custody, not a constructor for
an admitted context. Lowering is `weft.security.physical/0.1.0`: a nonempty ordered
installation array and one execution operation. Each is `{id,sql,parameters}`;
IDs are unique across the bundle, SQL is backend-generated, and typed parameter
slots retain the existing versioned lexical parameter contract. The host MUST
stage the installation under one qualified atomic activation protocol; it MUST
NOT execute individual statements outside that protocol or expose staged data.

resultContract is `weft.security.result-contract/0.1.0`, with encoding
`weft.security.cells/0.1.0` and ordered columns. Positions are contiguous from
one, column order/output names match the actual owner application plan, and
sourceFields retain exact revision-qualified field lineage. Each column has
uniquely identified outcomes. Original/transformed outcomes declare exact
scalar or selected model-field domains. A transformed outcome retains the closed
descriptor `{kind:"constant", version:"0.1.0", outputField, literal}` and a
nonempty `dispositionSources` list of exact `{ruleId,target,field}` references.
Target and field are both revision-qualified; a rule ID alone is insufficient
because owner normalization repeats a source rule for each selected target. The
literal is the complete shared-core typed literal, not merely its carrier type.
Every reference must resolve to the actual admitted permit-rule disclosure for
that exact normalized target, revision-qualified source field and selected action. Its transform,
version, output field, domain and literal must agree with the declared outcome;
multiple references may share an outcome only when their complete checked
transform meanings agree. All applicable source dispositions and composition
rules remain required; a list containing just one convenient rule cannot erase
another active obligation or permit a conflicting mask. Other transforms refuse
this protocol until separately defined. Withheld and absent outcomes carry no value
domain. Absence requires an admitted absent-allowed source/result meaning;
nullable original values are distinct from absent and withheld.

`resultContractJson` retains the compiler's exact checked UTF-8 serialization
of resultContract, and `resultContractSha256` is SHA-256 over those exact bytes.
Every consumer must reject duplicate members or malformed JSON in the retained
string, verify its digest, and require parsed structural equality with the full
resultContract object, including ordered arrays and literal token strings.
Object member order alone does not change structural meaning, but a different
serialization is different retained evidence and requires its own matching hash;
consumers must not reserialize before checking the supplied digest. Agreement of
bytes, hash and object is custody only, not independent admission of a supplied
contract or source request.

Cells are `{outcomeId,disposition,value}` for original/transformed and
`{outcomeId,disposition}` for absent/withheld. value uses the pinned shared core
0.8 literal vocabulary, including explicit JSON null and exact integer/decimal
lexical tokens. Runtime MUST check each cell against the selected column's
outcome and domain and check row width/order. A transformed constant cell must
also equal the selected checked literal under the exact admitted domain's
equality, without numeric coercion or substituting another valid domain member.
The batch pins exact serialized
result-contract bytes by SHA. Shape validity alone neither proves source-domain
membership nor authorizes any cell. A native null cannot stand in for withheld,
a missing result property cannot stand in for absence, and JS Number conversion
cannot replace exact integer/decimal carriers.

Each obligation is `{id,semanticSources,enforcementSite,prerequisites,
failureCode,evidenceCaseIds}`. IDs are unique; prerequisite references resolve
without cycles; semanticSources refer to selected source paths/scan-action
identities, and evidenceCaseIds name the independent required native cases.
The compiler MUST require complete dependency and disposition coverage,
source/inventory correspondence, private-fact completeness, ordinary bypass
closure, compatible current authority, guarded final release, result codecs
and atomic activation. None may be omitted or marked enforced by a backend
success flag. These are required native admission obligations.

The compiler validates physical/result declarations against the actual owner
plan and source domains before emission; any missing, incompatible, unknown or
unsupported selected requirement blocks without a partial artifact. Exact byte
limits are sixteen MiB for the request, four million bytes per policy/ontology/
profile/binding source, sixteen million bytes for ownerPlan.json, one million
bytes per SQL statement and thirty-two MiB for a response. Schema character
limits do not replace UTF-8 byte checks. Schema collection bounds apply in
addition to the owner parser/extraction budgets. Each result-cell batch has a
separate thirty-two MiB UTF-8 limit, maximum JSON container depth 64 (root depth
one), and maximum 1,000,000 JSON value nodes, counting each object, array and
scalar once. Parsing must enforce these limits and reject duplicate object
members before materializing or publishing rows. Oversize/deep batches refuse
with WFT-LIMIT; no prefix of a refused batch may be released. A host may stream
separate bounded batches only while independently preserving current authority,
contract identity and guarded final release for each batch.

### Error and compatibility rules

Missing security registration returns WFT-SECURITY-BACKEND-REQUIRED. Invalid or
incompatible registration returns WFT-SECURITY-BACKEND-VERSION. Incomplete or
unsupported physical/result/obligation coverage returns
WFT-SECURITY-LOWERING-UNSUPPORTED. Existing source/profile/type diagnostics
remain applicable before dispatch. Refusal does not authorize an ordinary,
report-mode or candidate fallback. Current native protection is untouched by
compile/refusal; host installation independently performs the staged inventory
verification and atomic commit/rollback required above.

The schema files define transport conformance only. Rust/native Python/browser
public entrypoint parity, actual security registry dispatch, native physical
lowering, result-domain validation and the real installer remain unimplemented
until their source-qualified conformance evidence is retained.


### Rust module ownership for the security registration boundary

`security_backend` owns closed declaration parsing, the private-constructor
borrowed owner context, explicit trusted security registry and exact registration
identity selection. It depends on existing immutable owner semantic objects and
shared nested backend declaration types; it never registers ordinary emitters.
`security_lowering` owns the backend's owned physical/result/obligation return
vocabulary. Public construction supplies declarations, never admitted output.
`compile` alone sequences actual source/logical/query/profile admission and
context construction before a security factory callback. It constructs all
compiler-owned envelope/source fields. Ordinary registration remains separate.

The initial implementation may dispatch security registration while keeping
physical lowering closed. Until independent source-domain, capability, binding,
result and complete dependency validation exist, every selected registration
must return blocked before invoking `SecurityBackend::lower`. Registration shape
and ID/version/target membership do not establish selected semantic coverage.
Boundary evidence must include real public-entrypoint callback counters,
unsupported source/profile/refusal controls, ordinary fallback controls, exact
selection failures, and an external-context-substitution compile-fail control.
No native fixture or supported/candidate declaration can remove that gate.


### Owner-issued requirement inventory

`security_requirements` owns the private-constructor borrowed SecurityRequirements,
with immutable scan/action inventories joined to complete actual logical Rule
objects, explicit Disclosed/OriginalAuthorized operator modes, and ordered actual
application Output occurrences. `security_query_profile` exposes its primary action
without inferring it from sorted actions. `security_backend` constructs requirements
only after exact source/profile/binding and actual-query identity rechecks, before
calling the registration factory. No external constructor/deserializer is admitted.

The derivation budget counts at most one million scan/action/rule and retained-entry
visits plus sixteen million selected identifier bytes; rule/condition/output trees
are borrowed, not cloned. Existing upstream source/container bounds remain required.
This is not a bound on every comparison or total CPU instruction. COUNT/SUM and
all permit/require/forbid conditions/disclosures remain explicit requirements; their
presence is not result-domain admission. The existing LOWERING-UNSUPPORTED guard
remains unconditional. Project boundary verification uses the locked weft-core
library/security_admission tests and external compile-fail documentation test.


### Pure direct-field result declaration checker

The private `security_result_check` module owns source-bound declaration
correspondence. SecurityBackendContext.check_result_declaration takes a borrowed
SecurityResultContract and returns Result<()> only. It rechecks source/profile/
binding custody and validates actual ordered outputs with internal scan identity,
exact original model domains, all conservative constant-transform classes and
all scoped source IDs, including false permits. Transform classes include exact
output-field/domain/version and normalized literal meaning. Original-bearing
fields require resolved required presence in this initial subset; Absent and
aggregate expressions refuse complete checking. A null constant remains a
transformed literal when its admitted domain permits it.

Closed versions/encoding, 256-column/outcome limits, unique IDs/classes and exact
coverage are mandatory. The one-million counted-visit and sixteen-million payload/
identifier-byte ledger charges both source literals and normalized payloads before
class retention/comparison. Existing individual normalization/source bounds still
apply; the ledger does not bound every CPU instruction or allocator byte.
Successful correspondence is not permission to choose any envelope member, does
not issue executable/admitted native artifacts, and never lifts the unconditional
lowering refusal. Runtime composition, cell validation, presence codecs, aggregate
semantics, capability/binding coverage and native admission remain separate gates.

### Bounded JSON transport reader foundation

The private `json::checked_json_bounded` accepts explicit UTF8 byte, maximum
container/value depth and JSON value-node limits. Root depth is one and object
names do not count as values. It inspects raw nested JSON, refusing duplicate
decoded member names before constructing the final Value tree, then parses the
complete value. Byte/depth/node overflow returns WFT-LIMIT, duplicates retain
WFT-JSON-DUPLICATE and malformed JSON retains WFT-INPUT. Ordinary checked_json
keeps its existing depth-zero,128-depth/100000-node behavior.

This foundation has no runtime cell caller yet. It does not verify a cell
contract hash, row width, selected domain/constant, policy selection, current
authority or release. Raw container inspection allocates bounded-by-input-byte
raw entries before recursive node admission; no constant-memory or instruction
bound is claimed. The complete cell admission pipeline remains required.

### Pure owner-context cell correspondence

`SecurityBackendContext::check_result_cells` accepts the owner-corresponding
declaration, retained contract JSON, its SHA256 and a batch JSON string. It
rechecks declaration/source correspondence, verifies exact retained-byte hash
before parsing, requires structural equality with the complete declaration and
checks the batch's closed version/hash/rows envelope. Both JSON inputs use
32MiB/depth64/1000000-node limits and duplicate-member refusal. Rows are limited
to4096 and must match ordered column width. Every cell names a declared outcome,
uses its exact disposition and closed member set, and has a checked model-domain
value where required. Constant transformations additionally require exact admitted
domain equality. Null is a value only where that model domain permits it; it
never substitutes for Withheld or Absent. Scalar result domains remain outside
the direct-field declaration subset.

The shared one-million-visit/sixteen-million-byte literal ledger also charges
normalized numeric payload. Invalid cell/custody correspondence returns
WFT-SECURITY-RESULT; cell ledger exhaustion returns WFT-LIMIT in result phase.
Parser limit/duplicate/input diagnostics remain explicit.
This pure method returns unit only and issues no checked batch or release token.
It does not establish actual policy truth, authorized outcome selection, native
codec/source correspondence, current authority, atomic installation or guarded
release. Successful empty-batch checking does not grant query permission. Public
compiler lowering remains closed.

### Conditional supplied-truth cell selection

`check_simulated_result_selection` first checks full owner declaration/cell
correspondence, then requires one supplied scan/action/rule truth inventory per
row. Every actual owner scan and action must appear exactly, with every matching
rule supplied once. The existing source-checked composition fold must Permit
every scan/action; Unknown, Deny and Conflict refuse the complete check. Primary
actions compose the deduplicated projected fields for each actual scan occurrence.
Each ordered cell must select that field's resulting Original/Withheld or exact
constant transformation. Self-join occurrences remain distinct even for identical
revision-qualified field identities. The counted ledger bounds one million
visits and sixteen million charged bytes: ontology source bytes before each
composition, supplied rule IDs, initial projected-field identifiers, and selected
transform literals and normalized payloads after composition. A separate
composition ledger is shared across all rows and actions of this check. It bounds sixteen million normalized payload bytes and sixteen million
encoded disclosure-copy bytes, reserving normalization cost before allocation.
Neither ledger resets between rows. Other policy/model inspection is outside
these ledgers; they do not bound every instruction or allocation.

Selection mismatch returns WFT-SECURITY-RESULT-SELECTION; ledger exhaustion
returns WFT-LIMIT/result, while prior declaration/cell/source errors retain their
boundaries. The unit-returning method is conditional simulation only. Caller
truths are not evaluated conditions, authenticated facts, native cuts, credentials
or proof of row provenance. Empty batches grant no query permission. Actual
condition evaluation/correlation, original-value row correspondence, query-wide
authorization, compatible current authority and guarded release remain required.
No compiled artifact, checked batch, installer dispatch or release token is issued.


### Conditional evaluated scoped facts

`check_simulated_fact_selection` accepts the existing simulated cut and a closed
`{version:"weft.security.scoped-facts/0.1.0",rows:[{scanId:Fact}]}` envelope.
Each row supplies every actual owner scan exactly once with its exact target.
No caller action, output inventory or truth values are accepted. The existing
finite Record fact interpreter eagerly preflights all actual required rules,
evaluates each scan/action/rule condition from that one simulated population,
and passes only those evaluated truths to the conditional selection checker.
Original cells must equal their declared scan fact's normalized field value and
have complete declared field coverage. Repeated identities for bags/self-joins
are allowed only with identical normalized field/absence assignments, including
any occurrence in the subject or association population. Absence cannot be
invented from a missing field. Graph-witness expressions outside the existing
Record interpreter refuse. The combined cut/scoped-row input is bounded to four
million bytes combined; each cut/rows text separately has depth64/one million
JSON-node parser limits. Rows are limited to4096, with existing maxFacts /
maxSteps / normalization / copied-payload limits. Generated truth-map rule,
action and scan IDs are charged before cloning against a separate one-million
visit/sixteen-million-byte metadata ledger. These are selected ledgers,
not a complete CPU/allocator bound. Scope/coherence/metadata refusals return
WFT-SECURITY-EVALUATION/model; underlying fact literal/source diagnostics retain
their existing boundaries, including WFT-SECURITY-LITERAL. Prior cell and
selection errors retain their own boundaries. No prefix is returned after a later failure.

Caller trust, coverage and generation assertions remain unauthenticated. This
method proves conditional source-expression/selected-cell/value correspondence,
not native query predicates, join provenance, complete result enumeration,
query-wide original-action admission on empty batches, current authority,
production fact issuer integrity or guarded release. Empty batches grant no
query permission. No compiler lowering, installed artifact, checked batch or
release token is produced.


#### Private semantic allocation issuer experiment

The private owner semantic checker retains all complete selected capability IDs
for each actual scan/action bundle and the whole application, rather than a
first matching boolean. OwnerCoverage retains the immutable registered
declaration, actual context and exact selected IDs, including zero-edge and
empty-obligation selections. Scope and ID retention are charged;4096 scope and
edge limits complement the existing traversal ledger. No public constructor or
transport is added. This declaration-coverage issuer does not determine native
case sufficiency, obligation kinds or enforcement sites, authenticate registration
or native evidence, or authorize SQL emission. Required evidence matching must
retain original obligation identity, failure behavior and independently required
prerequisite edges in addition to capability/source/site/case relation. The
backend-profile requirement issuer and public/native gate composition remain
unfinished. Draft UMF SPIKE-010 allocation and assignment annexes govern this
experiment; source-current tested subsets and limits must accompany any claim.

#### Private source-demand binding experiment

The bounded private source issuer now derives canonical source-to-scope edges
from the same actual traversal that issues source identities. OwnerSourceDemands
borrows the exact OwnerCoverage, its scope keys and complete candidate sets;
no backend-supplied source parsing, public constructor or transport is added.
Original-authorized use keeps primary/original/Application demands; query fields
accumulate every actual action, outputs retain their expression scan, and global
source custody reaches all actual scopes without asserting semantic support for
unused source contents. Selected zero-edge capabilities remain in coverage.
Source/scope retention is capped at65,536 edges within a separate charged ledger.
The original source-only path skips demand-only comparisons. The eventual matcher
must choose one complete capability per scope covering all required sources and
retain six-coordinate obligation identities with independently issued profile
requirements. It remains unfinished; source-demand issuance does not choose a
capability, authenticate evidence, establish native enforcement or emit SQL.
UMF SPIKE-010 source-demand-binding annex governs the exact tested subset.


## Main integration identity migration — 2026-10-10

Main already owns CONTRACT-005 for authored paths and ordinary compile 0.3/0.4 schemas. This security contract is explicitly migrated to CONTRACT-006. The pending blocked security 0.3 and registered security 0.4 transports become `weft-security-compile/0.1.0` and `weft-security-compile/0.2.0`, both paired with the existing `weft-sql/0.2.0` grammar. Separate security request/response schema paths preserve ordinary main contracts. Historical receipts retain original source identifiers and do not qualify the merged source. Both security transports remain blocked before emission; registration does not authorize physical execution.


#### Private independent template expansion experiment

The `weft.security.requirement-templates/0.1.0` private trusted premise supplies
explicit typed applicability selectors, kind, owner, site, failure, cases and
same-occurrence prerequisite template references. The issuer consumes actual
OwnerSourceDemands plus typed RuleOccurrences, never manifest obligation payloads
or parsed canonical source names, and emits the separate required-instances0.2
inventory. Original identities include profile/template/source/scope/address and
exclude capability origin; all eligible assigned origins retain full case atoms.
Semantic templates matching a selector are globally mandatory, even if another
kind is present at that source. Selected deployment templates are independently
assigned per capability, including selected zero-edge capabilities. Every authored
capability must cover the complete declared profile case inventory through its
Selected templates. This case inventory is a trusted premise, not original backend
qualification or independently authenticated catalog completeness.

The current applicability subset is all18 owner event categories and exact typed
Rule effect/condition/operand/disposition variants at retained occurrence paths.
False/empty branches and withheld dispositions remain distinct from stored reads.
Payload-specific domain, transform, operator/output and backend predicates are not
implemented by these selectors; an authenticated complete backend catalog remains
required before physical admission. No aggregate semantic support follows from
owner lineage. The initial prerequisite subset is a same-selector, same-occurrence
DAG, bounded to64 traversal levels; cross-target instantiation is unsupported.
Unknown/missing mappings and dependencies, cycles, mismatched registration/target,
foreign cases and incomplete selected deployment catalogs refuse atomically.

Template retention has a separate1m-visit/16m-text phase ledger, with charges before
retained copies and aggregate4096 expanded atom links/originals/instances limits.
Rule enumeration has its own bounded phase ledger. These are not one aggregate
CPU/memory or allocator guarantee. Conditional matching checks the original decoded
contracts and every required kind at one whole eligible capability per actual scope;
source completeness cannot authorize a fragmented codec/privacy result. This
private boundary does not authenticate its profile, select a production backend,
emit SQL, establish native cases or grant authorization/publication authority.


#### Private original case catalog custody

A separate catalog-bound template result retains the exact original UMF
required-case-plan0.1 bytes under an independently supplied SHA256 pin and an
explicit trusted backend-label/registration-hash association. Catalog admission
requires all132 original IDs:12 shared semantic plus30 for each of pg-raw, Truss,
delta-raw and Ashlar. A selected backend retains42 complete original case records,
including assertion text/IDs, coverage, ordered command arguments and source lists,
timeouts, absent procedures/evidence and counterexample status. S10's additional
`S10:disclosure` assertion cannot disappear. Status and procedure presence do not
remove a required case or supply execution evidence. Commands are never executed
by this compiler boundary. The full original byte snapshot is preserved.

The bridge requires exact registered-source correspondence and all42 profile case
IDs before template expansion. It hashes only the immutable one-MiB-bounded owner
registration; foreign/unbounded profile strings refuse before hashing. Existing
private trusted-case issuance remains unchanged. Catalog-bound issuance retains
pending cases and cannot match original declarations with a shorter case inventory.
Its output is still conditional requirement custody, not native qualification.

The reader has separate bounded JSON parsing and retention work (1m visits/16m
text,4m input bytes,depth64). It rejects duplicate JSON keys, incomplete/duplicate/
foreign IDs, nonrequired cases, missing original assertion IDs, unknown selected
record fields and exhausted ledgers. Parsing temporaries are bounded separately;
these ledgers are not total process memory/CPU bounds. Expected source pins and
backend/registration association remain trusted host premises; an agreeing caller
hash cannot authenticate a profile. Template sufficiency, live procedure source
pins, native observations, installation/authority freshness and publication gates
remain required. This implementation does not execute the132 acceptance cases.

#### Authored deployment duty catalog candidate

The separate private `weft.security.deployment-duty-catalog/0.1.0` candidate
supplies42 distinct Selected qualification duties: all12 semantic cases and all30
original backend cases. The [authored golden table](../../../../crates/weft-core/tests/security-deployment-duty-catalog.txt)
records each original suffix, exact kind, owner, site and prerequisite suffixes.
All duties use failure `WFT-SECURITY-QUALIFICATION-REQUIRED`; each retains exactly
one original case record, including all that record's assertion IDs. S10 therefore
retains both S10 and S10:disclosure through the original pinned case catalog.
This is an explicit candidate qualification interface, not a claim that a case
label or this table defines all physical enforcement steps.

Template IDs are `deployment:<backend>:<original-suffix>`. Semantic cases retain
their original S identifiers; backend cases use the exact selected backend home.
The four original backend assertion sets have identical suffix assertions, so this
revision shares duty interfaces while requiring separate backend procedures,
physical mappings, native identities and evidence. Twelve semantic verifiers and
B12 receipt custody are Host/host duties; the other29 are Backend/native duties.
The81 explicit prerequisite edges form a same-Selected-occurrence DAG. Prerequisite
original identities instantiate separately at each original capability and do not
cross capabilities or targets. These proposed dependencies constrain qualification
composition; they do not establish sufficiency of any procedure or semantic proof.

The strict private boundary requires the exact42 authored Selected templates,
including kinds, owners, sites, refusal codes, single-case assignments and
prerequisites. Every authored capability retains all42 duties, including unchosen
or zero-edge entries. Extra catch-alls, omissions, home substitution and agreeing
manifest/profile weakening refuse. The returned private DeploymentIssued type
records this candidate version and is distinct from weaker catalog-only issuance.
Earlier experiments remain unchanged. Semantic templates are still independently
trusted inputs: this catalog does not implement payload-specific domain/operator/
transform applicability, authenticated registration selection or native execution.
No SQL/public transport is activated and every case remains pending.

Strict checking occurs after the existing charged profile preflight and consumes
the same issuer ledger. The42 fixed authored records each reserve512 text bytes
before map construction; all template counts, expected comparisons and capability
membership lookups charge visits. No comparison map is constructed from incoming
Selected template IDs. Those reservations bound copied authored text only, not
allocator metadata or aggregate process resources. Existing4096 template/capability/
expanded-atom limits and1m-visit/16m-text phase budgets remain.


#### Typed payload applicability candidate — template0.2

The private requirement-template0.2 experiment adds mandatory duty dispatch from
the actual borrowed field, key, query and association payloads. It preserves
source/scope/occurrence identities, scalar families, required/absent-allowed/
unspecified carrier nullability, facet-family presence, stored classification,
ordered key members and primary absent/false/true, operator kind and disclosed/
original-authorized mode. Direct outputs use the exact descriptor identity and
retain required versus absent-allowed availability separately from logical type
nullability; COUNT/SUM use their actual typed output families and facets. Ordered
association endpoint/member positions are independent duty occurrences. Selected
unknown variants refuse. All expansion uses the existing issuer visit/text ledger
and aggregate atom/contract limits; payload JSON is neither cloned nor normalized.

This is a presence/family applicability subset. Facet numeric values, allowed-value
members, constant/transform domains/literals/revisions, endpoint direction and
physical join semantics remain retained in owner payloads but are not qualified
by these selectors. No authenticated template completeness, exact native capability
compatibility or semantic-to-physical refinement is established. The admitted
required-direct-cells result profile does not thereby admit optional original Field or
nullable SUM output. Template0.1 keeps its earlier experimental behavior and
refuses payload selectors; DeploymentIssued still denotes deployment catalog
provenance, not payload or native qualification. Public security lowering remains
closed; all original acceptance cases and the full goal remain binding.


#### Rule domain/literal obligation candidate — template0.3

The separate private template0.3 experiment includes template0.2 dispatch plus
borrowed typed rule operand/disclosure dispatch. Each stored/context/constant
operand domain and constant-transform output domain emits mandatory role/scalar/
nullability/refinement-family duties. Constant operands and transforms also emit
exact scalar-wrapper-family and typed absence/nonabsence verification duties.
Only the selected source subset's constant transform revision0.1.0 is admitted
by this dispatcher; unknown transform names/revisions refuse before expansion.
Identity/endpoint terms retain their existing structural and actual key duties.

Rule duty addresses frame the exact already-charged RulePath under rule-payload
and include the individual payload address. Equal sides, populated false branches
and identical disclosures at different ordered positions remain distinct. The
fixed five-token stack buffer bounds address assembly before downstream charged
encoding. Direct rule/literal visits, wrapper text, transform/revision text and
all generated duty copies consume the existing issuer ledger. No payload JSON or
normalized literal is copied or reparsed by the dispatcher. Original source/type
admission remains responsible for exact values and refinements. Earlier
template0.1/0.2 paths remain unchanged and reject new rule-only selectors.

This closes occurrence-specific verification-demand extraction only. The exact
original domain/literal remains in immutable typed owner custody; a family/absence
selector is not a numeric-value equality proof or physical backend compatibility
check. Native interpretation, complete authenticated profiles, original case
procedure adequacy and complete backend qualification remain open. The existing
conditional requirement-matching laws are not a Rust or database refinement
proof. No public lowering or required acceptance promotion follows.

### Original-source diagnostic result inspection — 2026-10-13

The Rust-only `security_mapping_transport::inspect_security_mapping_result_cells` reconstructs original model/policy/query-profile/SQL context from the existing closed mapping request and applies the existing declaration/cell checks to a typed result contract plus exact contract JSON/hash and cell batch. Source parsing, parameter preflight and private context construction remain owner-controlled. Success is conditional correspondence, never backend registration, native provenance, selection, authentication or a release credential. The ordinary mapping JSON protocol and blocked public security compile boundary are unchanged. The separate PostgreSQL diagnostic example's single-column resultProbe is a fixture producer, not an admitted compiled response; mismatched output/source/provenance/batch refuses without stdout. Complete selection and native lifecycle obligations remain required.

### Original-source diagnostic conditional fact selection — 2026-10-13

The Rust-only `security_mapping_transport::inspect_security_mapping_result_facts` reconstructs the same immutable original request/context as the cell inspector and invokes the existing `check_simulated_fact_selection`. The separate diagnostic fixture accepts paired `resultProbe.factCut` / `scopedFacts` only; it exposes no caller truth/action override. Success checks declaration/cells, original per-scan/action rule evaluation, disclosure selection and original-value correspondence to declared facts. The original resolver assigns occurrence IDs (`s0` for the single scan), independently of SQL aliases. Missing/foreign scopes, denied selection or inconsistent fact/value assignments refuse before stdout. The caller's population completeness, trust and generation remain assertions. This is conditional fact/cell correspondence; native query filtering/join provenance, complete enumeration, authenticated issuer/principal/current authority, empty-batch query permission and guarded release remain unimplemented. No registered backend admission or release token is produced.

### Multi-column diagnostic cell batches — 2026-10-13

The diagnostic `resultProbe` additionally accepts `{columns:[{target,field,outputName}],rows,...}` with paired optional `factCut` / `scopedFacts`. It refuses mixing legacy single-column members with the new form; column objects have exactly three members, column count is1–256 and one-based positions are generated in order. Before repeated declaration parsing/copying it reserves `(serialized original request bytes + serialized renderer output bytes) * columnCount` within16,000,000 bytes. This is a conservative diagnostic source expansion bound, not a complete CPU/allocator proof; original declaration/cell/fact validators retain their own phase limits. Immutable owner query outputs govern exact column/source/name/order correspondence, cell arity, rule selection and original values. All output remains buffered until every column and row passes. This extends fixture batch correspondence only, without native caller authentication, SQL compiler provenance, complete enumeration or protected release authority.


### Original ontology subject selection — 2026-10-13

The source-bound Weft inspector reconstructs the original catalog, ontology, policy, SQL and admitted query profile, then checks the selected source SHA/Record/Key against the original ontology Subject. A closed source-only diagnostic returns `securityReady:false`; no native admission is produced. Truss requires this synchronous inspector before native observation or original UMF key encoding. Deferred async/generator functions are refused before invocation, including across realms; ordinary rejected Promise returns are observed before refusal. Arbitrary configured callback effects, hidden asynchronous wrappers and modified Promise constructors/species remain trusted-code premises. Original helper authenticity, actual subject existence/current binding, issuer authority and protected release remain unqualified. Retained evidence: `../../04-build/evidence/ontology-subject-selection-20261013/receipt.json`. Seven core transport tests, sixteen module controls, twenty-one host tests, thirteen native subject groups and eight predicate regressions pass; independent actual-binary review passes thirty-nine controls with every refusal producing empty stdout. Owner-only selection callbacks are test doubles; native fixtures use the original Weft inspector. No required backend case is promoted; all132 remain required.


### Subject-selection consumer transport — 2026-10-13

`inspect_security_mapping_subject_key_json(request, selection)` delegates the same original bounded source inspector through core, runtime, Python and WASM. The original mapping request and closed selection JSON remain separate strings; no alternate ontology/key interpretation is introduced. The closed response uses `interfaceVersion: weft-security-subject-selection/0.1.0`, `status: inspected|refused`, and `securityReady:false`; refusal additionally carries original owner diagnostics. Success carries no key bytes, caller identity, native admission, issuer proof or release credential. Python/WASM wrappers only delegate to the original runtime owner. Existing compiler security refusals remain normative.


### Compiler-owned checked PostgreSQL candidate lowering

`SecurityRegistry::prepare_postgresql_candidate(context, capability_ids, allow_candidate)` is a Rust-only preparation boundary. It selects the exact retained original registry/target, requires the supported PostgreSQL18.6 declaration and original semantic coverage, and derives original source demands/occurrences before invoking the trusted pure lowerer exactly once. Failed source/profile/capability admission invokes it zero times. Lowering may not change selected capability identities. The complete returned parameter/query-slot, result declaration and semantic-source obligation coverage is checked before any prepared object is returned. Malformed native placeholders, changed output declarations, omitted obligations or substituted/duplicate capabilities refuse atomically.

The return `CheckedSecurityCandidate` has no public constructor/deserializer. It owns the declaration and retains exact original manifest JSON, compiler-produced physical-plan/result-contract JSON plus their SHA256 values, and the original owner context fingerprint. Getters borrow these immutable snapshots. These hashes establish byte correspondence only. They do not authenticate an issuer, source, principal, installed native inventory or current authority. This preparation boundary does not issue a native credential, expose a compiled response or lift either public security activation refusal. It provides the compiler-owned snapshot for the later authenticated native binding and guarded release implementation; all original132 required backend cases remain binding.


### Reified graph source and owner query candidate

`GraphSource` is a closed pure Rust carrier with version `weft.truss.security.graph-source/0.1-candidate`, separate `physicalModule: ModuleInput` and `nativeSchemaRevision`, and original `inventoryJson`, `correspondenceJson`, `logicalResolutionJson`, `logicalTransportJson` strings. Authored physical document revision `r1` and native counter `1` are distinct domains. Original logical domain pins stay unchanged. The carrier resides in exact original `bindingJson`; the query profile binds those bytes. Source references, literal scalar domains, authored key fields and native relationship declarations are checked against original owner implementations; same labels never equate opaque key/namespace bytes. Current carrier applicability is the explicitly named reified-graph fixture with singleton string keys/endpoint paths and JSON scalar homes; it is not a generic storage or native authorization profile.

Every endpoint path emits a complete exactly-one typed incidence/key-cell guard over all association nodes, including inactive assignments. Source/cardinality/schema-counter drift refuses; physical accepted authored revision/current custody must additionally be verified by the native host. The new `with_security_mapping_context` callback constructs the actual immutable owner source, profile and resolved query, with no context deserialization or native authority.

`prepare_graph_query_checked` returns an immutable, owner-checked `GraphQueryCandidate` with typed result contract and original result bytes/hash. The current direct scalar subset rejects joins, filters, aggregates, groups, order and limit rather than silently dropping them. Subject locator is a separate symbolic int8 slot supplied only by the genuinely authenticated guarded host, never model/request values. Policy parameters keep their original ordinals/types. Mandatory endpoint, subject and indeterminate-truth checks precede the original execution SQL.

Build `cargo build --locked -p weft-postgresql --example security_graph_candidate --example security_graph_result`. Candidate stdin is the original closed `weft-security-mapping/0.1.0` request with bound GraphSource; output remains `securityReady:false`, `nativeAdmission:required`. Result-check stdin is exactly `{version:"weft.truss.security.graph-result/0.1-candidate",sourceRequestJson,sourceOutputJson,batchJson}`. It reconstructs the original producer/context and requires exact original whole stdout bytes including its final newline, then delegates complete cell checks to the original owner typed contract. Substituted SQL/contract/context, invalid outcome/arity/members and extra input fields refuse with empty stdout. Its `inspected` result is correspondence only, not proof of native row selection, principal, current authority or guarded release. Native source/caller/role/fact closure and original132-case acceptance remain mandatory before public activation.


## Original parameter-source inventory — 2026-10-13

`SecurityBackendContext.parameter_sources()` issues a borrowed, immutable `SecurityParameterSources` from the actual admitted catalog, logical policy, resolved query and query profile. It is a pure Rust prerequisite for guarded physical lowering, with no constructor, deserializer or execution permission. No public request/response, backend registration version or activation gate changes. Existing `SecurityPhysicalPlan`0.1 and its exact original-query slot checks remain unchanged; policy operands and native subject locators cannot be inserted as invented query values.

The inventory keeps these sources distinct:

- Query literals and parameters retain every actual predicate occurrence, exact source path and original scalar/exact value object. Field-to-field uses contribute no parameter value.
- Condition constants borrow their original field/domain/literal, including explicit null and exact numeric token spelling. Context operands borrow field/domain but have no compiler-supplied value. Constant transformations borrow their original output field/domain/literal.
- The subject demand retains the original logical subject Record, authored key ID and ordered authored key fields. It contains no caller, native role/database identity, principal mapping, physical locator or credential. Those values require genuine host/native issuance under the admitted cut.

Policy source paths address the expanded original typed logical rule inventory: `logicalRules/{index}/condition` followed by actual `args/{index}`, `arg`, `where`, `left` or `right`; constant disclosures use `logicalRules/{index}/disclosure/{index}/transformed`. These paths identify occurrences inside this original owner context, not authenticated source labels. `require_context` compares original catalog/plan/query/profile object references plus exact binding/backend/target identity and rechecks source custody. Equal digests or newly reconstructed equal source bytes do not substitute another owner context.

Issuance traverses with at most65536 charged steps,1MiB cumulative retained/traversal path text,4096 combined query/policy/context parameter sources and64 subject-key components. A path is at most4096bytes. Exhaustion refuses the whole inventory, with no partial returned sources. Original literal/domain/query objects are borrowed, avoiding another payload copy; these ledgers do not claim a total core heap/CPU bound or qualify native codecs. Physical ordinal assignment, policy/context/native slot correspondence, registered guarded lowering, source/subject issuers, native execution and release remain unfinished. All132 original required cases and B-008 remain open.


## Graph policy ordinal/source correspondence — 2026-10-13

`GraphQueryCandidate.parameter_bindings()` now exposes immutable typed metadata checked from the original core parameter inventory. The predicate renderer records condition and transformation occurrence paths internally while assigning its actual parameter ordinals. The candidate checks every recorded path, original value and physical codec against `SecurityBackendContext.parameter_sources()` before retaining output. String/boolean condition constants use text/boolean carriers; constant transformations retain their exact CoreLiteral JSON payload. Equal values at different occurrences retain different source paths and ordinals. Unsupplied context operands cannot become constant values. This direct-projection candidate still refuses query operations and cannot drop or reclassify any original query value.

The subject binding remains a separate unvalued demand: original logical subject Record/key/ordered fields, checked graph native type ID, final ordinal and int8 physical locator type. It supplies no caller, role/database identity, principal mapping or locator value. Genuine Truss subject/source issuers under current held cuts remain mandatory. Private constructors and absence of deserialization prevent caller replacement of checked typed bindings; these bindings are source correspondence metadata, not execution or release credentials.

Policy source-path retention is bounded to1MiB and4096bytes per path. Parameter count remains1024. A separate1MiB serialized field/domain/path/subject metadata ledger charges before source metadata clones. These are finite component ledgers, not a total resource or native-codec theorem. Original declared source/semantic coverage, guarded physical plan registration, compiler/native admission dispatch and all132 required cases remain open.

The existing candidate producer/result JSON and SQL are unchanged for the retained original inputs: new provenance stays inside the Rust candidate until a separately governed guarded transport is defined. No existing request/response, backend manifest, source version, capability status or public activation gate is upgraded. [Evidence](../../04-build/evidence/security-graph-parameters-20261013/receipt.json) distinguishes typed source/ordinal controls, original producer-byte comparisons and native authority work.


## Installed source comparison separate from query lineage — 2026-10-13

`SecurityBackendContext.source_context()` issues an immutable `SecuritySourceContext` with original source JSON and its SHA256. The closed carrier is `weft.security.source-context/0.1-candidate`, governed by `security-source-context-v0.1-candidate.schema.json`, and always has `securityReady:false`. It retains exact ordered model inputs/pins/selections, original policy and ontology bytes, original query-use profile bytes, opaque binding bytes and backend/version/target identity. It preserves opaque source content; it neither strips nor interprets query-like or authority-like content embedded inside opaque strings.

The capsule has no top-level SQL, parameter values, result contract, principal, epoch or permit. Those query-specific facts retain the existing original `execution_query_context_sha256` and immutable result/parameter lineage. Different admitted SQL, projection order, alias, predicate, aggregate or parameter values can share source bytes while retaining distinct query contexts. Sharing sources does not admit previously unsupported lowering or query operations. Changes to retained source bytes, including semantically equal whitespace with updated source pins, are distinct source capsules.

`require_sources(context)` rechecks original owner source custody, reproduces the exact capsule and compares bytes; a digest alone is not accepted as native source or actor authority. Private fields, no public constructor and no deserializer prevent mutation/reconstruction of this Rust checked value. A newly admitted owner context with identical source bytes can correspond to the same capsule; per-query borrowed inventories still require their own actual owner objects.

Serialization charges a16,000,000-byte UTF8 output ledger before extending the retained buffer; exact-bound success and over-bound refusal return no partial capsule. This is a serialized-byte bound, not a total heap/CPU theorem. The capsule does not register a backend, authenticate an installed source/caller, attest a current generation, qualify native codecs, issue a credential or authorize execution/release. Existing compile and manifest validators reject it as their input.

Registered guarded dispatch must compare this original source capsule with genuinely installed/authenticated source state under held native cuts, then bind each actual query context, typed plan, native subject and result contract separately. A new admitted query against unchanged installed sources must not inherently require a source-authority epoch transition. The existing exact-query native fixture comparisons stay intact until that wider dispatcher is implemented and tested; no authority check is relaxed by this separation. B-008 and all132 original required cases remain open.


## Separate guarded graph handoff — 2026-10-13

`prepare_guarded_graph_candidate(raw_mapping_request)` is a pure Rust owner preparation API with a private-constructor `GuardedGraphCandidate`. The request remains the original closed `weft-security-mapping/0.1.0` plus original GraphSource binding. No native caller, subject value, epoch, permit or admission boolean is accepted. New `security_guarded_candidate` and `security_guarded_result` examples expose this original pipeline as bounded compiler-only processes. They perform no database execution or publication.

The distinct `weft.truss.security.guarded-candidate/0.1-candidate` output retains original request-byte SHA, exact source capsule JSON/SHA, separate query-context SHA, original owner-checked query/guards/result contract, typed policy source/value/codec/ordinal correspondence and a separate unvalued native subject/key/type/ordinal demand. Source fields and domains come from the private core inventory, not caller plan annotations. `securityReady` is always false and `nativeAdmission` is required. Existing backend manifests/registrations, public compiler gates and legacy graph producer/result protocols remain unchanged. The closed envelope/schema is `security-guarded-candidate-v0.1-candidate.schema.json`; its nested query is the existing original-owner declaration, not a new caller plan admission schema.

The candidate's `json()` is exact canonical producer output including one final newline. Serialization charges16,000,000UTF8 bytes, reserving the newline before buffer writes. Original request input is bounded to4,000,000bytes. The metadata and query retain their original separate source/parameter/result bounds; these ledgers do not claim a total heap/CPU theorem. The subject demand contains no runtime value and requires genuine host/native issuance under held cuts.

`inspect_guarded_graph_result(raw)` accepts the closed `weft.truss.security.guarded-result/0.1-candidate` input with original request bytes, exact original output bytes and complete batch JSON. It regenerates the complete candidate from actual original owner source/query context, requires byte-exact output, compares exact source capsule and query context, and invokes the original immutable result-contract cell checker. It returns inspected source/request/output/query/batch comparisons with `securityReady:false`. The input is at most32,000,000bytes, original output at most16,000,000bytes and batch at most4,000,000bytes. Extra authority fields and wrong versions refuse. The input/output schemas are `security-guarded-result-input-v0.1-candidate.schema.json` and `security-guarded-result-output-v0.1-candidate.schema.json`.

Inspection checks correspondence only: it does not establish native population, rule/row selection, installed source authenticity, actor/current authority, codec execution, permit or release ordering. Native dispatch must consume the original trusted compiler handoff under genuine source/Subject/authority cuts, execute every guard, check complete result lineage, and retain protection through settlement/publication. The legacy inspector refuses the new output and the new inspector refuses legacy output. All132 original required cases, full supported SQL operations, registered guarded dispatch and public security activation remain open.


## Per-scan query-use guards — 2026-10-14

`prepare_graph_query_use_guards` derives private immutable guards from the actual owner context and exact GraphSource. Each original scan retains its target, required action groups, original rule truth expressions and every predicate/order/group/join/aggregate field occurrence, including query-only fields. Both sides of a join retain separate scan guards. Prohibited use refuses during owner admission. Original-authorized use checks the exact designated action; an absent permission group refuses for primary-permitted rows. Disclosed use checks the composed disposition; withheld, conflict, refusal and SQL NULL refuse rather than becoming a filtered-out row. Primary indeterminacy refuses; primary-denied rows do not require an original-action grant.

These are guard expressions for future per-scan private relations, not relational lowering or native admission. Consumers must evaluate them before query filters, joins, ordering, grouping, aggregates and limits, retain complete native scalar/presence checks, and bind the original rule aliases locally to each scan. The existing direct-projection candidate still refuses unsupported operations. The work/text ledger bounds guard construction; it is not a total heap or CPU proof. [Evidence](../../04-build/evidence/security-query-use-guards-20261014/receipt.json) records seven guarded tests, six graph regressions, privacy and boundary checks. No native case is promoted; B-008 remains open under all132 required cases.


## Aggregate result-domain candidate — 2026-10-14

The separate `weft.security.result-contract/0.2-candidate` and `weft.security.cells/0.2-candidate` add a computed outcome for actual resolved COUNT/SUM outputs. COUNT has no authored source Field and is a nonnegative, non-null exact integer. SUM retains the exact original argument Field identity/revision and actual resolved output family, scale and nullability. Aggregate domains never inherit an input Field range, allowed-values set, integer width or decimal precision. Empty ungrouped SUM can be null. Grouped SUM remains non-null only for required non-null original operands; original absent/null inputs and null constant disclosures make the security result domain nullable, as derived from original sources and selected operand mode.

`check_aggregate_result_declaration` checks complete output positions/names, exact COUNT/SUM lineage and exactly one computed scalar outcome per aggregate. Mixed direct Field columns retain the existing original disposition/domain correspondence. `check_aggregate_result_cells` checks complete batches against exact retained contract bytes/hash and original owner output domains. Exact integer/decimal tokens remain strings; no floating-point conversion is introduced. Decimal normalization uses an internal resource ceiling rather than input precision; normalization remains subject to the existing work/byte ledger.

The original declaration/cell methods and both existing generic/PostgreSQL candidate registration checks reject this separate version. Existing guarded graph transport/result inspection remains direct projection and is unchanged. Pure cell/domain correspondence does not prove arithmetic, input selection or query-use authorization: native dispatch must evaluate all per-scan guards before relational operations, validate original operands, compute actual aggregates, and retain genuine source/Subject/current-authority cuts through release. Simulation of a direct output row is not aggregate provenance. No native acceptance case or public activation is promoted.


## Separate relational parameter channels — 2026-10-14

`prepare_relational_graph_parameters` derives a private immutable owner-bound parameter inventory for future relational graph lowering. Original policy values retain their checked source paths, domains, codecs and ordinals; the native Subject demand stays unvalued at the next ordinal. Every actual scalar SQL literal/parameter occurrence follows it in a separate query channel with original source path, borrowed resolved value/type and selected Boolean/Text/Numeric codec demand. Repeated named values retain distinct actual occurrences; equal query and policy values never establish interchangeable provenance. Query-only filters and keyset values remain query inputs, not output columns or Subject assertions.

The inventory requires the actual context object, not an equal reconstructed request or digest. It validates exact GraphSource correspondence and original native graph declaration before retaining channels. At most4096 combined policy/Subject/query slots and1MiB query-path metadata are retained; policy rendering has its existing1MiB value and metadata budgets. Query values/types are borrowed from original immutable owner objects rather than replicated into parameter metadata. These ledgers do not bound total heap/CPU or qualify native codecs.

Legacy parameter issuance still rejects every actual query value and the original direct-projection candidate still refuses filters, joins, grouping, ordering, aggregation and limits. The separate inventory emits no SQL or external transport, registers no backend, supplies no native Subject value, and authorizes no execution/release. Relational lowering must consume every channel, validate exact native operand domains/presence and apply all actual per-scan query-use guards before relational operations. No native case or public capability is promoted.


## Original relational SQL candidate — 2026-10-14

`prepare_relational_graph_sql` emits original scans, field-equality joins, equality/keyset filters, grouping, COUNT/SUM, ordered projection, ordering and limits against the explicit reified GraphSource. The private immutable program retains actual original context, parameter channels, result contract, mandatory checks and native operand demands. Per-scan truth relations are materialized before eligible relations; every scan retains its actual action/query-use guards independent of query filters or limits. Parameter type anchors retain all original ordinals in every statement. User values remain parameters and never enter SQL text.

Direct projections preserve composed cell envelopes. Aggregate cells use the separate computed contract/encoding. Group keys include an absence discriminator; missing and present-null operands are not silently merged. Grouped projection and order checks refuse nonfunctional disclosure/order values within a group before choosing a representative cell or ordering key. NULL participates in order-consistency checks. Boolean aggregate ordering uses bool_or, not an unavailable PostgreSQL min(boolean). Disclosed transformations with a different scalar family refuse rather than silently produce a missing scalar wrapper. These explicit refusals do not redefine the full B-008 acceptance scope.

This is SQL declaration correspondence, not native admission. The guarded native consumer must validate complete original/disclosed operand domains and presence, native parameter codecs, actual installed source/caller/Subject/current authority and all mandatory checks under held cuts before evaluating relational SQL, including group-consistency SQL. It must verify complete result lineage and retain protection through publication. Operand demands retain original model domains and disclosure sources; they are not caller assertions or proof that validation occurred. The existing registered/direct guarded path does not accept this program.

The bounded `security_relational_probe` example emits a separate `weft.truss.security.relational-probe/0.2-candidate` diagnostic with securityReady:false. Original mapping inputs are bounded to4M bytes and output to16M bytes before stdout; extra authority inputs remain rejected by the original owner transport. It is not a guarded execution transport. The controlled PostgreSQL18.6 probe uses an isolated Unix-socket cluster, synthetic rows, original Truss scalar helpers and an excluded administrator. Seven probes cover all five operators in one query, COUNT/SUM, keyset ordering, hostile bound strings and withholding/missing-grant refusal before an empty filter. This partial layout does not prove installed source authenticity, independent caller/current authority, native input validation coverage or release; no132-case entry is promoted.


## Original operand value/presence checking — 2026-10-14

The separate `weft.security.operand-values/0.1-candidate` batch retains an actual original query-context SHA and every original scan. Each supplied primary-permitted row has a distinct native row locator and one cell for each actual indexed operator occurrence on that scan. Native locators are tracking identities, not authored logical keys. `check_operand_values` derives all expected indices/modes/domains from the actual owner context. It checks complete supplied row widths, original literal domains/refinements and known absence. Disclosed originals must be possible in original classification/rules; constant transformations retain an actual original permit-rule ID and normalize against its original output Field/literal. Original-authorized inputs cannot be reclassified as transforms. Withholding, unknown states, duplicate locators, missing occurrences, foreign contexts and extra authority fields refuse.

Unbounded model integers remain unbounded subject to the existing resource ledger; no signed64 restriction is invented by this check. The batch is bounded to4M bytes,4096 rows per scan,64 JSON nesting levels and the existing1M work/16M normalization ledger. The private immutable receipt retains only context/batch hashes and row counts. It proves supplied value/presence correspondence, not native population, original value equality, policy truth/selection, source authenticity, caller/current authority, codec support, execution or release. A digest or a caller-supplied valid batch is never an authority credential.

Relational candidates now issue original operand-read SQL for each scan, before numeric field casts, query filters, grouping or limits. Reads retain every actual occurrence and transform source, distinguish absent from present-null, and use a4097-row sentinel against the4096-row checker bound. Native hosts must run source/Subject/action/truth admission first, capture a complete current population under held cuts, check the original batch and native codecs, then run group-consistency and relational SQL. The unqualified diagnostic probe includes these reads; the separate `security_operand_values` example checks original request/batch correspondence and always returns securityReady:false. Existing guarded dispatch remains unchanged.

Eight controlled PostgreSQL18.6 probes now route actual captured rows through the original core checker before relational evaluation, including a malformed later salary hidden by an empty filter. These are excluded-administrator partial-layout probes; no native source/caller/current-authority or guarded publication claim, and no132-case entry, is promoted.


## Separate guarded relational handoff — 2026-10-14

`prepare_guarded_relational_candidate` derives a private immutable `GuardedRelationalCandidate` from original closed mapping input. The distinct `weft.truss.security.relational-candidate/0.2-candidate` retains exact request-byte SHA, original source capsule bytes/SHA, separate query context, actual relational SQL/checks, operand reads/demands, original policy/query parameter channels and an unvalued native Subject demand. It retains the original result contract JSON/SHA and always declares securityReady:false. Native admission, codecs, population/selection and release protection are required. Original query values are never converted into Subject assertions or policy operands.

The closed `weft.truss.security.relational-result/0.2-candidate` input supplies original request/output bytes, operand batch and complete result batch. `inspect_guarded_relational_result` rebuilds the complete original producer, requires byte-exact output including its final newline, rechecks original source/query custody, then validates every supplied original operand and result cell using their distinct owner contracts. It returns inspected hashes/counts with securityReady:false, not a permit. Correspondence cannot prove native capture completeness/authenticity, rule truth/selection, arithmetic, source/caller/current authority or protected release.

Input is at most32M bytes, original request4M, producer output16M, operand/result batches4M each and result-contract JSON4M. Output buffering charges UTF8 bytes before extending its retained buffer and reserves the newline; no partial output escapes on refusal. This is an output ledger, not a total heap/CPU theorem. Closed envelope schemas are `security-relational-candidate-v0.2-candidate.schema.json`, `security-relational-result-input-v0.2-candidate.schema.json` and `security-relational-result-output-v0.2-candidate.schema.json`. Nested SQL/check/parameter declarations are original-owner outputs, not caller-plan admission schemas.

The separate `security_relational_candidate` and `security_relational_result` examples expose bounded pure compilation/inspection. Existing guarded producers/inspectors and original result registrations reject the new versions; no fallback or public activation is introduced. Tests and12 CLI controls refuse changed SQL/source bytes/ordinals/Subject values, missing newline, wrong encoding/version, positional arrays and injected authority. Genuine guarded native dispatch must independently satisfy all required obligations under held cuts and protect settlement/publication. B-008 and all132 original required cases remain open.


## PostgreSQL18.6 raw-cast codec correspondence — 2026-10-14

`check_postgresql_parameter_codecs` derives actual original policy/query channels and checks their Boolean, UTF8 text and core-literal JSONB representations before any SQL, including type anchors and operand captures. Subject remains unvalued; genuine native issuance must qualify its int8 locator. `check_postgresql_operand_codecs` first applies original model-domain/presence checking, then checks the supplied operand representations. The private receipt retains context and checked counts, not native authority.

Exact JSON-style numeric tokens are analyzed without floating-point conversion or decimal expansion. PostgreSQL18 numeric stores up to131072 digits before the decimal point and16383 after it; its source parser bounds exponent magnitude to INT32_MAX/2 and normalizes zero weight while retaining display scale. The checker follows those raw-cast limits, including large-exponent zero and trailing fractional zeros. Valid UMF values outside this codec profile refuse with a native-codec capability diagnostic; model domains remain unchanged. UTF8 text and JSONB strings/keys containing NUL refuse rather than being sanitized.

Sources: PostgreSQL18 [numeric documentation](https://www.postgresql.org/docs/18/datatype-numeric.html) and [numeric.c](https://github.com/postgres/postgres/blob/REL_18_STABLE/src/backend/utils/adt/numeric.c). Ten actual PostgreSQL18.6 isolated type probes match the selected boundaries. This qualifies value representation, not actual session/server settings, source/caller/current authority, native population/truth, arithmetic-range safety, result selection or release. Aggregate intermediates/results and authentic native target/session admission remain required.

The separate relational producer now checks original parameter representations before issuance; its result inspector checks supplied operand representations. Wire versions and bytes for existing admitted values are unchanged. `security_postgresql_codecs` provides a bounded pure checker for original requestJson and optional operandBatchJson, always securityReady:false. The guarded host must invoke parameter checking before any SQL and operand checking after genuine capture but before group-consistency/relational evaluation; post-result inspection alone is insufficient. No original registration/public activation or132-case qualification is promoted.


## Security SUM nullability from original operand sources — 2026-10-14

`aggregate_result_type` derives the separate candidate result domain from the actual immutable owner output occurrence, selected aggregate operand mode and original model/policy sources. It rejects a caller-reconstructed output object. Grouped SUM can be null when original input Fields allow absence/null or a selected disclosed constant can be null; COUNT remains non-null and nonnegative. Required non-null original inputs retain the existing non-null grouped result. Family and scale stay original; input width/range/precision refinements are never copied onto a sum.

The security-only catalog admits optional scalar domains for query operators while retaining exact original source bytes and separate availability semantics. Ordinary catalog preparation stays on its existing subset; old result/registration/public activation refusals stay intact. Known absence remains distinct from present-null in operand captures and grouping keys. The aggregate declaration and emitter use the same owner-derived result type, and complete result-cell checking accepts a legitimate grouped-null SUM without inventing a replacement value. Tests cover source absence/null, null constant disclosure, required grouped-null refusal, changed declared nullability and copied-output refusal. Native null/absence grouping and result parity remain separate qualification work; no132-case entry is promoted.


## Controlled native absence/null aggregate parity — 2026-10-14

The portable SQL probe now authors corresponding optional logical/physical salary Fields and native inventory, updates original document/resolution/source pins and retains original request custody. Actual PostgreSQL18.6 rows include one missing salary and one present-null salary. Controlled queries prove grouped original SUM returns null, missing and present-null form distinct grouping keys, and disclosed-original grouped SUM also returns null. Each successful query's actual operand captures and complete result batch pass the original core/guarded relational inspectors against exact producer bytes and source/query context.

Eleven controlled probes pass, including the prior withholding/missing-grant/malformed-later-value refusals before evaluation. The partial synthetic layout and excluded administrator still do not qualify native source/caller/current authority, target/session custody, capture completeness outside the fixture or protected publication. No132-case entry is promoted. The saved runner requires both the pure probe/operand inspector and guarded producer/result inspector binaries; new evidence is separate from historical probe receipts.


## Explicit guarded check execution stages — 2026-10-14

Every original relational check now carries a closed `stage`: `beforeOperandCapture` for integrity, Subject and query-use/truth checks; `afterOperandValidation` for grouped disclosure/order consistency. The latter statements evaluate relational operands and filters, so executing them before original operand-domain/presence and native codec validation is forbidden. Parameter codec and genuine native source/caller/current-authority admission still precede every SQL statement. Capture follows the initial checks; original value/native codec checking precedes the later checks and execution; result inspection and late native checks precede protected publication.

The separate0.2 producer retains stages in its exact canonical bytes. Changing a stage fails original producer correspondence, and the candidate envelope schema requires one of the two stage values. Native hosts must reject missing/unknown stages and enforce ordering, not infer it from error strings. Existing direct0.1 output/checks remain unchanged; updating0.2 requires the matching original compiler/binary pin. The controlled runner consumes the explicit stages and retains malformed-later-value refusal before any grouped or relational operand evaluation. Eleven controlled probes pass; no native authority/release or132-case entry is promoted.


### Explicit native Python relationship candidate bridge — 2026-10-15

The separate `truss-security-relationship-candidate` Cargo feature in runtime and
Python exposes `prepare_truss_security_relationship_candidate_json(request)` and
`inspect_truss_security_relationship_result_json(request)`. Both invoke the same
original Rust producer/inspector used by the separate relationship binaries.
Python accepts and returns original JSON strings; refusals raise ValueError with
original diagnostic code/message. The existing 4MB producer and 32MB inspector
input bounds, closed envelopes, exact producer-byte reconstruction, query/policy/
unvalued Subject parameter channels, codecs and result obligations are unchanged.
No new protocol, installed backend, caller identity or execution callback is added.
Ordinary builds, including the ordinary PG18 feature, omit these two methods.
Existing security mapping and compiler registrations remain closed to this profile.

Fresh macOS arm64/Python3.12.14 installed-wheel checks compare exact Rust/Python
candidate and inspector outputs for both original permitted and observation-denied
sources. SQL/newline/source/version/authority overrides, missing source/terminal
scan declarations, mismatched result hashes, duplicate JSON keys, malformed and
oversized envelopes refuse. Empty supplied batches only exercise correspondence:
they do not establish native population completeness, source truth, original
operand capture, current authority, authenticated Subject or guarded publication.
This binding increment promotes no original backend case and does not activate
public security compilation. Native enforcement and full B-008 remain open.


### Original AST query property candidate — 2026-10-15

The separate `security_mapping_transport::prepare_security_query_properties`
function and `weft-core` example `security_query_properties` consume the existing
closed original mapping request and emit `weft.security.query-properties/0.1-candidate`.
The closed schema is `security-query-properties-v0.1-candidate.schema.json`.
No existing QueryUse, mapping handoff, candidate SQL or registered compiler wire
is extended or reinterpreted. The report has securityReady:false and authority:false.

Each actual resolved expression position retains its path, scan occurrence,
qualified target/Field, original revision-qualified Field identity and logical
type. Equal predicates and both sides of join equality require equality;
lexicographic comparisons and ordering require order; grouping requires equality.
Equal-valued or repeated Field expressions remain distinct occurrences. Hidden
Fields used only by filters/order/join/group remain dependencies. Source scans
survive field-free COUNT(*); contradictory predicates do not erase dependencies.
Directed one-hop relationship terminal keys derive equality at their own tN scan
when the owner invokes derive with an admitted relationship context.

SUM retains actual argument/output numeric type and the resolved mode/action,
but required:null/propertyStatus:unsupported. No numeric transform property is
invented or classified as equality. The actual legacy mode and originalAction
are marked modeSource:legacy-ontology-query-profile. Full exact source capsule,
SQL, source/query hashes and pins retain their provenance. This report does not
resolve the current UMF catalog, canonical stored Field or storageTransform:
the original catalog/owner consumer must resolve those independently. A selected
compatible transform property never supplies a read/native permit.

Derivation refuses missing original requirements and stale identities; caller
property lists, paths, modes or actions are not accepted inputs. Work retains at
most4096 dependency occurrences with a conservative text charge and16MB final
serialized report ceiling. The ceiling is an endpoint/output bound, not a
preallocation, CPU or whole-compiler resource theorem. Existing4MB request and
source admission limits remain. JSON facets remain opaque original type content.

Actual parser refusal of COUNT(DISTINCT), LIMIT0 and WHERE FALSE remains explicit
in this first candidate. They require language/IR and null/duplicate/zero-result
conformance, with no optimization that bypasses permissions. Numeric SUM rules,
current catalog/storage-transform source consumption, effective disclosure and
original action/current-authority/native before-dispatch checks remain required.
This component does not close P03/B08, promote an original native case, change
any case assertion/allocation, or activate protected execution.


Current-source dependency: this first report requires original legacy0.1 ontology/
query-profile admission. Current UMF catalog/ontology0.2 source bytes must never be
rewritten into a synthetic legacy policy to obtain this context. A later explicit
current-source admission or policy-independent application-AST lineage surface
must derive requirements from the actual current query/model, retain unknown
source bytes and omit unestablished modes/actions. This report does not establish
current catalog compatibility by carrying a legacy-source mode label.


### Policy-independent application lineage candidate — 2026-10-15

The distinct `application_lineage::prepare` API and `application_lineage` core
example accept `weft-application-lineage/0.1-candidate` and emit the closed
`weft.application.lineage/0.1-candidate` artifact. Request/report schemas live in
`application-lineage-request-v0.1-candidate.schema.json` and
`application-lineage-v0.1-candidate.schema.json`. A private immutable result is
constructed only by resolving the original pinned core0.8 modules and SQL through
the existing application parser/resolver. No SecurityLogicalPlan, policy0.1 adapter
or synthetic legacy policy is constructed. Old registrations/transports stay
separate. No executable plugin, SQL override or caller AST/required/mode/action
input is accepted.

A shared internal traversal of the resolved application AST derives original
occurrence paths, scan/Field identities with revisions, equality/order properties
and actual numeric SUM argument/output types. It is used by both the earlier
legacy property report and this new source-only report. Legacy output bytes must
remain unchanged. Self-join sides and repeated expressions are distinct, hidden
Fields retain uses, terminal keys retain their own tN equality occurrence, and
COUNT(*) preserves source scans with no invented Field dependency. Unsupported
SUM properties remain required:null/propertyStatus:unsupported. ASC/DESC direction
and required capabilities are preserved consistently; unsupported descending
cursor combinations refuse. COUNT(DISTINCT), LIMIT0 and WHERE FALSE still require
language/IR/conformance work and remain refused.

The report retains exact sourceRequestJson/hash, original model pins, SQL hash,
actual applicationPlanJson/hash, scan identities and uses. It contains no mode,
modeSource or originalAction. Optional external opaque source records retain
role, declared document/revision identity, media type, exact UTF8 bytes and digest
inside the original request. Their status is always uninterpreted; matching a
supplied digest proves local byte correspondence, not ownership, current policy
meaning, storageTransform, effective disposition, original action, key/runtime
scope, authentication or native permission. Unknown opaque source content is never
parsed as a compiler override. Original model content remains retained; selected
unestablished core element meaning still refuses.

Limits are4MB input bytes,16 opaque sources,4096 dependency occurrences,8MB
parameter replication/retained-use/serialized-plan budgets and16MB serialized
report output. The bounded writer refuses without partial output. These are
explicit endpoint/component bounds, not a whole-process allocation/CPU theorem.
The pure parameter preflight is reused from the original mapping transport.

The UMF owner's explicitly new named-current-stored-token-r1 fixture is retained
unchanged with source SHA2216a7ad3093c90437db035fb337d21ac67589dee5eb148b0eb4d7fd1ff75059.
Its current policy0.2, ontology0.2, catalog0.1 and storage owner values are extracted
with recorded compact UTF8 JSON encoding; the exact whole fixture bytes are also
retained as an opaque source. Five actual SQL scenarios independently derive
operation/property lineage, including selfjoin/group, output-omitted token order,
unsupported numeric SUM and field-free COUNT. Expected stored uses remain authored
test expectations, never producer inputs or attestations. Opaque expectation
mutations cannot change AST uses. The original unnamed UMF fixture is preserved
as a refusal: Weft never adds names/members to make it queryable.

UMF must separately consume these original issued requirements against its actual
catalog/storage-owner program and effective read policy. No current catalog
compatibility, native before-dispatch/source/current authority, protected release,
P03/B08 completion or original backend case promotion follows from this artifact.
All original assertions, allocations and deadlines remain mandatory.


### Selected availability regression repair — 2026-10-15

An independent owner replay exposed that the source-only0.1 parser could resolve
an absent-allowed scalar as an ideal nonnull Field while its dependency report
omitted actual availability. Predicate, projection, hidden order, selfjoin, group
and SUM argument paths could therefore emit misleading domain lineage. This is a
source-report counterexample, not evidence of a native data leak or permission.
The original six-case and expanded eight-case before witnesses are preserved under
`tests/lineage-availability/`; the old frozen producer passed only the required
predicate and field-free count controls. The earlier legacy property report also
accepted an original absent salary source; its actual before output is retained.

Both `application_lineage::prepare` and `security_query_properties::derive` now
require actual selected Field descriptors to have availability required. The
shared check resolves each original scan Record/Field membership from Catalog,
then inspects the original descriptor; it does not infer presence from the ideal
LogicalType.nullable flag or the existence of a predicate. It covers plain output
Fields as well as every hidden/operator/aggregate/terminal-key dependency before
any report is returned. Unrepresented availability refuses with
WFT-APPLICATION-LINEAGE-AVAILABILITY and no partial stdout. Absence is not NULL;
nullable/absent source semantics need a separately explicit qualified descriptor
and protocol before these reports can claim them.

Field-free COUNT(*) remains supported as structural lineage: it selects no Field,
retains every original scan and source duty, and does not blanket reject an unused
optional Record member. A required projection similarly survives an unselected
optional sibling. Neither positive control establishes native source completeness,
policy eligibility, current authority or guarded release.

All existing all-required0.1 reports, current five-source scenarios and historical
mapping outputs remain byte-identical. There is no envelope/schema/registration
upgrade or compatibility fallback. Fresh292 core tests,16 boundary controls and
10 host availability controls pass; exact original before witnesses, repaired
binary/source pins and current/legacy correspondence evidence are retained under
`evidence/lineage-availability-20261015/`. Frozen old producers remain historical
counterexample evidence, not qualification of selected absence. Full P03/B08 and
all native backend acceptance duties remain open. The in-progress0.2 grammar is
kept separate from this repair.

### Application lineage0.2 candidate: complete read intents

This separate candidate uses request `weft-application-lineage/0.2-candidate`,
report `weft.application.lineage/0.2-candidate` and plan
`weft-ir/application-lineage/0.2-candidate`. Its closed request/report schemas are
`application-lineage-request-v0.2-candidate.schema.json` and
`application-lineage-v0.2-candidate.schema.json`. It extends the original
application parser/resolver; caller-supplied plans/read requirements, policy modes
and action assertions remain invalid inputs. The0.1 protocol and ordinary/backend
registrations MUST retain their original bytes and refusals. The0.2 plan MUST NOT
enter an older registration, even when its SQL uses only older operations.

Only this candidate admits COUNT(DISTINCT one scalar Field), flat Boolean TRUE/
FALSE WHERE conjuncts, and integer LIMIT0 through1000. COUNT DISTINCT produces a
required integer count and demands equality for its original argument domain.
A null-capable argument would additionally demand nullity correspondence; actual
selected source Fields currently require original descriptor availability
`required`. Absent-allowed and unspecified descriptors refuse: absence MUST NOT
be interpreted as SQL NULL. Nullable-source admission is not implemented by this
candidate. COUNT(DISTINCT expression/multiple arguments/*), JOIN ON constants,
negative/fractional/exponent LIMIT and readProfile combinations remain refused.
SUM argument/output types are retained, but numeric aggregate preservation remains
required:null/propertyStatus:unsupported and an explicit unsupported source.

The mandatory `readRequirements` object has four ordered arrays:

| Member | Required meaning |
| --- | --- |
| scans | Every source/right-join/relationship-terminal occurrence, with original path, scan, target, complete Record identity/revision and kind rowRead. |
| fields | Every projected Field and every hidden predicate/join/group/order/aggregate/terminal-Key occurrence, including duplicates; original path, scan, owner target/Record identity, Field reference/full identity/revision, role, original descriptor and sourceStatus. |
| relationships | Each original relationship observation, with path, source/terminal scan and original relationship/source/target identities. |
| unsupportedSources | Original field-shape and numeric-SUM preservation gaps that consumers MUST NOT treat as supported. |

Projection occurrences precede the shared operator traversal; scan order is
source, right joins, then relationship terminals in filter order. Descriptor
availability is the original model's value, never inferred from the ideal IR
nullable bit or native data. Non-scalar read shapes retain unsupported-shape and
an explicit unsupported entry. These are source read intents, not policy actions,
permissions or effective disclosure modes. The current UMF owner must separately
derive row/field/relationship actions and effective read programs from its exact
policy/catalog/ontology/storage-owner sources. Fields needed only for predicates,
ordering, joins or aggregation retain their duties. FALSE and LIMIT0 MUST NOT
erase a scan, Field use, unresolved source or required duty. Missing names and
parameters therefore refuse even when SQL would return no rows.

Existing4MB request,4096 operation occurrences,8MB retained-use/plan and16MB report
bounds continue. Read requirements additionally permit at most8192 Field entries
and an8MB serialized Field-entry budget. The final report remains bounded; errors
emit no partial report. WFT-APPLICATION-READ-REQUIREMENTS reports failed original
read derivation; existing syntax/name/parameter/availability errors retain their
owner codes. The schemas describe output shape; only rebuilding the original
producer can establish report correspondence. Source/report hashes alone are not
current authority, population completeness, native codec/refinement or release
proof. Both authority and securityReady MUST remain false.

Example: `SELECT COUNT(DISTINCT s.token) FROM Resource r JOIN Resource s ON
r.token=s.token WHERE FALSE LIMIT 0` retains both row scans, both join Field
occurrences and the aggregate argument. `SELECT COUNT(*) FROM Resource r WHERE
FALSE LIMIT 0` retains the Resource row scan with no invented Field dependency.
Original0.1 and existing SQL backends refuse these extensions. No native protected
backend case or original132-case acceptance item is promoted by this candidate.

### Opt-in application lineage embedding candidate

The explicit Cargo feature application-lineage-candidate enables
prepare_application_lineage_json(request) in weft-runtime, native Python and
browser WASM. It calls the same original application_lineage::prepare; requests
remain the exact0.1/0.2 candidate schemas above, with no host plan/action/mode
argument. Success returns the exact report JSON string with no new envelope or
newline normalization. Rust returns Result<String, Diagnostic>; Python raises
ValueError and browser WASM throws a string containing the original diagnostic
code/message on refusal. No partial report is returned. Invalid UTF8 cannot enter
these string APIs; malformed/duplicate/over-budget JSON retains core refusal.

Without the feature these functions are absent. Existing compile entrypoints,
backend features and release defaults do not enable it. No new parser, sidecar,
SQL execution, policy interpretation or current/native authority is added. Exact
source/output custody and subsequent native obligations remain host requirements.

### In-process lineage preparation custody

With application-lineage-candidate enabled, runtime prepare_application_lineage
returns an immutable ApplicationLineagePreparation, constructed only by the
actual core issuer. Python prepare_application_lineage(request) returns the
frozen native ApplicationLineagePreparation object. Its json() method returns
exactly the existing report bytes; there is no Python constructor, report/dict
import, mutable field, dynamic instance dictionary or serialization round trip
that creates this native object. The existing JSON-returning API remains exact.
The Python class belongs to module weft; default-feature builds omit both the
class and factory. Browser WASM retains its original JSON-string bridge.

The object is only evidence of an actual local producer invocation over the
retained source request. Any caller can invoke that producer on its own source
inputs. Possession MUST NOT authenticate policy/catalog ownership, installed
source bytes, native caller/Subject, current generation, population completeness
or release. Those require the native consumer's private invocation and original
same-store transaction/source/authority checks. A native receiver MUST retain
its original typed object/procedure reference; duck typing, a user-overridable
module/class attribute, matching report hashes or a normalized JSON copy MUST NOT
serve as native admission. No object contains an executable plugin or database
connection. Opaque sources remain byte-retained and uninterpreted, including
content that is not valid source policy. All report authority/securityReady flags
remain false, and all original native security obligations stay open.

The core ApplicationLineage preparation retains the actual resolved Catalog,
ApplicationPlan and original ordered OrderDirection values. Its catalog(), plan()
and order_directions() accessors return immutable references; no report JSON is
reparsed to reconstruct these values. The native runtime/Python preparation owns
this same core result for its lifetime. Native lowerers MUST consume the complete
preparation: the older Plan.order list alone does not carry DESC/ASC direction.
The original order_directions list is occurrence-aligned with Plan.order.
Retaining typed context does not admit a native registration or change report
bytes. Old registrations still reject the new IR before binding callbacks, even
when the original SQL uses only older syntax. Current-source SQL lowering and
its separate effective operand/domain and original-action designation contracts
remain unfinished. Legacy security.query-profile0.2-candidate is a relationship
extension of the earlier policy profile, not canonical policy/catalog0.2 support.

The preparation additionally retains exact original source_request_json() bytes
as UTF8, including outer request whitespace and uninterpreted Artifact bodies;
Python exposes this same read-only getter. This avoids a report reconstruction
for a native source handoff. Source identity MUST include complete original
request and directions, not only applicationPlanSha256: aligned-span ASC/DESC
queries can have identical older Plan bytes and different direction vectors.
The host's resource account includes retained original input/model/plan/report
and preparation lifetime. These remain bounded component values, not native
credentials or a total process-memory proof.

A preparation/report created by a native host may retain non-public policy,
metadata or opaque fact/context bytes supplied by that host. The native adapter
MUST keep it inside the trusted preparation/execution boundary; authority:false
is not a public-disclosure classification. Neither json() nor source_request_json()
may be included in ordinary query responses or unguarded diagnostics. The native
host releases only the separately checked query result under its current release
obligations. Test fixtures containing logical-cut bytes are source-correspondence
evidence, not a protocol that permits returning private facts to an ordinary caller.

## Current-source original-use designation candidate

The separate profile version is weft.current-query.designation-profile/0.1-candidate;
the issued artifact is weft.current-query.designation/0.1-candidate. Their closed
schemas are current-query-designation-profile-v0.1-candidate.schema.json and
current-query-designation-v0.1-candidate.schema.json. These are not aliases for
legacy security.query-profile0.1 or its0.2 relationship extension.

The original0.2 source request retains exactly one application/json opaque source
for each selected role: query-designation, native-binding, policy, ontology,
catalog and owner. The profile source identity matches its id/revision; its
modelPins equal the actual Catalog pins. Sources bind the four original semantic
source roles by identity/revision/byte SHA. Binding scope binds backend ID/version,
target profile and the exact native-binding source identity/bytes. These hashes
prove local correspondence only. No source body loads an executable backend.

| Profile member | Rules |
| --- | --- |
| version/id/revision | Exact new version; id/revision1..4096 UTF8 bytes. |
| modelPins | Exact ordered owning core0.8 pins. |
| sources | Exactly policy/ontology/catalog/owner, each once, matching original source bytes/identity. |
| binding | Exact backendId/backendVersion/targetProfile and native-binding source identity/SHA. This does not admit its physical layout. |
| targets | Unique actual Record references, covering every original query scan;1..1024 entries. |
| primaryAction | An exact declared read-purpose action. |
| bindings | At most4096 unique target/Field/operator→originalAction entries; each Field belongs to that Record and each action is distinct from primaryAction with declared original-authorized purpose. |

The issuer consumes the actual immutable preparation. It reuses the original
operation traversal, preserving scan/path/Field/operator occurrences and operation
names, including selfjoin sides, hidden uses, COUNT DISTINCT and zero/false source
duties. A binding attaches its exact originalAction to each matching occurrence.
No match yields originalAction:null, never implicit disclosure or permission.
Projection read/disposition requirements remain separate. Relationship observation
designation is unsupported in this first candidate and refuses; no legacy fallback.

The issuer checks only the bounded action/purpose subset of original ontology0.2:
exact source identity/version, exhaustive unique action-purpose closure, known
purposes and the owning-operation distinctness rules. The full UMF source/model/
ontology/policy/catalog/storage admission remains unresolved and mandatory. Its
source subset bounds every action ID to4096 UTF8 bytes and256 actions. This is
stricter than UMF's UTF16 identifier ceiling for some long Unicode IDs; it is not
universal ontology support. Missing/unknown/incomplete purpose closure refuses.
Names do not select purpose, and grants do not supply absent purpose declarations.

The output retains exact profile JSON/SHA, original request SHA, model/binding/
source scope and per-use designations. It always has authority:false and
securityReady:false, with explicit unresolved whole-source semantics, joint query
modes, per-use effective/native value scope and current authority/release. Neither
ontology nor catalog modes are selected by this Weft profile. UMF must compose
applicable modes and effective value routes from its original source program;
conflicting/absent/unknown selected meaning cannot be overwritten by a designation.
Selected original-authorized routes require the exact issued action and separate
current permission. Disclosed routes cannot be upgraded by property compatibility.

Core current_query_designation::derive(&ApplicationLineage) returns an immutable
object tied to that original preparation. Native Python's opt-in preparation
method current_query_designations_json() accepts no profile/mode/action override;
WASM prepare_current_query_designations_json(request) calls the same factory.
Their success is exact report JSON; refusal uses the existing diagnostic exception
surface. WFT-CURRENT-QUERY-DESIGNATION refuses invalid source/profile linkage,
unknown/missing actions/purposes, duplicate bindings, invalid membership, unsupported
relationship designation or exhausted budgets, with no partial output. Existing
request4MB/report16MB bounds apply;4096 derived uses share an8MB serialized-use
budget. This is a component ledger, not a process-wide heap/CPU proof.

Native consumers MUST bind the original private issuer invocation and approved
profile/source/registration at a live same-store cut. Caller replacement of the
entire profile, a rehashed source, an action name or this report is not an approved
registration. Native caller/Subject/current authority, complete preflight, effective
value domains, exact paired key/runtime scope and guarded release remain required.
No SQL is emitted and no original132-case entry is promoted by this candidate.
The source/report may retain private host metadata and must remain host-private.
