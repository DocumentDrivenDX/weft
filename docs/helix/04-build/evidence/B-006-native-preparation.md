# B-006: Ashlar native preparation

B-005 merged as PR #7 at `3ad557b76d6b8239d6033b993cd564bea48c2dd1`.
B-006 work continues on `codex/b006-ashlar-databricks` in the main Weft checkout.
This checkpoint establishes native prerequisites, not backend acceptance.

## Owned storage sources

Ashlar checkout inspected at `e3ab6648e0340c59a077459647eb2df11afe1a5a`.
The sibling is advancing independently; these hashes pin the consumed documents:

| Ashlar source | SHA-256 |
| --- | --- |
| CONTRACT-003-delta-graph-tables.md | ec73e77e2023d192ed3c6eb3916b395fa04f8f1695380667b206855fcf765632 |
| CONTRACT-004-publication-resolver.md | 52492ad626cf046d2ff595d5aa91c20d086561c1aba0df3b6b70338bbd08514b |
| delta-layout-v03.sql | ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e |

`ashlar-delta/0.3` remains a draft. Canonical objects and edges carry signed64
physical identities qualified by source and type; logical keys remain separate.
Property maps are exact JSON text keyed by producer property IDs. Typed serving
projections are explicit, rebuildable mappings with presence columns. Weft must
not infer a property's home from its scalar type or invent projection tables.
Parallel edges retain independent identities; ordinary Weft joins retain bags.
Ashlar's consumer graph count/distinct conventions do not replace SQL semantics.

One validated immutable publication supplies every consumed table UUID/version.
The host checks custody, policy, schema, retained files and all read dependencies;
the compiler emits exact version references and obligations without database IO.
No latest-head fallback or trust in stored validation flags is permitted.
Ashlar's UMF binding remains deferred. Candidate implementation will require
explicit supplied logical identities/mappings against these owned layouts;
production binding and runtime adoption are separate qualification work.

## Executed prerequisite evidence, 2026-10-08

Run `tests/ashlar-databricks/native-prerequisites.py` with the existing authenticated
`aidev-cus` SDK profile and `WEFT_ASHLAR_EVIDENCE_OUTPUT` set to an owned output
directory. Five read-only statements passed on existing warehouse
`2439e1f2e37ac563`; no tables or compute configuration changed.

[Summary](B-006-prerequisites/summary.json) and
[full statement responses](B-006-prerequisites/statements.jsonl) retain native
column metadata, submitted parameters, terminal outcomes and statement handles.
The engine reports `4.2.0` with a zero build hash; this alone does not identify
the Databricks warehouse release/channel or qualify a broader runtime profile.

- Exact equality distinguishes composed/decomposed Unicode and trailing spaces.
- Two maximum DECIMAL(28,2) inputs sum to exact widened decimal text.
- DECIMAL(38,0) SUM overflow fails with ARITHMETIC_OVERFLOW rather than NULL.
- A UInt64 JSON token is observed as exact text in this one probe; absent and
  JSON null both extract to SQL NULL, requiring separate presence/type guards.

These are independently authored engine probes. They do not execute emitted
compiler SQL, establish arbitrary JSON numeric lexical preservation, qualify
delegation/publication, or close US-004-AC1–AC4. Next add failing mapped corpus and
binding/refusal tests, implement the registered backend, then execute emitted SQL
with independent expected values and actual Python/browser parity.
