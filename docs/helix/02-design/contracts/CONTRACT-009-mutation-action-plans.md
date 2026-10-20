---
ddx:
  id: CONTRACT-009
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.prd
      kind: informed_by
    - id: FEAT-001
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# CONTRACT-009: Mutation compilation to UMF action plans

**Version:** proposed `weft-mutation/0.1.0`, `weft-mutation-compile/0.1.0` and `weft.update-selection/1`.

## Purpose

Compile the first bulk UPDATE into the shared `umf.actions.plan/1` profile and an inert typed selection program. UMF CONTRACT-902/903/904 in repository `umf`, paths `docs/helix/02-design/contracts/CONTRACT-902-portable-action-plan.md`, `CONTRACT-903-frozen-action-manifest.md` and `CONTRACT-904-bulk-action-coordination.md`, are the upstream semantic authorities. Consumers MUST supply/pin their original published artifacts; no local copy becomes a competing contract.

## Scope and Boundaries

Pure Rust compiler with native Python and browser WebAssembly (WASM) bindings over the same core. No database connection, queue submission, credential, action registration, state discovery or manifest sealing. Current read dialects/compile schemas remain unchanged; mutation requests use a separate entry point and version. One supplied core0.10.0 document/module/Record/Key/action snapshot, prepared by the original UMF owner inspector and explicitly installed in the trusted compilation registry. Native executor registration/revision resolution occurs separately at materialization.

## Normative Surface

### Source dialect

```ebnf
statement = "UPDATE", record, "SET", assignment, { ",", assignment }, "WHERE", predicate, [ ";" ];
assignment = field, "=", value;
predicate = disjunction;
disjunction = conjunction, { "OR", conjunction };
conjunction = atom, { "AND", atom };
atom = "(", predicate, ")" | field, comparison, value;
comparison = "=" | "<>" | "<" | "<=" | ">" | ">=";
value = string_literal | integer_literal | "TRUE" | "FALSE" | named_parameter;
named_parameter = ":", identifier;
```

Keywords are ASCII case-insensitive. identifier is `[A-Za-z_][A-Za-z0-9_]*`, quoted names use double quotes with doubled quote escaping; logical name equality/resolution is existing CONTRACT-001 exact UMF resolution (ambiguous unqualified Record refuses). record is a Record name or `module-name.Record-name`; field is an unqualified member name. No table aliases, comments, multiple statements, missing WHERE, NOT, NULL, IN, joins, subqueries, ORDER/LIMIT, RETURNING, arithmetic or key edits in this profile. String literals use single quotes and doubled single-quote escaping, no backslash escapes; integer_literal is optional minus followed by canonical decimal integer digits (`0` or nonzero leading digit sequence), no exponent/decimal part. Business type/domain admission remains existing UMF CoreLiteral validation. Same-shaped different Fields do not make parameters interchangeable.

Predicate Fields and SET Fields are required, cardinality one, scalar boolean/integer/string. `=`/`<>` apply to all three; ordering applies only to integer/string. Exact mathematical integer comparison; string equality and order use Unicode scalar sequence binary comparison, no locale/case normalization. Boolean ordering refuses. Required values make predicate truth two-valued in this first profile; missing/null stored predicate values violate backend input obligations and MUST refuse materialization, never silently filter. SET NULL and nullable types refuse this subset explicitly, while retained core/action documents preserve their meaning.

Duplicate SET destinations or unused/missing parameters refuse. Each marker's occurrences resolve to the SAME exact FieldRef; reuse across different Fields refuses even if types match. supplied parameter map values are CoreLiteral, admitted against that Field including facets. All SET bindings are constant after compilation. SQL literal/parameter spelling is retained in provenance. Parentheses preserve explicit precedence; AND binds before OR. Assignment order is source order; evaluation cannot use prior assignment results.

### Compile API and selection carrier

`compileMutation(request:MutationRequest, registry:MutationRegistry): MutationResponse` is the common core operation exposed idiomatically by Rust/Python/WASM. Registry is trusted host-installed code/data, never loaded by model content. MutationRequest is `{interfaceVersion:"weft-mutation-compile/0.1.0",dialect:"weft-mutation/0.1.0",sql:string,parameters:{[name]:CoreLiteral},model:Artifact,modelPin:ModelPin,action:ActionPin,target:KeyRef,selectionProfile:{id:"weft.update-selection",version:"1"},batchSize:integer,limits:Limits}`. Shared types are exact UMF CONTRACT-902 types; model digest and all action bytes must correspond to the original UMF-produced preparation artifact installed in the trusted registry. Requests cannot install or replace a preparation artifact. ActionPin is portable identity/document digest; executor-assigned revisions are not compiler inputs. No credentials/store names appear in the logical plan.

