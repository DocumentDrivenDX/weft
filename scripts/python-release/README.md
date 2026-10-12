# Python wheel release

Package `weft-sql==0.4.2`, native import `weft`. Source baseline is
`cdd8fa698683cd87bb126916a75200e80e19d831`; the release patch is separate.
Python binding version is independent from compiler/backend/dialect versions.

The owner selected MIT OR Apache-2.0, newer candidate features, and a regular 0.4.2 release.
Maturin pins four features: both qualified registrations plus explicit PathsKeys
and implicit-arithmetic candidate transports. Synthetic test registrations and
public security activation are excluded. Candidate APIs preserve candidate opt-in.

## Build and validate

Use Rust 1.90.0 and Maturin 1.9.6. Build with:

```sh
maturin build --release --locked --manifest-path crates/weft-python/Cargo.toml --out dist --interpreter python
python -m venv .wheel-venv
.wheel-venv/bin/python -m pip install --no-index dist/*.whl
.wheel-venv/bin/python scripts/python-release/validate.py --wheel dist/EXACT_WHEEL.whl --out reports/installed-wheel.json
```

On Windows use `.wheel-venv/Scripts/python`. The workflow builds five platform
wheels and installs each on six conventional GIL CPython versions, 3.9–3.14.
Windows x64 and macOS Intel stay prospective until hosted validation passes.
Linux uses manylinux2014 (glibc >=2.17); musl, PyPy, free-threaded Python and
other architectures are excluded. Local x86_64 Docker execution uses emulation;
it does not establish performance or broad hardware compatibility.

Validation checks complete RECORD hashes/sizes, installed native payload identity,
ABI3 tags, package/version/license metadata, both licenses and dependency notices,
absence of JavaScript/WASM, ordinary 2,181 response objects, 174 candidate/control
response objects, deterministic repeats, invalid types/Unicode and input refusals.
Subprocess calls are disabled during compilation. The retained ordinary baseline
has 53 changes from historical native outputs that still lack native requalification.
The package is experimental; wheel compatibility does not grant new database
semantics or production execution claims.

`licenses.py` produces notices from locked Cargo metadata; missing texts fail.
SIMD license text comes from upstream commit
`d74c030d9dc4f3cae02146d1f497ff62726ef09a`; r-efi licensing is in its AUTHORS file.

## Trusted Publishing setup

The user intends to own the PyPI project and maintain releases. PyPI returned 404
for `weft-sql` on 2026-10-11; this does not prove name reservation or account rights.

1. Owner signs in to PyPI with their account and creates a pending Trusted Publisher
   for project `weft-sql`, GitHub owner `DocumentDrivenDX`, repository `weft`,
   workflow `python-wheels.yml`, environment `pypi`.
2. Owner configures the GitHub `pypi` environment with a required release reviewer
   and deployment restricted to the approved release ref. Without environment
   protection, the workflow's `publish` Boolean is its only manual gate.
3. Integrate and review this patch at a clean commit, then dispatch
   `python-wheels.yml` with `publish=false`. Require all build/validation jobs.
4. Inspect the exact source, wheel hashes, platform evidence and candidate limitations.
   Explicitly authorize upload; rerun that approved ref with `publish=true` and
   approve its protected `pypi` environment. The publish job downloads only wheels
   built and validated in that same run and uses OIDC, with no stored PyPI token.

GitHub release is independently selected with `github_release=true`, using a
protected `github-release` environment. It creates a regular `v0.4.2` release
at the validated workflow source ref and attaches exact same-run wheels.
The owner has authorized cutting that GitHub release in this session; PyPI
upload still requires publisher setup and separate final authorization.

Only wheels are published by this workflow. An sdist is deliberately absent until
its extracted source closure has been built and validated. A failed platform job
holds the entire publication. To release a narrower platform matrix, edit and
review the matrix before dispatching; do not skip failed jobs or claim support.

Publication has not occurred at preparation time. Version selection and license are settled. Remaining
human input is the owner's PyPI account identity and authorization to integrate
this isolated patch; a final upload authorization follows the reviewed CI result.
