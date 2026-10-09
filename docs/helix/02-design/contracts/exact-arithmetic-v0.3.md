---
ddx:
  id: weft.exact-arithmetic-0.3
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.prd
      kind: informed_by
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
    - id: CONTRACT-004
      kind: informed_by
---

# Exact arithmetic reads

Select `weft-sql/0.3.0`, `weft-ir/0.3.0` and `weft-compile/0.3.0`
explicitly. Older named application-read subsets MUST NOT expand implicitly. Preserve every 0.1/0.2 acceptance and refusal rule; arithmetic MUST
NOT become accepted in an older profile merely because a backend supports it.
The frontend owns expression meaning; backends own exact representation and
execution obligations. Original UMF model versions, identities, constraints,
unknown content and mathematical integer domains remain unchanged.

## Expressions and relational meaning

The explicit 0.3 grammar admits bare authored Field names in field positions.
Resolve each name against every visible source Record member before interpreting
its value type or capability. Exactly one member must match; missing or ambiguous
names refuse, including self-joins and same-name unsupported or nullable fields.
Quoted names preserve exact case and embedded punctuation. Unquoted names retain
the existing identifier rules. Projection aliases are not source members and do
not become an implicit lookup scope. JOIN ON sees only its current join prefix.
Relationship arguments in HAS_RELATED and RELATED_KEYS remain qualified; bare
relationship-name resolution is outside this Field-only extension. Fully qualified
names retain their existing meaning. Older 0.1/0.2 grammars remain
qualified-only; their acceptance and refusal boundaries do not widen.


Add `+`, binary `-`, `*`, unary negation and parenthesized numeric expressions to
scalar projections and comparison operands in WHERE and INNER JOIN ON.
GROUP BY, ORDER BY and aggregate arguments retain their field-only forms;
arithmetic in those positions refuses in this first profile. A computed grouped
projection may reference only grouped fields and constants. Multiplication binds before addition
and subtraction; unary negation binds more tightly than multiplication.
Equal-precedence binary operations associate left. An explicit
projection alias names each computed output. Operands are pinned required scalar
numeric fields, exact numeric literals or typed numeric parameters. Strings,
booleans, unavailable values and unknown numeric meanings MUST refuse.

Evaluate expressions over each joined row combination. Retain bag multiplicity;
filters remain before grouping. Existing SUM(Field) and COUNT semantics remain
unchanged. Division, rounding, floating arithmetic, implicit string conversion,
aggregate-expression nesting and SUM(expression) are outside this first profile.

Integer-only expressions have mathematical integer results without inferred
machine width. Decimal-containing expressions have exact decimal results:
addition/subtraction use the maximum operand scale; multiplication sums scales.
Derived precision is unbounded unless an independent expression proof establishes
a finite bound. A backend width MUST NOT become the logical result precision.
Comparisons align exact numeric values without rounding or floating substitution.
The IR MUST distinguish derived numeric domains from original UMF Field facets,
and retain every operand identity, literal token and source span.

Integer operands, integer literals and integer parameters have scale 0. A decimal
literal has its lexical fractional scale; a decimal parameter derives that scale
from its exact supplied token. An authored Field keeps its own declared scale.
Insignificant trailing zeros remain retained in the original token even when a
field-domain check admits an equivalent value at its declared scale. Numeric
promotion embeds integers into decimal arithmetic exactly at scale 0. Every named
parameter MUST satisfy all occurrence-specific logical constraints before binding;
repeated uses cannot overwrite or widen another occurrence's domain. Native
capacity is checked separately and never defines a literal or parameter type.

Source validity, native integrity and operand capacity cover every row in the
bound selected scans, including rows later filtered out. JOIN ON arithmetic
intermediates cover every candidate pair at that join stage, before its ON
predicate; WHERE intermediates cover the full joined bag before WHERE. Projection
intermediates cover surviving rows after WHERE, or the grouped rows when grouping
applies, before ORDER/LIMIT. Preserve these logical row sets through optimization;
predicate pushdown, short-circuiting and unused-result elimination MUST NOT hide a
required check. A backend may discharge a check by an independent exact proof,
but MUST NOT infer safety solely from the final selected result.

Expression parsing/resolution MUST enforce common bounded node and nesting-depth
limits before recursive work or allocation. Publish exact limits with the profile
and preserve source spans on resource refusals. Deep nesting, large expressions,
hostile literal lengths and boundary-sized valid expressions require fixtures.

## Backend admission and results

Require separately declared exact arithmetic capabilities for the selected
operators and integer/decimal domains. The backend manifest MUST admit the explicit
language/IR pair. Version and capability admission MUST precede backend binding,
so an older backend never receives a new derived domain. Unsupported profiles,
operators or derived domains block before executable SQL is returned. Candidate behavior still requires allowCandidate.

A finite native representation MUST either return the exact result or raise an
explicit capability failure. Check every intermediate and final result; overflow,
underflow, scale reduction, rounding and overflow-to-null MUST NOT become values.
Cancellation MUST NOT disguise a failed intermediate operation. Preserve source
validity, native scalar integrity and finite representation capacity as separate
obligations, including sources later excluded by a user filter. Unknown or
unfulfilled obligations block host execution.

Results carry exact integer or decimal text plus the declared derived scale and
nullability. No JSON number or host float may carry an exact computed value.
Original source presence and aggregate NULL remain governed by their owning
contracts. Compilation establishes no source authority, publication or read pin.

### Spark coefficient candidate

The opt-in `spark4-delta4-arithmetic-candidate` target MAY admit nonaggregate
required-scalar row queries at the explicit 0.3 language/IR pair. It MUST preserve
source facets and use signed `DECIMAL(38,0)` coefficients with separately retained
scales, at most 18 in this finite profile. Native capacity MUST NOT appear as a
logical integer width or derived decimal precision. Literal/parameter slots MUST
retain their original tokens and logical domains; derive physical coefficients
inside SQL. Precisionless decimal parameter metadata requires explicit 0.3 exact
decimal capability admission and its original lexical scale.

Every negation, subtraction, addition, multiplication and comparison scale-alignment
product MUST use exact-or-null native operations, with pre-execution checks that
turn any null intermediate into complete-query capability refusal. These checks
MUST use the logical candidate row sets above, including every ON/WHERE conjunct.
Final decimal text MUST be formed from the coefficient's sign and zero-padded
digits; division, floating-point conversion and rounding are forbidden. Hosts MUST
fulfil source-integrity, arithmetic and publication obligations in order, execute
compiler SQL unchanged, buffer the full result, and recheck the complete original
publication context before release. This finite candidate requires independent
native qualification; registration or compilation alone establishes no support.

## Required evidence

Freeze executable grammar, IR/request/response/result schemas and positive/refusal
cases before advertising this profile. Native Python and browser WASM must compile
the same versioned requests through the Rust core; no host SQL rewriting is allowed.

Use independent arbitrary-precision arithmetic and row bags for precedence,
negative values, mixed integer/decimal expressions, large coefficients, duplicate
joins, exact scale, cancellation and finite intermediate/result overflow. Preserve
older-profile refusal and backend-feature registration controls.

The original commerce scenarios require actual unchanged compiled SQL for partial
return (`ordered - fulfilled + returned`), over-fulfillment, exact settlement and
refund (`returned * original price`). Retain the original model/data; compare full
result bags against the independent source oracle. A compile-only result or named
refusal does not satisfy any promised native scenario pass.
