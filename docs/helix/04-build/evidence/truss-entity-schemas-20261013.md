# Versioned Truss entity schema foundation — 2026-10-13

Separate0.2.2 request, logical-plan, response and backend-manifest schemas preserve every older schema byte. Explicit weft-truss-entity-read/0.1.0 complete-entity profile admission gates native metadata and complete relationship-key output expressions. Native metadata retains actual scan/Record identity and fixed exact type/nullability. Response carriers are text with exact-integer version decoding and text decoding for the other metadata. CompleteRelatedKeys uses a separate tag and its bound is a whole-query refusal ceiling, never truncation.

Four Draft2020-12 schema checks pass. New metadata expressions validate under2.2, are rejected by the old2.1 expression schema, and wrong version nullability is refused. All five metadata response branches reject incompatible boolean carriers and decimal decoders. Astra Ultra reviewed the actual schema source and the carrier/bound corrections were applied.

These schemas are unactivated: no compiler dispatch, resolver, backend manifest or Python API selects them yet. Collision rules, runtime metadata/relationship lowering, result validation/decoding and the unchanged public whole-entity consumer workflow remain pending. No new runtime support or whole-entity passing claim is made.
