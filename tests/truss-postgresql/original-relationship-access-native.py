"""Native directed-edge access primitive, prior to full relationship lowering."""
import csv,hashlib,io,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
path=ROOT/'tests/truss-postgresql/fixtures/original-relationship-access.json';raw=path.read_bytes();captures=json.loads(raw);results=[]
for e in captures:
    for case in ['empty','scoped-directed']:
        sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int); CREATE TEMP TABLE edge(rel_type_id int,source_id bigint,source_type int,target_id bigint,target_type int);\n'
        sql+='INSERT INTO object VALUES (1,-1),(1,-999),(2,-2),(2,-999),(3,-1);\n'
        if case!='empty':
            sql+='INSERT INTO edge VALUES (0,1,-1,2,-2),(0,3,-1,2,-2),(1,1,-1,2,-2),(0,1,-999,2,-2),(0,1,-1,2,-999),(0,2,-2,1,-1);\n'
        args=','.join("'"+p['value']+"'" for p in e['parameters']);sql+=f"PREPARE q(int4,int4,int4) AS {e['sql']}; EXECUTE q({args}); ROLLBACK;\n"
        out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
        rows=list(csv.reader(io.StringIO(out)));expected=[] if case=='empty' else ([['2','1'],['2','3']] if e['inverse'] else [['1','2'],['3','2']])
        assert rows==[['owner','target']]+expected,(case,e['inverse'],rows)
        results.append(dict(case=case,inverse=e['inverse'],rows=expected,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original admitted directed physical edge access only; full HAS_RELATED/RELATED_KEYS, target key projection, multiplicity and host prerequisites remain unqualified',server=server,captureSha256=hashlib.sha256(raw).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-original-relationship-access-native.json').write_text(json.dumps(receipt,indent=2)+'\n')
print('4 original directed edge native access cases passed')
