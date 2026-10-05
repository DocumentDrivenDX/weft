---
ddx:
  id: US-006
  type: user-stories
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-004
      kind: informed_by
---

# US-006: Qualify versions with a large independent conformance corpus

**Feature:** FEAT-004. **Feature Requirements:** CONF-01–03.
**PRD Requirements:** FR-11, FR-12. **Priority:** P0. **Status:** Draft.

## Story

As an compiler maintainer, I want to qualify versions with a large independent conformance corpus so that I can evolve the language safely without losing unknown meaning.

## Context and Walkthrough

I supply Weft SQL and pinned UMF modules, select a registered backend, and
inspect the complete artifact or refusal. Backend configuration comes from
owned physical binding metadata. I only execute through a host that can fulfill
reported obligations; compilation does not certify database state or policy.

## Acceptance Criteria

- **US-006-AC1:** Given the release corpus, when a backend/host claims support, then every applicable expected result/refusal has reproducible version-qualified evidence.
- **US-006-AC2:** Given altered model/binding bytes or incompatible interfaces, when compilation runs, then pin/version guards refuse stale or unsupported inputs.
- **US-006-AC3:** Given unknown unselected content, when models prepare/compile, then full source is retained; selected unknown semantics block.
- **US-006-AC4:** Given resource-limit boundaries or malicious inputs, when each host compiles, then termination and refusal agree without partial SQL.

## Edge Cases and Test Scenarios

- `support_evidence_audit`: Missing/failed/skipped evidence cannot count as supported.
- `pin_and_version_guards`: SHA-256 bytes and identity guards; no implicit migrations.
- `unknown_content_retention`: Original bytes/content recoverable; no opaque-to-known inference.
- `resource_and_fuzz_guards`: Duplicate keys, bounds, nesting, malformed text and plugin failures.

## Dependencies

CONTRACT-001–003; FEAT-004; corresponding TD-006 / STP-006.
Backend integration requires the owning layout/profile contracts, not inferred
spike table names. Planned tests do not constitute delivered coverage.

## Out of Scope

Database execution inside the compiler, undisclosed fallback/loss and unsupported
broader SQL. User-visible surfaces are governed by Contracts, not invented here.
