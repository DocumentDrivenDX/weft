---
ddx:
  id: weft.current-0457-controller-pairs
  type: implementation-plan
  activity: build
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.release-0.4.57
      kind: informs
    - id: CONTRACT-003
      kind: informed_by
    - id: CONTRACT-008
      kind: informed_by
    - id: TP-001
      kind: informed_by
---

# Current 0.4.57 controller-pair qualification addendum

## Scope

This release-specific profile keeps product source R at
`c623b94b66b0b46d492d457c62faddc896e2e452`, tree
`cc552d691c92c7abdd44fdf4e4ca585936ae68c5`. Compiler, manifests, dependencies,
features, resources, build scripts, packaging inputs and runtime bytes remain R.
The release tag, source distribution, repository source archive and all three
wheel payloads name R. No source archive or receipt is relabelled.

Two separately registered controllers may qualify the missing platforms:
Mac controller Q and Linux ARM64 controller A. Their actual GitHub heads, trees,
workflow bytes, original jobs/attempts and upload windows remain distinct from
product R. `weft-current-three-wheel-sdk-controller-pairs/1` consumes the
original successful R x86_64 job, an explicit
`weft-current-macos-controller-pair/1` component and an explicit
`weft-current-arm64-controller-pair/1` component. Neither platform registration
admits the other. The frozen `weft-current-three-wheel-sdk/1` validator is
unchanged; this profile never constructs false `/1` identities to call it.

Governing artifacts are [release qualification](../../05-deploy/releases/v0.4.57.md),
[compile/host boundary](../../02-design/contracts/CONTRACT-003-compile-and-host-boundary.md),
[reliability runner](../../02-design/contracts/CONTRACT-008-reliability-runner.md)
and [compiler conformance](../../03-test/TP-001-compiler-conformance.md).
The hosted acquisition budgets below are separate from `weft-runner/1`'s
operator configuration and do not change its bounds or compiler semantics.

## Implementation slices

1. Validate both complete actual Git tree inventories before comparison. Preserve
   object type/mode; reject malformed equal entries, symlinks/submodules and every
   undeclared R-to-controller difference. Only the exact enumerated control and
   governing-document paths may differ. All production files remain byte/mode
   identical. Bind nine refusal requests independently to the immutable R corpus
   before checking response/receipt hashes.
2. Freeze the reviewed Mac controller Q, then the separate ARM controller A.
   Record actual commits/trees, workflow SHA256s, executed controller hashes and
   complete changed-path inventories. No controller contains a claim about its
   own not-yet-known commit SHA.
3. After review, commit exact pair data in registration metadata P. P differs
   from the combined reviewed controller tree only at the registration JSON.
   The tracked registration is initially `unadmitted`; it refuses execution.
   Each platform entry pins its own exact controller/ref/workflow/tree/bytes.
   Current-main registration API/contents/blob verification is mandatory before
   a dispatch can acquire tools. Caller envelopes alone confer no provenance.
4. Admit each reviewed platform run explicitly. Build from a separate initially
   clean R checkout; execute controller files from Q/A outside that checkout.
   Hash every tracked R byte/mode before and after, and reject unexpected source
   additions. Keep actual GitHub workflow identities unchanged.
5. Authenticate original successful artifacts, then stage exact qualified bytes
   through the explicit current publisher branch. Existing immutable release
   API readbacks, archive/metadata/checksum validation and normal PyPI OIDC
   publisher remain required. No direct upload bypass or generic differing-
   controller fallback is added.

## Execution contract

Preparation and pure controls are authorized. No new producer dispatch, tag or
publication is admitted by this document alone. Sol must review complete source,
Q/A trees/workflow hashes and the exact registration before platform execution;
actual qualification and publication evidence require later review.
No heavy local job is allowed; the shared local ledger/caps remain untouched.
Source-only controls do not claim native/browser execution. No new dependency,
cache restore, source URL/path override or runtime change is in scope.
Token usage ceiling is not specified; the hosted limits are explicit below.

