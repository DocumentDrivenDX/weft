# Native privilege successor source and DML analysis prototype

The selected source controls pass two bulk list-facet tests and nine owner
privilege tests with zero skips. Original scalar bulk artifact digest remains
f34aa692fefcf4b225e3e793c56fc9f4e4bf9b62220d6b2ff40bd365a6caa5f1.
List-only slots retain nullable/container/elementNullable/maxItems; outer and
selection declarations match all facets, and action input values stay scalar.

The successor API recompiles original read/Related/search/action/bulk programs
and retains exact original statement paths and hashes. Search adds index/domain
admissions, actual locks, and base-vector typed Vars preserving original typmod
casts. Mutation inventory covers action first-false/collect-all branch union and
bulk actual preview/selection/update, with explicit nonexecuting RULE analysis
metadata for DML. Source tests cover five original authored actions, including
CREATE and condition branches. No embedded unused action effects are claimed as
executed bulk statements.

A separate owned PostgreSQL18.6 prototype passes eight checks: original UPDATE
RETURNING remains commandType2 with RETURNING and typed-Var subqueries in native
pg_rewrite; the rule never fires, input/event relations are empty, business rows
stay unchanged, role is NOSUPERUSER/NOBYPASSRLS, and server stops. It establishes
the analysis primitive only. INSERT/DELETE, selected-catalog function/operator/
cast closure, independent grants and complete runtime/receipt coverage still
require exact native proof. All compiler reports remain complete:false and
nativeAuthority:false; new search/mutation grant inventories remain unresolved.

Initial source reds are preserved: action test fixture needed its changed model
hash reflected in security ontology; keyword query also had vector projection
metadata and the test had incorrectly inferred query kind from vectorBinding.
Prototype r1 lacked psycopg and r2 sandbox shared memory was refused before any
server; authorized r3 native run passed. No product condition was weakened.

R16 wheel/browser/native receipts remain immutable. This source batch has no
new installed wheel or browser/native selected-operation qualification yet.
Stable defaults/versions/publication are unchanged. Temporal remains HELD.
