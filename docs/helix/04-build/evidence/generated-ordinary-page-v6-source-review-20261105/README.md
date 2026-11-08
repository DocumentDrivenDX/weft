# Generated ordinary Page `/6` source review evidence

**Class:** local Weft issuer/schema evidence only. This is not a wheel, UMF
consumer, generated-client, host, service, or release qualification.

## Source and schema checks

The targeted Rust test command was:

```sh
RUSTC=/Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin/rustc \
  /Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin/cargo test \
  -p weft-postgresql issues_closed_ordinary_page_and_keeps_v5_independent
```

It passed 1 test. The test compares the generated `/5` response byte-for-byte
with the retained v5 `issued-v5.json` evidence, then checks `/6` eligible and
unsupported roots, SQL slots, identifier escaping, marker-collision refusal,
identity fields, and the issued exact-full continuation rule.

The generated response vectors are `issued-v6.json`,
`issued-v6-escaped-identifiers.json`, and
`issued-v6-unsupported-root.json`. `validate-v6.mjs` uses Ajv 8.17.1 to
validate all three against the closed response/root and descriptor schemas. It
also rejects five mutations covering unknown top-level, root, descriptor,
codec, and unsupported-branch members. The result is in
`schema-validation.log`; Cargo output is retained in `issuer-test.log`.

## PostgreSQL SQL execution

PostgreSQL 18.6 executed SQL rendered from the retained issued instances. The
temporary cluster used data path `/private/tmp/weft-v6-review-pgdata`, socket
path `/private/tmp/weft-v6-review-pgsock`, and port `55492`. It was fast-stopped
successfully after the queries, and only its temporary data, socket, and log
paths were removed. No shared or pre-existing PostgreSQL cluster was used.

`native-page.sql` and `native-guard.sql` are the executed SQL; their psql
outputs are `native-page.log` and `native-guard.log`. The fixture confirms:

- The native collation precondition returns exactly `pg_catalog`, `C`, and
  deterministic `true`.
- A size-2 initial page SQL returns two visible rows plus one lookahead row.
- A next page after Key `b` begins with `c` and returns `c`, `d`.
- An empty public projection remains valid and emits ordinal plus hidden Key.
- A quoted identifier containing `%` executes after DB-API percent escaping.
- A fetched row exceeding the 8 MiB native byte limit triggers SQLSTATE 22012.

This does not test UMF materialization/slicing, continuation object creation or
resume identity checks, deleted-anchor handling, live role/head checks, native
application admission, installed packages, or exact-full continuation
materialization. Those remain required downstream evidence. Mixed orders,
null/missing sort markers, composite Keys, joins, and target snapshots remain
explicitly unsupported by this profile.
