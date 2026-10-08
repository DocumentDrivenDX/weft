# B-007 application resolver semantic branch accounting

Scope: `application_resolve.rs`, Weft IR 0.2. Model/type-graph construction,
lexing/parsing, exact arithmetic and backend lowering are separate scopes.
This is a semantic-family review, not a claim about every private subcondition
or every possible model/query combination.

| Family | Assertion evidence |
| --- | --- |
| Source occurrence, repeated alias and ON visibility | `join-count`, `forward-alias`; new repeated-alias refusal and same-record self-join assertions retain different scan occurrences. |
| Field lookup and four exact scalar families | Retained application corpus and the 303 unchanged positive 0.1 cases ported to explicit 0.2; catalog lookup/refusal has separate accounting. |
| Descriptor capabilities, presence and graph reuse | `whole-entity`, `scalar-page` and `plans_keep_domains_presence_join_bags_and_cursors`; native value/compound corpora exercise map/sequence/structured shapes through the public compiler. The new self-join keeps shared logical identities and distinct output scans. |
| Field/literal/parameter value selection | Equality/join/cursor corpus; incompatible-field-family refusal, exact numeric boundary parameters and repeated parameter use-domain intersection. |
| Parameter profile/count/name normalization | Profile version refusal, 1,024 used bindings admitted and 1,025 refused; invalid ASCII identifier, case-fold collision, quoted parameter and surplus/missing bindings refuse. |
| Exact lexical parameter text | `injection-text`, integer/decimal boundary corpus; exponent/plus/numeric injection refusals; new uppercase-boolean and NUL-string refusals. Numeric range checking remains owned by `exact.rs`. |
| Equality versus lexicographic predicates, AND and JOIN | `join-count`, single/composite cursors; new 1,024-predicate plan asserts AND. ON literals/greater comparisons refuse field-equality admission; the initial literal refusal is retained. |
| HAS_RELATED placement, key arity and key value restrictions | Related/inverse page/filter fixtures, arity refusal, new ON-placement and field-key-operand refusals; inverse EXISTS asserts both exists and inverse capabilities. |
| Grouping uniqueness and scan-sensitive membership | `duplicate-group`, `count-ungrouped-field`; new self-join grouping refusal demonstrates that matching logical identity in another scan does not satisfy grouping. |
| Entity, field, COUNT, SUM and related output dispatch | Whole entity/scalar/count/related corpus; whole-entity and RELATED aggregation refusals; boolean SUM refusal. New integer/decimal global/grouped SUM assertions retain result facets and nullability. |
| Labels, expansion bound and empty output | `duplicate-expanded-name`; new 256/257 unique expanded member boundary and empty typed projection refusal. This expansion boundary follows valid entity syntax rather than bypassing the parser's projection-count limit. |
| Aggregate ordering | `count-nongrouped-order`, grouped-count positive; ordering must reference grouped fields. |
| Count-summary recognizer | Global/grouped/join count successes; unbounded group and invalid output refusals; new missing-COUNT, global-LIMIT, cursor and relationship-filter refusals. |
| Entity/related-page recognizer | Single/composite cursor and related/inverse page successes; missing order/limit/wrong key and relationship-required/excluded refusals; new joined-source, field-valued cursor, missing key and ambiguous equivalent-key refusals. |
| Final typed plan, pins and complete parameter usage | Existing plan/retention assertions, named-parameter intersection, surplus refusal and same-record scan assertions. Backend emission/parameter transport receives separate checks. |

The new checks supplement the unchanged 620-case application corpus and 303
positive queries ported from the original 636 cases. All nine application resolver
tests pass, with no ignored or filtered tests, in B-007-application-resolver-audit.
The families above are accounted for at the resolver boundary. This does not
close the remaining parser/exact/type-graph/backend branch review or native
supported-profile qualification. No new language or native domain is claimed.
