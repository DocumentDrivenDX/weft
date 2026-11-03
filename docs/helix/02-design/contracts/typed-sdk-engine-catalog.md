---
ddx:
  id: weft.typed-sdk-engine-catalog
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
---

# Typed SDK engine catalog custody

**Contract ID:** weft.typed-sdk-engine-catalog. **Type:** library.
**Version:** `weft.engine-read/0.1`. **Status:** draft implementation candidate.

## Purpose

Support UMF CONTRACT-069/071 engine adoption without moving SQL lowering or
engine admission into the SDK. Weft owns original compilation correspondence;
Truss and Ashlar retain their protected read paths and native authority.

## Scope and Boundaries

Non-temporal scalar engine reads use original source-bound compiler requests.
Temporal qualification, engine mutation authority and publication resolution
are outside this interface. A successful comparison grants no permission.

## Normative Surface

`compile_engine_read_json` accepts exactly `{version,compiler,request}`.
`compiler` is `truss-runtime`, `registered` or `ashlar-spark4`; it selects an existing original
Weft compiler, never artifact-supplied executable code. `ashlar-spark4` explicitly selects the independent Spark4.0.1/Delta4.0.0 candidate registration; it does not relabel local execution as managed Databricks qualification. The complete original
request is passed unchanged to that compiler. Only a compiled result can emit
`{version,status,compiler,request,artifact,outputs,structuralDigest}`. Blocked
compiler responses retain their complete original diagnostic result. Outputs
retain the original compiler's scalar families and semantic nullability.

`check_engine_read_json(program,current_request)` first recompiles the original
request and compares the complete program. The current request MUST equal the
original in every member except declared parameter values. Parameter names and
families MUST remain identical. The current request is then compiled through
the same original compiler; its complete program is returned. The host MUST
compare this artifact with the artifact it is about to execute inside its own
native admission interval. No comparison may remove binding, model, SQL,
backend, policy, publication or unknown members to obtain a match.

The first profile requires explicitly present scalar `parameters` and an empty
`exactParameters` object when supplied. String, boolean, integer and decimal
parameters retain exact original text. Result containers, timestamp and binary
carriers require separately qualified profiles. Transport framing, duplicate
members and nonfinite values use the shared compiler JSON admission.

## Precedence and Compatibility

Original engine/native contracts override this custody interface. Source,
binding or backend drift refuses. The structural digest covers the complete
request, artifact and output metadata using the owner's canonical JSON; it is
integrity, never authority. Changes to comparison meaning require a new version.

## Error Semantics

Unknown compiler/profile, context drift, altered parameter inventory or unsupported
carriers return `blocked` with `WFT-ENGINE-READ`. No automatic retry or fallback
compiler is permitted. Native authorization, snapshot and cleanup failures remain
engine-owned and MUST NOT become empty rows.

## Examples

A catalog retains an original Truss request with `price` as decimal text `1.23`.
A current native-context request may bind `price` to `2.50`. Changing the source
model, accepted catalog binding, backend version or SQL refuses comparison.

## Validation Checklist

Real compiler tests cover valid binding, exact decimal text, model/binding/SQL
drift, parameter-family drift and complete original diagnostics. Engine E2Es
must additionally prove protected execution and original source correspondence;
compiler custody tests alone do not qualify engine adoption.

## Truss cursor export candidate — 2026-10-16

`compile_engine_export_json(program,current_request)` is a proposed explicit
`weft.engine-export/0.1` interface for an original initial Truss engine-page
program only. It first performs the complete existing page-program comparison
with no continuation. Only declared current parameter values may differ. The
original page program, request, source/backend/binding context and authored Key
metadata remain retained. A successful result grants no native permission.

The owner compiler resolves the same original page source and authored Key,
retaining hidden Key outputs and ordering, then constructs an unbounded logical
cursor plan (`limit: null`, no generated continuation filter). PostgreSQL lowering
uses a distinct backend `truss.postgresql.runtime` version `0.4.42` and target
`truss-runtime-pg18.6-export/0.1`; it emits no result LIMIT. The ordinary0.4.41
registration and all existing read/page artifacts remain unchanged. No host may
remove LIMIT text or reinterpret a bounded logical plan as an unbounded one.

The export result retains the complete original program, checked current page
program, current original request, complete derived export request/artifact and
output/Key signature, covered by the owner structural digest. The derived
request explicitly selects the export backend; this transformation is owned by
the compiler, not guessed by the SDK. The full export artifact is independently
checked for its own compile/IR versions, model pins, binding digest, backend and
closed obligations. Its additional `truss.runtime.cursor-export/0.1` obligation
requires a Truss-owned fixed data portal and fresh current native binding/actor/
scope admission before and after each bounded fetch. Hosts without this handler
must refuse before data execution.

