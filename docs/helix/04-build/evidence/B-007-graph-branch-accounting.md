# B-007 model graph and authored endpoint semantic branch accounting

Scope: `Catalog` application descriptors in `application_model.rs`, after pinned
UMF 0.7.0 envelope admission, reached through SQL/frontend catalog selection.
Arbitrary manually constructed `Record`, `Literal` or typed-plan objects are not
public compiler inputs. This map names semantic families rather than private
condition/line coverage.

| Family | Assertion evidence |
| --- | --- |
| Owning document and module selection | B-007 catalog admission/boundary tests: exact bytes/hash/version, document and selection limits; no invented or partial owning document. |
| Local dependency resolution | Missing selected element and new missing dependency module refuse; a map item in an unselected local module is retained from the whole owning document. Missing reference fields themselves violate the envelope. |
| Ordered authored membership | Existing ordered members and new missing members, non-Field member and missing authored name diagnostics. Exact duplicate references are envelope refusals; no downstream execution is claimed for them. |
| Member name/identity lookup | New missing/unowned and exact by-name/by-identity equivalence; new duplicate-name ambiguity. The defensive repeated-identity lookup path is protected by ordered-members validation/envelope uniqueness. |
| Identity reuse, cycles and resource bounds | Existing recursive record points back by identity; depth128/129 and identities4096/4097 assertions. Seen identities terminate cycles without copying or losing dependencies. |
| Required/optional Field availability | Existing required and absent-allowed descriptors; native null remains distinct and unsupported. Unspecified/future availability refuses. |
| Scalar and cardinality dispatch | Four scalar families belong to scalar-resolution accounting; unknown selected scalar meaning refuses. New unspecified/future cardinality diagnostics. |
| Sequence/map item role | Existing sequence and new map descriptors retain exact leaf identities/types. Non-Field item refuses; scalar container collisions violate the envelope. Nonempty container references do not imply a supported scalar/structured interpretation. |
| Structured target and compound facets | Existing cyclic structured graph; wrong target role and simultaneous scalar/reference meaning refuse. Compound facets violate envelope admission before the defensive graph guard. |
| Graph role preconditions | Public roots are selected Records; members/items must be Fields and structured targets must be Records. Other visited roles are excluded before recursion; no fabricated direct Record bypass counts as SQL-input proof. |
| Authored key identity/membership/types | Existing missing key and optional key scalar refusals; new duplicate key-ID ambiguity and non-member field diagnostic. Exact duplicate key references violate envelope uniqueness. |
| Relationship discovery/orientation | Existing forward/inverse identities and ambiguous name; new missing name, duplicate relationship ID, and unselected-module exclusion. |
| Endpoint shape, lifecycle and multiplicity | Existing directed/association/lifecycle/target-multiplicity controls; new polymorphic source/target, non-Record target/source and source multiplicity refusals. |
| Source/target key selection | Existing target authored key and unkeyed inverse refusal; new single-key fallback, primary-key precedence, multiple-primary and no-primary ambiguity. No storage key is invented. |

B-007-graph-final-audit records all 18 model integration tests passing, none ignored
or filtered. B-007-typegraph-audit remains the earlier 15-test checkpoint. These
families are accounted at the catalog/descriptor boundary; native value integrity
and storage correspondence belong to the backend/native scopes. This review does
not qualify unknown graph roles, native null, unregistered extension semantics or
every possible graph/model combination.
