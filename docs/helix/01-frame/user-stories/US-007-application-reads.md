---
ddx:
  id: US-007
  type: user-stories
  activity: frame
  status: draft
  authoring:
    home: repo
  links:
    - id: FEAT-005
      kind: informed_by
---

# US-007: Serve bounded application reads

As an application host, I need complete entities, ordered pages, counts and related keys through Weft, so I can replace a limited recognizer with the compiler without changing accepted meaning.

## Acceptance Criteria

- **US-007-AC1:** Complete declared-member order and exact absent/null/value/list/structured descriptors.
- **US-007-AC2:** Stable ascending complete-key order, exact resume comparison and enforced bound; no silent cut.
- **US-007-AC3:** COUNT(*) preserves duplicates; empty global count 0, grouped empty output empty.
- **US-007-AC4:** Existential key filtering, bounded related keys with exact truncation and inverse resolution.
- **US-007-AC5:** Marker values match resolved UMF types and remain bound data; literal quoting/exactness is normative.
- **US-007-AC6:** Named subsets have independent positive/refusal fixtures and identify excluded constructs without changing valid syntax.

## Requirements and Boundaries

FR-13 through FR-18. Weft SQL 0.2 introduces the application-read profile; 0.1 preserves existing acceptance/refusals. A fixture-only mapping cannot close native adapter acceptance. Unknown selected member or relationship meaning blocks.
