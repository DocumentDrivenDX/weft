# B-007 exact-value and parser semantic branch accounting

This review concerns public Weft SQL 0.1/0.2 input. It assumes the pinned UMF
profile's validated scalar facets and parser-admitted literal text; it does not
claim arbitrary manually constructed Rust `Literal` or `LogicalType` objects
are safe public inputs. Native evaluation and backend support are separate.

| Exact-value family | Assertion evidence |
| --- | --- |
| String/Boolean identity and family mismatch | New complete three-literal-kind/four-field-family matrix: exact Unicode and Boolean token retention; all eight mismatches refuse without a plan. |
| Integral token requirement | New `1.0` against unsigned64 refuses, despite an integral numeric value. |
| Integer parser-range overflow | Both signs beyond i128 refuse before selected-domain admission; no panic or partial plan. |
| Signed/unsigned selected bounds | 4,200 independent generated mathematical-bound cases, seed 95955044271873, B-007-properties; new unsigned64 max, max+1, negative, negative-zero and leading-zero assertions. |
| Decimal exact scale/precision | 3,200 independent coefficient cases across precision 1–28 and valid scales, seed 95955044271874, B-007-properties; new decimal(2,2) scale-overflow and whole-digit overflow assertions. |
| Decimal zero/trailing-zero text | New negative zero and padded 0.99 retain original text; generated trailing-zero cases admit without rounding. |
| Facet preconditions | Selected UMF scalar resolution and the pinned envelope validate integer widths and decimal precision/scale before `exact::literal`. Unknown facets are selected-meaning refusals; no default domain. |
| Numeric lexical preconditions | Shared SQL lexer admits base-ten digit tokens with optional minus/fraction; exponent/plus/trailing punctuation do not become an exact literal. Named parameter lexical admission is separately accounted in the application resolver audit. |

B-007-exact-audit records two new tests passing, 25 assertions through the public
frontend, no ignored tests and five existing property tests filtered by the
explicit `exact_guard` selector. The 10,000-case property receipt remains at its
original source scope; it is not relabeled as a fresh execution. Every branch of
`exact::literal` has a named supported/refused semantic family above. This is
not native qualification or an exhaustive cross-product claim.

| Lexer/parser family | Assertion evidence |
| --- | --- |
| SQL byte/token limits | B-007 parser resource boundary test: both dialects, 65536/65537 bytes and 4096/4097 tokens. |
| Whitespace, ASCII words, quoted names and quote doubling | New names/source-form assertions and existing case corpus; quoted reserved label and doubled double quote retain exact identity. |
| String/Boolean/number/minus token dispatch | New literal-token retention in both parsers, empty string, apostrophe/Unicode, FALSE and negative decimal; negative Boolean and plus refuse. |
| Quoted lexical failure and UTF-8 spans | New unclosed Unicode and NUL cases, explicit code/message and in-range character boundaries; B-007-parser supplies 5,000 deterministic generated span checks. |
| Comments and trailing syntax | New both-comment-style refusals; trailing word, greater-than, star, other punctuation, second statement, exponent and dot refuse in both versions. |
| Identifier, source namespace and alias dispatch | New implicit/explicit/default aliases, qualified and quoted names; empty quoted identifier refuses. Existing name resolution corpus owns actual schema identity matching. |
| EOF, keyword, punctuation and operand failures | New exact EOF/name/operand diagnostics; existing malformed application cases and initial 333 refusals cover malformed statement grammar. |
| Output, aggregate and join dispatch | Initial 636 cases and application fixtures plus four original syntax tests; output256/257 and joins16/17 resource boundaries. |
| Application entity/COUNT/RELATED outputs | Existing structural assertions and wildcard/entity-alias/COUNT-argument refusals. |
| Equality, tuple cursor, field/literal/parameter and HAS_RELATED | Existing structural/arity/tuple-equality/missing-parameter cases; resolver audit separately owns type and recognizer admission. |
| GROUP/ORDER/LIMIT and finishing | Existing groups/order ASC and excluded DESC/OFFSET/multiple statement cases; bounds1/1000 and six refused literal bounds. |
| Explicit version boundary | Existing 0.1-refused/0.2-admitted application syntax assertions. |

B-007-syntax-audit records all six application syntax tests passing with zero
ignored or filtered tests. Source review finds no additional critical grammar
family beyond this map. This is semantic-family accounting, not private line
coverage or every possible malformed string. Shared parser helpers unused by a
public entry point are not counted as supported grammar. Remaining backend
lowering review and qualified native support reports are still required.
