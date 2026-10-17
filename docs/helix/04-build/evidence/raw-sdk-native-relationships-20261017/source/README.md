# R13 original PostgreSQL admission controls

Source-only preparation. No native execution has occurred for this packet.
The three authored requests compile with exact Weft source
`6640db8f0ccdabd0e34206a136c5ae88a3e6b95d`, native payload
`41a907705278dd80f07c1761303948fb24cb59ab57e7d4f57695836e18518583`.
Each request and original compiled program is retained beside this file.

Run `native_controls.py` only during an assigned native lease in an owned
PostgreSQL18 cluster. `UMF_PG_TEST_DSN` must be a nonsuperuser, NOBYPASSRLS
principal with CREATE-schema permission. `UMF_PG_ADMIN_DSN` is the same owned
cluster's admin, used only to create temporary native operator classes (which
PostgreSQL reserves to superusers). No DSN or credential is recorded.

The harness creates fresh UUID-suffixed schemas, checks exact installed native
module bytes, recompiles each original request with that schema, and executes
the emitted named Boolean admission SQL unchanged. It runs the original owner
query only after positive pre-admission and repeats the unchanged admission
before releasing its observation. It removes every owned schema in `finally`.
An outer owned-cluster runner must also stop the server in its `finally`.

The eleven native controls are:

1. Deterministic C collation and default opclass: FK and alternate Key admit.
2. Genuine nondeterministic ICU casefold UNIQUE/FK: lowercase `alpha` is accepted
   by native FK referencing `Alpha`, while the original binary query would not
   equate them. Both original admissions must refuse before query execution.
3. Genuine custom target B-tree opclass using native built-in operators and
   comparator: original profile refuses the unqualified custom class.
4. Genuine custom source unique opclass for inverse max-one: FK-degree admission
   refuses while the independent standard alternate target Key still admits.
5. Original finite inverse degree 1..2 admits complete read-only RR scope; a NULL
   FK row contributes no edge.
6. The same degree at READ COMMITTED refuses.
7. The same degree in a read-write transaction refuses.
8. A zero-neighbor target refuses.
9. A target with three distinct source neighbors refuses max-two.
10. FORCE RLS hiding a target refuses the incomplete native scope even when the
    authorized visible counts would satisfy the bound.
11. Restoring the complete scope and valid graph admits again.

The receipt explicitly labels these original-owner Boolean/native-query
controls, not full SDK E2Es. Existing R12 full schema/instance/SDK E2Es remain
distinct; the successor SDK must additionally exercise its original admission
and isolation hooks against the immutable R13 compiler/runtime cohort.

Temporal qualification remains held. Native privilege inventory remains
incomplete; CREATE-schema and temporary-opclass admin permissions here are
fixture setup authority, not proposed production SDK grants.
