# Weft 0.4.55 hosted and public distribution proof

`umf-weft==0.4.55` passed the bounded hosted three-wheel release profile and
actual PyPI OIDC publication. The import remains `weft`. All source, workflow
and tag pins for this cohort target
`ea0fef0209aaf8e08ef165b6afe00b408873354f`; the compiler behavior retains the
qualified 0.4.54 base. This record does not relabel predecessor artifacts.

| Gate | Actual run | Result and scope |
| --- | --- | --- |
| Linux x86_64 and ARM64 | [40422686820](https://github.com/DocumentDrivenDX/weft/actions/runs/40422686820) | Two fresh exact installed-wheel packaging checks passed; publication skipped |
| macOS ARM64 | [40422688981](https://github.com/DocumentDrivenDX/weft/actions/runs/40422688981) | Actual Darwin ARM64 preflight, exact installed wheel and nine frozen Rust/Python/Chromium refusal-response comparisons passed |
| Promotion | [40424462909](https://github.com/DocumentDrivenDX/weft/actions/runs/40424462909) | Verified source/producer/ZIP/wheel/native pins and exact draft asset bytes; immutable release 424900540 published |
| PyPI | [40424569422](https://github.com/DocumentDrivenDX/weft/actions/runs/40424569422) | Source verification, actual OIDC upload, all three public archive-byte checks and fresh public Ubuntu install/import passed |

The [public release](https://github.com/DocumentDrivenDX/weft/releases/tag/v0.4.55)
contains exactly three wheels and their qualification metadata. No sdist was
produced. The [publication receipt](publication-receipt.json) records
`publication_mode: oidc_upload`, `oidc_trusted_publisher: true` and
`registry_bytes_verified: true`. The separate [consumer receipt](consumer-receipt.json)
records the fresh `https://pypi.org/simple` installation/import. Future qualified
releases use this main-only OIDC route without stored registry tokens.

The [verification record](verification-record.json) retains all seven Actions
artifact ZIP API digests, raw receipt/file hashes, twenty public release asset
IDs/digests/sizes and the executed cross-checks. Every ZIP matched its API
SHA-256 before members were read. Raw receipts equal the ZIP members. Every
public GitHub asset matched its API hash; the manifest and
[release pins](release-pins.json) matched the wheels, native modules, Mac
producer/preflight/tool receipts and source. All three PyPI-served archive
bytes equaled the qualified GitHub archives. [Registry metadata](pypi-registry.json)
and the [verified tuple](verified-summary.json) retain the exact file/hash cohort.

The six PostgreSQL/string vectors return `WFT-BACKEND-MISSING`; the three
Databricks vectors return `WFT-BACKEND-VERSION` under the selected wheel feature
composition. Generic source fixtures remain unchanged. The retained
[roster](compiler-roster-0.4.55.json), [Rust responses](rust-compiler-reports-0.4.55.json),
[Python](python-compiler-parity-0.4.55.json) and
[Chromium](browser-compiler-parity-0.4.55.json) receipts preserve that scope.

These results establish installed packaging and the named refusal-response
profile. They add no positive-query, native PostgreSQL, managed database,
executor or security authority claim. No native build/import/parity was run on
the local Mac. Consumer admission remains with its owning integration gates;
no consumer repin is established here. Broader compiler conformance
[PR run 40420146047](https://github.com/DocumentDrivenDX/weft/actions/runs/40420146047)
and [main run 40422641352](https://github.com/DocumentDrivenDX/weft/actions/runs/40422641352)
were still running at this evidence checkpoint and are not reported complete.
