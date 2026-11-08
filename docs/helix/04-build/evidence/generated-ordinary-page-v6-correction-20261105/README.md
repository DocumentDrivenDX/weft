# Generated ordinary Page `/6` correction evidence

**Class:** local Weft source, schema, and issuer-generated native SQL evidence
only. This is not a wheel, UMF consumer, generated-client, host, service, or
release qualification.

## Source and schema corrections

This correction follows Sol's review of the prior source evidence. The `/6`
page descriptor now carries a closed `fieldOutputs` inventory. Each selectable
Field is joined by exact source Field identity to one source table column and
one bound native column, with carrier, nullability, native type, and PostgreSQL
OID checked before issuance. Empty selection remains represented by the empty
projection slot.

`issuedProgramDigest` now hashes the canonical complete root, with only its own
digest member omitted. This includes inherited `selectSql`, predicates, output
metadata, and all Page fragments. The test recomputes the digest and changes
`selectSql` to prove that the changed root no longer matches. The v6 schema
closes the emitted OID set to issuer-supported PostgreSQL types.

The retained `issuer-test.log` is the final focused Rust run. It passed
`issues_closed_ordinary_page_and_keeps_v5_independent`, including the frozen
`/5` byte comparison, exact root digest coverage, field inventory, nested
repeated predicate, six-slot DB-API order, and v6 unsupported-root behavior.
The initial edit attempt briefly placed the Field inventory in the v5 path and
failed compilation; that source mistake was removed before the passing run. The
final generated response is `issued-v6.json`.

Ajv 8.17.1 validation of the corrected instance and five schema-drift negatives
passed in `schema-validation-retry.log`. The earlier probe accepted OID 25,
which is valid for the emitted text Field; that incorrect negative probe is
retained in `schema-validation.log`. The schema now rejects unsupported OID
9999.

## Native SQL proof

PostgreSQL 18.6 executed a prepared statement shaped from the issued Page SQL
with nested repeated equality filters. It binds the three filter occurrences
as `$1`–`$3`, continuation positions as `$4`–`$5`, and page size as `$6`.
`native-filter-retry.log` shows the query returned the expected single row
`(ordinal 1, key b, selected value B)`. Rust `dbapi_sql_with_order` asserted
`filterValues, filterValues, filterValues, after, after, size` for the same
nested predicate shape.

The clean retry used a fresh temporary cluster at port `55494`, data path
`/private/tmp/weft-v6-correction-pgdata-retry-20261105`, and socket path
`/private/tmp/weft-v6-correction-pgsock-retry-20261105`. PostgreSQL was fast
stopped and the task-owned cluster, socket, and server log were removed. The
first attempt at port `55493` included a malformed psql `\\set` line; its
output is retained in `native-filter.log`, and its separate cluster was also
fast stopped and removed. PostgreSQL `55441` was not used or modified.

This proof validates issuer SQL and parameter order only. It does not claim
continuation object creation or resume identity verification, UMF
materialization/slicing, deleted-anchor behavior, installed packages, or
consumer admission. Exact-full continuation remains the conservative ordinary
Page rule: a full visible page carries continuation even when lookahead found
no additional row. Unsupported branches and broader ORM semantics remain as
documented in CONTRACT-018.