The new core cursor-plan factory must reject caller ORDER/LIMIT, continuations,
read-profile/security overrides and unsupported selected carriers under the same
closed authored-Key page rules. Its logical plan must describe actual unlimited
cursor delivery; ordinary bounded factories may not select this mode. Python
and browser WASM expose the same Rust custody implementation with bounded strict
JSON inputs. Compiler proof must cover original/context/program drift, ordinary
profile refusal, true limit-free IR/SQL and full signature correspondence.
Truss native snapshot/authorization/cleanup evidence is separately required;
compiler acceptance does not qualify a stream, protected policy or temporal SDK.


## Explicit guarded Truss temporal engine — freeze1

Adopt integration contract `truss.temporal-interface-freeze/1`, F1–F7, authored
at Truss `docs/helix/04-build/temporal-interface-freeze.md` (isolated review copy:
`/private/tmp/umf-stack-release-truss/docs/helix/04-build/temporal-interface-freeze.md`).
This additive profile qualifies neither native execution nor publication by
itself. Historical temporal HELD text continues to apply to old profiles;
this section selects the new bounded implementation contract.

Select compiler truss-runtime-temporal, engine read/page0.4,
umf.weft-binding-temporal/0.2, compile0.2.10/backend-interface0.2.10,
truss.postgresql.runtime backend0.4.47 and target
truss-runtime-pg18.6/0.7-temporal-candidate with explicit ABI1.3. Keep ordinary
ABI1.2, compiler selectors, backend registrations and wrapper0.1/0.2 unchanged.
The captured binding retains truss-runtime-binding/0.5 shape and exact actual
native tuple; only explicit native temporal capture supplies its ABI1.3 pin.

compile_engine_read_json accepts the closed
{version:'weft.engine-read/0.4',compiler:'truss-runtime-temporal',request}.
compile_engine_page_json accepts the closed
{version:'weft.engine-page/0.4',compiler:'truss-runtime-temporal',request,
 selectedKey:KeyRef,pageSize:integer1..999}; result is the initial page after:null.
The request is the closed F5 compile0.2.10 request with REQUIRED
 temporalProfile:'umf.weft-binding-temporal/0.2',
 temporalParameters:{name:{kind:'date'|'instant'|'local',value:string}},
 parameters:<existing ordinary map>, exactParameters:{}.
No temporal value is inferred from ordinary strings; normalized names across
maps are disjoint, complete and at most1024 total. Reject read/security/timestamp
comparison overrides. Original documentJson/bindingJson strings and exact pins
remain unchanged. CONTRACT-075 strict32/1–6 token grammar applies without the
historical128-character or zero-residue conversion.

Public Python and WASM exports take and return JSON strings:

```text
check_engine_temporal_read_shape_json(program, original_request)
check_engine_temporal_page_shape_json(program, original_request)
check_engine_temporal_read_json(program, current_request)
check_engine_temporal_page_json(program, current_request, after_json)
```

Every success returns exactly {status:'checked',program:<whole checked program>}.
Shape checks independently recompile the entire original program and compare
supplied original_request in full; return original compiled read/initial page.
Current checks first perform full original recompilation, allow only declared
parameter/temporalParameter values to change (never kind/family/name/inventory),
then compile and return the whole current read/page. Every source, SQL, slot,
output, cap, ABI, backend and unknown member participates in full correspondence.
Structural digest owners remain weft-runtime-engine-read/page with version0.4.
A digest never grants execution authority.

Blocked response is {status:'blocked',diagnostics:[<ordered original diagnostics>]},
plus original:<whole blocked compile response> when a compiler blocks. No partial
program/artifact. Preserve WFT-ENGINE-READ/PAGE categories and original underlying
parse/compile/resource errors. Maximum program/request UTF-8 size4,194,304 each,
after_json1,048,576, and any stricter existing limits. Shared strict Rust framing,
resource and cancellation controls remain; bindings MUST NOT diverge.

F5's closed artifact.sourceDomainChecks binds one full source, exact ABI, original
caps and one-based binding/requirements slots. Requirements use
truss.temporal-source-guard/0.2 and cover EVERY temporal member of the selected
RecordView, including unprojected Fields. Lower only through the F7 buffered
STABLE person_query_temporal_objects/page_objects signatures; all original rows,
including filtered-out/later rows, validate before user filters, continuation,
ordering or limit. No selected/seek source optimization or separate preflight.
Projection keeps original exact cells; comparison/order uses the ABI1.3 strict
query_temporal_canonical_text(text,text) helper with explicit C collation.

Read/page is single-Record, no joins/navigation/aggregates/containers/refinements.
Temporal Keys and non-string selected-Key paging refuse. F6 owns the exact
closed page metadata, full effective sort tuple, missing/null states and
weft.engine-temporal-after/0.2 continuation. SQL is the sole ordering authority;
append the complete required-string authored Key ASC suffix and preserve
ASC/DESC/FIRST/LAST. The original opaque native token/actor/snapshot/authority
is Truss-owned; JSON continuation data confers no authority. Truss owns native
execution, settlement and final authorization; Access retains full original
program text and performs typed signature/decoder custody only.
