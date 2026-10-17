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
