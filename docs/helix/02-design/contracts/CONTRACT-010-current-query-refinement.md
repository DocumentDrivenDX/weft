---
ddx:
  id: CONTRACT-010
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: US-009
      kind: informed_by
    - id: CONTRACT-006
      kind: informed_by
    - id: CONTRACT-007
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
---

# CONTRACT-010: Conditional current-query result refinement

**Type:** library / candidate data protocol. **Version:** 0.1-candidate.

## Purpose

Independently check a native scalar result against the original Weft
CurrentQueryPreparation (CQP), its validated materialization binding and
independently captured complete eligible input populations. A returned SQL
contributor bag MUST NOT serve as its own expected input population.

US-009-AC3/AC4 and CONTRACT-006/007 govern meaning and host authority. This
candidate checks supported relational semantics; its report grants no native
permission and cannot establish that supplied cells are genuine or complete.

## Scope and Boundaries

Weft owns the original typed relational plan, exact comparisons/arithmetic,
expected public cells, contributor multiplicity and result comparison. The
existing selected backend MUST first validate the exact original materialization
binding through its actual typed lowering path. No imported AST, compiled-artifact
JSON, callback or caller-built success report can replace that invocation.

Hosts own authenticated source/caller/Subject/current authority; original UMF
read/disclosure and separate action decisions; native value/type/comparison
correspondence; complete primary-eligible population and independent counts;
statement/result custody; snapshot retention; cancellation; closing revalidation;
and guarded public release. Those obligations remain required on empty results.
The checker performs no I/O, SQL execution or policy evaluation. Reports MUST be
consumed only with the same direct invocation, held operation, actual captured
input/result and cut. They MUST NOT be cached or reused for admission across
operations, even when all data/query digests match.

Initial supported targets are PostgreSQL current rows ABI2, contributors ABI3 and
explicit original SUM ABI4, and Databricks contributors1 / original SUM2. Their
existing closed binding/context versions remain unchanged. Scalar ABI1 without
private contributor lineage refuses this refinement profile. Unknown domains,
operators, null/absence/withholding carriers or mixed profiles refuse atomically.
Required String and signed 64-bit Integer per-use cells inherit the existing bounded
current-scalar subset; this does not widen it.

## Normative Surface

The optional feature is `current-query-refinement-candidate`; it MUST NOT be
automatically selected by an older profile or protected consumer registration.
Candidate Python methods on the actual CQP, and corresponding Rust/runtime
entrypoints, accept `(original_binding_json, refinement_input_json)`:

- `check_postgresql_current_rows_candidate_json`
- `check_postgresql_current_contributors_candidate_json`
- `check_postgresql_current_sum_candidate_json`
- `check_databricks_current_contributors_candidate_json`
- `check_databricks_current_sum_candidate_json`

These names are reserved draft surfaces until implemented and qualified. The
actual retained parent and typed plan MUST be used; no original preparation or
context argument is accepted from JSON. The corresponding actual backend lowerer
MUST run before data evaluation, including for FALSE, empty scans and LIMIT0.

The [input schema](current-query-refinement-input-v0.1-candidate.schema.json)
and [result schema](current-query-refinement-result-v0.1-candidate.schema.json)
define structural shape; the following byte, identity, type and relational rules
remain mandatory beyond JSON Schema validation. The UTF-8 JSON input is a closed
object with exactly these fields:

| Field | Type | Rule |
| --- | --- | --- |
| `version` | string | Exactly `weft.current-query.refinement-input/0.1-candidate` |
| `sourceRequestSha256` | string | Exact digest of the actual original request bytes |
| `designationSha256` | string | Exact actual CQP designation bytes |
| `materializationBindingSha256` | string | Exact supplied original binding bytes |
| `compiledArtifactSha256` | string | Exact complete artifact JSON emitted by the actual selected lowerer |
| `scans` | array | Every original scan, once, in original order |
| `resultRows` | array | Complete native rows in observed order; each row contains public carriers followed by its private JSON text carrier |

