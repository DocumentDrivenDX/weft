---
ddx:
  id: CONTRACT-007
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

# CONTRACT-007: Security compilation admission foundation

FR-19–22 govern this slice. Owner direction is the shared UMF security implementation goal. UMF owns
CONTRACT-052/053 policy/ontology/lifecycle meaning; Weft owns compiler lowering;
hosts/backends own authenticated execution and publication.

`weft-compile/0.5.0` retains `weft-sql/0.2.0` grammar. Its request schema requires
UMF 0.8.0 source pins and a security source object with version
`umf.security/0.1.0`, policyJson and ontologyJson. Strings preserve authored bytes;
these bytes are not credentials, understood policy, or globally current revisions.
They must eventually undergo exact bounded semantic admission under the shared
UMF contracts before a security-enabled artifact can be emitted.

This initial foundation validates the transport and exact model byte/identity/
version/schema custody through a separate Catalog::prepare_security path. The
canonical upstream schema is pinned from UMF; source correspondence is retained
in the sibling integration evidence. Ordinary Catalog::prepare retains main
admission of core 0.7.0 and 0.8.0; prepare_security alone requires core 0.8.0.
Unknown versions never downgrade.

All well-formed 0.5 requests currently return WFT-SECURITY-UNSUPPORTED before
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

The initial response schema admits only a blocked 0.5 artifact. No compiled
0.5 result-domain is selected yet; that owner contract must be authored and
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

Compile 0.5 accepts optional queryProfileJson inside security, admits it before
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

Compile 0.5 with a query profile now resolves its actual SQL and derives admitted
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
self-signing a packet does not establish these obligations. Compile 0.5 security
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
   contract must be selected explicitly; the current blocked-only 0.5 contract
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


### B-009 aggregate disclosure budget repair

Original logical simulation shares one normalized/copy payload budget across
fact admission, expression evaluation and disclosure composition for the entire
collection. Transform normalization reserves a conservative scalar-size bound
before allocation, then charges actual normalized bytes. Selected transformed
disclosures charge encoded domain/literal/identity metadata before cloning and
retention; this conservative copy accounting includes metadata beyond scalar
payload. Standalone original/draft composition uses the same limits per call.
Dominant withholding skips unused transform normalization; conflicting transforms
still refuse without returning an admitted prefix. Short numeric exponent
expansion is included in the pre-allocation reservation.

These are logical payload/work bounds, not a claim of total heap accounting or
native memory sandboxing. B-009 retains exact boundary and repeated-resource
controls; an old simulation pass does not cover this repaired path.


## Explicit main integration migration

At historical reliability checkpoint 2b88a9a this security contract had ID
CONTRACT-005 and used compile 0.3 / SQL 0.2. Main already owns CONTRACT-005
for authored two-hop paths and compile 0.3 / SQL 0.3 for ordinary nullable reads.
The owner approved preserving main: this current security contract is
CONTRACT-007, using blocked-only compile 0.5 / SQL 0.2. Historical receipts and
schema hashes retain their original checkpoint meanings; fresh integrated
qualification is required. Native security acceptance remains open.


## Optional original result presence component — 2026-10-12

The pure direct-field declaration checker now includes both Original and Absent
for an absent-allowed field whenever Original is a conservative policy outcome.
Both must be declared exactly once; missing/extra outcomes, required-field Absent,
unsupported nullability and the aggregate256-outcome limit retain refusal.
Transforms and Withheld do not fabricate absence. A transformed null remains its
explicit transform outcome; Original(null) remains a present original value.

Cell validation checks the exact outcome/tag/value shape. Supplied-truth selection
checks active disposition only and cannot establish original values or absence.
The separate supplied-fact selector requires complete original field coverage:
Absent requires the exact field in the validated resource's explicit absent set,
with no present field; Original requires an exact present value, including null.
An omitted field without explicit absence refuses. These inputs remain simulated
facts, not authenticated native observations, and no method releases results.

