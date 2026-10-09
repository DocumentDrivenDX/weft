---
ddx:
  id: CONTRACT-001
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-001
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
---

# CONTRACT-001: Weft SQL v0.1 and logical plan

**Type:** language/library. **Version:** `weft-sql/0.1.0`, `weft-ir/0.1.0`.
**Status:** draft; specified syntax is not implemented support.

## Purpose

Define one source SQL dialect over supplied UMF modules, independent of the
backend's target dialect. The frontend MUST resolve meaning before physical
lowering. The normative grammar is [weft-sql-v0.1.ebnf](weft-sql-v0.1.ebnf).

## Scope and Boundaries

One read-only query targets one registered backend. Multiple supplied UMF modules
and owning documents MAY participate in one query if the target binds all scans.
No network/catalog discovery, physical schema inference or cross-target federation
occurs. Module preparation is CONTRACT-003; physical lowering is CONTRACT-002.

## Normative Surface: Naming and Types

A queryable source is a known core record with explicit members and nonempty
`name`; its logical qualifier is the owning module's `namespace`. Queryable
properties are referenced member fields with nonempty `name`. Display names
resolve to exact `{documentId,revision,module,element}` identities; names never
become identity. Unqualified sources MUST be unique across supplied modules.
A qualified source is `namespace.record`; namespaces containing punctuation/dots
require quoting as a single identifier. Record member references stay local to
their owning UMF document; dependencies must be supplied, never guessed.
Ambiguity/missing names, duplicate identities or unsupported selected references
MUST block. Unknown unselected contents remain retained and diagnostic. Reserved keyword tokens require quoting as names.

Unquoted identifiers are ASCII letter/underscore followed by ASCII letters,
digits/underscore and fold to lowercase for lookup; ASCII letters in model names also fold for an unquoted lookup. Collisions after folding are ambiguous. Double-quoted identifiers
match exactly, preserve Unicode scalar sequence and escape quotes by doubling.
No Unicode normalization occurs. Bare properties and wildcard projections are
outside v0.1. Qualified columns name their scan alias; a source without explicit
alias uses its record name. ON scope includes previous scans and its own scan,
never a later alias. Duplicate scan aliases and output labels MUST block.

Selected fields MUST be known `field`, `cardinality:one`, `nullability:required`,
without record/item type assertions. Initial families: boolean, string, integer,
decimal. Integers require known width 1..64 and signedness; decimals require known
precision 1..28 and scale 0..precision. Original core 0.8 facetless integer Fields
additionally establish mathematical integer meaning without an authored width;
their LogicalType MUST remain integer with empty facets. Selection MUST require
`type.integer.unbounded` in addition to `type.integer`. Core 0.7 behavior remains
unchanged. Exact integer literal tokens MUST remain lexical arbitrary-length
base-ten integers within common resource limits, without float or i128 coercion.
Other/unknown selected facets block.
Native refinements MUST agree through the backend binding's qualified semantic
profile; core scalar labels alone never establish equality/encoding.
Optional/absent, explicit null fields, arrays/maps, floats/binary/temporal fields,
unknown kinds/types and relationship-path expressions block in this version.
Later dialect versions may add them with explicit meanings and fixture coverage.

## Normative Surface: Relational Meaning

Scans and INNER JOIN are bags. Equality predicates combine with AND; TRUE/FALSE
values are logical booleans. Literal domains must match the field family without
implicit string/numeric conversion. Integer literals MUST fit selected width;
decimal literals MUST fit precision/scale exactly (insignificant trailing zeros
may be accepted; rounding may not). String equality compares Unicode scalar
sequences exactly, including trailing spaces and composed/decomposed differences.
Only well-formed scalar text is accepted; NUL is refused in the initial target
profile. Empty strings are valid values.

A join preserves every matching row combination. The compiler MUST NOT infer
relationship traversal, DISTINCT, key uniqueness, existential joins or a missing
edge table. Explicit property joins need no relationship assertion. WHERE applies
before grouping. GROUP BY exact value tuples includes one group per distinct
logical tuple; output order is unspecified. A plain projection in a grouped or
aggregate query MUST be one of the grouped fields on the same scan.

