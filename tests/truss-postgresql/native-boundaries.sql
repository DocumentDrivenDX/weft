-- Independent PostgreSQL meaning probes; deliberately no application objects.
-- Run with psql -X -A -t. Output exactly one JSON text row.
WITH numbers(v) AS (
  VALUES ('{"p":9223372036854775807}'::jsonb),
         ('{"p":9223372036854775807}'::jsonb),
         ('{"p":0.0000000000000000000000000001}'::jsonb)
), strings(v) AS (
  VALUES ('a'::text), ('a '::text), ('é'::text), ('😀'::text)
), cursors(v) AS (VALUES ('2'::text), ('10'::text), ('-1'::text)),
states(v) AS (VALUES ('{}'::jsonb), ('{"p":null}'::jsonb), ('{"p":[]}'::jsonb))
SELECT jsonb_build_object(
  'serverVersion', current_setting('server_version'),
  'serverEncoding', current_setting('server_encoding'),
  'exactSum', (SELECT sum((v->>'p')::numeric)::text FROM numbers),
  'stringOrder', (SELECT jsonb_agg(v ORDER BY v COLLATE "C") FROM strings),
  'trailingSpaceDistinct', 'a'::text COLLATE "C" <> 'a '::text COLLATE "C",
  'numericOrder', (SELECT jsonb_agg(v ORDER BY v::numeric) FROM cursors),
  'lexicalOrder', (SELECT jsonb_agg(v ORDER BY v COLLATE "C") FROM cursors),
  'states', (SELECT jsonb_agg(jsonb_build_object('present',v ? 'p','kind',jsonb_typeof(v->'p'))) FROM states),
  'emptyCount', (SELECT count(*)::text FROM numbers WHERE false),
  'emptySumIsNull', (SELECT sum((v->>'p')::numeric) IS NULL FROM numbers WHERE false)
)::text;