The component evidence is under
`../../04-build/evidence/security-optional-presence-20261012/`. Public security
compilation still refuses lowering and emission; native codec/presence, current
policy and final publication obligations remain required before integration can
claim support. This extends pure correspondence only, not the supported SQL
surface or a qualified native backend.


## Bounded physical declaration shape — 2026-10-12

The owned context exposes `check_physical_shape` for
weft.security.physical-plan/0.1.0 declarations. It rechecks original query/profile
sources, requires unique operation IDs across installation and execution, and
bounds512 installation operations,1MiB SQL per operation,1024 parameters per
operation and16384 parameters across the complete plan. SQL must be nonempty and
NUL-free. Parameter positions are contiguous within each operation; provenance
must be an object and nullable string-carried slots refuse until a separately
qualified codec exists. Empty non-null parameter strings remain representable.

The shared declaration ledger bounds work and16,000,000 retained bytes, including
parameter value/provenance/facets and original opaque JSON number lexemes. It
charges numeric spelling without interpreting or normalizing opaque metadata.
Depth/collection bounds remain enforced before any checked success. No accepted
prefix is returned. This method checks shape only: nonempty SQL is not parsed or
proved to realize the original query, parameter origins are not semantically
matched, and no native privilege, installation, current authority or publication
obligation is admitted. It issues no execution credential or immutable checked
plan. Callers cannot convert success into emission or bypass future correspondence.

The component receipt is
`../../04-build/evidence/security-physical-shape-20261012/receipt.json`.
Public security compilation remains blocked. Complete physical/result/dependency
coverage, backend capabilities, installed native behavior and host lifetime remain
required. No independent SQL feature allowlist is introduced by these finite bounds.


## Exact physical parameter-domain component — 2026-10-12

The owned context's separate `check_physical_parameter_domains` first validates
physical shape, then applies Weft's existing exact ordinary parameter semantics.
This scoped component admits non-null strings/booleans with no selected facets,
bounded signed/unsigned integerWidth1–64, and decimal precision1–28 with
scale0–precision. Only the exact known facet members are admitted; opaque unknown
facets remain retainable in shape but refuse semantic interpretation. Unbounded
integer, derived/arbitrary decimal, nullable carriers and broader selected string
facets require their own existing capability/profile integration rather than
being silently reinterpreted. These are component limits, not a product SQL ceiling.

Empty and valid Unicode strings pass; native NUL and noncanonical booleans refuse.
Integer overflow/fractional spellings and decimal precision/scale violations or
exponent spellings refuse. Values retain their original text. No parameter origin
is matched to an actual source query, native placeholder layout or codec by this
check; those mandatory obligations remain before lowering/emission. Success
issues no authority and the public security compiler stays blocked.

`../../04-build/evidence/security-physical-domains-20261012/receipt.json` retains
source-qualified component checks. No separate SQL feature allowlist, new local
literal algebra or native backend qualification is introduced.


## Original execution query-slot correspondence — 2026-10-12

The owned context exposes `check_execution_query_slots` for an explicitly fully
parameterized relational-query profile. It reuses shape/domain checks and derives
each scalar literal/parameter occurrence from the actual original resolved plan,
in join/on order then filter order, including page tuples and relationship keys.
Field-to-field comparisons create no parameter slot. The bounded derived inventory
matches execution-slot count, exact text value, original logical type and exact
origin `{kind:queryValue,sourcePath:...}`. Each repeated occurrence stays distinct;
parameter names or same-typed values cannot substitute another occurrence.

This is an optional correspondence component, not a rule that all backends must
parameterize every literal. Alternate inline/deduplicated/mixed strategies need
separate complete correspondence under their actual owned lowering. The component
does not inspect SQL placeholder placement or prove SQL implements the original
query; installation and policy/subject parameters remain separately unadmitted.
Successful checks still produce no security-enabled compiler response or authority.

`../../04-build/evidence/security-query-slots-20261012/receipt.json` retains
source-qualified tests. Complete SQL/capability/obligation/native/host validation
and original backend acceptance remain required.


