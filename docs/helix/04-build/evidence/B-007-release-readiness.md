# B-007 release readiness — 2026-10-08

State: preparation only; no distribution is authorized or qualified by this record.
Governing exit criteria are the implementation plan, TP-001, SD-004 and TD-006.

The source owner is DocumentDrivenDX, as established by the requested repository.
The current Rust workspace version is 0.1.0 with `publish = false`; the Python
project is `weft-sql` 0.1.0, built by Maturin 1.9.6 with Python >=3.9; the browser
package is `@documentdrivendx/weft` 0.1.0 and private. These are local manifest
names. Registry availability, namespace ownership and publishing credentials
have not been verified. No individual release maintainer is inferred.

Before any distribution:

1. Record the owner's license decision and add matching license metadata/files.
   That decision is pending. A suggested choice is not approval.
2. Verify registry ownership for each published package and designate the actual
   release maintainer. Do not upload until those facts are established.
3. Audit all 30 criteria against their required evidence layers. Require the
   unchanged 636 fixtures, expanded assertions, deterministic properties,
   semantic mutations, parser/resource checks and supported/refused branch review.
4. Pin source commit, dependency lockfiles, build tools, model/binding bytes,
   compiler/dialect/IR/backend versions and native engine/settings profiles.
   Build artifacts from that commit and retain SHA-256 manifests. Run matching
   native, Python and real-browser checks on the artifacts to be distributed.
5. Publish an exact support inventory. Candidate capabilities remain candidate;
   unavailable/skipped/unknown engines do not count as supported. Evidence must
   prove each advertised operation/type/domain on its named profile.
6. Inspect package contents, licenses, platform tags, imports and browser exports.
   Check Python's native ABI without a JavaScript sidecar, browser WASM without
   runtime Node APIs, and host-owned connection/authorization boundaries.
7. Review the release candidate and its evidence manifest before changing private
   or publish flags. Only then perform an explicitly authorized publication.

The observed macOS ARM64 Python and Chromium artifacts establish their recorded
subset only. Python >=3.9 metadata does not prove every interpreter, operating
system or architecture. The TypeScript source export does not itself establish
an installable built browser distribution. Production storage/runtime adoption,
package registry reservations and a fleet-wide build matrix remain unverified.
No performance SLA is added.

Rollback means disabling the affected backend/profile or declining a release.
Do not alter original UMF modules, migrate a database, or reinterpret stored
values to make a failed support claim pass. An already published version would
require the relevant registry's documented withdrawal/deprecation process and
an explicit owner instruction; no registry action is taken here.
