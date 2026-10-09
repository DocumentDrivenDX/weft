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

## Additive scalar comparisons

The explicit 0.3 grammar additionally admits `<`, `<=`, `>=` and `<>` in
scalar WHERE and INNER JOIN ON comparisons. Multi-character operators MUST be
contiguous; `!=`, `=>`, `=<` and new tuple operators refuse. Existing `=` and `>`
IR serialization and old-profile acceptance/refusal rules remain unchanged,
including the existing lexicographic `>` tuple and Boolean `>` behavior.

`scalarCompare` retains the original left Field and right typed Field, literal
or named parameter; `arithmeticCompareExtended` retains original numeric
expressions. Both use the closed operator values `less`, `lessEqual`,
`greaterEqual` and `notEqual`, with separately required `compare.less`,
`compare.lessEqual`, `compare.greaterEqual` and `compare.notEqual` capabilities.
New operators in ON additionally require `compare.scalarJoin`; this does not
widen the original field-equality ON policy for operator-free `=` or `>`.

Scalar operands MUST satisfy the original selected Field/presence/family/facet
rules and every occurrence-specific literal or parameter constraint. Scalar
fields of different families refuse. Numeric expressions retain the established
exact integer/decimal promotion. Strings preserve Unicode scalar sequences with
explicit UTF8_BINARY comparison, without normalization or coercion. New Boolean
ordering operators refuse; `<>` admits exact Boolean operands. Nullable/absent
values and unknown meanings remain outside this profile. All numeric comparison
scale-alignment guards cover complete pre-ON and pre-WHERE candidate bags before
any user predicate can eliminate rows. New operator IR/capability admission MUST
precede backend binding; hosts MUST execute emitted SQL unchanged.

## Positional outputs with repeated Field labels

The explicit 0.3 profile MAY admit repeated implicit unaliased scalar Field
output labels with `project.positionedOutputs`, gated before backend binding.
Explicit duplicate aliases, computed duplicate labels and whole-entity label
collisions remain refused. Older profiles retain their duplicate-label refusal.
Original Output names and Field identities/scan occurrences MUST remain unchanged;
ordered output array positions identify each cell independently.

A positioned artifact MUST declare a unique bounded physical `carrierName` for
EVERY output, separate from the original logical `outputName`, and retain
contiguous integer positions, exact output count/order/type and original lineage.
Backends MUST generate collision-free physical aliases across all outputs rather
than preserve only some logical labels. Unique-label artifacts MUST omit
`carrierName` and retain their original bytes. Older response schemas stay closed.

The `weft.output.positioned` host obligation requires explicit carrier-name
admission, actual native column count/order/name checks, original logical-plan
ordinal/scan lineage checks, and complete exact ordered row arrays. Unknown
carrierName or this obligation MUST refuse before user SQL. Named dictionaries
MUST NOT collapse repeated logical labels; hosts MUST NOT rewrite SQL or rename
results to bypass missing compiler support. Python/browser transport MUST preserve
all repeated cells and metadata through the shared Rust implementation.

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

## Explicit scalar null tests and tagged optional outputs

The 0.3 language admits scalar `Field IS NULL` and `Field IS NOT NULL`. These predicates retain original Field identity, ideal scalar domain and availability; they do not equate absence with null or rewrite source declarations. Older language/IR profiles retain their required-only scalar refusal. Numeric zero, Boolean false and empty String are nonnull values. Arithmetic on optional operands remains unsupported.

The finite Spark4 candidate supports these tests only with an explicitly admitted original Props JSON home. The exact optional scalar Field/property home must explicitly select Props `encoding:"ashlar-weft-json-native-null/0.1-candidate"`. Required/compound Fields and older profiles refuse that encoding. Guard obligations retain original property ID, identity and encoding. Every consumed optional property must exist: source-valid absence refuses backend representation before user SQL. Present JSON null and present exact scalar values are separately admitted; malformed nonnull values never become null through a failed cast. Required properties containing null retain their original source/integrity refusal. Original schema revision, carrier type and numeric precision/scale checks cover every consumed row, including rows later filtered away.

Optional String equality in INNER JOIN ON uses original binary String equality with native-null UNKNOWN/nonmatch semantics; null/null is not equality. Full join-prefix candidate guards precede ON evaluation. WHERE null tests do not erase original optionality or remove full-source guards. Other optional comparisons and optional arithmetic require separately governed support.

Optional outputs retain `Representation::Value {descriptor,nativeNull:true}` under an explicit selected native-null capability. Their exact JSON tagged carrier is `{state:"null"}` or `{state:"value",value:...}`; nonnull String/Integer/Decimal use exact text and Boolean uses a Boolean value. The ideal scalar descriptor stays nonnullable, while availability remains independently authored. Absent state is not synthesized or silently collapsed into the null tag. Hosts must verify the exact selected descriptor/property home, decode tags without coercion and withhold results until source, publication, guard and cleanup contexts close.

The new predicate/native-null capabilities must be assessed before backend binding/lowering callbacks. Acceptance must cover original optional String join/projection, decimal18,2 output, missing key refusal, null/null nonmatching joins, malformed scalar refusal, required null refusal, zero/false/empty-string distinctions, unknown homes/facets and unchanged older-profile refusals/artifacts.
