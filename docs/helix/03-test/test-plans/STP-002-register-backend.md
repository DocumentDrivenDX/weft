---
ddx:
  id: STP-002
  type: story-test-plan
  activity: test
  status: draft
  authoring:
    home: repo
  links:
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# STP-002: register-backend

## Story Reference

[[US-002-register-backend]], [[TD-002-register-backend]], [[SD-002]], TP-001.

## Scope and Objective

Prove this P0 journey under its selected dialect/model/backend/host versions.
Every covering test must cite its AC as `@covers US-002-ACm`. All tests below are
planned; no compiler implementation or executed coverage exists at bootstrap.

## Acceptance Criteria Test Mapping

| AC | Planned test | Observable assertion | Layer | State |
| --- | --- | --- | --- | --- |
| US-002-AC1 | `third_backend_registration` | No frontend backend-ID switch; explicit registry selection. | Rust + host/contract | planned |
| US-002-AC2 | `capability_negotiation` | No fallback; missing manifest/interface versions identify refusal. | Rust + host/contract | planned |
| US-002-AC3 | `candidate_qualification` | Candidate cannot masquerade as verified conformance. | Rust + host/contract | planned |
| US-002-AC4 | `plugin_trust_boundary` | Injection/version/failure guards; trusted registration only. | Rust + host/contract | planned |

## Executable Proof and Data

Planned implementation tests: `tests/register-backend/`; shared language-neutral fixtures
in `docs/helix/03-test/fixtures/`. The existing `bun run specs:check` validates
spec/fixture integrity only; it does not execute this story or certify coverage.
Add runnable Rust/host/native commands with pinned versions before implementation.

## Edge Cases and Build Handoff

100% of applicable positive/refusal/boundary cases must pass. Do not count a
skipped platform test as a pass; missing native/host access blocks qualification.
Expected results are authored independently of emitted SQL/compiler code.
Require tests-before-code, retained logs/input hashes, and zero phantom claims.
