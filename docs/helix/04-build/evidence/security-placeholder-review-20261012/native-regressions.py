"""Native PostgreSQL parameter-boundary evidence; no application/security behavior claims."""
from pathlib import Path
from datetime import datetime, timezone
import hashlib
import importlib.metadata
import json
from truss.local_runtime import LocalPostgres
from truss._runtime_profile import RUNTIME_DISTRIBUTION
import psycopg
from psycopg import sql as pgsql

ROOT = Path('/private/tmp/weft-tokenizer-review-probe')
CASES = [
    ('baseline-one', 'SELECT $1'),
    ('repeated-one', 'SELECT $1, $1'),
    ('operator-block-comment-hides-slot', 'SELECT 1%/*$1*/1'),
    ('operator-line-comment-hides-slot', 'SELECT 1%-- $1\n1'),
    ('operator-nested-comment-hides-slot', 'SELECT 1%/* outer /*$1*/ end */1'),
    ('operator-comment-retains-real-slot', 'SELECT 1%/*$2*/1, $1'),
    ('operator-or-comment-hides-slot', 'SELECT 1|/*$1*/1'),
    ('operator-minus-comment-hides-slot', 'SELECT 1-/*$1*/1'),
    ('operator-block-comment-unterminated', 'SELECT 1%/*$1'),
    ('operator-line-comment-cr', 'SELECT 1%-- $2\r1, $1'),
    ('line-comment-eof', 'SELECT $1 -- $2'),
    ('ordinary-operators', 'SELECT 7%3, 1|2, 1#2, ~1, 1+2, 4/2, $1'),
    ('nested-comments', 'SELECT $1 /* $2 /* $3 */ $4 */'),
    ('unterminated-nested-comment', 'SELECT $1 /* $2 /* $3 */'),
    ('escape-string-continuation-hides-slot', "SELECT E'foo'\n'\\' $1 --'"),
    ('escape-string-continuation-cr', "SELECT E'foo'\r'\\' $1 --'"),
    ('escape-string-continuation-line-comment', "SELECT E'foo' -- $2\n'\\' $1 --'"),
    ('escape-string-quote', "SELECT E'foo\\' $2', $1"),
    ('escape-string-double-quote', "SELECT E'foo'' $2', $1"),
    ('escape-string-literal-backslashes', "SELECT E'foo\\\\', $1"),
    ('escape-string-invalid-unicode', "SELECT $1, E'\\uZZZZ'"),
    ('escape-string-unterminated', "SELECT $1, E'foo\\'"),
    ('standard-string-backslash', "SELECT '\\', $1"),
    ('standard-string-doubled-quote', "SELECT 'it''s $2', $1"),
    ('standard-string-continuation', "SELECT 'foo'\n'$2', $1"),
    ('standard-string-unterminated', "SELECT $1, 'unterminated"),
    ('parameter-ascii-junk', 'SELECT $1abc'),
    ('parameter-unicode-junk', 'SELECT $1😀'),
    ('parameter-underscore-junk', 'SELECT $1_'),
    ('parameter-leading-zero-native-acceptance', 'SELECT $01'),
    ('parameter-zero', 'SELECT $0'),
    ('invalid-numeric-dollar-tag', 'SELECT $1, $2$abc$2$'),
    ('invalid-zero-dollar-tag', 'SELECT $1, $0$abc$0$'),
    ('invalid-leading-zero-dollar-tag', 'SELECT $1, $01$abc$01$'),
    ('dollar-quote-empty-tag', 'SELECT $1, $$ $2 $$'),
    ('dollar-quote-ascii-tag', 'SELECT $1, $_tag1$hello $2$_tag1$'),
    ('dollar-quote-unicode-tag', 'SELECT $1, $😀$hello $2$😀$'),
    ('dollar-quote-combining-tag', 'SELECT $1, $\u0301$hello $2$\u0301$'),
    ('dollar-quote-mismatched-tag', 'SELECT $1, $a$hello $2$A$'),
    ('identifier-dollar-suffix', 'SELECT 1 AS x$1'),
    ('identifier-unicode-dollar-suffix', 'SELECT 1 AS 😀$1'),
    ('quoted-identifier', 'SELECT $1 AS "$2"'),
    ('quoted-identifier-doubled-quote', 'SELECT $1 AS "a""$2"'),
    ('quoted-identifier-unterminated', 'SELECT $1 AS "$2'),
    ('unicode-quoted-identifier', 'SELECT $1 AS U&"$2"'),
    ('unicode-string-default-escape', "SELECT $1, U&'d\\0061t $2'"),
    ('unicode-string-custom-escape', "SELECT $1, U&'hello\\world $2' UESCAPE '!'"),
    ('unicode-quoted-identifier-custom-escape', 'SELECT $1 AS U&"hello\\world $2" UESCAPE \'!\''),
    ('unicode-string-custom-quote-escape', "SELECT $1, U&'it''s !0061 $2' UESCAPE '!'"),
    ('unicode-string-invalid-default-escape', "SELECT $1, U&'hello\\world'"),
]
report = {
    'kind': 'weft.postgresql-placeholder-native-lexical-evidence/0.1.0',
    'generatedAt': datetime.now(timezone.utc).isoformat(),
    'purpose': 'Independent native parameter inventory and lexical boundary regression observations.',
    'method': 'PREPARE without supplied parameter types; inspect pg_prepared_statements; EXECUTE accepted cases using a text sentinel for each inferred parameter.',
    'limits': ['Finite lexical regression cases only; no application query equivalence or security lowering qualification.', 'Native preparation also parses statements; errors retain native SQLSTATE and message rather than being classified as all scanner errors.', 'Native parameter inventory reflects prepared parameter type array length; queries intentionally avoid gaps between used parameter positions.', 'The project profile independently forbids noncanonical and zero ordinals even where native PostgreSQL accepts leading zeros.'],
    'sourceReference': 'https://github.com/postgres/postgres/blob/REL_18_STABLE/src/backend/parser/scan.l',
    'cases': [],
}
with LocalPostgres(ROOT / 'pgdata-regressions') as pg:
    with psycopg.connect(pg.info.connection_uri, autocommit=True) as conn:
        executable = pg.psql_path.parent / 'postgres'
        report['engine'] = {
            'name': 'PostgreSQL', 'version': pg.info.server_version,
            'serverVersionNum': int(conn.execute('SHOW server_version_num').fetchone()[0]),
            'versionDescription': conn.execute('SELECT version()').fetchone()[0],
            'binarySha256': hashlib.sha256(executable.read_bytes()).hexdigest(),
            'runtimeDistribution': RUNTIME_DISTRIBUTION, 'runtimeDistributionVersion': importlib.metadata.version(RUNTIME_DISTRIBUTION),
            'driverDistribution': 'psycopg', 'driverVersion': importlib.metadata.version('psycopg'),
        }
        report['sessionSettings'] = {key: conn.execute(f'SHOW {key}').fetchone()[0] for key in ['standard_conforming_strings','backslash_quote','server_encoding','client_encoding']}
        for index, (case_id, statement) in enumerate(CASES):
            entry = {'id': case_id, 'sql': statement}
            name = f'lexical_{index}'
            prepare = f'PREPARE {name} AS {statement}'
            try:
                conn.execute(prepare)
                count, types = conn.execute('SELECT cardinality(parameter_types), parameter_types::text FROM pg_prepared_statements WHERE name=%s', (name,)).fetchone()
                entry['prepare'] = {'status': 'accepted', 'parameterCount': count, 'parameterTypes': types}
                args = [pgsql.Literal('native-sentinel') for _ in range(count)]
                execute = pgsql.SQL('EXECUTE {}').format(pgsql.Identifier(name))
                if args:
                    execute += pgsql.SQL('({})').format(pgsql.SQL(', ').join(args))
                cursor = conn.execute(execute)
                entry['execute'] = {'status': 'accepted', 'rows': cursor.fetchall(), 'columnNames': [col.name for col in cursor.description]}
            except psycopg.Error as error:
                entry['prepare' if 'prepare' not in entry else 'execute'] = {
                    'status': 'rejected', 'sqlstate': error.sqlstate,
                    'message': error.diag.message_primary,
                    'statementPosition': error.diag.statement_position,
                    'context': error.diag.context,
                }
            report['cases'].append(entry)
report['summary'] = {
    'caseCount': len(report['cases']),
    'prepared': sum(c['prepare']['status'] == 'accepted' for c in report['cases']),
    'refused': sum(c['prepare']['status'] == 'rejected' for c in report['cases']),
}
print(json.dumps(report, ensure_ascii=False, indent=2))