## PostgreSQL query-placeholder coverage component — 2026-10-12

`check_postgresql_query_placeholders` rechecks the original execution query slots
and uses the workspace-pinned sqlparser0.63.0 PostgreSQL tokenizer with conservative
token preflight. The selected lexical profile assumes UTF8 and
`standard_conforming_strings=on`; it distinguishes canonical `$1`–`$n` placeholders
from ordinary quoted identifiers, ordinary/standalone escape strings, ASCII-tagged
or untagged dollar strings and comments. Every declared execution slot must appear;
repeated use of a slot is allowed, undeclared/zero/noncanonical ordinals and
identifier junk immediately following an ordinal refuse. Tokenizer errors refuse.
This checker does not establish complete native literal or syntax validity, prove
original query equivalence, inspect installation placeholders, select a native
codec or certify the context's backend dialect. Its scope must be selected as a
PostgreSQL ordinal profile; other strategies/dialects require their own coverage.

The existing1MiB SQL bound limits tokenizer input. No allocation/instruction-level
or concurrency-memory proof is claimed. This introduces no SQL keyword/feature
allowlist and leaves public security compiler emission blocked. The source-qualified
receipt is `../../04-build/evidence/security-postgres-placeholders-20261012/receipt.json`.

The lexical correction retains `WFT-SECURITY-PHYSICAL` refusal for operator tokens
containing a comment opener, continued escape strings, non-ASCII dollar tags and
Unicode `U&` string literals/custom `UESCAPE`. These forms require a separately
qualified scanner even when PostgreSQL accepts them. Ordinary operators remain
admitted; whitespace before an operator-adjacent comment avoids that ambiguous
token form. PostgreSQL18.6 native controls demonstrate that `$1` in
`SELECT 1%/*$1*/1` and an escape-string continuation can be non-parameter text;
the corrected checker refuses these cases. Native observations and before/after source checks are in
`../../04-build/evidence/security-placeholder-review-20261012/receipt.json`.
This is finite lexical correspondence evidence, not complete lowering or backend
security acceptance; the public compiler remains blocked.


## Source-bound execution slot origins — 2026-10-12

Query-value origins now additionally require `sourceContextSha256`, obtained from
the owned context's `execution_query_context_sha256`. Versioned length-framed
hash inputs cover exact query SQL, binding JSON, backend/target identity, policy
and ontology bytes, query-profile bytes, original model documents/pins/selected
modules and the resolved application plan (including supplied parameter values).
Large plan JSON is hashed through a streaming writer; original model bytes are
borrowed. Parameter occurrence paths cannot be replayed under another context
merely because its value/type/path have the same shape.

This fingerprint is comparison data, not a signature, native credential or proof
of issuer trust. Its correspondence relies on SHA256 collision resistance and
original compiler-context source custody. A caller computing the expected hash
still gains no execution authority or SQL-semantic validation; complete lowering,
native qualification and host release remain mandatory. The source-qualified
receipt is `../../04-build/evidence/security-slot-context-20261012/receipt.json`.


## Required lowering-obligation coverage and settlement design — 2026-10-12

This section defines the next admission gate; it is not implemented by the
current context checkers. `SecurityLowering.obligations` is a declaration. An
empty list, a generic Supported capability, or all submitted obligations marked
enforced cannot establish complete coverage. The compiler derives the required
source inventory from its actual immutable SecurityRequirements and original
source context, before considering a backend's declarations.

The inventory must include each scan occurrence and action, each applicable rule
and its complete condition/disclosure, each ordered identity key and component,
each live field/context/association dependency, every operator use and its
disclosed/original-authorized mode, and each ordered output occurrence. Repeated
outputs and scan occurrences stay distinct even when their dependencies match.
Zero-rule scans and COUNT-only queries still require scan/action eligibility.
Empty association populations require completeness, not omission of their source.
The inventory also contains mandatory source custody/current native mapping,
authenticated original subject/context, complete current fact/authority cut,
query/native codec correspondence, resource/diagnostic privacy and final
publication/lifetime duties. Those duties cannot disappear because a query has
no scalar outputs or a backend reports no installation operations.

