---
ddx:
  id: CONTRACT-011
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: US-009
      kind: informed_by
    - id: CONTRACT-004
      kind: informed_by
    - id: CONTRACT-010
      kind: informed_by
---

# CONTRACT-011: Bounded Unicode strings in current queries

**Status:** proposed explicit compiler candidate; no native admission.

## Requirement and original meaning

The existing UMF PostgreSQL protection catalog preserves required String Fields
with `facets.length = {min, max, unit: "unicode-scalar"}`. Original and effective
Fields may have different bounds. The current facetless String subset refuses
these models. The extension MUST preserve their actual identities and facets;
removing bounds, substituting a different model or changing a Field identity is
not an implementation of this requirement. Raw relational and graph-derived
carriers use the same per-occurrence source model and binding rules.

CONTRACT-004/010 continue to own source meaning and conditional refinement.
Authentication, transformation execution, disclosure decisions, source truth and
completeness, native constraints/comparison semantics and final guarded release
remain host obligations. A successful compiler/checker report grants none of them.

## Explicit selection and isolation

New application lineage request/report/plan versions are respectively
`weft-application-lineage/0.3-candidate`,
`weft.application.lineage/0.3-candidate` and
`weft-ir/application-lineage/0.3-candidate`.
The request retains the existing 0.2 grammar, original opaque inputs, parameter
preflight and read requirements. It selects bounded String interpretation;
0.1/0.2 continue to refuse selected bounded Strings.

The new designation profile/report/native-input version is 0.3-candidate with
exact capability list `["bounded-unicode-scalars"]`. It must pair with the new
lineage version, not an old source context. SUM is not admitted by that capability.
Old designation profiles refuse the new context. Source/pin/action custody and
all original per-occurrence completeness checks remain mandatory.

New PostgreSQL current-values binding version is 0.5-candidate, explicitly naming
the capability. Separate bounded rows and contributor APIs produce scalar ABIs5
and6 respectively. New Databricks contributor binding/artifact version is
0.3-candidate, naming the same capability. Old entrypoints, binding versions and
result ABIs continue to refuse bounded contexts. No existing registration or
public module pin is upgraded implicitly. New runtime/Python/WASM lower/check
methods require the separate `bounded-current-string-candidate` feature; it is
not selected by the existing standard-wheel candidate feature set.

The new refinement input version is 0.2-candidate. Its complete-population and
whole-result schema otherwise retains CONTRACT-010, including all budgets and
private result/custody requirements. Old refinement profiles remain closed.

## Exact scalar domain

The new profile admits required cardinality-one Strings with either empty facets
or exactly one length facet. A length facet has exactly `min`, `max` and `unit`;
unit is exactly `unicode-scalar`. Bounds are nonnegative exact mathematical JSON integers no
larger than 9007199254740991, with min <= max. Decimal/exponent spellings of the same integer and negative zero retain their
original representation and receive the same bound meaning, without f64
conversion or exponent-sized allocation. Missing members, booleans, nonintegral
values, alternate units and unknown selected facets refuse. No allocation or
iteration may depend on the declared maximum.

Length is the number of Unicode scalar values in the original string. UTF-8
bytes, UTF-16 code units and grapheme clusters are different measures and MUST
NOT substitute. Supplementary characters count once; combining sequences retain
each original scalar. No normalization, case folding, replacement or truncation
is performed. Invalid Unicode source JSON refuses. Existing framing/text budgets
still apply independently of declared length.

Existing required exact signed64 Integer support is retained in the new profile.
Other families/facets, nullable or absent operand domains, withheld cells and
nullable token carriers remain unsupported. This chunk does not add three-valued
predicates, null-sensitive aggregates, operators or disclosed SUM.

## Original operands and effective values

Original literal/parameter values are validated against their exact original
Field domain during preparation, before FALSE/LIMIT0 or other optimization.
Their values and original logical types remain unchanged in parameter metadata.
Each selected effective cell is independently validated against its own exact
selected domain. No transform is applied to a literal or parameter.

For literal/parameter equality in the new profile, original and effective String
types share the Unicode scalar comparison family even when their bounds differ.
Each operand retains its own domain: the literal must satisfy its original
bounds, not the other operand's bounds. A literal outside the effective cell's
range can compare unequal; it is not truncated or automatically transformed.
Cross-family comparison still refuses. Field-to-field comparisons continue to
require semantically identical complete logical types and the same comparison
scope. Equivalent exact JSON spellings of a bound have the same domain meaning;
the original facet representations remain retained. This
explicit extension does not relax the older profile's compatibility checks.

Grouping/projection/hidden ordering retain the exact selected effective domain,
carrier and scope. Identical source Field identity alone cannot merge different
occurrences. Predicate, order, group, join, COUNT/DISTINCT and existing legal
LIMIT/tie behavior retain CONTRACT-010. Prohibited, unresolved, missing or
mismatched per-use modes refuse before SQL generation, including empty queries.

## Validation order and host boundary

The conditional checker validates every supplied eligible cell and public cell
against its own domain before deriving a relation, even for FALSE, empty groups
or LIMIT0. Complete source populations/counts, exact bags and public result
occurrences remain independent inputs. An invalid cell cannot disappear through
filtering, grouping, ordering or pagination. Private diagnostics do not include
private offending values.

SQL still uses the existing required binary comparison/encoding rules and native
parameter types. It does not claim that TEXT/STRING alone enforces these facets.
The host must establish exact producer/native constraint correspondence and
complete value-domain validation before public release, including complete
eligible inputs that a query would filter away. The compiler cannot inspect a
live installation. PostgreSQL/Databricks native support must be independently
qualified; an offline checker comparison is not native execution evidence.

## Acceptance-linked verification

This is a prerequisite for UMF pg-raw.P03/US-056-AC6, not its completion. Nullable
Token semantics, native operator gateway, actual catalog-selected transforms,
actor/source/ACL/OID correspondence and the existing weakened-barrier control
remain required before that backend case can be accepted. All original132
shared/native requirements and all current158 allocated cases remain required.

STP-009 allocates the implementation checks below to US-009-AC3/AC4. No complete
criterion is closed by this candidate. Independent Astra Ultra review is required
before owner main integration; return a frozen patch and exact source/test pins.
