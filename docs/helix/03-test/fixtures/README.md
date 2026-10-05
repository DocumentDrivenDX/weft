# Conformance inputs and independent expectations

This is a specification corpus, not compiler execution evidence. It contains
636 language/numeric/model/relational scenarios, including exhaustive
signed/unsigned 1..64-bit boundary values. Expectations are independent of emitted
SQL. `sales.rows.json` is synthetic non-PII data; exact numbers are text.

All requests use explicitly named fixture-only backend profiles and illustrative
binding data. A conformance harness must register real fixture backends before
execution; these mappings must never claim production Truss/Ashlar compatibility.
`setup.schemaNegative` marks deliberate transport-schema violations; the checker
requires them to fail schema validation. Corpus guards are distinct from actual
query/engine/host tests. Expected rows compare as unordered bags (duplicates count),
with tagged exact values; NULL uses kind null without value.

Syntax refusals choose WFT-UNSUPPORTED for well-formed excluded syntax and
WFT-SYNTAX for malformed tokens/grammar. Implementations must stabilize this
classification; a parser's native error code is not the language diagnostic.
