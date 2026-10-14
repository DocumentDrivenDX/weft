---
ddx:
  id: weft.exact-instant-v0.2.6
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: CONTRACT-001
      kind: informed_by
---

# Exact timestamp equality transport

Compile/IR/backend0.2.6 retains SQL0.2.0 and adds explicitly selected instant
equality. The request MUST include `timestampComparisonProfile` equal to
`umf-rfc3339-exact-instant/1.0.0`. The normative recipe is committed UMF
[CONTRACT-065](https://github.com/DocumentDrivenDX/umf/blob/cbb6fb87/docs/helix/02-design/contracts/CONTRACT-065-rfc3339-exact-instant.md).
Generic core timestamp, civil time and temporal Key meanings remain unchanged.

`exactParameters` and `exactEqual` operands may use `scalarType: timestamp` in
this version. Both operands MUST have the same exact family. Required or
absent-allowed singular fields are supported; absent/null comparisons yield SQL
UNKNOWN/nonmatch. The compiler checks literal/parameter validity before emission,
including on empty sources. Original tokens MUST remain unchanged in typed
parameter slots, origin records and projections. The backend MUST normalize both
stored operands and query operands using the same recipe, never native timestamp
casts. Exact binary equality remains supported under its existing semantics.

The capability names are `compare.timestamp.instantEqual` and
`compare.timestamp.optionalInstantEqual`. Unsupported backend selections refuse
before execution. Backend0.4.29 selects Truss native59; backend0.4.28 retains
native58 and its binary-only0.2.5 interface. Old schemas remain unchanged.
Ashlar admission remains unavailable until its adapter retains exact original
timestamp text and adopts this recipe. No ordering, grouping or temporal-key
capability is added.

The four matching JSON schemas define the closed request, plan, response and
backend shapes. Invalid timestamp tokens refuse with `WFT-TIMESTAMP-DOMAIN`;
tokens beyond1MiB refuse with `WFT-LIMIT`. A missing or unknown profile violates
the request envelope. These failures do not initiate retries.

Offset-equivalent tokens with fractions `.123456789` and `.1234567890` compare
equal. Tokens differing at the seventh fractional digit compare unequal.
`-00:00` identifies a known UTC instant with separately retained unknown local
offset knowledge. Pre-epoch fractions use floor whole seconds, not a truncated
signed decimal or finite host datetime.
