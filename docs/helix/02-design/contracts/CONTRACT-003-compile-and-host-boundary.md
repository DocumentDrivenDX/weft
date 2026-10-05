---
ddx:
  id: CONTRACT-003
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-003
      kind: informed_by
    - id: FEAT-004
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
---

# CONTRACT-003: Compile and language binding boundary

**Type:** library/data. **Version:** `weft-compile/0.1.0`. **Status:** draft.

## Purpose and Scope and Boundaries

Input is Weft SQL plus supplied UMF modules. The selected registered backend and
its owned mapping configuration determine output SQL: initially Ashlar/Databricks
or Truss/PostgreSQL. Backend selection is compile context, not another input SQL
dialect. Output is a compilation artifact; Weft never executes it.

## Normative Surface: Request

`compile(sql, modules, target, options)` is the conceptual operation. The stable
cross-language transport is `compile_json(requestJson) -> responseJson`, UTF-8 JSON
text on all hosts. Python returns str; the TypeScript wrapper returns string after
explicit WASM initialization. Convenience typed wrappers may follow this boundary
but may not use binary floats for exact values. The transport schema is
[compile-request.schema.json](compile-request.schema.json).

| Field | Required rule |
| --- | --- |
| interfaceVersion | `weft-compile/0.1.0` |
| dialect | `weft-sql/0.1.0` |
| sql | String, one fully consumed source query |
| modules | 1..32 entries; each has documentJson, pin and selectedModuleIds |
| pin | documentId, opaque nonempty revision, umfVersion `0.7.0`, sha256 of exact documentJson UTF-8 bytes |
| selectedModuleIds | Nonempty unique list of module IDs in that owning document |
| target | backendId, backendVersion, targetProfile, bindingJson, bindingSha256 |
| options | allowCandidate defaults false; no implicit lossy mode |

Each owning document occurs once. The compiler verifies hash/ID/core version,
parses JSON without duplicate keys/numeric coercion, validates the pinned UMF
schema and semantic participation subset, and retains the complete source text.
Revision is a caller label tied to verified bytes, not an assertion of UMF's
unimplemented cross-document revision semantics. Selecting a module defines query
surface; required local referenced elements may resolve through its supplied owning
document. A missing referenced dependency blocks rather than fetching/flattening.
Unknown uninterpreted native extensions survive; selected unknown semantics block.

`targetProfile` names the registered backend manifest's exact profile. bindingJson
is opaque backend-owned JSON validated under CONTRACT-002 and hash-verified first.
It may also be supplied by the host from native binding metadata carried in UMF;
there is no guessed mapping from logical names. Query parameters arise from
literals/physical discriminators in v0.1; external source parameter syntax is deferred.

## Normative Surface: Response

[compile-response.schema.json](compile-response.schema.json) defines the structural
response; semantic checks enforce parameter contiguity, type domains, pin consistency
and required qualifications. Every response contains interfaceVersion, status and diagnostics. A compiled
response additionally contains dialect, compilerVersion, modelPins, backend,
bindingSha256, resolved logicalPlan, sql, parameters, columns, obligations and
qualification. Blocked responses MUST contain no SQL/parameters/partial plan.

Parameters are ordered `{position:positive integer,logicalType,value,origin}`;
position is contiguous from one, value is an exact string, logicalType names
boolean/string/integer/decimal and established facets. origin is logical-literal
with SQL byte span or backend-binding with identity/path. Boolean lexical values
are `true`/`false`; integer is signed base-ten text; decimal is exact base-ten text.
String parameters preserve scalar text, including empty string and quote characters.

Columns record position, outputName, logicalType, nullable, carrier, decoder,
sourceIdentities and aggregate. Integer/decimal carriers MUST be text with
exact-integer/exact-decimal decoder obligations; boolean/string use declared exact
carriers. Numeric decode MUST reject malformed/out-of-domain values, never round.
SUM nullable distinguishes empty global aggregation. Unordered bags remain unordered.

Backend identifies backendId/backendVersion/targetProfile/interfaceVersion;
qualification is candidate or conformance-verified with evidence references and
remaining host assumptions. Neither means the compiled query has been executed.
Obligations are typed IDs with parameters, enforcement owner and failure outcome;
hosts must check required obligations before execution. Required contexts include
prepared execution, stored-value domains, session numeric/collation behavior,
effective authorization and publication. Compiler output is not proof of enforcement.

Diagnostics contain code, severity, message, phase, sourceSpan or module identity/
JSON pointer when known, and recoverability. Deterministic ordering is phase,
source byte offset (missing last), code and pointer. Unknown unselected content
is informational; blocking diagnostics are errors. Never leak credentials/data.

