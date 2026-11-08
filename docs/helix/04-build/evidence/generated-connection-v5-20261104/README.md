# Generated connection v5 issuer SQL qualification

Date: 2026-11-04

This evidence qualifies only the Rust-issued forward/prior SQL templates after filling the issued `{{SELECT_COLUMNS}}` and `{{FILTER}}` slots. It does not qualify UMF cursor authentication, UMF lifecycle/head checks, pre-lease cursor validation, request-snapshot behavior, Batten hosting, or end-to-end connection semantics.

## Artifacts

- `issued-v5.json`: final actual Rust output from `prepare_generated_read_algebra_v5_json`, ordinary physical table `notes`.
- `issued-v5-percent-table.json`: final actual Rust output from the percent-escaping test, physical table `n%s` (the issued DB-API SQL template contains `n%%s`).
- Both artifacts validate against UMF `docs/helix/02-design/contracts/generated-read-algebra-v5.schema.json` using Ajv 8.20.0 Draft 2020-12. The checked root output was `/5`, readiness `/5`, ABI 1.5.
- `postgresql18-qualify.py.txt` is the Python/psycopg fixture and runner used for native execution.
- `postgresql-version.txt` and `postgresql18-results.jsonl` preserve the PostgreSQL version and observed results.

## Issued SQL template hashes

| Artifact | Forward template SHA-256 | Previous template SHA-256 |
| --- | --- | --- |
| `issued-v5.json` | `6eb2e8f20ac1eee1d89ff51e445ee6f09e4063627ed881b8cc3583f7292cd18a` | `5e461f59b191d1ced46c4f99b719b6c242422418d54652cb6961ff1d812a3e07` |
| `issued-v5-percent-table.json` | `7bd83a0dd1a6f0c387f86a94d132c7d6617fd6bb730f6f0cb6705419b84faff3` | `3a80e8cbc092894de423cd8279918f35a0d5a9ef540c275d93628b2b0947c7c3` |

## Commands

Rust tests and feature checks were run from the Weft repository with the installed Rust 1.90 toolchain:

```sh
cargo test -p weft-postgresql v5_ -- --nocapture
cargo test -p weft-postgresql v4_issues_ordered_key_batch_codec_tuple_and_rls_left_join_template -- --nocapture
cargo check -p weft-runtime -p weft-python
cargo check -p weft-runtime -p weft-python --features raw-postgresql-reads
```

The actual output artifacts were captured from the Rust issuer tests with `WEFT_V5_CAPTURE=1` and `WEFT_V5_CAPTURE_PERCENT=1`; temporary capture instrumentation was removed before the final focused test run. Both final artifacts were then schema-validated with Ajv 8.20.0 from the UMF repository. Native SQL execution used PostgreSQL 18.6 (Homebrew), psycopg 3.3.6 from Batten's existing virtual environment, an ephemeral cluster in `/private/tmp/weft-v5-pgdata`, socket `/private/tmp/weft-v5-sock`, and port 55439. The cluster was stopped and its data/socket paths removed after the run.

The fixture used a C-collated `text` primary key and a parameterized `owner = %s::pg_catalog.text` filter. It executed the final issued SQL for zero-first one-row probing, `first=2` with limit 3, continuation after `a`, prior false with a null cursor, and prior true at `b`, for both table names. All assertions in the preserved script passed.
