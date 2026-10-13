# Compatible physical revisions candidate

Register `weft_databricks::compatible_profile::CompatibleRevisions` in Rust, or build runtime/Python/WASM with only `ashlar-databricks-compatible-revisions-candidate` for this backend.

Request selection:

- backendId: `ashlar.databricks`
- backendVersion: `0.1.1-compatible-revisions-candidate`
- targetProfile: `dbsql-compatible-revisions-candidate`
- options.allowCandidate: `true`
- binding capability: `binding.compatibleSchemaRevisions`

The existing binding profile remains unchanged. Object/Edge `records` may add `compatibleSchemaRevisions`: 1–32 unique nonempty strings, each at most 256 UTF-8 bytes, including the required primary `schemaRevision`. Omission retains the old single-revision guard. Explicit empty/null selections and projection selections refuse. Original logical model/field pins, scalar domains, result descriptors and IR remain unchanged.

Every selected physical revision becomes a bound UTF8_BINARY comparison in a parenthesized OR in the existing integrity checks. Unselected/null revisions remain violations rather than being filtered from the result. The host must enforce all emitted integrity checks before releasing buffered results.

The new `ashlar.compatibleSchemaRevisions` host obligation identifies the selected record/revision sets and original publication/model pins. Hosts must establish accepted revision custody and original logical field/key correspondence for all selected revisions. This is not a compiler proof of schema compatibility.

Existing Candidate, NativeReview, Qualified and other registrations delegating to historical candidate admission reject the extension. Their versions and declared capabilities are unchanged. Runtime feature combinations with other Ashlar registrations are refused at build time.

This is a source-distributed candidate, not a released package or native qualification. The focused compiler tests cover legacy artifact preservation, parameterized guards, invalid selections, historical-registration refusal, explicit capability/obligation emission and candidate opt-in. Native execution against evolved Ashlar publications remains a separate integration gate.