Source identities must be owner-issued, versioned and tied to the exact context
fingerprint. They must identify the original semantic path and its qualified
model lineage. Labels, rule names alone, hashes alone, caller-created paths and
unqualified element IDs cannot identify a requirement. The concrete source-token
wire encoding and native evidence validators remain open implementation decisions;
no current string in semanticSources is admitted by this section alone. Neither
post-serialization reindexing nor a copied source-context digest can manufacture
owner custody. The implementation must preserve the original inventory objects.

Coverage compares the complete owner inventory with declared source references:
all required sources must be covered, all referenced sources must resolve to
that original inventory, and duplicate/ambiguous obligation IDs refuse. Multiple
obligations may cover one source when mechanisms cooperate; matching sets is a
necessary comparison, not proof that the SQL implements those requirements.
Each declared prerequisite must name exactly one declared obligation. Missing
references, self dependencies and cycles refuse before installation or output.
Use bounded topological validation, not recursive unchecked traversal. Initial
budgets are4096 obligations,32768 dependency edges and16,000,000 charged UTF8
bytes, including IDs, source tokens, failure codes and evidence case IDs.
Exhaustion returns one refusal, without a partial admitted prefix.

Enforced/refused/unverified outcomes belong to the independently checked receipt
and its exact original operation, not to trusted backend declarations. A consumer
can settle enforced only after all prerequisites settle enforced and the selected
validator accepts its evidence for the current original source/native context.
Compiler, backend, host and native sites remain distinct. Host/native authority
or final publication duties cannot be discharged by an ordinary compiler's
static claim. A refused, missing, stale or unverified dependency blocks its
consumers and positive profile qualification. No retry may reinterpret an unknown
original native outcome as a completed replacement operation. All duties must
remain valid through their declared use and publication boundaries. A readiness
check before execution alone does not discharge that interval.

Required implementation controls include deleting every individual required
source, zero-rule COUNT scans, duplicate aliased outputs, wrong scan/action/key
component/endpoint direction, unowned source substitution, stale context replay,
missing prerequisites, self/two-node/long cycles, conflicting enforcement sites,
forged proof outcomes and bounded graph exhaustion. Native controls must mutate
SQL, codec, mapping, policy and authority after comparison and before final
disclosure. Prior protection and caller work must survive each refused activation.
These controls belong inside the original B09/B11/B12/B15 and lifecycle runners;
a shape checker or this design model cannot replace those required cases.

The finite five-source/five-obligation model in
`../spikes/security-obligations/model.py` executes nine solver checks. Six UNSAT
results exclude uncovered sources, stale-only coverage, refused/unverified
prerequisites, a two-node cycle and unowned sources under the stated constraints.
Three SAT witnesses establish a usable dependency chain, the weakness of
all-declared-enforced without complete owner coverage, and the fact that accepted
flags alone do not prove actual native enforcement. This is conditional design
analysis, not Rust refinement, authenticated evidence or backend qualification.
Retained results: `../../04-build/evidence/security-obligation-model-20261012/receipt.json`.
Public security compilation remains blocked until complete lowering, evidence
validation and mandatory host/native admission are implemented and qualified.


## Bounded obligation declaration graph component — 2026-10-12

`SecurityBackendContext.check_obligation_shape` rechecks original source context
and validates declaration structure only. It requires unique nonempty bounded
IDs, nonempty source/evidence-case lists, unique members within each list,
well-formed WFT failure codes and resolvable distinct prerequisites. It bounds
4096 obligations,4096 entries per list,32768 total prerequisite edges and the
shared1,000,000-work/16,000,000-byte declaration ledger. Text has an early16384-byte
check and at most4096 Unicode scalar values; NUL refuses.