MutationResponse is `{interfaceVersion:"weft-mutation-compile/0.1.0",status:"compiled",plan:Artifact,provenance:Provenance,diagnostics:Diagnostic[]}` or `{interfaceVersion:"weft-mutation-compile/0.1.0",status:"blocked",diagnostics:Diagnostic[]}`. Diagnostic is `{code:string,phase:"input"|"parse"|"resolve"|"type"|"action"|"limit",path:string,message:string}`. Compiled plan is exact JSON Plan artifact; no generated action registration, target SQL or runnable job. Provenance is `{sourceSha256:string,parameters:Artifact,entityParameter:string,action:ActionPin,assignments:{ordinal:integer,field:FieldRef,input:string}[]}`. Plan.source retains the exact JSON source envelope `{dialect,sql,parameters}` as an Artifact with mediaType application/json; sourceSha256 refers to that envelope. Artifacts use original UTF-8 encoding and must agree across bindings.

Selection artifact mediaType is application/json and its utf8 decodes as `{protocol:"weft.update-selection/1",record:RecordRef,key:KeyRef,predicate:Predicate}`. Predicate is exactly `{compare:{field:FieldRef,op:"eq"|"ne"|"lt"|"le"|"gt"|"ge",value:CoreLiteral}}`, `{and:Predicate[]}` or `{or:Predicate[]}`; and/or arrays have 2..64 operands. No raw SQL, free property-path or executable text is admitted as an AST node. All refs are local under modelPin; key equals plan.target. dependencies are all predicate fields once in first source-use order. Backend assessment/lowering must use this exact profile and supply independent semantic/native evidence; new executor registration requires no frontend switch by backend ID.

Compiler consumes the host-installed `umf.actions.plan-model/1` preparation artifact generated by CONTRACT-902's original Registry-backed inspector for THESE model/action bytes. It checks original byte/digest/identity correspondence and matches the SQL's complete SET destination/input list to the artifact's exact inspected binding map. This bounded transport/matching component is implemented in Rust against UMF-owned schemas and independent positive/refusal corpus; it does not call TypeScript, spawn a JavaScript sidecar or reinterpret arbitrary action AST/authorization semantics. Arbitrary new declarations require new owner inspection/export before compilation; a corpus or client-set eligibility flag cannot substitute for that preparation. It cannot choose a different action, change authorization or infer a business operation from a field name. Refusal for mismatch occurs before producing a compiled plan. Directly supplied typed plans must still pass upstream inspection; compilation is not execution admission.

### Selection backend boundary

MutationRegistry is `{actions:ActionRegistration[],selectionBackends:SelectionBackend[]}` supplied by trusted host code. ActionRegistration is `{pin:ActionPin,snapshot:Artifact,preparation:Artifact}`; the preparation is an original owner-produced build input installed by trusted host code, not caller data. Full snapshot/preparation/model byte pins and portable identity must match. Compiler checks transport/binding consistency but claims no native registry/delegation authority. SelectionBackend implements `describe():SelectionCapabilities`, `assessSelection(selection:Artifact,model:Artifact,binding:Artifact):SelectionAssessment` and `lowerSelection(selection:Artifact,model:Artifact,binding:Artifact,maxItems:integer):SelectionLowering`. SelectionCapabilities is `{profile:Profile,claims:{id:"selection.interpretation"|"selection.completeRead",status:"supported"|"unsupported"|"unknown",evidence:string[]}[]}`; it declares selection support only and cannot issue per-action, snapshot or bulk evidence. These declarations do not verify execution. SelectionAssessment is `{status:"supported"|"unsupported"|"unknown",diagnostics:Diagnostic[]}`. SelectionLowering is `{status:"lowered",sql:string,parameters:{name:string,field:FieldRef,value:CoreLiteral}[],columns:{name:string,kind:"identity"|"version"|"key",field:FieldRef|null}[],preflight:{sql:string,parameters:{name:string,field:FieldRef,value:CoreLiteral}[]}[],ordering:{kind:"native-integer-identity",direction:"asc"},bounds:{maxItems:integer,fetchLimit:integer},obligations:string[]}` or `{status:"blocked",diagnostics:Diagnostic[]}`. identity/version descriptors have field=null; key descriptors have the exact Key component Field in declared order. preflight programs MUST validate every referenced required predicate Field in the complete admitted authorized Record population BEFORE applying WHERE, so NULL/absent/malformed values cannot disappear through SQL three-valued filtering or short-circuiting. They run under the same coherent snapshot/authority/source admission as selection, including empty-result queries, and any violated domain aborts materialization with WFT-MUTATION-DOMAIN. If the backend cannot provide this behavior, assessSelection refuses. ordering compares actual native positive integer identities numerically, never returned decimal-text lexically. fetchLimit MUST equal maxItems+1 and the host refuses overflow; it does not trim. Results must include exactly identity, version and every Key component; all parameter values remain data, not SQL fragments. The native result decoder/host guard issuer uses the qualified backend binding; a lowered SQL artifact cannot itself issue a guard or seal work.

