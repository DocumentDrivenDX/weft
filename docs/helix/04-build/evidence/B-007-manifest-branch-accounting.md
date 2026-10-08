# B-007 manifest admission branch accounting

Scope: `validate_manifest_json`, `validate_manifest` and their admission helpers
in `crates/weft-core/src/backend.rs`. This is a source review against the seven
executed registration tests retained in B-007-manifest-admission-boundaries.
It does not account for frontend resolution, backend lowering or native support.

| Guard family | Positive and refusal assertions |
| --- | --- |
| Raw one-MiB bound | `manifest_collection_and_byte_limits_admit_the_boundary`: exactly one MiB admits; next byte is WFT-LIMIT. |
| Duplicate JSON members and unknown manifest members | `duplicate_declarations_and_unknown_shapes_refuse` and `manifest_versions_and_profile_capability_binding`. |
| Interface and dialect/IR pairing | `manifest_versions_and_profile_capability_binding`; `manifest_identity_evidence_and_language_guards_are_independent` covers empty/duplicate language pairs. |
| Backend ID/version/binding profile nonempty and NUL-free | Independent edits of each field in `manifest_identity_evidence_and_language_guards_are_independent`. |
| Target/capability count lower and upper bounds | Empty collections in identity test; 256/257 targets and 4096/4097 capabilities in boundary test. Unique IDs prevent duplicate guards from concealing the upper-bound assertions. |
| Manifest evidence ID validity and uniqueness | Empty, NUL and duplicate IDs in identity test. |
| Target ID/engine/version/layout/publication identity | Each field edited independently for empty/NUL in `target_profiles_require_each_identity_and_object_settings`; baseline retains observed declaration version/settings. |
| Object session settings and duplicate target IDs | Five non-object settings in target test; duplicate target in declaration-shape test. |
| Capability ID validity and uniqueness | Empty/NUL IDs in capability test; duplicate capability in declaration-shape test. |
| Target references nonempty/unique/declared | Empty, empty ID, repeated ID and undeclared target in capability test. |
| Capability language valid and declared | Empty and mismatched pair in capability test; valid pair excluded from manifest in independent-language test. **Repeated valid capability language pairs lack an isolated assertion.** |
| Logical/result domains object and nonempty | Both fields independently tested as empty object, array, boolean and null. Baseline has explicit domains. |
| Constraints and evidence ID validity/uniqueness | Both collections tested with empty/NUL/repeated IDs. Evidence additionally tests undeclared references and supported-without-evidence refusal; candidate without evidence remains distinct. |
| Obligation ID/parameters/failure code/uniqueness | Valid object baseline; empty/NUL ID, array/null parameters, absent prefix, empty suffix, lowercase/non-ASCII suffix, duplicate IDs each refuse in capability test. |

Remaining isolated admission assertions: malformed JSON syntax versus duplicate
members; malformed known-member deserialization versus unknown members; repeated
valid capability language pairs. These cases cannot be inferred from the passing
seven-test count. The broader critical semantic branch gate remains open.

Registration test source SHA-256: 4c2cfeec0b518fb8f68b5d9407e50e91788c959af62f891931883eb8d13a46ef.
