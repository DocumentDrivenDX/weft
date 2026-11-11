# Changelog

Older releases: see [GitHub Releases](https://github.com/DocumentDrivenDX/weft/releases).

## 0.4.58

- Generated read algebra v6 to v8 and ordinary pages (PR #59).
- The query-codec-instructions schema reference uses the target's URN `$id`, so
  `bun run specs:check` passes again.
- Release process simplified: one `release.yml` (tag-driven; tests, builds Linux
  and macOS arm64 wheels, an sdist and WebAssembly tarballs, smoke-installs each
  wheel, then publishes to PyPI with trusted publishing and creates the GitHub
  Release with SHA256SUMS and a build provenance attestation).
- CI tests every pull request: Rust workspace, installed wheel, schema checks.
- Repository cleanup: committed run output, receipts and candidate binaries were
  removed from the tree (they remain in git history).
- UMF consumed by release tag instead of a commit pin.

## 0.4.57

- Published to PyPI as `umf-weft` with wheels for Linux x86_64, Linux aarch64 and
  macOS arm64. See the GitHub Release for details.

## 0.4.56

- See the GitHub Release.

## 0.4.55

- The Python distribution was renamed from `weft-sql` to `umf-weft`. The import
  stays `weft`. Compiler behavior is unchanged from 0.4.54.

## 0.4.54

- Unity current aggregate compiler work integrated; typed-expression compiler
  (G02) retained.

## 0.4.53

- Closed compiler profile for typed same-row backfill expressions in UMF table
  operation 0.2 (G02).
