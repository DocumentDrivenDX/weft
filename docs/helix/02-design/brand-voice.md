---
ddx:
  id: weft.brand-voice
  type: brand-voice
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.vision
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
---

# Weft brand voice

Weft speaks like an engineer who cares about the meaning of a query. Its promise is one logical SQL dialect over UMF, with inspectable compilation and explicit limits. Its audience is developers embedding queries and engineers implementing storage backends.

## Position and message

Primary line: **One query. Its meaning intact.** Supporting line: **A universal UMF-powered SQL dialect with pluggable storage backends.** Explain the input and output immediately: Weft SQL + UMF modules + storage binding → target SQL and an execution contract.

## Voice rules

Use direct sentences, concrete nouns and named responsibilities. Describe what developers supply, what the compiler returns, and what the host must enforce. Prefer “Inspect the SQL” to “Unlock seamless interoperability.” Explain a technical term when it first becomes necessary. Use weaving as a restrained visual metaphor, never as a replacement for an interface explanation.

The tone is calm, exact and curious. Headlines can be short; evidence needs its scope. “Universal” describes the extensible dialect, not support for every SQL feature or database. Say “initial backends” rather than implying arbitrary target compatibility. Never claim production readiness, released packages or completed security enforcement from compiler fixture evidence.

## Examples

- Hero: “One query. Its meaning intact.”
- Mechanism: “Resolve logical records and fields against your UMF modules. Let a registered backend map them to storage.”
- Refusal: “If a backend cannot preserve selected meaning, compilation stops with a diagnostic.”
- Boundary: “Weft compiles. Your host authenticates callers, executes SQL and enforces the returned obligations.”
- Action labels: “Read the contracts”, “Explore the source”, “Inspect support”.

## Content provenance

Site content is model-primary, commissioned by the human operator. Innsigle signatures identify published bytes and issuer; they do not certify technical correctness, human authorship or access-control safety. Omit invented human-input percentages. The website links to exact support evidence and labels in-progress work.
