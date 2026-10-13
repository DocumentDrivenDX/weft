# SQL page resolver source increment — 2026-10-13

The owner compiler now resolves a single-source SQL page against the document’s actual primary key. It adds complete required-string key order, hidden key outputs, a strict continuation predicate and one-row lookahead. It refuses caller ordering/limits, aggregates, joins and unsupported key domains. Existing resolution refuses relationship page predicates.

A focused Rust test uses the unchanged original commerce 0.8 owner document, projects a non-key field and verifies both the hidden primary key and the next-page comparison. It passes (one test). The Python-feature Cargo check passes. Module boundary checking passes after declaring the added public source types.

Astra Ultra reviewed the increment and approved it as unactivated source. Public Python page compilation and the original Truss consumer paging workflow remain pending. Generated AST spans are diagnostic anchors only: before public emission, continuation parameter origins must be distinguished explicitly from authored SQL literal origins. This increment does not claim public paging support or native accepted-catalog evidence.

The existing supported Truss backend remains 0.4.6, runtime 0.33 candidate, PostgreSQL 18.6 through DocumentDrivenDX pgserver-pg18 18.6.1 and psycopg 3.3.6.

## Public composition completed

The runtime and Python owner page entrypoints now wrap an unchanged validated compile artifact, retain explicit generated-node provenance, and assign continuation parameter origins by filter/component occurrence before emission validation. Separate continuation bytes are bounded. The Truss original consumer page workflow passed installed bindings on DocumentDrivenDX PostgreSQL18.6, then all six original read variants passed together (10.790 seconds). Astra Ultra approved the focused source subject to that unchanged workflow; it now passes. This does not establish broad page domains, replicas or full corpus interchange.
