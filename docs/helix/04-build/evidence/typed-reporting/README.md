# Typed SDK reporting — 2026-10-15

The first raw PostgreSQL 18 reporting profile is implemented behind feature `raw-postgresql-reads` and `compile_raw_read_json`. UMF's located interface references are WR-B7 (schema-neutral search-path binding) and WR-A14 (relationship join lowering), from checkout `20bc/umf`'s `docs/helix/04-build/weft-requirements-typed-access.md`. These are interface requirement IDs, not invented tracker issue IDs.

Core owns the pinned core 0.8 source and SQL admission artifact (`crates/weft-core/src/raw_read.rs`). The PostgreSQL backend borrows that artifact for native lowering (`crates/weft-postgresql/src/raw_read.rs`). Python and WASM share this Rust path. Unsupported targets/shapes produce blocked responses without executable SQL.

The profile admits explicit scalar projections, up to three declared to-one inner/left equijoins, grouped count/sum/avg/min/max, HAVING comparisons and nullable keyset continuations. It emits bound parameter slots, output carriers, schema/role/read-only/timeout obligations and access-share locks. Native execution and connection ownership remain in the SDK host adapter; the compiler obtains no connection or execution authority.

Five Rust tests pass. The Python and WASM targets build. UMF's installed SDK/native-Weft wheel tests on Python 3.11.17 and PostgreSQL 18.6 prove caller-role RLS across joins and aggregates, generated joins, nullable paging and typed timeout recovery. Full source hashes and logs are retained in the calling UMF checkout under `docs/helix/04-build/evidence/typed-reporting/`.

Lakebase 18 compilation passes; native service execution remains unverified without an endpoint. This experimental scalar/to-one profile does not qualify broader targets, native aggregate DSL, arbitrary SQL functions, snapshots, async/streaming or a release. Reviewed SQL AVG retains PostgreSQL numeric semantics; the separate proposed explicit-scale half-even DSL AVG contract is not implemented here.