## Limits and Host Boundary

Initial maxima: request 16 MiB UTF-8; source SQL 64 KiB; each owning model 4 MiB;
32 owning documents; 256 selected modules total; 100,000 JSON values/document;
JSON nesting 128; 4,096 SQL tokens; 16 joins; 256 outputs; 1,024 literal parameters.
All limits are checked before/while allocation; no partial artifact may escape.
Host cancellation/deadline and total memory sandboxing are host responsibilities;
limits do not claim that trusted plugin code is sandboxed.

No model lookup, code loading, database connection or remote validation occurs.
Unknown request envelope/version fields block (allowCandidate is the only initial
option). Output changes use versioned compatibility rules, not wrapper coercion.

## Precedence and Compatibility

CONTRACT-001 owns dialect/IR semantics; CONTRACT-002 owns lowering/capabilities;
this contract owns transport/pins/limits. Python/WASM wrappers share identical
canonical results excluding explicitly nonsemantic build metadata. JSON objects
compare structurally; exact value strings and ordered arrays compare bytewise.
Canonical serialization orders object keys by Unicode scalar sequence, emits compact UTF-8 JSON without a BOM, preserves array order and exact value strings, and performs no Unicode or numeric normalization. Source spans are zero-based half-open UTF-8 byte offsets `[start,end)` into the original SQL string. Phase ordering follows the schema enum order: input, model, parse, resolve, type, capability, lower, emit, host.
Cross-language public names/package versions and released schema definitions must
be pinned before implementation; the transport described here is the draft target.

## Error Semantics

WFT-INPUT, WFT-UTF8, WFT-JSON-DUPLICATE, WFT-PIN, WFT-MODEL,
WFT-MODEL-VERSION, WFT-LIMIT and WFT-VERSION block. Model/dialect/backend errors
remain their own diagnostic codes. Invalid outer JSON returns a normal blocked
response when transport permits; native misuse outside the string interface is a
host type error. No retry/network fallback is implied.

## Examples and Validation Checklist

A bundle with sales and shared UMF modules compiles the same named query against
both registered backends. Changed model bytes with the old digest block before
resolution. The same record name in two modules requires an explicit namespace.
Two module documents in one logical query are allowed only if the chosen backend
maps both. Preserve unrelated extension content and compare exact reports through
native Python/browser; neither wrapper may implement its own resolver or emitter.

## B-004 executable boundary in progress

The pure Rust `compile::Compiler` owns an explicitly registered backend registry
and `compile_json` dispatches the exact 0.1/0.2 interface/dialect pairs. Unknown
members, malformed versions/options, duplicate JSON, limits and stale model or
binding bytes refuse atomically. Body/schema validation precedes resolution;
binding byte pins are checked before resolution. Recoverability is added at the
public boundary. Default runtime composition has no production backends until
B-005/B-006; it never invents or falls back to a mapping.

The unreleased draft 0.1 response schema now names B-003's actual backend interface
0.2 and permits targetContext plus per-operation qualification declarations/
assessments. This is initial public-schema finalization, not compatibility evidence
for a released 0.1 response implementation. Source dialect/IR 0.1 remains unchanged.
[The 0.2 response schema](compile-response-v0.2.schema.json) uses explicit scalar,
type-graph value and related-key representations. Value/related carriers are typed
JSON text decoded through retained descriptors with exact numeric string leaves;
optional availability alone never authorizes native null.

Native package names are `weft-sql` / Python `weft.compile_json`, Rust
`weft-core` plus pure `weft-runtime` composition, and `weft-wasm` for browser
embedding. They are foundation version 0.1.0, not released support claims. Python
and WASM wrappers implement transport only. An explicit `test-third` build feature
links the fixture backend for conformance; content cannot activate it. Full native
Python, real-browser wrapper, artifact/schema and platform evidence remains the
B-004 exit gate. A successful build alone does not qualify either host surface.

### Browser scalar transport and fatal host failure

The thin browser wrapper accepts JSON as a Unicode scalar string and rejects
unpaired UTF-16 surrogates before wasm-bindgen encoding. It binds an explicitly
initialized trusted module. It performs no name/type/backend interpretation.
A WebAssembly RuntimeError retires that wrapper permanently: subsequent calls
return the same blocked `WFT-BACKEND-FAILURE` host-action diagnostic without
re-entering WASM. The fatal envelope uses interface 0.1 because a damaged host
cannot reliably determine the active request version; hosts must create a fresh
module context/realm to recover. Ordinary compiler refusals retain the selected
request interface version and do not poison the instance.