Current UMF 0.10 preparation admission uses the versioned owner interface
(`umf.actions.plan-model`, version `1`), its exact schema identity/digest and
the existing model/action/frame/source byte correspondence. The bounded producer
build string is retained as provenance. It is not an authentication claim or a
compiler-cohort admission gate: labels cannot prove that an owner implementation
ran. Trusted host installation of owner-produced preparation remains the boundary.
SDK build or generated-comment changes that preserve this interface do not require
a compiler update merely to recognize another build hash.


Selection lowering transport has finite output ceilings: each SQL/preflight string is nonempty and at most65536 UTF-8 bytes; at most64 preflight programs,1024 uniquely named parameters per program,64 Key components plus exactly one identity/version column,128 nonempty obligation strings,128 diagnostics,256 UTF-16-unit metadata names/codes and4096-unit messages/paths/evidence/obligations. Each scalar parameter CoreLiteral JSON is at most1MiB; the complete serialized lowering (including all SQL, parameters, columns, preflight and obligations) is at most16MiB. Oversize refuses WFT-MUTATION-LIMIT without truncation. Common binding transport is at most4MiB/depth64/100000 nodes; original backend admission may impose stricter selected limits; these transport ceilings do not claim a global process heap limit. Callback success and error diagnostics use the same finite metadata/phase validation. A supported SelectionAssessment requires diagnostics=[]; nonempty diagnostics with supported status refuse rather than being discarded. Original Key Fields retain their UMF domains; boolean/integer/string restrictions apply to predicate/SET Fields, never universally to Key descriptors. Common dispatch requires exact model bytes and target Key from an installed original owner preparation before calling a backend; other retained Records do not inherit that preparation's semantic admission.

Backend-local original binding owns exact physical table/column/type/value codec and predicate/current-authority mapping. Reuse original backend mapping admission/export and add immutable identity/whole-row-version access under separately pinned mutation-read capability. Absent access or unqualified exact order/value/null/policy behavior returns unsupported, never guesses a column. Compiler frontend stays backend-independent; native correspondence and execution/publication guard tests belong to the backend/Truss stories. SQL remains host-executed only after all original admission/authority obligations, including empty selection. Binding-specific immutable identity order must match the selected manifest guard profile. Compilation of logical intent does not require one physical backend; lowering is a later explicit host call.

### Deterministic source/AST/artifact encoding

Collapse grouping-only parentheses after applying AND-before-OR precedence, flatten adjacent same-operator AND/OR nodes in left-to-right source order (2..64 children), and retain different-operator nesting. Never reorder operands or infer commutativity for canonical bytes. Literal `-0` remains `{integerToken:"-0"}`; exponent integer tokens supplied as parameters retain their spelling and are admitted by the original UMF exact integer domain. SQL integer literals still exclude exponent notation. Provenance records both entityParameter and every assignment input/Field binding from the preparation artifact.

Generated Plan, Selection, preparation/provenance and source-envelope JSON use one owner corpus-pinned compact encoding: recursively sort object keys by UTF-16 code-unit order, preserve array order, write structural safe integers in shortest decimal, emit true/false/null literally, and escape strings only for quote, backslash and control characters (standard short escapes for backspace/tab/LF/formfeed/CR, otherwise lowercase four-digit Unicode escape). Other valid Unicode scalars remain literal UTF-8; unpaired surrogates refuse. CoreLiteral numeric tokens are strings and are never reserialized as JSON numbers or normalized. No whitespace outside strings. The original SQL string/parameter carriers are retained exactly inside the envelope. Consumers MUST check the expected generated bytes against independent golden fixtures; retained external artifacts are never rewritten into this form to disguise changed intent.

### Bounds

