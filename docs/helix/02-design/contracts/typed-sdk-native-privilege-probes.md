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

# Original raw-read native privilege analysis candidate

This source candidate supplies nonexecuting analysis programs for the selected
original raw-read catalog. It does not grant native authority or assert complete
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
 request:<complete original weft.raw-read/0.1 or weft.related-read/0.1 request>,
 program:<complete original compiler output>,
 probeTable:"__weft_privilege_<owned bounded identifier>"}
```

The original owner recompiles the full request and requires complete program
equality. Model/binding/parameter/source/profile/admission changes refuse before
probes are issued. The original request must be retained at catalog build;
reconstructing it from generatedSQL or runtime augmentation is not an API.
Search/action/bulk/engine programs are outside this first profile and refuse
explicitly until their own original statement inventory/probes are implemented.

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
