# Generated ordinary Page `/6` native output correction evidence

**Class:** issuer unit-test, schema, and source-admission evidence only. The char
vector is helper/schema evidence, not a public eligible `/6` artifact. This is
not a wheel, consumer, host, installed runtime, or release qualification.

## Exact native output types

The closed `/6` root output schema now relates native type, Weft carrier, and
OID exactly, including `char` + `string` + 1042 and `bytea` + `bits128` + 17.
The genuine issued root vector `issued-v6-bytea.json` proves a Bits128 Field
can be joined from source Field to native binding and emitted with `bytea` OID
17. The targeted test also exercises the production metadata helper for a
synthetic `char` Field and confirms OID 1042 is retained. Ajv validates the
genuine bytea root and the separately labeled helper output; four native
type/carrier/OID drift negatives refuse in `schema-validation-retry.log`. The
initial incorrect drift probe is retained in `schema-validation.log`.

The fixture combining authored char storage fails before Page issuance with the
existing lifecycle admission error `native column type/mapping/availability
differs`. That profile currently accepts text/varchar for string columns. This
evidence does not claim authored char source support. A char primary Key also
remains unsupported; the focused test checks the issuer key-type predicate. No
lifecycle semantics were broadened.

`issuer-test.log` records the focused Rust test
`v6_emits_bytea_public_metadata_and_closes_char_helper_but_upstream_refuses_authored_char`.
PostgreSQL native decoding and runtime OID guards are downstream qualification
work and are not claimed here. The preceding correction evidence remains
unchanged in `generated-ordinary-page-v6-correction-20261105/`.
