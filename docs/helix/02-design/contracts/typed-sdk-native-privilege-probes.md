---
ddx:
  id: weft.typed-sdk-native-privilege-probes
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: weft.typed-sdk-raw-collections
      kind: informed_by
---

# Original native privilege analysis candidate

This source candidate supplies nonexecuting analysis programs for the selected
original raw-read or native PostgreSQL search catalog. It does not grant native authority or assert complete
least privilege. UMF CONTRACT-071's required per-target native qualification and
runtime metadata/control inventory remain separate. Temporal stays HELD.

## Closed source API

Under existing raw-postgresql-reads, Python/WASM/native runtime expose
`compile_native_privilege_probes_json(request)` and
`check_native_privilege_probes_json(program)`. WASM framing refuses malformedUTF16
and inputs beyond4MiB. Native compiler admission still enforces its exact JSON
framing and duplicate-member rules. Neither API opens a connection or executesSQL.

The request is exactly:

```text
{version:"weft.native-privilege-probes/0.1",
 request:<complete original weft.raw-read/0.1, weft.related-read/0.1 or weft.search/0.1 request>,
 program:<complete original compiler output>,
 probeTable:"__weft_privilege_<owned bounded identifier>"}
```

The original owner recompiles the full request and requires complete program
equality. Model/binding/parameter/source/profile/admission changes refuse before
probes are issued. The original request must be retained at catalog build;
reconstructing it from generatedSQL or runtime augmentation is not an API.
Engine programs remain outside this source profile and refuse explicitly.
The action/bulk successor inventories their original statements and uses an
explicit nonexecuting RULE analysis profile for DML; it remains unqualified
until the new closed metadata, analyzed trees and independent native grants are
proved for the same selected operation.
The search extension is a successor source candidate; the earlier raw-read
native/browser evidence does not qualify its new statements or vector protocol.

The output retains request/program, exact sourceProgramDigest, statements, locks,
grantSemantics and an owner structural digest. Each probe also provides owner-derived unique ordinal outputColumns for native
view analysis, so the resolver never guesses output arity from SQL. Its version is
weft.native-privilege-probes/0.1; digest owner is
weft-postgresql-native-privilege-probes, projection0.1. Check recomposes the entire
program and compares exact JSON, including every nested probe and protocol type.

## Statement and protocol custody

Each business or original native admission statement retains its exact JSON
pointer, role, SQL bytes/SHA256, complete original parameters and positional order.
Roles cover business-read, projection admission, Key integrity, relationship
integrity and declared term-index admission. Related source selection remains
inside the one actual business statement; no separate source query is invented.
Original LOCK statements retain exact bytes/path/hash and their table-level
SELECT requirement; column-only SELECT does not establish LOCK sufficiency.

Every compiler-emitted parameter hole must have its exact explicit pg_catalog
cast. The owner scanner skips literals, quoted identifiers and comments, refuses
unestablished dollar quoting/casts or incomplete slot inventories, and replaces
only those owned holes in analysisSQL. Each replacement is a scalar Var subselect
from an owned temporary input relation with explicit native column types. This
preserves cast analysis that a typedNULL constant can erase. Input relation and
scalar-subselect dependencies are probe support, distinguished from original
business privileges. Original executableSQL remains byte-for-byte unchanged.

The candidate protocol inventory records PostgreSQL180006 and the pinned
psycopg3.3.6 extended scalar/array adaptation family. Integer variants retain
int2/int4/int8/numeric; other primitive and array types retain their exact native
OIDs. Unknown String, nullable input and empty-list OID0 cases retain the inferred
original cast type. Each per-parameter pre-cast variant is analyzed independently:
every post-cast business operand type is fixed by the original SQL. This proof
does not authorize a different driver/adapter or erase source value constraints.
Native qualification must independently observe actual protocol OID/format,
inference, casts, type input/receive functions and array-element behavior.

Probe setupSQL creates only the owned temporary input relation and contains no
business rows. AnalysisSQL is intended for native parse/analyze metadata such as
an owned temporary view's pg_rewrite expression tree; never execute it as a
substitute business query. The host owns temporary fixture identity, isolation,
cleanup and native-version/context checks, and must distinguish probe-only objects
and constructs from required real dependencies. Compiler output is metadata,
not permission to inspect or alter unrelated relations.

