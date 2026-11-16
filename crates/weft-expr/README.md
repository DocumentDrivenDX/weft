# weft-expr

The typed expression and predicate core shared by weft's compiler (`weft-core`)
and other compilers that type programs against UMF models (Fireweed's
`fireweed-compiler`): Field references, literal and parameter operands,
comparisons and their typing rule, exact arithmetic domains, and the
`weft.update-selection/1` predicate format of CONTRACT-009.

It depends on `weft-model` for identities, logical types, family sets and
diagnostics. It parses no source text and lowers nothing to a backend.

## Version

Published to crates.io with every weft release, at the release version: the
`vX.Y.Z` tag, the `umf-weft` wheel and all four shared crates (`weft-model`,
`weft-expr`, `weft-backend`, `weft-postgresql-emit`) carry the same number, and
`weft_expr::VERSION` reports it. Depend on them at one exact version, for example
`weft-expr = "=0.4.58"`. While weft is 0.x any release may change the public
surface below; `CHANGELOG.md` records each change. Wire shapes and diagnostic codes are part of the surface.

## Public surface

| Module | Items | Meaning |
| --- | --- | --- |
| `operand` | `Field<F>`, `Value<F>` | A Field read through a scan occurrence; a literal, typed parameter or Field operand. Serialized exactly as weft's 0.2/0.3 logical plans serialize them. |
| `compare` | `CompareOp`, `check`, `ComparisonOperator` | The six comparisons (`eq ne lt le gt ge`) with SQL text, and the typing rule: same family and facets on both sides, ordering defined on every family except boolean. `ComparisonOperator` is weft-sql 0.3's four-member wire enum. |
| `arithmetic` | `Domain`, `Expression<F>`, `Kind<F>`, `token_domain`, `MAX_LITERAL_BYTES` | Exact integer/fixed-scale decimal arithmetic: `*` adds scales, `+`/`-` take the larger scale; tokens are kept. Field operands must be required integers or scaled decimals. |
| `selection` | `PROTOCOL`, `Predicate`, `Compare`, `Refusal`, `walk`, `MAX_DEPTH`, `MIN_OPERANDS`, `MAX_OPERANDS`, `MAX_COMPARISONS` | `weft.update-selection/1`: decode with a pre-order visitor for the caller's domain checks, encode, comparisons in source order, plan `dependencies` in first-use order, connective splicing. |

`F` is a `weft_model::types::FamilySet`. weft's profiles use the default
`Family`; a compiler that types timestamps and floats uses `ScalarFamily`.

## What stays in weft-core

Each weft dialect version's plan tree (`ir::Expression`, `application_ir::Predicate`,
`arithmetic_plan::Predicate`) keeps its own frozen wire shape and stays with the
dialect that owns it; those trees are built from the operands above. Source
parsing and name resolution (`arithmetic_resolve::resolve`, the mutation SQL
parser) stay in weft-core because they read weft-sql text. Checking that a
CoreLiteral is in a UMF Field's domain is `weft_model::core_literal`.
