# Working in Weft

Weft compiles UMF-described queries to backend SQL (Truss on PostgreSQL, Ashlar on
Databricks). Rust core in `crates/`, Python bindings in `crates/weft-python`, WASM in
`crates/weft-wasm`, browser wrapper in `packages/`, specs and contracts in `docs/helix/`.

- Test: `cargo test --workspace --locked`, `bun install --frozen-lockfile && bun run specs:check && bun run modules:check`.
- Wheel: `maturin develop --manifest-path crates/weft-python/Cargo.toml`, then `python examples/quickstart.py`.
- Run output belongs in CI artifacts or a temp directory, never in git.
- Ownership: UMF owns metamodel meaning; backends own physical mapping and target SQL;
  hosts own connections, credentials, execution, authorization and publication.
  Preserve unknown model content and refuse what cannot be preserved.
- Python bindings share the Rust core. No JavaScript sidecar for the Python API.
- Release: bump the version in `pyproject.toml`, `crates/weft-python/pyproject.toml` and
  `crates/weft-python/Cargo.toml`, add a `CHANGELOG.md` section, then push tag `vX.Y.Z`
  from main. `scripts/check-release-version.py` enforces agreement.
- Only claim what CI tests; label the rest experimental.
