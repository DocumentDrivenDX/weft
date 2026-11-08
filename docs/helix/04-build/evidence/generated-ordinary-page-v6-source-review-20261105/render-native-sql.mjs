import fs from 'node:fs';
const dir = 'docs/helix/04-build/evidence/generated-ordinary-page-v6-source-review-20261105/';
const normal = JSON.parse(fs.readFileSync(dir + 'issued-v6.json', 'utf8'));
const q = JSON.parse(fs.readFileSync(dir + 'issued-v6-escaped-identifiers.json', 'utf8'));
function rawPage(artifact, after, size, projection) {
  const root = artifact.roots['demo/Note'];
  const page = root.ordinaryPage.page;
  const source = page.sourceSqlTemplate
    .replace('{{FILTER}}', 'TRUE')
    .replace('{{PUBLIC_PROJECTION}}', projection)
    .replace('$1', after === null ? 'NULL' : `'${after}'`)
    .replace('$2', after === null ? 'NULL' : `'${after}'`)
    .replace('$3', String(size));
  return root.nativeReadGuard.template.sql.replace('{{SOURCE_SQL}}', source).replace('{{RETURN_COLUMNS}}', 'f.*');
}
function dbapiPage(artifact, projection) {
  const root = artifact.roots['demo/Note'];
  let sql = root.ordinaryPage.page.sqlTemplate
    .replace('{{FILTER}}', 'TRUE')
    .replace('{{PUBLIC_PROJECTION}}', projection)
    .replaceAll('%%', '%');
  for (const value of ['NULL', 'NULL', '2']) sql = sql.replace('%s', value);
  return sql;
}
function precondition(artifact, table, column) {
  return artifact.roots['demo/Note'].ordinaryPage.nativeCollationPrecondition.sourceSqlTemplate
    .replace('$1', "'plugin_demo'")
    .replace('$2', `'${table.replaceAll("'", "''")}'`)
    .replace('$3', `'${column.replaceAll("'", "''")}'`);
}
const publicProjection = ', "r"."owner" COLLATE pg_catalog."C" AS "__weft_field_1"';
const page1 = rawPage(normal, null, 2, publicProjection);
const page2 = rawPage(normal, 'b', 2, publicProjection);
const emptyProjection = rawPage(normal, null, 2, '');
const quotedSql = dbapiPage(q, publicProjection);
fs.writeFileSync(dir + 'native-page.sql', `\\set ON_ERROR_STOP on
CREATE SCHEMA plugin_demo;
CREATE TABLE plugin_demo.notes (id text COLLATE pg_catalog."C" PRIMARY KEY, owner text COLLATE pg_catalog."C" NOT NULL);
INSERT INTO plugin_demo.notes VALUES ('a','A'),('b','B'),('c','C'),('d','D');
${precondition(normal, 'notes', 'id')};
${page1};
${page2};
${emptyProjection};
CREATE TABLE plugin_demo."note%""s" ("key%""id" text COLLATE pg_catalog."C" PRIMARY KEY, owner text COLLATE pg_catalog."C" NOT NULL);
INSERT INTO plugin_demo."note%""s" VALUES ('a','A'),('b','B'),('c','C');
${precondition(q, 'note%"s', 'key%"id')};
${quotedSql};
`);
const oversize = `\\set ON_ERROR_STOP on
INSERT INTO plugin_demo.notes VALUES ('z', repeat('x', 8388609));
DO $test$ BEGIN BEGIN ${rawPage(normal, null, 10, publicProjection)}; RAISE EXCEPTION 'byte guard failed to stop oversized result'; EXCEPTION WHEN division_by_zero THEN RAISE NOTICE 'byte guard raised SQLSTATE 22012'; END; END $test$;
`;
fs.writeFileSync(dir + 'native-guard.sql', oversize);
