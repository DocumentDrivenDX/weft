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
| pin | documentId, opaque nonempty revision, umfVersion `0.7.0` or `0.8.0`, sha256 of exact documentJson UTF-8 bytes |
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

Owning core version admission is additive within the existing 0.1/0.2 transport
shapes. The original document `umf` and pin `umfVersion` MUST match exactly;
validation dispatches to the independently pinned schema for that owning version.
No version rewrite, inferred upgrade or relabel is permitted. Request, retained
module, logical-plan and result pins preserve the original version and bytes.
Core 0.7 selected behavior remains unchanged. Core 0.8 admission does not imply
support for every newly expressible value meaning: selected defaults,
allowed-values constraints, range/collection refinements, unknown element members
and nonempty uninterpreted selected extensions MUST refuse until independently
implemented with corresponding guards. Existing established scalar facets and
presence/collection subsets remain governed by CONTRACT-001 and the backend
capabilities. Unselected source content remains retained.

Facetless mathematical integer declarations MUST NOT acquire a synthesized width.
Selection requires an explicit unbounded-integer backend capability. Without an
admitted exact representability profile, these fields refuse before backend binding
dispatch and host execution. Original core 0.8 string/fixed-decimal queries,
string-key joins and COUNT are a partial domain; they do not complete the required
original commerce integer or general arbitrary-precision query scope.

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

## CLI byte transport

The `weft-cli/0.1.0` component accepts one complete UTF-8 request on stdin
through EOF and writes the unchanged `compile_json` response plus one LF on
stdout. Exit zero means a complete transport response, including a normal
blocked compiler response; the caller must inspect its status. Candidate and
backend registration remain governed by CONTRACT-002 and build selection.

Before constructing a string, the CLI MUST capture at most 16 MiB plus one
sentinel byte. Requests longer than 16 MiB MUST fail transport. Input I/O
failures and malformed UTF-8 MUST fail before compilation and emit no stdout.
Fatal transport failures use exit 2 and one payload-free stderr code:
`WEFT_CLI_INPUT_LIMIT`, `WEFT_CLI_INPUT_IO`, `WEFT_CLI_UTF8` or
`WEFT_CLI_OUTPUT_IO`. Output I/O failure can leave a prefix; consumers MUST
reject it on nonzero exit and MUST NOT release a partial artifact. No retry,
compiler substitution or target fallback is implied. Total process memory,
execution deadlines and cancellation remain host responsibilities.

These byte-transport failures do not alter the string library interface, its
versioned schemas or normal compiler diagnostics. Valid inputs preserve exact
response strings and ordered arrays under the existing comparison rules.
The produced CLI must be checked at the exact byte boundary, multibyte boundary,
oversized input and malformed UTF-8, alongside ordinary response framing and
the full declared compiler-profile corpus. A build alone establishes none of
that conformance or native execution.


## Compiler distribution boundary

`weft-distribution/0.1` describes a concrete CLI build, separately from language,
IR, backend and compile-interface versions. A build record binds the complete
ordered tracked-source inventory and exact commit, lockfile/toolchain digests,
observed build command/features/target and tool versions, executable bytes, backend
manifest identities, public schemas, and conformance receipts. Record unknown
linker, cache and environment properties explicitly; the same source revision does
not imply identical executable bytes or a hermetic build.

A build record is inert data. Only an independently reviewed entry in a public
Weft distribution index selected from a trusted release/package revision can
register the record and executable digests. A caller-supplied manifest, local
checksum or passing smoke test cannot promote a build. Configuration selects the
trusted distribution identity and local installation location outside compile
requests. Requests retain ordinary fresh publication IDs, table UUIDs, versions
and source bindings; registration cannot become a fixed-fixture request allowlist.

The closed [index schema](distribution-index.schema.json) defines
`weft-distribution-index/0.1` entries with a realization identity, relative manifest
and executable descriptors (path, SHA-256 and byte length), target triple, and
an assembly custody descriptor that binds every file in the realized package.
All entry artifact paths resolve relative to the caller-selected realization
package root, never the index file or repository directory.
Consumer admission rejects duplicate realization identities, checks all selected
descriptors against the record and actual bytes, and verifies the independently
selected immutable index revision and digest before reading a caller record.
Schema validity cannot establish index trust or artifact correspondence.

