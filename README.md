# Weft

Weft compiles a request (a UMF document, a storage binding and a query) into SQL
for a chosen backend, with bound parameters, result types, decoding instructions
and diagnostics. If a selected meaning cannot be preserved, compilation returns
an explicit refusal instead of guessing. Weft never connects to a database or
executes SQL: execution, authorization and result publication belong to the host.

Weft is a Rust compiler with Python bindings (`umf-weft`, import `weft`) and a
WebAssembly build. It consumes [UMF](https://github.com/DocumentDrivenDX/umf) and
lowers to two backends: Truss on PostgreSQL and Ashlar on Databricks.

**Status: experimental.** The public surface is the JSON request and response
contracts in `docs/helix/02-design/contracts/`.

## What CI tests

- The Rust workspace test suite (`cargo test --workspace`) on Linux x86_64.
- The Python wheel, built from this source and installed into a fresh virtualenv:
  the quickstart below plus the installed-wheel checks listed in
  `.github/workflows/ci.yml`.
- Nightly: backend feature matrices, WebAssembly parity in Chromium, and native
  PostgreSQL 18.6 table-operation tests (see `.github/workflows/ci.yml`).
- Releases build abi3 wheels (Python 3.9 or later) for manylinux x86_64,
  manylinux aarch64 and macOS arm64, an sdist, and two WebAssembly tarballs. Each
  wheel is installed into a fresh virtualenv and runs the quickstart before
  anything is published.

The Truss/PostgreSQL and Ashlar/Databricks backends are compile-only in this
repository. Nothing here is tested against a running Databricks warehouse.

## Install

```sh
pip install umf-weft
```

## Quickstart

```sh
git clone https://github.com/DocumentDrivenDX/weft && cd weft
python examples/quickstart.py
```

`examples/quickstart.py` loads `examples/plain-field.request.json` and compiles it:

```python
import json, pathlib, weft

request = json.loads(pathlib.Path(__file__).with_name("plain-field.request.json").read_text())
result = json.loads(weft.compile_truss_runtime_json(json.dumps(request)))
assert result["status"] == "compiled", result
print(result["sql"])
```

The request is chosen by the host: a UMF document, a Truss runtime binding and the
query to compile. The output is one PostgreSQL statement, which begins:

```
SELECT (("s0".props->'1'->>'text') COLLATE pg_catalog."C")::pg_catalog.text AS "A-label" FROM "truss"."query_objects"($1::pg_catalog.jsonb,1::pg_catalog.int4) AS "s0" LIMIT 101
```

CI runs this script against a freshly built wheel on every pull request.

## Build from source

```sh
rustup show                                   # installs the pinned toolchain
cargo test --workspace --locked
pip install maturin==1.9.6
maturin develop --manifest-path crates/weft-python/Cargo.toml
bun install --frozen-lockfile && bun run specs:check && bun run modules:check
```

## More

- [Architecture](docs/helix/02-design/architecture.md) and [ADRs](docs/helix/02-design/adr/)
- [Request and response contracts](docs/helix/02-design/contracts/)
- [Changelog](CHANGELOG.md) and [roadmap](ROADMAP.md)

## License

Dual-licensed under MIT OR Apache-2.0, at your option. See [LICENSE.md](LICENSE.md).
Third-party materials retain their original notices.