## Honest completeness boundary

All outputs currently retain complete:false and nativeAuthority:false. Native
function/operator/cast/type-I/O analysis, independent grant/revoke sufficiency and
runtime metadata/control/closing statements remain explicit unresolved items.
Original compiler grantSemantics is retained conservatively; symbol names are
not overload identities. Selected semantic fields and actual native used columns
must be reconciled against native analyzed trees before an exact claim.

The native resolver must bind its evidence to this original program digest,
statement/probe hashes, actual target/driver/principal context and coverage
manifest. It must classify direct required grants, native internal operations and
probe support separately. Unknown control/data-dependent branches, unsupported
roles and unavailable native context remain unresolved, never fake-complete.
Source tests or successful user queries alone do not prove least privilege.

Probe metadata is bounded to4096 input-column/variant units and4MiB serialized
artifacts, and must pass the same faithful JSON reader as the checker. Exceeding
analysis capacity refuses explicitly without changing the original query or
issuing a completeness claim; no parameter or statement is silently dropped.

## Search successor source profile

The exact original search compiler is re-invoked before any inventory is issued.
The report retains the executed `/statement/sql`, `/nativeAdmission/sql`, all
`/preflights/*/sql`, and `/selectionTemplate/lockSql/*` paths. The embedded source
selection is not invented as a separately executed business query. Positional
parameters receive owner ordinal names while retaining their original `source`
and carrier metadata. Search native object/write-grant closure is explicitly
unresolved; `grantSemantics` is null and `complete` remains false.

Vector casts are admitted only from the original checked `vectorBinding` schema,
dimensions and extension version. The temporary Var has the base original
`schema.vector` type; the original `vector(dimensions)` cast remains in the probe,
preserving its typmod coercion. Candidate protocol variants are native-inferred
OID0 and text OID25. No native vector type OID or driver format is invented.
Actual native inference, coercion and type-I/O dependencies require new proof.

Action/bulk DML analysis is not represented by a SELECT approximation: assignment
coercions, write operations and RETURNING dependencies must remain original.
A separately reviewed, never-fired owned temporary RULE prototype establishes
the viability of native analyzed-tree custody for original UPDATE RETURNING.
Its eight PostgreSQL18.6 checks include typed Vars, original UPDATE/RETURNING
nodes, empty event/input tables, unchanged business rows, a non-superuser without
BYPASSRLS, and stopped owned server. This prototype is not a selected-catalog
privilege or grant qualification.

## Mutation successor source profile

Action inventory covers exact principal admission, pre/post images, all declared
pre/postconditions, change guards and effect statements. Its coverage union names
invoke-first-false and validate-collect-all; it does not promise every branch runs
in every invocation. Bulk inventory covers only the executed principal admission,
preview, locked selection and array-bound UPDATE. It does not enumerate unused
individual effects from the embedded action template as executed statements.

Each original DML retains SQL/hash and source-owned operation, entity, assigned
Fields and RETURNING Key descriptors. Its typed-Var probe explicitly declares
`analysisKind:pg-rewrite-rule/0.1`, a temporary `analysisRelation`, original rule
identity/event/expected business command type, and `neverFire:true`.
`setupStatements` creates the bounded input relation, a temporary event relation
with original native Key column types through `WITH NO DATA`, and an unconditional
INSTEAD rule storing the original DML. The consumer must read native pg_rewrite;
it must never execute the DML or fire the rule. The existing SELECT/view consumer
must refuse this kind until it has an explicit closed dispatch implementation.

Input/event relations, scalar subselects, rule and event-type projection are
probe support, not business privilege evidence. Native command relation
permissions, assignment coercions and RETURNING expressions must be interpreted
without replacing DML with SELECT. ROW EXCLUSIVE locks retain table privilege
alternatives INSERT/UPDATE/DELETE/TRUNCATE/MAINTAIN; the raw-read ACCESS SHARE
SELECT requirement is not reused for them. All write-grant, runtime receipt,
closing/reconciliation and independent native proof obligations stay unresolved;
`grantSemantics:null`, `complete:false` and `nativeAuthority:false` are deliberate.
