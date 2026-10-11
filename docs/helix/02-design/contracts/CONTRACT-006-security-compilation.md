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
