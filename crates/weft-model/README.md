# weft-model

UMF model loading, identity resolution, logical typing and exact values, shared by
weft's compiler (`weft-core`) and other compilers that type programs against UMF
models (Fireweed's `fireweed-compiler`).

This crate reads pinned UMF 0.10 documents, resolves authored names to original
identities, derives logical scalar types, and checks exact literal tokens. It does
no SQL parsing, planning, backend lowering, execution or IO. It depends on no
other workspace crate.

## Version

Published to crates.io with every weft release, at the release version: the
`vX.Y.Z` tag, the `umf-weft` wheel and all four shared crates (`weft-model`,
`weft-expr`, `weft-backend`, `weft-postgresql-emit`) carry the same number, and
`weft_model::VERSION` reports it. Depend on them at one exact version, for example
`weft-model = "=0.4.58"`. While weft is 0.x any release may change the public
surface below; `CHANGELOG.md` records each change. Diagnostic codes are part of the surface.

Two build settings of this workspace do not travel with the published crate.
weft patches `ahash` to its `no-rng` feature (`vendor/README.md`), so the schema
validator uses no runtime entropy; without the same `[patch.crates-io]` entry a
dependent gets upstream `ahash` with runtime entropy. And wasm32 builds need
`--cfg getrandom_backend="unsupported"` (`.cargo/config.toml`). The compiled-in
UMF 0.10 schema ships in the crate (`schema/`).

## Public surface

| Module | Items | Meaning |
| --- | --- | --- |
| `catalog` | `Catalog`, `ModuleInput`, `Record`, `Admission`, `ScalarRules`, `scalar_type`, `scalar_type_projected`, `scalar_type_with`, `selected010*` | Prepare a bundle of pinned UMF 0.10 documents (digest, identity, schema and limit checks), keep the original bytes, resolve Records and Fields by authored name or identity, and type selected Fields. |
| `core_literal` | `field_domain`, `validate_field`, `check_literal`, `PreparedLiteral`, `ScalarLiteral`, `normalized_facets`, `normalized_literal`, `coefficient`, `unsigned`, `MAX_TEXT` | UMF CoreLiteral JSON (`{"integerToken":..}`, `{"string":..}`, ...) admitted against a selected Field's domain: facets, integer widths, decimal precision/scale, ranges, allowed values, examples and defaults. Refusals keep weft's historical codes `WFT-SECURITY-LITERAL` and `WFT-SECURITY-ONTOLOGY`. |
| `descriptor` | `Shape`, `Descriptor`, `EntityDescriptor`, `Member`, `AuthoredKey`, `ExactScalarType`, `RelationshipRead`, `RelationshipIdentity` | Identity-based type graphs of Records and their members, authored keys and relationship reads. |
| `types` | `Span`, `Identity`, `ModelPin`, `Family`, `ScalarFamily`, `FamilySet`, `LogicalType<F>`, `ScalarType` | Resolved identities and logical scalar types. |
| `name` | `Name` | Authored names; quoted names match exactly, unquoted names ASCII-case-fold. |
| `exact` | `Literal`, `LiteralKind`, `check_literal`, `coefficient`, `validate_float_token`, `validate_mathematical_decimal`, `validate_binary_token`, `validate_date_token`, `MAX_TOKEN` | Exact literal tokens and value-domain checks; exact decimal coefficients. No conversion through native floats. |
| `exact_instant` | `PROFILE`, `key` | UMF CONTRACT-065 RFC 3339 exact instants (`umf-rfc3339-exact-instant/1.0.0`): an exact `(epoch seconds, fractional digits)` comparison key at any precision. |
| `string_domain` | `Length`, `bounds`, `supported`, `validate`, `validate_facets`, `same_domain`, `literal_compatible` | Bounded Unicode-scalar string domains. |
| `json` | `sha256`, `checked_json`, `checked_truss_runtime_catalog_json`, `checked_json_bounded`, `checked_json_bounded_charged` | Bounded, duplicate-refusing JSON reading and digests. |
| `diagnostic` | `Diagnostic`, `Result`, `ResolutionReason` | Refusals: stable `code`, `phase`, `message`, optional source span. |

## Family sets: how new types stay out of old profiles

`LogicalType<F = Family>` is generic over the families a profile admits.

- `Family` is the closed set every existing weft dialect profile admits: boolean,
  string, integer, decimal. weft's IR and backends use `LogicalType` with this
  default, so they cannot hold a timestamp or a float. The wire format of
  `Family` is unchanged and refuses `"timestamp"` and `"float"`.
- `ScalarFamily` adds UMF `timestamp` (an exact UTC instant, any fractional
  precision, millisecond resolution included) and `float` (a finite IEEE 754
  binary64 value written as a UMF float token, kept as its token). A caller opts
  in by asking for it: `Catalog::field_as::<ScalarFamily>`,
  `Catalog::entity_descriptor_as::<ScalarFamily>`, and the other `*_as` methods.
- Nullability is a flag on every `LogicalType`. A selected `absent-allowed`
  scalar Field types as `nullable: true` only when the catalog's
  `Admission::nullable_fields` is set; otherwise it refuses as before.

`Admission::default()` is weft-sql 0.1's profile. weft-core sets the existing
flags exactly where it set the former crate-private fields; it never sets
`nullable_fields`.

## Not in this crate

SQL text, logical plans, expression trees (see `weft-expr`), backend manifests,
lowering and SQL emission stay in `weft-core` and the backend crates.

UMF extensions that only one compiler interprets (for example Fireweed's
`umf.fireweed` text sets and JSON bodies) are refused by the selected-element
checks here unless an admission flag names them; they belong to that compiler.