The checker builds bounded indices only after charging declaration text, then
uses an iterative topological traversal. Self dependencies and cycles refuse
with WFT-SECURITY-OBLIGATIONS. No recursively traversed caller graph, admitted
prefix, settlement or execution credential is returned. The typed enforcement
site is retained but not admitted. An empty declaration list passes this shape
check; unknown semantic source strings and evidence case IDs are opaque data.
Thus a successful shape check cannot imply complete owner-source coverage,
valid evidence or all-enforced qualification. Those gates from the preceding
design remain mandatory and unimplemented.

The original compiler-context test compares all512 directed three-node graphs
against an independent transitive-closure oracle:25 acyclic graphs pass,487
cyclic graphs refuse. It separately checks a4096-node chain, its long cycle,
exact32768/32769-edge boundaries, missing references, duplicate IDs/source tokens,
malformed/empty members, count exhaustion and total text exhaustion. This is
finite algorithm correspondence and boundary evidence, not a general formal
refinement or native authority proof. The final compiler still refuses with no
SQL/parameters/plan. Receipt:
`../../04-build/evidence/security-obligation-shape-20261012/receipt.json`.


## Context-bound owner event-source coverage component — 2026-10-12

`semantic_source_ids` reuses the existing private security_obligation_sources
issuer; it does not replace OwnerSourceDemands, typed RuleOccurrences, independent
requirement templates, deployment-duty catalogs or their matching gates. Its
returned strings project actual immutable owner requirements as compact JSON
string arrays `["weft.security.semantic-source/0.1.0", contextSha256, eventId]`.
`eventId` is the unchanged compact JSON string-array identity from that issuer.
The exact context fingerprint covers original query, binding, model/pin selection,
policy/ontology/profile and backend/target bytes under the existing framed-hash
contract. SHA256 collision resistance is a correspondence premise, not authority.

The old private source-only protocol stays unchanged. The new frame is a separate
component encoding; no implicit conversion admits an old raw event ID or a new
frame into independently authored profile/template obligations. Future full
registration integration must select and validate that encoding explicitly.
Owner issuance and framing each retain their existing finite1,000,000-work/
16,000,000-byte ledger. Framing charges exact JSON expansion before allocating,
with4096-byte IDs and4096 distinct sources. These are separate finite phases; no
single aggregate ledger or general allocation/runtime proof is claimed.

`check_semantic_source_coverage` first checks the bounded declaration DAG, then
requires every declared source to belong to the actual current inventory and
every inventory source to appear. Joint mechanisms may repeat a source across
different obligations. Empty declarations, omissions, unowned sources and
noncanonical/stale frames refuse. It derives the required set itself; a caller
cannot supply a shortened required set. Returned inventory strings remain data.

This checks owner event coverage, not complete typed condition/disclosure
occurrence, semantic capability bundle, independently authored site/kind/evidence
matching, deployment duty coverage or native enforcement. A generic all-events
host obligation can pass this component while remaining unqualified. All those
stronger gates, current authority and final publication remain mandatory before
security compilation can emit. COUNT retains its scan/action/output source
events here, without admitting computed result contracts or native count behavior.

Tests erase every source from one actual inventory, add an unowned source, keep
repeated aliases distinct and verify field-free COUNT lineage. Original contexts
with changed SQL, binding or profile bytes refuse retained declarations; binding
and profile controls explicitly keep all underlying unframed event IDs identical.
The original compiler remains blocked with no SQL/parameters/plan. Retained
receipt: `../../04-build/evidence/security-source-coverage-20261012/receipt.json`.


## Combined registry-bound candidate declaration check — 2026-10-12

`SecurityRegistry.check_candidate_declaration(context, candidate, allow_candidate)`
connects existing correspondence components without invoking a backend callback.
It rechecks original sources, selects the exact original registry declaration,
issues conservative selected scan/action and whole-application capability
coverage, retains typed owner demands and rule occurrences, then checks physical
parameter domains, original query slots, result declaration and context-bound
event source coverage. Unknown selected capability/profile/constraint, stale
model revisions, insufficient declared limits and incomplete declarations refuse.
The exact registry entry cannot be replaced by a separately supplied manifest.