SUM accepts integer or fixed-scale decimal. Integer SUM has an exact mathematical
integer result; decimal SUM has an exact decimal result with the argument scale.
Global SUM over an empty input is SQL NULL, never zero. A grouped query over no
rows has no groups. Duplicate input rows contribute repeatedly. Arithmetic must
be exact or fail with an explicit numeric-domain error; wrapping, rounding,
floating substitution and overflow-to-null are forbidden. A target's finite
aggregate domain is an explicit obligation/refusal under CONTRACT-002, never
silently assigned to the logical type. Null aggregate output is distinct from
UMF absent/present-null field semantics.

## Logical Intermediate Representation

The resolved IR is typed data, not source SQL. Nodes are Scan, InnerJoin, Filter,
Aggregate and Project. Expressions are FieldRef, Literal, Equal, And and Sum.
A FieldRef names scan occurrence and exact UMF field identity; repeated self-join
scans have distinct occurrence IDs. A Scan names exact record identity and module
pin. Numeric literals retain exact lexical text. Each expression/output records
scalar family, presence/result nullability and established facets/domain.

The plan retains SQL source byte spans, required capability IDs, selected module
pins and diagnostic origins. It MUST NOT contain target table/column names or
backend SQL fragments. The first plan is a deterministic tree with Scan occurrence
IDs in FROM/JOIN order. Filter/aggregate/project order preserves the semantics
above. Versioned IR types are internal library contracts; the serialized structural schema is [logical-plan.schema.json](logical-plan.schema.json).
Semantic checks enforce type/operator rules, identity resolution and ordered byte
spans beyond that schema. Optimizer/plugin Rust types must match this shape.

## Precedence and Compatibility

UMF governs model meaning; this contract governs query language meaning. Backend
capability absence cannot redefine accepted SQL semantics. New syntax/meaning
needs a new dialect profile; unknown AST nodes are refused even if a parser accepts
them. A source-language version is selected explicitly and never auto-detected
from the target. The EBNF excludes comments, parameters supplied in source SQL,
functions other than SUM, outer joins, subqueries, ORDER/LIMIT, DML and DDL in v0.1.

## Error Semantics

WFT-SYNTAX, WFT-UNSUPPORTED, WFT-NAME-MISSING, WFT-NAME-AMBIGUOUS,
WFT-TYPE, WFT-GROUP and WFT-NUMERIC-DOMAIN are blocking diagnostics with source
spans/module identity when known. A failure returns no partial executable SQL.
Correct the query/modules/selected profile; no fallback backend or retry is implied.

## Examples

```sql
SELECT c.name, SUM(o.total) AS total
FROM Customer c
JOIN Orders o ON o.customer_id = c.id
GROUP BY c.name
```

This is Weft SQL. A backend may read catalog property IDs or typed warehouse
columns, but must preserve the same named logical fields, bag and exact totals.
`SELECT name FROM Customer` and `SELECT * FROM Customer` block in v0.1.

## B-002 concrete Rust representation

The versioned representation is `crates/weft-core/src/ir.rs`: `LogicalPlan`, `Node`, `Expression`, `LogicalType`, `Identity`, `ModelPin` and `Span`. Serde encodes node/expression enum tags as schema `op` strings and uses camelCase member names. Binary expression variants box operands; nodes box inputs; scans allocate `s0`, `s1`, ... in source order. No physical identifiers occur in IR.

Default output names are the UMF field name and `sum` for an unnamed aggregate; AS supplies the parsed identifier value. Duplicate output names block. GROUP BY without SUM is supported with an aggregate node containing groups and an empty aggregate list; an aggregate node must have at least one group or aggregate. Global SUM is nullable; grouped SUM is non-null for the required input subset. SUM result facets carry decimal scale without a precision bound, or no integer width bound; argument facets remain on FieldRef. Numeric equality compares exact values within the same family even if input field domains differ. Required capability IDs are `scan`, `project`, `innerJoin`, `filter`, `equal`, `and`, `group`, `sum` and `type.<family>` as used, sorted and unique. These IDs are logical needs, not backend support declarations.
