# Ordinary Page `/6` varchar schema correction

The accepted, installed Weft `0.4.65.dev1` compiler wheel emits an eligible
ordinary Page for a derived `varchar(32)` Key. The source request is the
retained `request-v6.json` from the `/6` wheel evidence. The check changes only
the authored Key column's `nativeType`, recomputes the exact model digest, and
calls the installed compiler. No database or runtime is involved.

The earlier descriptor schema (SHA-256
`aedbdebdcff598bbc285f6eb207b0fe961fc871ffc6629dfb4f8b20154c3266d`)
rejected the wheel's preserved `{kind:"varchar",length:32}` in both
`key.output` and `page.hiddenKeyOutput`. UMF CONTRACT-081 requires a positive
integer length for varchar. The current descriptor schema requires that facet
and pairs `text` with OID 25 and `varchar` with OID 1043.

From the Weft repository root, run
`node docs/helix/04-build/evidence/generated-ordinary-page-v6-varchar-schema-20261105/validate.mjs`.
Set `WEFT_V6_PYTHON` to an installed `0.4.65.dev1` wheel's Python interpreter
when the recorded local venv is unavailable. The check validates the actual
compiler output against the complete `/6` schema and refuses omitted or invalid
length, `char`, and swapped key OIDs.

This correction changes the descriptor schema only. The historical wheel,
source inventory, source hashes, and original schema-validation receipt remain
unchanged. It does not establish native execution, host admission, or deployment
qualification.
