---
ddx:
  id: CONTRACT-014
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: US-009
      kind: informed_by
    - id: CONTRACT-012
      kind: informed_by
    - id: CONTRACT-013
      kind: informed_by
---

# CONTRACT-014: Issued current-presence runtime and Python bridge

**Type:** library / candidate embedding protocol. **Version:** 0.1-candidate.

## Purpose

Expose the separate CONTRACT-012 presence context and CONTRACT-013 conditional
PostgreSQL transport through actual issued runtime/Python objects. Preserve the
actual original preparation across language boundaries; never import an issued
object from a JSON report or reuse the uniform scalar current-query type.

## Scope and Boundaries

The optional feature is `current-presence-candidate` in weft-runtime and
weft-python. It MUST explicitly enable application lineage and the new Core/PG
presence features. Older scalar features alone MUST NOT expose the class or
factory. Default wheel configuration and all published0.4.36 pins remain separate;
this contract does not enable publication or select a backend implicitly.

This bridge returns compiler/data reports only. It accepts no database connection
or execution callback and authenticates no source/caller/Subject/current cut.
CONTRACT-012/013 whole-UMF producer, source/native custody, complete populations,
closing checks, guarded release, cancellation and cleanup obligations remain
required, including empty results. Full original132 B-008 acceptance remains
required. Runtime/Python tests do not qualify a native backend.

## Normative Surface

The existing issued `ApplicationLineagePreparation` exposes the feature-gated
`prepare_current_presence()` factory. It MUST issue the child only by invoking
the actual Core presence factory on its retained immutable typed original parent.
The original .4 request and all profile/model/source pins remain unchanged.

Runtime `CurrentPresencePreparation` has private fields and no public data
constructor or Deserialize. It MUST retain shared ownership of the actual
original object, not merely equal source bytes. The new child MUST remain usable
after the caller drops its parent handle. `is_from(original)` MUST compare actual
object custody: a separately prepared byte-identical request MUST return false.

| Runtime method | Required behavior |
| --- | --- |
| `json()` | Exact issued Core presence report bytes |
| `is_from(&ApplicationLineagePreparation)` | Actual original object identity |
| `native_request_json()` | Actual Core native-input JSON from the same original context |
| `lower_postgresql_current_presence_candidate_json(binding_json)` | Invoke the actual new typed PG lowerer; exact artifact JSON |
| `check_postgresql_current_presence_candidate_json(binding_json, transport_input_json)` | Invoke the actual new typed PG transport checker; exact report JSON |

Every lowering/check/native-request operation MUST derive its scoped Core context
from that same retained immutable original and require exact correspondence to
the issued preparation report. It MUST NOT reparse/import a replacement SQL/AST,
caller-provided preparation, success report or context JSON. No public callback
or borrowed Core context escape is exposed by this runtime wrapper.

Python exposes a distinct frozen `weft.CurrentPresencePreparation` type, with no
constructor, subclassing, instance field mutation, copy/pickle reconstruction or
report importer. Its actual-parent factory lives only on the issued
`weft.ApplicationLineagePreparation`. The child MUST retain that actual Python
parent object and the matching runtime original. Methods MUST check this custody
before using the child. `source_preparation()` MUST return that same Python
object, not a new equivalent wrapper.

| Python method | Arguments / result |
| --- | --- |
| `source_preparation()` | Same actual parent object |
| `json()` | Exact Core preparation JSON string |
| `native_request_json()` | Exact Core native-input JSON string |
| `lower_postgresql_current_presence_candidate_json(binding_json)` | Exact original binding string; returns actual artifact JSON string |
| `check_postgresql_current_presence_candidate_json(binding_json, transport_input_json)` | Exact strings; returns actual transport report JSON string |

Binding/input values MUST be exact builtin Python str, not str subclasses,
bytes, mappings, arbitrary objects or coercible wrappers. No caller `__str__` or
`__len__` override may execute. Binding UTF8 MUST fit4,000,000 bytes and transport
input16,000,000 bytes; invalid Unicode conversion MUST refuse. The underlying
closed data versions/capabilities and conditional semantics remain exactly
CONTRACT-012/013. Reports retain false authority/securityReady and mandatory
nativeAdmission. The bridge MUST NOT mint native authority from type possession.

## Precedence and Compatibility

`CurrentQueryPreparation` remains distinct. Its old scalar methods MUST NOT
accept the new class, and its factory MUST continue to reject .4 parents. The
new type MUST NOT expose old scalar lower/check methods or accept old binding
versions. New-class methods MUST reject wrong self types before backend handling.
Feature-disabled and legacy-wheel imports MUST expose neither the class nor its
parent factory. Existing ABI1 result meanings, public compiler registrations and
Databricks/Spark paths remain unchanged. No WASM class is added by this increment.

## Error Semantics

Bad Python values/custody MUST refuse with ValueError carrying
`WFT-CURRENT-PRESENCE-BRIDGE: Current presence bridge refused`. Wrong constructors,
self types and signatures MAY produce TypeError or AttributeError before Rust
entry. Core source preparation, actual lowering and transport failures retain
their existing scoped diagnostic codes and fixed messages. No failure may return
a partial artifact/success report or disclose private keys, values, counts or
policy census. A later check failure cannot erase already issued SQL; hosts MUST
release nothing and clean up their entire owned native stage on any failure.

## Examples

`parent = weft.prepare_application_lineage(original_request_json)` followed by
`child = parent.prepare_current_presence()` issues the new child when its explicit
feature and original .4 profile are admitted. `child.source_preparation() is parent`
MUST be true. After deleting the parent's local variable, the child still owns the
same original source context. `parent.prepare_current_query()` refuses .4;
`weft.CurrentPresencePreparation(child.json())` cannot import it.

## Validation and Unresolved Evidence

Use real compiled Python types and exact actual-producer fixture outputs to test
parent identity/lifetime, byte-identical Core/native/lowering/check reports,
constructor/copy/subclass/pickle refusals, old-descriptor type rejection, exact
string/no-coercion rules, invalid UTF8/size bounds and explicit feature absence.
Retain wheel/native file hashes and interpreter/toolchain qualification. These
embedding checks do not replace actual native producer/stage/result/current-cut
composition or the original132 acceptance obligations.
