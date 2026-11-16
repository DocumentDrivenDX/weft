# weft-postgresql-emit

PostgreSQL emission primitives shared by weft's PostgreSQL backends
(`weft-postgresql`) and other compilers' PostgreSQL bindings (Fireweed): quoted
identifiers, numbered parameter slots, string literals, typed casts with the
`C` collation, and exact result decoders.

It depends on `weft-model` (logical types, family sets, diagnostics) and
`weft-backend` (the SQL carrier's `ParameterSlot` and `ScalarDecoder`). It
knows no storage layout: Truss's table and property layout stays in
`weft-postgresql`, and a Fireweed layout is a different binding over the same
primitives.

## Version

Published to crates.io with every weft release, at the release version: the
`vX.Y.Z` tag, the `umf-weft` wheel and all four shared crates (`weft-model`,
`weft-expr`, `weft-backend`, `weft-postgresql-emit`) carry the same number, and
`weft_postgresql_emit::VERSION` reports it. Depend on them at one exact version, for example
`weft-postgresql-emit = "=0.4.58"`. While weft is 0.x any release may change the public
surface below; `CHANGELOG.md` records each change. Emitted text and refusal codes are part of the surface.

## Public surface

| Item | Meaning |
| --- | --- |
| `Identifier`, `qualified` | A quoted identifier in the standard `NAMEDATALEN=64` profile (1–63 UTF-8 bytes, no NUL, `"` doubled); namespace and object quoted separately. |
| `Parameters`, `CatalogDomain` | Per-query numbered slots `$1..$1024`. Values never enter SQL text. Typed (`push`), exact binary (canonical base64), exact timestamp and date slots, and signed native catalog discriminators (`int4`/`int2`). |
| `string_literal`, `escape_string_literal` | `'...'` and `E'...'` literals; `None` for text containing NUL. |
| `COLLATE_C`, `collate_c` | The bytewise `pg_catalog."C"` collation used for every text comparison and ordering. |
| `cast_type`, `typed` | `expr::pg_catalog.text COLLATE pg_catalog."C"`, `::bool`, `::numeric` (exact integers and decimals); for opt-in families `::timestamptz` and `::float8`. |
| `decoder`, `decoder_for` | The exact result decoder for a column's family (`ScalarDecoder::exact`). |
| `decodable` | Projects an expression to the text its decoder reads: `to_char(.. AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"')` for timestamps, `encode(float8send(..), 'hex')` for floats, `::text` for integers and decimals. Instants outside years 0001–9999 or infinite are not decodable. Experimental: only unit tests use it. |