The first composition is the release CLI with exactly
`ashlar-databricks-candidate`, Rust 1.90.0 and one actually executed target triple.
Every other composition/platform requires its own record and qualification.
Candidate opt-in, source/model gates and all emitted host obligations remain in
force. Distribution qualification does not establish authorization, stored-value
validity, publication consistency or native engine support.

Production follows build → digest/profile/full-corpus qualification → independent
index admission. Consumer installation follows trusted indexed selection → fetch
or local realization match → verify → install → available. A local rebuild with
different bytes remains an unindexed candidate until independently admitted.
Failure at any consumer stage leaves that component unavailable; there is no
compiler fallback. Changing executable/source/features/target/schema/corpus
invalidates that concrete realization. Index admission requires actual produced-byte execution over the
complete declared public-transport corpus, full responses, current refusal and
candidate-opt-out controls and valid fresh-binding variation. Static inconsistent
vectors refuse at compile time; actual native drift refuses through host
observations. A coherent retained older publication remains usable under host
retention rules. Explicit reviewed historical expected-artifact migrations retain
original receipts and record exact old/new values, paths and hashes; general SQL
or response normalization is forbidden.

| Property | Required invariant or progress condition |
| --- | --- |
| DIST-F1 | An inert build record cannot promote itself; availability requires the independently trusted index entry |
| DIST-F2 | Executable, source, features, target, backend, schema and complete receipt identities agree with the selected realization |
| DIST-F3 | Registration discharges no source, authorization, publication or stored-value host obligation |
| DIST-F4 | A failed verification/install step leaves the component unavailable and cannot select a fallback |
| DIST-L1 | With a trusted valid realization, available artifact, complete passing receipts, writable installation and completion of pending steps, installation reaches available; unavailable external inputs do not imply unconditional progress |

The distribution harness lives outside compiler crates and receives explicit
build/check paths and typed options. It verifies opening/closing source and
executable custody, reports stage/profile/digests and stable refusal codes, and
keeps protocol output clean. Diagnostics exclude credentials and raw user model or
query payloads. These state transitions are a precise specification with
executable positive/adversarial tests, not a mechanical proof claim.

## Two-hop transport proposal

[CONTRACT-005](CONTRACT-005-authored-two-hop-paths.md) proposes new closed
request/response/IR/result versions for typed path occurrences and exact grouped
counts. Old schemas, historical artifacts and immutable distribution realizations
remain unchanged. Hosts must decode the new typed descriptor and discharge every
required obligation before releasing results; wrapper coercion or SQL repair
cannot provide compatibility.

## R1 CLI byte admission and prepared-source ownership

The CLI MUST read at most sixteen MiB plus one sentinel byte before UTF-8
conversion or compiler invocation. An exact-limit request is admitted to normal
validation; overflow stops after one sentinel without draining the suffix.
The current CLI byte-transport contract above governs failures: no stdout,
exit 2, and the fixed WEFT_CLI_INPUT_LIMIT / WEFT_CLI_UTF8 /
WEFT_CLI_INPUT_IO / WEFT_CLI_OUTPUT_IO stderr code. The historical R1
structured-error protocol was superseded during main integration; library
compile_json refusals retain their existing structured versioned responses.
Input wait/cancellation remains host-owned.

Rust Catalog owns retained inputs privately; inputs() returns a read-only slice.
Changes require cloning inputs and preparing a new snapshot, with renewed exact
pins. Source-bound plans/packets MUST refuse a different prepared snapshot. This
is an intentional source API migration for unreleased Rust callers; it does not
change compile request/response schemas or permit version relabeling.

The CLI reads a cloned, unbuffered OS stdin handle on Unix and Windows; it does not use Rust stdin read-ahead. Unix shared-offset evidence verifies the unread suffix after the sentinel. Windows is an implemented branch without execution evidence in R1; other target families refuse host input. Handle-clone failures follow the safe host I/O refusal.
