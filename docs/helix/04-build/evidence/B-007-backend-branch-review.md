# B-007 named backend branch review

State: partial critical-path review, with fresh executable component receipts in
B-007-backend-branches/. All 18 Ashlar and 22 Truss candidate tests pass. This is
an explicit branch/test review, not a line-coverage percentage or a claim that
every possible binding/native domain has been exercised.

| Critical path | Positive evidence | Refusal evidence |
| --- | --- | --- |
| Ashlar layout/publication/home admission | Explicit props/column homes, snapshot version zero, original local cross-module fields | Unknown layout/hash, stale pins, malformed UUID/version/name, duplicate mapping/table, unknown selected home/plugin meaning |
| Ashlar native scalar domain selection | Required signed widths, admitted unsigned BIGINT subdomains, exact Decimal/string/Boolean slots | DOUBLE or mismatched selected native carrier; UInt64 BIGINT; unknown type/facet/encoding |
| Ashlar application lowering | Entity/COUNT/keyset, optional scalar in both homes, forward/inverse/EXISTS relationships, recursive compound descriptors | Changed/missing relationship definitions, key/cursor mismatch, unknown selected compound dependency/encoding |
| Truss original-byte/home admission | Both props and fixed row homes retain admitted original definitions | Changed bundle hashes/Base64/profile, out-of-range catalog IDs, duplicate records, arbitrary SQL, rehashed physical redirection, silent location fallback |
| Truss result/application lowering | Typed entity/count/relationship/page contracts and ordered composite keys | Wrong owner/key/component, substituted original definitions, unknown codecs/join/execution/home/relationship profiles; sequence-to-scalar fallback |
| Common emission meaning | Owner type selection and bag projection; exact parameter tokens and explicit presence decoding | Actual source mutants detect removed type filter, DISTINCT insertion, binary-double rounding and null-to-absence substitution |

The native evidence complements component checks: B-007's 76 fresh PostgreSQL
application cases and B-006's exact Ashlar native cases/guards test storage results
at their own engines. It does not follow that every admitted numeric width,
precision/scale pair or recursive graph is natively qualified. In particular,
small-value unsigned-column observations are not full-domain boundary evidence.
Host publication/authority paths remain separate from pure compiler admission.
The support inventory must keep these candidate profiles and their exact subsets;
this review does not close the broader release/production qualification gate.