All digests are 64 lowercase hexadecimal characters. A scan object has exactly `scan`,
`expectedRowCount`, and `rows`. `scan` is the original occurrence identifier.
`expectedRowCount` is canonical unsigned decimal text; it MUST equal the complete
supplied row count. It is an independently captured host count for the same original scan occurrence,
source, primary read eligibility and held cut, before query predicates, joins or
LIMIT; neither result rows nor returned contributors may supply it. It is not a
Boolean claim of completeness or an authority credential. Matching JSON counts alone
MUST NOT be described as proof of native population completeness.

Each row has exactly `key` and `cells`. Keys are nonempty exact UTF-8 strings,
at most 256 bytes, unique within that scan. Reuse between distinct scans is valid.
`cells` has exactly the union of physical columns needed by all validated per-use
selections for that scan. Count-only scans have empty cell maps but retain every
key. Repeated uses of a physical column share its one captured value. Conflicting
typed column declarations refuse. Evaluation MUST retain the exact validated
`(scan,path)` selection for every occurrence, including original Field, effective
domain, column, comparison scope, mode/action and exposure. An original Field
identity alone MUST NOT collapse per-use mappings: one salary predicate may use
an original Integer while its projection uses a transformed String. Hidden ORDER
keys MUST come from their own validated per-use captured cells, not public DTOs.

Each cell has exactly `family` and `value`. `family` is `string` or `integer` and
MUST match the validated effective domain. `value` is text: strings preserve all
Unicode and empty-string distinctions; integers are canonical signed decimal
within signed 64-bit bounds. NULL, absent, withheld, float coercions and unknown cell
members refuse. These tagged values are data supplied by a trusted host; they
are not authenticated facts by construction.

Each native result row has exactly the expected public column count plus one
private carrier. Public carriers are strings, except the existing nullable empty
global SUM result may be JSON null. No other native null is accepted. Scalar
integer outputs use exact canonical decimal text. The selected actual artifact
owns result column order, names, logical types and codecs; these cannot be
replaced in the input frame.

ABI2's private `weft.stage-row-tuple/0.1-candidate` text is checked as one complete
ordered scan tuple. Other admitted profiles use
`weft.stage-contributor-bag/0.1-candidate` text. Versions, members, scan order,
arity and key carriers MUST match exactly. Duplicate JSON members refuse.
Contributor order within a bag is immaterial; occurrence multiplicity is not.
No contributor or group may be deduplicated, truncated or inferred from count.

A successful report is a closed object containing exactly `version` =
`weft.current-query.refinement-result/0.1-candidate`, `matched:true`,
`authority:false`, `securityReady:false`, `nativeAdmission:"required"`, and the
four exact input digests plus `refinementInputSha256`, the SHA-256 of the complete
original refinement frame bytes (including populations and native result rows).
The host MUST associate that exact frame digest with its actual held capture and
cut; a matching query/binding/artifact alone cannot reuse a report for other data.
It contains no cells, keys, population sizes, private
comparison results or partial expected result. The report is private host data,
not an importable native receipt or activation permit. Public query output still
requires the original native owner's guarded decoder and closing checks.

### Relational semantics

All input shapes, cell values, binding obligations and bounds MUST be checked
before query filtering or LIMIT can hide invalid data. The original typed plan
owns stage order: scans/inner joins, filters, grouping/aggregation, projection,
ordering, then limit. No SQL text is parsed or executed to derive expected rows.

Boolean/equality predicates and lexicographic ORDER BY comparisons preserve the
admitted primitive/domain
and paired comparison-scope requirements from actual lowering. Strings use exact
UTF-8 binary comparison; signed 64-bit values compare numerically. Parameters and
literals come from the original typed CQP, never a replacement input expression.
Relational inequalities remain refused by the unchanged selected lowerers; this
checker cannot independently widen their supported predicate subset.

