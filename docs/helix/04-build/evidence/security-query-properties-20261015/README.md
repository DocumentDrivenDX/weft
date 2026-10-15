# Original query property candidate evidence

Legacy source-only structural requirement derivation: 282 core tests, 16 boundary
controls, and three retained original request/report probes. The independent host
correspondence test verifies exact original source/hash/SQL identity, occurrence
paths and properties, explicit unsupported numeric SUM and field-free counts.
Caller annotations, changed pins and malformed/oversized requests refuse without
partial stdout. Exact prior-main mapping bytes were captured from an archive of
c492444 and are freshly compared to current mapping output for all three requests.
The distinct report remains refused by the historical mapping entrypoint.

Fixtures live under tests/query-properties/fixtures; receipt.json pins source,
schema, fixture and executable bytes. Reproduce with freshly built examples:
`WEFT_QUERY_PROPERTIES=<security_query_properties> WEFT_SECURITY_MAPPING=<weft-security-mapping> python tests/query-properties/check.py`.

No native database or current catalog source is admitted by this receipt.
COUNT(DISTINCT), LIMIT0, WHERE FALSE, numeric SUM properties, actual current
catalog/storage-transform consumer and native before-dispatch/current authority
remain required. Preserving a legacy mode source is not current catalog support.
No original case, assertion, allocation or acceptance deadline is changed.
