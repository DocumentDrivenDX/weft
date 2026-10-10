---
ddx:
  id: US-009
  type: user-stories
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-006
      kind: informed_by
---

# US-009: security-compilation

**Feature:** FEAT-006. **PRD Requirements:** FR-19, FR-20, FR-21, FR-22. **Priority:** P0.

## Story

As an application developer, I want to compile a protected logical read so that native hosts can enforce its meaning without hidden loss.

## Context and Walkthrough

Supply pinned policy/ontology/model inputs, inspect admitted meaning and lowering, then hand an artifact to an authenticated native host. Present public activation refuses; foundation simulations are scoped component evidence.

## Acceptance Criteria

- **US-009-AC1:** Given pinned supported security sources, when the separate versioned transport is admitted, then exact custody and previous ordinary transport behavior survive without relabeling or fallback.
- **US-009-AC2:** Given selected policy and ontology meaning, when admission and composition run, then complete qualified identity and private-fact semantics are preserved or refused before emission.
- **US-009-AC3:** Given a protected read, when logical and physical lowering run, then disclosure, replacement and protected predicate/order/group/join/aggregate semantics preserve null, absent and withheld distinctions.
- **US-009-AC4:** Given a security artifact, when a native host executes it, then authenticated custody, current authority and guarded final release satisfy mandatory versioned obligations without inferred credentials or qualification.

## Edge Cases and Test Scenarios

- `security_transport`: Given pinned supported security sources, when the separate versioned transport is admitted, then exact custody and previous ordinary transport behavior survive without relabeling or fallback.
- `security_admission`: Given selected policy and ontology meaning, when admission and composition run, then complete qualified identity and private-fact semantics are preserved or refused before emission.
- `security_lowering`: Given a protected read, when logical and physical lowering run, then disclosure, replacement and protected predicate/order/group/join/aggregate semantics preserve null, absent and withheld distinctions.
- `security_host_release`: Given a security artifact, when a native host executes it, then authenticated custody, current authority and guarded final release satisfy mandatory versioned obligations without inferred credentials or qualification.

## Dependencies

CONTRACT-001–005, architecture, corresponding technical design and story test plan.

## Out of Scope

B-009 allocates these requirements but leaves all four full acceptance outcomes open. Existing admission/simulation passes do not prove physical lowering or native enforcement.