Mutation request UTF-8 bytes at most 1 MiB; SQL at most 65536 bytes; model at most 1 MiB; at most 64 assignments/parameters, 256 predicate comparison nodes, 128 source grouping levels before collapse,24 predicate nesting levels (decoded Predicate nodes; full emitted Plan JSON depth must independently be at most64). Existing stricter input/model bounds also apply. Plan limits and batch size are CONTRACT-902. Excess refuses, no truncation. Structural transport limits do not assert a total process heap/time ceiling.

## Precedence and Compatibility

Existing UMF action/core semantics and CONTRACT-902 take precedence over compiler conveniences. A separately versioned dialect/IR preserves read compatibility; no read parser silently treats UPDATE as SELECT or adds mutation behavior to old interfaces. Thin bindings preserve exact parameter/source/artifact bytes and ordered diagnostics. Different compilation artifacts require new manifest materialization; compiler parity proves no native execution or current authorization.

## Error Semantics

WFT-MUTATION-INPUT (envelope/digest), WFT-MUTATION-SYNTAX (outside grammar), WFT-MUTATION-RESOLVE (unknown/ambiguous ref), WFT-MUTATION-TYPE (domain/marker incompatibility), WFT-MUTATION-ACTION (registered declaration/SET mismatch), WFT-MUTATION-LIMIT (bound exceeded). Underlying PAP_* diagnostics retain source path/meaning. All branches block without executable plan; unknown model semantics retain original content and block selected meaning. No fallback or silent action substitution.

## Examples

`UPDATE Customer SET active = FALSE WHERE last_seen < :cutoff` with cutoff bound as `{integerToken:"100"}` resolves active to the exact boolean Field and last_seen to an exact required integer Field. The selection program has one lt comparison and the action template supplies `{boolean:false}` to the action's active value input. `UPDATE Customer SET id = :id WHERE active = TRUE`, `SET active = NULL`, `SET balance = balance + 1` and missing WHERE refuse in this profile.


## Current Truss adapter and native Python bridge — 2026-10-14

The explicit current registration selects PostgreSQL18.6, Truss layout1.0,
`truss-catalog-runtime/0.65-candidate`, accepted `truss-runtime-binding/0.4`
and its owner renderer. The first native domain is required singular string
and boolean Key/predicate fields; unsupported types, facets, homes and NUL
text parameters refuse. Older PostgreSQL17.11/source-epoch0.16 selection
receipts remain historical and cannot qualify this registration.

`CurrentTrussMutationBackend` validates exact model/binding bytes and matching
accepted model revision, then reuses the original current runtime's mapping
admission and tagged scalar interpretation. It emits no caller-selected SQL
and loads no executable plugin from a model. Native identity/version carriers
are the original layout's `object.id` and `object.ver`; logical Key columns
retain authored component order and original Field references. Preflight covers
the whole selected type population before WHERE, including scan bounds, native
identity/version domains, required scalar cells and duplicate typed keys.
The host executes all preflight and selection under its original admitted cut,
rejects any violation, and refuses the maxItems+1 sentinel without trimming.
The renderer's original source/codec meaning is separate from native source,
caller and release admission.

The native Python API is in-process Rust:

```python
compiler = weft.MutationCompiler(original_action_configuration_json)
handle = compiler.register_truss_selection(original_registration_json)
plan_json = compiler.compile_json(original_mutation_request_json)
lowering_json = compiler.lower_selection_json(handle, original_selection_request_json)
```

Registration is a closed `{model, modelPin, binding, evidence}` object. Model
and binding use the original `{mediaType, utf8, sha256}` Artifact carrier;
`modelPin` must match the accepted revision exactly. The prior illustrative
`demo1` revision is refused against a SHA-revision binding. Registration lives
in the host-owned compiler instance; request JSON cannot install backend code.
The lower request is exactly `{selection, model, binding, maxItems}`. Selection
is the original compiled plan's `selection.artifact`. Its return is the existing
SelectionLowering JSON contract with typed CoreLiteral parameters and original
SQL/preflight/identity/version/Key descriptors. Unknown members and uninstalled
handles refuse. Connections, credentials, authority, action execution and
publication remain host responsibilities.

Fresh same-store Python/native evidence tests the original products model,
bounded selection, typed dollar parameters, preflight failures, overflow,
stale revision/binding rejection and a public Truss version-checked update.
The native selector in that test is an explicitly excluded administrative
assessor. This is compiler/driver/SDK effect correspondence, not protected RR,
queued-action authorization, sealing or complete security acceptance.
