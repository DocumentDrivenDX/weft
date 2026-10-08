"""Execute captured owner-wide row-home structural count queries; component only."""
import csv, hashlib, io, json, subprocess, sys
from pathlib import Path
capture_path = Path(sys.argv[1])
captures = json.loads(capture_path.read_text())
assert [c['kind'] for c in captures] == ['object', 'edge']
scenarios = [('absent', True, 1), ('valid', True, 1), ('missing-root', False, 1),
             ('duplicate-state', False, 2), ('duplicate-root', False, 2),
             ('duplicate-scalar', False, 2), ('parented-root', False, 1),
             ('null-state-id', False, 1), ('wrong-owner', True, 1),
             ('compound-root', True, 1)]
statements = ['BEGIN;', 'CREATE SCHEMA "schema.with.dot";', '''
CREATE TABLE "schema.with.dot".object(id bigint,type_id int);
CREATE TABLE "schema.with.dot".edge(id bigint,rel_type_id int);
INSERT INTO "schema.with.dot".object VALUES (987654321,-1),(123,-2);
INSERT INTO "schema.with.dot".edge VALUES (987654321,-1),(123,-2);
CREATE TABLE "schema.with.dot".row_home_state
(state_id bigint, owner_kind text, object_id bigint, object_type_id int,
edge_id bigint, relationship_type_id int, property_owner_type_id int,
property_id int, root_node_id bigint);
CREATE TABLE "schema.with.dot".row_home_node
(state_id bigint, node_id bigint, parent_node_id bigint);
CREATE TABLE "schema.with.dot".row_home_scalar(state_id bigint, node_id bigint);
''']
expected = {}
for capture in captures:
    kind = capture['kind']
    query = capture['check']
    assert [p['value'] for p in capture['parameters']] == ['-1','42','-1']
    statements.append(f'PREPARE probe_{kind}(int,int,int) AS {query};')
    for label, valid, count in scenarios:
        tag = kind + ':' + label
        expected[tag] = [str(0 if valid else count)]
        statements.append('TRUNCATE "schema.with.dot".row_home_state, "schema.with.dot".row_home_node, "schema.with.dot".row_home_scalar;')
        if label != 'absent':
            owner_id = 123 if label == 'wrong-owner' else 987654321
            sid = 'NULL' if label == 'null-state-id' else '10'
            object_values = f'{owner_id},-1,NULL,NULL' if kind == 'object' else f'NULL,NULL,{owner_id},-1'
            state = f"INSERT INTO \"schema.with.dot\".row_home_state VALUES ({sid},'{kind}',{object_values},-1,42,20);"
            statements.append(state)
            if label == 'duplicate-state': statements.append(state)
            if label != 'missing-root':
                parent = '999' if label == 'parented-root' else 'NULL'
                node = f'INSERT INTO "schema.with.dot".row_home_node VALUES (10,20,{parent});'
                statements.append(node)
                if label == 'duplicate-root': statements.append(node)
            if label != 'compound-root':
                scalar = 'INSERT INTO "schema.with.dot".row_home_scalar VALUES (10,20);'
                statements.append(scalar)
                if label == 'duplicate-scalar': statements.append(scalar)
        statements.append(f"SELECT '{tag}' AS scenario;")
        statements.append(f'EXECUTE probe_{kind}(-1,42,-1);')
statements.append('ROLLBACK;')
sql = '\n'.join(statements)
result = subprocess.run(['docker', 'exec', '-i', 'weft-b005-pg17', 'psql', '-U', 'postgres', '-X', '-q', '--csv', '-v', 'ON_ERROR_STOP=1'], input=sql, text=True, capture_output=True, check=True)
observed = {}; current = None
for row in csv.reader(io.StringIO(result.stdout)):
    if not row or row[0] in ('scenario','violations'): continue
    if row[0] in expected: current = row[0]; observed[current] = []
    else:
        assert current is not None
        observed[current].append(row[0])
assert observed == expected, (observed, expected)
version = subprocess.run(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-At','-c','SELECT version()'],text=True,capture_output=True,check=True).stdout.strip()
report = {'server': version, 'captureSha256': hashlib.sha256(capture_path.read_bytes()).hexdigest(), 'sqlSha256': hashlib.sha256(sql.encode()).hexdigest(), 'harnessSha256': hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), 'scope':'Rust-emitted owner-wide row-home structural count queries on synthetic unconstrained tables; not row codec, host visibility, logical edge selection or native Truss profile qualification', 'scenarios':len(expected), 'observations':observed}
Path(sys.argv[2] if len(sys.argv)>2 else '/private/tmp/weft-row-scan-native-report.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'scenarios':len(expected),'passed':True}))
