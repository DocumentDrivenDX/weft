# Ordinary Page v6 schema omission correction — 2026-11-05

Sol found that the inherited output schema permits nativeType and oid to be omitted. Page fieldOutputs now require both, in addition to the existing closed kind/carrier/OID checks. The generic inherited root output schema is unchanged.

The retained command validates the genuine emitted bytea root from correction-2 and the explicitly synthetic char helper output. It rejects twelve omission mutations: deleting nativeType, oid or both from each of three public Field outputs and from the synthetic helper output. Raw stdout, command, tooling versions and hashes are retained here. The earlier value-pair, issuer, SQL and upstream char-refusal evidence retain their original scope.

This is a schema-only correction. No issuer bytes, table lifecycle semantics, runtime consumer, continuation resume behavior, wheel or ABI freeze is established by this test.
