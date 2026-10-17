---
ddx:
  id: weft.typed-sdk-compiler-release043-plan
  type: implementation-plan
  activity: build
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.implementation
      kind: informed_by
    - id: weft.typed-sdk-engine-order-pages-plan
      kind: informed_by
---

# Proposed stable SDK compiler release0.4.43

This is a local review plan. No version/default change, public push, tag or
publication is authorized by this document. The prior public-push approval
review rejected Weft publication scope; the coordinator is obtaining explicit
human authorization. Never retry through another transport while that hold
stands. Public0.4.41 and frozen0.4.42 candidate source/assets stay untouched.

## Freeze and default features

After remaining source/native prerequisites settle, freeze a clean combined
source commit and obtain the existing original Truss/Astra release review through
the coordinator. Choose canonical stable`0.4.43` in Python project, Python Cargo
crate and Cargo.lock together. No dev/prerelease spelling in stable release.

The six existing Python defaults remain an explicit baseline:
raw-postgresql-reads, ashlar-databricks-qualified, truss-postgresql-pg18,
current-postgresql-scalar-candidate, current-databricks-scalar-candidate and
current-query-refinement-candidate. Any addition must be individually reviewed.
The separately reviewed0.4.42 Graph packaging candidate is not silently folded
into0.4.43; confirm its source/feature disposition with the original owner.
Private eight-feature proof builds do not grant stable eligibility to
managed-delta-mutation-candidate, presence/Graph candidates or other experimental
registrations. Unqualified native targets remain closed. Temporal stays HELD.

Publish only the exact reviewed feature closure, with negative default-build and
public API tests proving disabled registrations and experimental constructors
remain absent. New source APIs may be callable yet return their exact closed
profile refusal; wheel availability never grants native capability.

## Existing full release gates remain mandatory

Retain every current workflow gate: version agreement, packaging and license
contents, current-query designation, mutation/native066, Truss068, native070,
Date, Decimal12, query ABI, current Databricks, query refinement and SDK owner
smokes plus every closed report verifier/self-test. Preserve full required source
conformance/graph checks. A new SDK corpus extends, rather than replaces, the
existing fixed inventory. Report validators reject missing/extra/duplicate cases,
wrong feature/version/source identity and mismatched wheel/native payload hashes.

Run exact installed-wheel probes on all five existing matrix targets:
Linux x86_64/aarch64, macOS arm64/x86_64 and Windows x86_64. Preserve ABI3 Python3.9
compatibility and Maturin1.9.6/Rust1.90.0 pins. Add source-bound calls and deliberate
negative controls for newly enrolled original Related plain/page/export,
Truss/Ashlar related-key metadata, raw full projections, list/term/aggregate/HLL,
source/nativeColumn custody, selected cursor profile and direct Field-only profile
as applicable to the final frozen source. Every API output retains complete
original requests, obligations and structural digest owner.

The public stable wheel must replay the SDK generated schema-load, instance-create,
real sync/async native read/page/stream, action/backend and original Truss
correctness/performance cohorts at their pinned versions. Native DB identity,
authorization, snapshot/fencing/cleanup and semantic negative controls remain
separate from five-platform compiler smoke results. Required native prerequisite
failures block the affected release claim; source tests cannot replace them.

## Exact tag and public asset verification

Once public authorization and completed source/artifact review are recorded,
normal reviewed source merge uses exact current head and required protections,
without admin bypass or force. Use the established exact-tag workflow. The
verified promotion path currently requires an eligible tagged push run; do not
promise pre-tag artifact promotion that its verifier rejects. Tag the exact
reviewed source, run all five wheel gates and existing publication logic, then
inspect the actual public assets before downstream default enrollment.

Record tag target/source/workflow commit, five exact wheel filenames and SHA256,
all native/shim payload hashes, closed report inventories and public release
asset hashes. Download and freshly install the actual public ARM wheel, rerun its
closed corpus and compatible SDK cohort; cross-platform public hash correspondence
must cover all five assets. Verify release-pins and source equality, not merely
latest tag/version. Attach created PRs through the app. CI status/jobs/log reads
use CodeReview; metadata-only tag-run discovery may use the approved GitHub
connector when CodeReview cannot resolve the tag. No gh CI fallback.

## Rollback and evidence

Never overwrite old wheels/tags or erase failed attempts. Preserve source commits,
recovery bundles, exact native/browser receipts and public digest proofs. Retain
explicit profile/native qualification boundaries in release notes and SDK
manifest. A failed candidate stays opt-in or refused; it does not weaken gates.
Current coherent3f64056 native42/API and Chromium90 proof is a preserved precursor,
not evidence for later source changes or permission to publish.
