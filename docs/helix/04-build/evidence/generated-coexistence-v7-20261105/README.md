# Generated Page and forward-connection `/7` issuer candidate

**Status:** local, unpublished issuer candidate; source/schema review passed. This packet covers compiler issuance and closed-schema behavior only. It does not qualify UMF verification, the compiler-free runtime, Batten Host composition, native PostgreSQL execution, browser, Workerd, or release.

The explicit Python issuer `prepare_generated_read_algebra_v7_json` accepts request profile `weft.generated-read-algebra-request/7`. It independently issues existing `/5` forward-connection and `/6` ordinary-Page semantics from one admitted `/4` root and joins the output. Each family keeps its own support/refusal branch. Both may refuse while the inherited `/4` root remains available. Standalone `/5` and `/6` still require at least one eligible root. Default and frozen profile entry points are unchanged.

The combined Page `issuedProgramDigest` is SHA-256 over canonical JSON of the complete combined root with only `ordinaryPage.continuation.fixed.issuedProgramDigest` omitted. The preimage therefore includes the full forward descriptor and its template digests. Issuer checks shared root, Record, Key, Field, codec, source Field, output, and native mapping identities when both families are eligible. The new closed schema is [`generated-read-algebra-v7.schema.json`](../../../02-design/contracts/generated-read-algebra-v7.schema.json); the corrected ordinary-Page schema remains at SHA-256 `242bc54428d4e0d4a2e7efa147ca40c8e445725cee4894e1c5b885c334b10dc2`.

## Candidate and source

- Wheel: `/private/tmp/weft-generated-v7-wheelhouse-review-20261105/weft_sql-0.4.66.dev1-cp39-abi3-macosx_11_0_arm64.whl`
- Wheel SHA-256: `1214a26beb73b351dcf7c684baf0465562e512798afe5138dce39d5ac1588660`
- Isolated installed interpreter: `/private/tmp/weft-v7-wheel-review-venv/bin/python` (Python 3.9.6)
- Issuer request: [`request-v7.json`](request-v7.json), derived from the retained v6 wheel fixture by changing only the request version
- Issued output SHA-256: `7a9a44cd10274dcc724a695cca2dbb12ca9a47adb73352e456df47821acaabd3` (34,336 bytes)
- The source inventory pins HEAD and the candidate source, schema, and governed input paths listed in [`source-inventory.json`](source-inventory.json). It does not enumerate every dirty worktree file or evidence directory.

## Checks

- Rust command `cargo test -p weft-postgresql generated_read_algebra --lib -- --nocapture`: 20 passed, 0 failed. Raw command output is retained in [`rust-tests.log`](rust-tests.log), SHA-256 `0bb0c0e17b5d95cd3825760bed69b9b3e49f4b28ee5ac6f96c6e9622348508c7`, including co-issuance/digest and both-refused `/7` root coverage.
- Installed `.66` wheel imports and exposes the explicit v7 function; a v6 request profile passed to it is rejected.
- Installed `.66` outputs for the retained v5 and v6 requests are byte-equal to the frozen `.65` wheel vectors: 20,377 and 28,659 bytes respectively. The existing `.65` wheel remains SHA-256 `d6650eb65226d94928526014b815e041cb7c00bf29cecb55bc828972c0f275b8`.
- Ajv 8.17.1 / Draft 2020-12 accepts the actual combined v7 artifact and both mixed support/refusal schema branches. It rejects missing varchar length and incorrect OID mutations at six output locations: Page field, Page Key, Page hidden Key, forward Key, forward execution Key, and forward Field.
- Independent Sol review confirmed the wheel/output hashes, recomputed the combined root digest, deep-compared the forward payload with historical `/5` and the Page payload with `/6` except the intentional combined digest, and repeated the output mutation checks.

This candidate does not imply acceptance for execution, deployment, publication, or version release.
