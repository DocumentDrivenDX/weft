---
ddx:
  id: weft.typed-sdk-raw-terms
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
---

# Source-bound raw term filters

**Contract ID:** weft.typed-sdk-raw-terms. **Type:** library/host obligations.
**Version:** `umf.raw-term-filter/0.1`. **Status:** implementation candidate.

## Purpose

Define the WR-A11/S14 unranked predicate boundary between Weft compilation and
the PostgreSQL18/Lakebase18 raw-read host. Compilation grants no native authority.

## Scope and Boundaries

The closed logical forms are `MATCHES_ALL_TERMS(field, value)` and
`MATCHES_ANY_TERM(field, value)`, where field is an explicitly aliased scalar
string Field and value is a string literal or declared string parameter.
Parentheses and existing boolean/group guards retain their meaning. A selected
Field MUST have exactly one original single-field exact keyword index. The
original pinned core0.8 source MUST declare a concrete `simple`/nonstemming or
`english`/stemming analyzer and plain syntax. The existing supported PostgreSQL
keyword ranking profile is implementation-defined; no BM25 ranking support is
implied by these boolean predicates. Its target binding MUST match PostgreSQL18
or Lakebase18, be transactional/complete, and name the original native index.
Missing/ambiguous sources, unsupported selected content and inferred analyzers
MUST refuse. Ordinary ranked-search admission remains unchanged.

## Normative Surface

The compiler MUST retain complete original source, index, target binding and raw
program custody under the covering digest. Query and field are tokenized by the
same declared native analyzer, as plain text without search operators. ALL is
query-token containment; ANY is token intersection. Empty query terms yield TRUE
for ALL and FALSE for ANY on non-null text; null text/query yields SQL UNKNOWN.
No ranking, approximate matching, embedding, normalization substitution or local
analyzer is introduced.

A compiled raw program with terms MUST include `obligations.termFilter` exactly
`{version:"umf.raw-term-filter/0.1",nativeCatalogBeforeAfter:true,indexes:[...]}`.
The deterministic nonempty index list contains exactly
`{source,index,nativeBinding,logicalTable,nativeTable,nativeColumn,configuration,nativeAdmission}`.
Source/index/binding retain their full original selected values. Configuration
is `pg_catalog.simple` or `pg_catalog.english`.

Native admission is exactly `{sql,dbapiSql,dbapiParameterOrder,parameters}`:

- `sql` is the complete positional PostgreSQL statement, retained for owner
  correspondence; `dbapiSql` is its complete compiler-owned named DBAPI form.
- Named parameters are `__umf_term_1` through `__umf_term_5` in that order.
  `dbapiParameterOrder` lists those exact names and `parameters` has matching
  `{name,source,carrier:"string"}` entries, with no duplicates/unknown members.
- Sources in order are current owned `{appSchema:true}` (or authored
  `{literal:{string:schema}}`), then literal native index, table, analyzer language
  and column. Every literal MUST be retained exactly. Hosts MUST NOT infer names.
- DBAPI text uses `%(__umf_term_N)s`; its literal PostgreSQL format `%L`/`%I`
  escapes are already doubled. Hosts MUST execute that statement unchanged and
  bind a mapping from the original five sources; they MUST NOT lower/rewrite SQL.

The host MUST execute each original native admission statement inside its owned
caller-role interval before the verified/user query and after all rows are
buffered but before release. Each execution MUST return exactly one row with one
value whose native driver type is Boolean and whose value is TRUE. Missing,
extra, NULL, integer1 or FALSE results MUST refuse. The program checks the valid,
ready/live, unpredicated single-expression GIN index and exact original native
`to_tsvector` expression, plus the complete default parser/dictionary mapping for
the selected built-in analyzer. The candidate pins nineteen mapped token kinds,
sequence1, pg_catalog dictionary/template names and initialization options;
English word/hyphen-word tokens use english_stem/snowball with English stopwords,
other mapped tokens use the original simple dictionary. Unknown native catalog
content MUST NOT be accepted merely because before/after values agree.

The original table/source/role/Key/timeout/read-only obligations remain mandatory.
A host without this handler MUST refuse the term-bearing program. Zero returned
user rows do not waive native admission. No database error may be retried or
translated into an empty successful result.

## Precedence and Compatibility

This is an additive obligation on `weft.raw-read/0.1`, not a replacement for its
complete original program. Programs without selected term predicates retain their
original obligation inventory. Native obligations take precedence over compiler
success. Altering token/analyzer or source/native correspondence meaning requires
a new qualified profile. Temporal qualification remains HELD.

## Error Semantics

Source, analyzer, target, program or native correspondence drift refuses before
release. Native errors propagate through the existing operation/cleanup handling;
there is no implicit retry. Previously buffered rows MUST NOT escape a failed
closing admission. Native Lakebase execution, platform dictionary behavior and
managed engines require separate evidence.

## Examples

`MATCHES_ALL_TERMS(d.body, :terms)` emits same-analyzer token containment and the
original keyword-index obligation. A query containing `OR`, `%` or quotes remains
plain text. `MATCHES_ANY_TERM(d.body, '')` matches no non-null text. A Field with
no selected keyword index, an implementation-defined analyzer, a changed GIN
expression or a changed English dictionary mapping refuses.
