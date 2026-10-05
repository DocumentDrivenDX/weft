# Discovery input: reads an application needs from Weft SQL

Proposed 2026-10-05, for the owner's review. This is source material from an
application-facing interface that serves reads from a mutable store (Truss) and a
warehouse (Ashlar), not an approved requirements document. It names no particular
application, and it changes no existing document.

## Why now

Weft SQL v0.1 deliberately defines a small exact subset: qualified required scalar
columns, equality predicates, inner equijoins, `SUM` and `GROUP BY`. An application
that wants Weft SQL as its read language, and needs to recognize a subset itself until
a compiler is integrated, found the common reads it needs fall just outside v0.1.
The gaps are small and each can be stated as exactly as the rest of the dialect.

## Requirements

### 1. Whole-entity projection

Reading one entity by its key, or a page of entities, needs every declared member
of a record, including optional, many-valued and structured members. v0.1 refuses
them, so an application has no way to ask for "the entity". Proposed:

- A projection form that selects all declared members of one source (for example
  `alias.*`), distinct from a bare `*`, with a deterministic column order.
- Explicit result descriptors for each member kind: presence for an optional member,
  a list for a many-valued member, and a representation for a structured member
  chosen by the binding and stated in the artifact.
- Presence semantics stated once: an absent optional member and an explicit null are
  different, and the descriptor says which the backend can distinguish.

### 2. Bounded, deterministic results

Interfaces that serve agents and screens must never return an unbounded result.
Proposed:

- `ORDER BY` on columns of the key, ascending, and `LIMIT n`, with the order made
  deterministic by the key so paging is stable.
- Ordered comparison (`>`) between a key column and a literal of the same scalar
  family, with the backend's collation or numeric order stated in the capability
  profile, so a host can continue a page from the last key (keyset paging) without
  an offset.
- A refusal, not a silent cut, when a backend cannot honor the limit and order.

### 3. Counting

Summaries by a property are a common read. Proposed `COUNT(*)` with `GROUP BY`,
with an exact integer result type and a stated empty-group behavior, alongside
`SUM`.

### 4. Relationships as queryable

An entity is opened together with what it relates to, and found by what it relates
to. Proposed, in whatever syntax the owner prefers:

- Filtering a source by its relationship to another entity identified by key.
- Projecting the keys of the entities on the other end of a named relationship,
  bounded, with a truncation marker.
- Using a relationship's inverse name from the other side.

The existing equijoin may already express part of this where a backend binds the
relationship to key columns; the requirement is the outcome, not a syntax.

### 5. Literals and parameters

A host that accepts query text from an untrusted caller, such as a model, must not
build backend statements from the caller's text. v0.1 accepts literals only and no
parameter markers. Proposed: either typed parameter markers in the source that the
host binds, or a normative statement of the literal grammar's exactness and quoting
that a host can rely on to extract typed values, with fixtures.

### 6. A recognizable subset before a compiler

A host may serve a fixed subset of Weft SQL before it embeds a compiler. Proposed:
publish the v0.1 grammar's normative subsets and fixtures so that such a host can
state exactly which weft-valid queries it accepts, refuse the rest with the construct
named, and replace its recognizer with the compiler without changing what it accepts.

### 7. Backend notes

A Truss backend would need key lookups, equality filters over property maps and a
bounded ordered scan; an Ashlar backend would need the same over gold tables with
published feed position reported by the host. Neither changes the frontend, per the
existing backend interface.

## Open

- Whether wildcard projection belongs in the dialect or in a separate entity-read
  contract that returns a typed whole record.
- How a many-valued or structured member's representation is declared in a binding.
- Which of these the owner considers in scope for a version after 0.1.
