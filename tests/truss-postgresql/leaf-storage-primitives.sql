-- Physical extraction primitive probe; not an adopted codec/profile qualification.
PREPARE leaf_storage(text) AS
SELECT label,
CASE WHEN pg_catalog.jsonb_typeof(owner.props)='object' THEN owner.props ? $1::pg_catalog.text ELSE NULL END AS present,
pg_catalog.jsonb_typeof(owner.props -> $1::pg_catalog.text) AS kind,
(owner.props ->> $1::pg_catalog.text) AS token,
CASE WHEN pg_catalog.jsonb_typeof(owner.props -> $1::pg_catalog.text)='boolean' THEN (owner.props ->> $1::pg_catalog.text)::pg_catalog.bool ELSE NULL END AS boolean_carrier
FROM (VALUES
('unicode', '{"member":"é😀  "}'::jsonb),
('boolean', '{"member":false}'::jsonb),
('integer', '{"member":"18446744073709551615"}'::jsonb),
('decimal', '{"member":"-0.00"}'::jsonb),
('wrong-number', '{"member":9007199254740993}'::jsonb),
('wrong-boolean', '{"member":"not a boolean"}'::jsonb),
('absent', '{}'::jsonb),
('null', '{"member":null}'::jsonb),
('array-root', '[]'::jsonb),
('sql-null-root', NULL::jsonb)
) AS owner(label, props);
EXECUTE leaf_storage('member');
