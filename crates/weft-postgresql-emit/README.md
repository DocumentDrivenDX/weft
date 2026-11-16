# weft-postgresql-emit 0.1.0

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

Own semver, independent of the workspace and the `umf-weft` wheel;
`weft_postgresql_emit::VERSION` reports it. While the major version is 0, a
minor bump may break the public surface below; a patch bump may not. Emitted
text and refusal codes are part of the surface.

## Public surface

| Item | Meaning |
| --- | --- |
| `Identifier`, `qualified` | A quoted identifier in the standard `NAMEDATALEN=64` profile (1–63 UTF-8 bytes, no NUL, `"` doubled); namespace and object quoted separately. |
| `Parameters`, `CatalogDomain` | Per-query numbered slots `$1..$1024`. Values never enter SQL text. Typed (`push`), exact binary (canonical base64), exact timestamp and date slots, and signed native catalog discriminators (`int4`/`int2`). |
| `string_literal`, `escape_string_literal` | `'...'` and `E'...'` literals; `None` for text containing NUL. |
| `COLLATE_C`, `collate_c` | The bytewise `pg_catalog."C"` collation used for every text comparison and ordering. |
| `cast_type`, `typed` | `expr::pg_catalog.text COLLATE pg_catalog."C"`, `::bool`, `::numeric` (exact integers and decimals); for opt-in families `::timestamptz` and `::float8`. |
| `decoder`, `decoder_for` | The exact result decoder for a column's family. Timestamp and float columns have no decoder in the SQL carrier yet (`decoder_for` returns `None`). |