| Platform | Job/global limit | Cold acquisition/build | Termination grace | Evidence reserve |
|---|---|---|---|---|
| Mac ARM64 only | 90 minutes / 5,400 monotonic seconds | 1,800 seconds for the original locked wheel build | original 30 seconds | 60 seconds |
| Linux ARM64 only | 90 minutes / 5,400 monotonic seconds | 4,200 seconds beginning before Maturin action acquisition, image acquisition and setup | 30 seconds | 60 seconds |

Other Mac tool, install, packaging, comparator, WASM and parity stage limits and
acceptance checks remain unchanged. Installed SDK steps have explicit bounded
commands within the original remaining global time. Partial failure evidence is
retained within the original per-file/70MiB aggregate caps; upload is not a pass.
These are cold builds, not cache resumes or completion guarantees.

ARM uses the genuine Maturin action commit
`e83996d129638aa358a18fbd1dfb82f0b0fb5d3b`, observed in the original R run,
with Rust1.90.0, Maturin1.9.6, manylinux2014/aarch64, unchanged locked compiler
arguments and one Cargo worker. Its controller names the same workspace target
path explicitly, avoiding the pinned action's otherwise implicit un-locked host
metadata lookup solely to discover that path. This is not a compiler cache or a
dependency override. The outer bounded process starts before acquiring/executing
the pinned action, so both previously opaque acquisition intervals are covered.

## Continuation evidence

Original R source/WASM run41679478777 passed. Original Linux x86_64 job126094501575
in run41686658658/attempt1 passed14 temporal,5 ordinary/padded-route and42 SDK
controls with genuine wheel/module/shim correspondence. Its ARM sibling's
30-minute job timeout does not change that successful job's original attempt.
Original Mac run41686410364/attempt1 timed out at its900-second wheel stage;
its preflight-only artifact is not wheel or SDK evidence. No failed job is
blindly retried and no successful original job is replayed.
Actual source-only independent controls initially found three false acceptances
and retained their RED output. Complete typed-tree validation and independent
R request custody must close those controls without changing their assertions.
New hosted results and final registration identities remain unexecuted/unadmitted
until their actual reviewed records exist.

## Exit criteria and validation plan

Required controls reject wrong Q/A/tree/ref/workflow/run head, undeclared
production changes, malformed equal Git entries, altered requests with relabelled
self-hashes, preflight-only artifacts, missing/reordered SDK42, missing any of
nine Mac refusal-parity results, wrong byte/module/shim/packaging bindings,
wrong original attempts/upload windows/digests, and old receipts relabelled under
new identities. Mocked controls are not qualification evidence.
Each new platform requires real installed packaging, normal resolver/pip-check,
14 temporal compiler cases,5 ordinary/padded-route controls and closed ordered42
SDK controls. Mac additionally requires all nine original Rust/installed-Python/
Chromium refusal responses and request/response byte custody. Shared temporal
Node/Chromium14 plus carrier refusals remain bound to the original successful R
source run. None grants database snapshot/isolation/security authority.
Final publication must authenticate exactly three wheels and the original R
archives, preserve original successful attempts, and reuse exact qualified bytes.
Actual wheel METADATA and public resolver selections govern dependency claims;
unpublished Codegen/Access versions cannot be replaced by older cohorts to pass.
Truss119/HF62/security/consumer admission and public-registry gates remain
independent. No benchmark or extra platform/Python matrix is scheduled.

## Risks and rollback

Hold on any source, controller, pair-data, budget, artifact or qualification
mismatch. A successful SDK upload before parity is only a partial. Preserve all
failed artifacts and successful original proofs; do not raise bounds or change
pair identities without review. Disabling the explicit pair selection leaves
historical source/controller equality and validators unchanged. Retain the prior
admitted public release until the new release and consumer gates are genuinely
qualified.
