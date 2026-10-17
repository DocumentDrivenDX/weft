# Raw PostgreSQL binary prefix — 2026-10-16

WR-A11 / S13 additive compiler candidate. The closed raw-read predicate grammar
admits `STARTS_WITH(aliased.string_field, 'literal' | :declared_string_parameter)`.
Both arguments can be parenthesized. It lowers to `pg_catalog.starts_with` with
explicit `pg_catalog."C"` collation on each string operand; it forms no LIKE
pattern and preserves original source, typed parameter inventory/order, native
obligations and covering digest. Function qualification, modifiers, arity,
non-string operands, non-scalar prefix expressions and ungrouped HAVING fields
refuse before executable SQL is returned. NOT/AND/OR retain the existing guards.

The original [PostgreSQL 18 string function documentation](https://www.postgresql.org/docs/18/functions-string.html)
defines the prefix operation. Its [18 source implementation](https://github.com/postgres/postgres/blob/REL_18_STABLE/src/backend/utils/adt/varlena.c)
uses byte comparison for deterministic collations; forcing C avoids accepting a
nondeterministic database default. Exact native Unicode, null, literal/backslash
and independent-role E2Es remain required; pure compiler checks do not establish
database behavior or performance.

Rust 1.90.0: all nine raw-read regressions pass, including literals with empty,
percent/underscore/backslash, composed/decomposed Unicode, U+1D11E and quote values;
PostgreSQL18 and Lakebase18 compilation; nullable/non-null string parameters;
wrong carriers, unsupported/custom functions and invalid modifiers/arity;
ungrouped HAVING refusal and grouped HAVING acceptance. Retained log includes
pre-existing warnings. No tests skipped.

The private next compiler uses Python `0.4.42.dev0` / native `0.4.42-dev.0` so
it cannot be confused with the immutable stable 0.4.41 release cohort. This packet
is not public wheel evidence. Term filters, aggregate/bucket additions, broader
search and managed mutations remain separate prerequisites. SDK temporal
qualification remains **HELD**.
