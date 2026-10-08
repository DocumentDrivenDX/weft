# B-007 scalar resolver branch accounting

Scope: `crates/weft-core/src/resolve.rs`, the retained 0.1 resolver. Catalog,
literal validation, parser, application resolver and backend lowering have their
own branches; this review does not claim to close those scopes.

| Semantic branch | Executed assertion |
| --- | --- |
| Admit a source and retain distinct scan occurrences | `on_scope_self_joins_and_group_only` asserts s0/s1 identity for a self join; original sales fixtures assert qualified plans. |
| Duplicate scan alias | `semantic-refusal-6`, unchanged 636-case frontend corpus, WFT-NAME-AMBIGUOUS. |
| Visible and missing scan aliases | Original sales/projection fixtures; `semantic-refusal-0` and the later-alias ON assertion in `on_scope_self_joins_and_group_only`, WFT-NAME-MISSING. |
| Selected string/boolean/integer/decimal capability | Unchanged corpus's exact string, integer-width, decimal and boolean predicates; module/field identity and types are checked by the resolution audit. Catalog field admission is a separate scope. |
| Column and typed-literal equality | Original sales join plus exact literal corpus; `semantic-refusal-5` rejects incompatible field families, WFT-TYPE. Literal parsing/range is owned by `exact.rs`. |
| First predicate and AND composition | Single-predicate corpus and the direct-AST 1,024-literal success, which asserts the AND capability. |
| Defensive literal upper bound | New direct internal resolver test: 1,024 admits; 1,025 refuses with exact WFT-LIMIT/type/message. This is not a new public AST API. |
| Empty JOIN predicate defense | New internal test supplies a distinct admitted join alias with zero predicates; exact WFT-SYNTAX/parse/message. Parsed SQL does not produce that input. |
| JOIN and optional filter nodes | Original sales join and authored filtered integer/decimal/string/boolean fixtures; ON visibility is separately asserted. |
| Explicit/default output labels and duplicate label refusal | Original aliased sales/default projection and SUM outputs; `semantic-refusal-4` rejects duplicate names, WFT-NAME-AMBIGUOUS. |
| Numeric SUM, integer versus decimal result facets, global versus grouped nullability | Exact numeric corpus and independent global/grouped empty bags; `semantic-refusal-2` rejects string SUM, WFT-TYPE. |
| Grouped projection membership | Original grouped sales query; `semantic-refusal-3` rejects an ungrouped projection, WFT-GROUP. |
| Aggregate with SUM, group-only aggregate and plain projection | Original grouped/global SUM cases; group-only test asserts empty aggregate expressions; plain projections occur throughout the corpus. |
| Final model pins, projection and distinct capability IDs | Initial resolution audit plus unchanged retained modules and identity-qualified plans. Registration/pipeline tests independently reject altered pins and duplicate required capabilities. |

The source review found two internal defensive guards that were not reached by
the public parsed-SQL suite. They now have direct assertions, retained in
B-007-resolver-direct-guards: nine core library tests pass, none ignored/filtered.
The named semantic families in this file are accounted for; this does not infer
full critical-path completion or native support from this one resolver scope.