The candidate argument remains caller-owned declaration data. Success returns
unit; it retains no admitted plan and cannot authorize subsequent mutation or
execution. It does not call SecurityBackend.lower, native installation or result
publication. Public compile continues to select registration and return blocked
without dispatch. allow_candidate only selects Candidate metadata for this
comparison; Supported/evidence strings are still declarations, not independent
native evidence or permission. No new compile envelope or executable artifact
version is introduced.

The selected private semantic domain still requires complete required direct
scalar outputs; unsupported computed/optional profiles remain explicit
requirements for later integration. This component does not choose a smaller
product SQL surface. Complete independently authored template/site/kind/failure/
case and deployment-duty matching, selected binding interpretation, SQL/query
equivalence, native codecs and original fact/authority/publication evidence are
still mandatory before lowering dispatch or emission. No conditional template
match or generic all-events host obligation qualifies those gates.

The original-context integration fixture uses a backend whose lower method
panics if invoked. Candidate and Supported metadata can pass declaration
comparison while the compiler still returns WFT-SECURITY-LOWERING-UNSUPPORTED
without SQL/parameters/plan. The physical SQL deliberately contains `SELECT 1`
for an original resource-field query: its acceptance demonstrates that this
checker does not establish SQL meaning. Eight isolated candidate corruptions,
six selected-registration corruptions, Candidate refusal without explicit
selection and foreign registry identity controls all refuse. Receipt:
`../../04-build/evidence/security-candidate-declaration-20261012/receipt.json`.


## Private matched-instance candidate projection — 2026-10-12

The existing InstanceCorrespondence now has a private candidate comparison. It
uses the retained original registry declarations after independent requirement
instance matching, never a caller-supplied shortened expected list. Returned
obligation IDs, failure codes, sites, prerequisite sets and evidence-case sets
must match those originals exactly. Each original semantic subject projects to
the current owner context-bound source frame. A selected-capability deployment
subject contributes no invented semantic path; its returned semantic source set
may be empty only when the matched original subject set has no semantic source.
Duplicate IDs/list members, omitted obligations, foreign sources/cases/dependencies
and stale frames refuse. Metadata order is immaterial, while original IDs retain
the previously checked typed source/scope/kind/occurrence/origin correspondence.

The combined private comparison additionally requires the exact original selected
capability set, including selected zero-edge capabilities, and checks physical
parameter domains, original query slots and result declaration. It does not use
the legacy unscoped shape checker to reinterpret scoped deployment subjects. The
legacy public check deliberately continues to refuse empty semantic-source lists.
This is an explicit private projection bridge, not implicit protocol migration.
The matcher takes its context from the retained owner correspondence, so a caller
cannot supply a different expected context to this comparison.

The comparison bounds4096 entries per collection and charges original/candidate
text and owner-generated frame decoding under1,000,000 visits/16,000,000 bytes.
The owner source issuer and other called components retain their own stated
phase ledgers. No combined allocation/runtime proof or single aggregate budget
is claimed. No admitted plan, native evidence outcome or execution credential is
returned. Independent template/profile authenticity and sufficiency, deployment
duties and actual original native enforcement remain separate mandatory gates.

Tests build returned metadata from the independently authored template side of
the existing fixture, not by copying decoded custody. Every omitted obligation
and ten isolated metadata corruptions refuse; selected-only duties reject
invented semantic sources. Reordered metadata passes. Six composed candidate
corruptions cover capability omission/substitution/duplication, output omission,
empty SQL and obligation omission. A deliberately semantically unproved `SELECT 1`
can pass declaration comparison; SQL equivalence and native disclosure are not
established. Public compilation/dispatch remains unchanged and blocked. Receipt:
`../../04-build/evidence/security-matched-candidate-20261012/receipt.json`.
