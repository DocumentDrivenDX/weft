# Weft generated required to-one inner join issuer v8

This evidence covers Weft Rust artifact issuance and closed descriptor validation
only. It does not claim a UMF installed package, runtime admission, native
execution, PostgreSQL behavior, or browser/Workerd support. There was no database
execution or catalog mutation.

## Source and physical witness

The positive original UMF source compiled with the installed UMF `codegen8`
build package and passed `umf_codegen.verify_table_deployment` with canonical
bundle equality. Source and verified deployment bundle are preserved under
`/private/tmp/weft-join8-source-20261105/positive/` along with the exact binding,
`nativeSource3` projection and v8 request.

- UMF source SHA-256: `a0489fea5ce87b3ca294007a9987d681ebefa07d52ed26beb83f4f76b55dcb5d`
- Verified bundle digest: `d6c6bacf0041f56b99052f806d010f73f2ad01a4cc808561df17c759b495ab8a`
- Binding configuration digest: `87e78860b98bb7fbfbf1535b85806e10691f10700f9906985f7952879efe7161`
- Authored Relationship: `demo/TaskNote`, directed Task to Note, target multiplicity exactly `1..1`, target Key `primary`
- Verified native FK: `taskNoteFk` / `fk_tasks_note`, `tasks.task_id` references `notes.id`
- Verified physical target Key: `notes` primary constraint `umf_k_5d660f25e1c2d3203a1bc5981910bfc2`

The same-name source vector adds authored names `owner` to Task.taskOwner and
Note.owner, and `value` to Task.taskValue and Note.payload. Binding logical names
use those authored names; the Weft v8 resolver maps them to exact source Field
refs within their owning Records. The source and verified bundle are under
`/private/tmp/weft-join8-source-20261105/same-name-fields/`.

- UMF source SHA-256: `e31d5c9740f9a53ca686aa9852ba76667778dd68ceaedbc8920c557575d0ff86`
- Verified bundle digest: `a0eac010cde9deb2fe51bbdb948eadf79fadb9ec3ec8a53ba22780824c01b658`

A separate verified nullable-source vector under
`/private/tmp/weft-join8-source-20261105/nullable-source/` uses Task.noteRef with
`absent-allowed` nullability and an exact FK pair to Note.primary. The issuer
refuses that relation as `nullable-source-foreign-key` while preserving its base
read, Page and Relay branches.

- UMF source SHA-256: `981dfa06a2e8e197b88d0c4171e137cee27832bb03d3ca6425231009db1a8678`
- Verified bundle digest: `78aa36d499fdd113669fad4a1ec38e7a65c897aff57d2a63abc104ae637309e2`

Each fixture has a reproducible builder and captured `build-output.json` in its
directory. Builders use the installed environment at
`/private/tmp/batten-page7-final-dev-20261105/source-environment` and public UMF
compiler/deployment verification APIs.

## Issued schema and artifact

The authoritative schema is registered by Weft `CONTRACT-018` at
`docs/helix/02-design/contracts/generated-read-algebra-v8.schema.json`.
SHA-256: `2b41e67e7fb2727d04c1a3d34da8c24c42bec26228caa1c261da024781720f83`.

The positive full artifact and isolated descriptor are preserved under
`/private/tmp/weft-join8-source-20261105/positive/`.

- Full `/8` algebra SHA-256: `64c810649a63b24e4f24026c04141946d62de3af24c4d8a1c1ecc32be452a564`
- Required inner join descriptor SHA-256: `2953a3233aa21fdc1053526d29d919a15dc08a5510b0ea428795740e5bfc5879`

The descriptor catalogs every original module Relationship per root. Eligible
entries bind the original Relationship identity and full normalized cut, exact
ordered binding Field/column pairs, verified FK and Key rows, owner-qualified
field/predicate/order fragments, source-derived `output.valueCodec`, global
output ordinals and OIDs, all-only terminal, empty projection behavior, and the
inherited PostgreSQL server byte guard. `/8` recomputes the ordinary Page digest
over the full root while omitting only its existing feedback slot. The v7
builder path retains its prior authored Field-ID lookup behavior; authored-name
resolution is private to `/8`.

## Verification

Focused Rust command:

```sh
env PATH=/Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:/usr/bin:/bin \
  /Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin/cargo \
  test -p weft-postgresql generated_join_v8::tests -- --nocapture
```

Result: 6 v8 unit tests passed. They cover the positive verified join, inherited
Page and Relay preservation with recalculated Page digest, owner-qualified
same-name fields, percent escaping in native identifiers, nullable source refusal with base/Page/Relay preservation,
and tampered FK, multiplicity, physical pair and raw binding rejection.

Legacy regression commands also passed: `cargo test -p weft-postgresql v5_ -- --nocapture` (5 tests), `cargo test -p weft-postgresql v6_ -- --nocapture` (2 tests), and `cargo test -p weft-postgresql v7_ -- --nocapture` (2 legacy tests plus the v8 coexistence test matched by the filter).

Public Rust/Python compile check:

```sh
env PATH=/Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:/usr/bin:/bin \
  /Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin/cargo \
  check -p weft-runtime -p weft-python
```

Result: successful. Existing unrelated compiler warnings remain in vendored and
other workspace modules.

The JSON Schema passed Draft 2020-12 schema self-validation. The positive
algebra's non-Page portion, including closed join catalogs and output codecs,
passed Draft 2020-12 instance validation after localizing the existing Page and
codec schema references for the validation process.

Additional schema vectors validated UUID/OID 2950, bytea-16/OID 17, date/OID
1082 and UTC timestamptz/OID 1184 result codecs. The Rust vector refuses
unsupported authored refinements.
