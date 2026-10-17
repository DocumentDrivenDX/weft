---
ddx:
  id: weft.typed-sdk-source-exact-verification
  type: contract
  activity: design
  status: draft
  links:
    - id: weft.typed-sdk-raw-collections
      kind: informed_by
    - id: CONTRACT-070
      kind: informed_by
    - id: CONTRACT-072
      kind: informed_by
  authoring:
    home: repo
---

# Original source-exact native column verification

The optional original RawRead and ActionRequest selector
`verificationProfile:"source-exact/0.1"` admits PostgreSQL18/Lakebase18 only.
It requires complete source-pinned original ColumnSet/binding facts for every
referenced read Record or touched action entity/Field table. Missing selectors
preserve legacy serialized bytes and generic lowering; those legacy artifacts
do not qualify exact initial native verification. The host must refuse their
promotion or execution where this exact guarantee is required.

Each relevant original NativeColumn retains its type, notNull and generation,
and additionally authors `collation:null` for noncollatable physical types or
`collation:{schema,name}` for collatable String columns. Both qualified identity
components contain1–63 UTF-8 bytes and no NUL. There is no inferred C/default
collation or portable locale equivalence. An explicitly authored
`{schema:"pg_catalog",name:"default"}` names that native identity only. Legacy
consumers refuse the additional fact unless they explicitly admit it.

The original output carries `obligations.nativeProjection` with
`version:"umf.native-columns/0.1"`, `verificationProfile:"source-exact/0.1"`,
`nativeCatalogBeforeAfter:true`, and nonempty `columns` containing unchanged
original Boolean `nativeAdmission` programs. The host executes every program
before and after the business operation and releases no result on refusal.
Exact native type/typmod, NOTNULL, generation kind, qualified collation identity,
and complete sorted non-dropped table-column inventory are checked even on an
empty table. Inheritance is refused. Existing native context/fence and later
shape fingerprint checks remain necessary; these facts are not credentials.
Generation remains a kind policy: no expression/default equality is invented.

Read metadata uses the original explicit app-schema source where schema-neutral.
Actions and bulk executions resolve their own unqualified quoted table through
`pg_catalog.to_regclass(pg_catalog.quote_ident(originalTableLiteral))`, matching
the original business/lock SQL search path. They do not assume current_schema()
is the application schema. Qualified binding schemas use original literal
namespace identity. Action/bulk admission parameters are literal-only.

Related requests require identical source/target verification profiles and merge
both original admission inventories. Search propagates the original read packet
to its executed top-level obligations. Bulk requires both action and selection
profiles to agree and unions their independently emitted original facts under
the bulk native context. Embedded read templates remain their original artifacts;
the SDK neither rewrites their SQL nor manufactures a stronger profile.

Source tests do not qualify native support. Actual initial-empty-table drift,
search-path resolution, pre/post withholding, serialized legacy refusal and
same-source wheel/browser/native proofs remain separate gates. Temporal remains
HELD; native privilege completeness and managed mutation qualification are not
changed by this profile.
