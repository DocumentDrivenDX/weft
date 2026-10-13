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

ROOT = Path('/private/tmp/weft-pg-candidate-native-20261012')
CASES = [('undeclared-ordinal', 'SELECT $1'), ('zero-ordinal', 'SELECT $0'), ('noncanonical-ordinal', 'SELECT $01'), ('ascii-junk', 'SELECT $1abc'), ('unicode-junk', 'SELECT $1é'), ('string-hides-slot', "SELECT '$1'"), ('comment-hides-slot', 'SELECT 1 /* $1 */'), ('dollar-string-hides-slot', 'SELECT $$ $1 $$'), ('baseline', 'SELECT 1')]
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