COUNT counts complete input tuples. DISTINCT removes duplicates only from the
selected aggregate value domain, never from contributors. SUM accumulates exact
mathematical integers. Empty global aggregation produces one result with count0,
SUM null and an empty bag; empty grouping produces no rows. Group contributors
include every original matching joined tuple. Public projections preserve their
validated effective domains and source occurrences.

Without ORDER BY, results compare as multisets of whole public-row/contributor-bag
pairs. With ORDER BY, key order and direction MUST hold; equal keys may appear in
any relative order. LIMIT without ordering admits any submultiset of the correct
size. LIMIT within an ordered tie admits any submultiset at that boundary, while
all preceding rows are mandatory and later rows are excluded. The checker MUST
NOT invent a deterministic tie-breaker or require its own iteration order.
Result cardinality is exactly min(limit, complete pre-limit cardinality), or the
full cardinality without LIMIT. Each matched occurrence is consumed once.

### Bounds

Input text and actual lowered artifact text are each at most 16,777,216 UTF-8 bytes.
Each parsed frame/private carrier is at most 32 levels and 1,000,000 JSON nodes.
The existing source/context/binding limits remain applicable. There are at most 16
scans, 10,000 total supplied source rows, 4,096 validated per-use selections, 1,024
public output columns, 10,000 intermediate tuples and 10,000 pre-limit result rows.
A Cartesian join candidate product over 1,000,000 refuses before iteration.
No prefix, sampling or approximate arithmetic may bypass these limits. Limit0
still requires complete checked inputs and bounded derivation. These are declared
frame/cardinality bounds, not a whole-process memory or timing proof; hosts retain
native statement deadlines, resource isolation and no-partial-release duties.

## Precedence and Compatibility

A new optional feature and exact result-refinement version require explicit
selection. Existing lowerers, native-input profiles, binding versions, consumer
cohorts and public security refusal paths are unchanged. Matching compiler or
package versions do not select this checker or permit a newer result ABI.

Host factual admission precedes refinement. Conditional success cannot override
missing/indeterminate policy, action, source, caller, cut, retention or final
release evidence. Repeated checks on identical actual contexts and input bytes
are deterministic within SQL's specified ordering and limit semantics.

## Error Semantics

All failures from actual lowering, frame/private-carrier parsing, typed evaluation
and comparison MUST normalize into the same private diagnostic. Malformed,
duplicate, unknown, stale, incompatible, incomplete-declared,
over-budget or mismatching input refuses with the coarse
`WFT-CURRENT-QUERY-REFINEMENT` diagnostic and no successful/partial report. Native
host errors remain owner errors and cannot be repaired with a synthetic frame.
No diagnostic includes a key, cell, hidden count, failing row index or private
branch. Match/failure outcomes, timing, private expected data and every report
digest MUST remain within the trusted host. Coarse text is not diagnostic/privacy
isolation; public failure behavior follows the native owner's disclosure contract
and this helper cannot close nativeB10. Retry requires a complete fresh admitted host cut, not relaxed checks.

## Examples

For `SELECT COUNT(*) FROM Resource r`, a complete eligible two-row input with
scan `s0` and keys `k1`,`k2` supplies `cells:{}` for both and
`expectedRowCount:"2"`. The exact expected public value is `"2"`; its complete
bag contains both singleton scan tuples. A native value `"2"` with `k1` repeated
and `k2` missing refuses. An empty native result also refuses without LIMIT0.

For an ordered equal-key group of three eligible scalar rows with LIMIT2, any
two distinct permitted row occurrences may match. Returning one occurrence twice
refuses; returning a row beyond a smaller preceding key while omitting that
preceding row refuses. None of these conditional checks establishes native read
permission or an authenticated source population.


## Non-authority countermodel

A caller can consistently truncate supplied populations, lower the declared
counts, and forge a correspondingly matching native result frame. This pure
checker may then match those conditional inputs. The native host MUST refuse
that substitution using its independently authenticated complete source/capture
custody. Tests MUST retain this countermodel: a count mismatch is a refinement
refusal; a consistent forged world is not proof that this checker can discover
missing native rows. A report is never source completeness or permission.
