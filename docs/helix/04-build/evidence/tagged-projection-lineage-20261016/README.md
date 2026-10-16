# Tagged projection lineage frontend (.4)

This source-only optional Core feature resolves one-scan direct scalar Field
projections while retaining original required/absent-allowed availability.
It requires an explicit .4 request and tagged-current-projection capability.
It emits no SQL, issues no existing current-query designation, grants no
disclosure permission, and is not part of public Python0.4.36.

R1 admitted complete-entity wildcard syntax after resolution expanded it into
Field expressions. Independent review found P2 PRESENCE-LINEAGE-001. R2 checks
the original AST before expansion; original failures and both review rounds
are retained. Proper public-UMF-validated array/map controls distinguish
selected-container refusal from successful scalar reads with unselected
containers. The older unknown-many refusal is separately identified.

Final source passes352 enabled Core tests,328 disabled Core tests,70artifact/
89schema checks and11package/16test boundary checks. Independent review passes
53enabled and53disabled outcomes and verifies1196 reconstructed repository
inputs. The original source observation is retained byte-exact; the new fixture
changes only the explicitly authored frontend envelope version.

No original132 shared/native acceptance case is promoted. Tagged SQL/result
handling, actual owner-selected disclosure provenance, bridge registration,
authenticated current authority and guarded native qualification remain open.
Before a new current-query context can be issued, every legacy scalar lowerer
must explicitly refuse that context rather than infer its profile from older
feature booleans.
