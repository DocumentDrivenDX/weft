"""Owned PG prototype: analyze original DML without firing its rule."""
import hashlib,json,os,subprocess,sys
from pathlib import Path
import psycopg
out=Path(sys.argv[1]);out.mkdir();(out/'socket').mkdir()
pg=Path('/opt/homebrew/opt/postgresql@18/bin');started=False;checks=[]
def run(label,args):
 with (out/(label+'.log')).open('w') as f:
  p=subprocess.run(list(map(str,args)),stdout=f,stderr=subprocess.STDOUT)
 if p.returncode:raise RuntimeError(label)
def check(name,value):
 assert value,name
 checks.append({'name':name,'passed':True})
try:
 run('init',[pg/'initdb','-D',out/'data','-A','trust','-U','prototype_admin','--no-sync'])
 run('start',[pg/'pg_ctl','-D',out/'data','-l',out/'server.log','-o',f"-F -k {out/'socket'} -c listen_addresses='' -p 55511",'-w','start']);started=True
 dsn=f"host={out/'socket'} port=55511 dbname=postgres"
 with psycopg.connect(dsn,user='prototype_admin',autocommit=True) as c:
  c.execute('CREATE ROLE prototype_owner LOGIN NOSUPERUSER NOBYPASSRLS')
  c.execute('GRANT CREATE ON DATABASE postgres TO prototype_owner')
 with psycopg.connect(dsn,user='prototype_owner',autocommit=True) as c:
  c.execute('CREATE SCHEMA prototype_owner AUTHORIZATION prototype_owner')
  c.execute('SET search_path=prototype_owner,pg_catalog')
  c.execute('CREATE TABLE orders(id text PRIMARY KEY,status text NOT NULL)')
  c.execute("INSERT INTO orders VALUES('a','old')")
  original=json.load(open('/private/tmp/weft-action-original.json'))
  sql=original['effects'][0]['statement']['sql']
  check('actual-original-update-returning',sql.startswith('UPDATE ') and 'RETURNING' in sql)
  typed=sql.replace('$1','(SELECT p1 FROM pg_temp.__weft_privilege_inputs)').replace('$2','(SELECT p2 FROM pg_temp.__weft_privilege_inputs)')
  (out/'original-program.json').write_text(json.dumps(original,indent=2))
  (out/'original-dml.sql').write_text(sql)
  with c.transaction():
   c.execute('CREATE TEMP TABLE __weft_privilege_inputs(p1 text,p2 text) ON COMMIT DROP')
   c.execute('CREATE TEMP TABLE __weft_privilege_event AS SELECT id FROM orders WITH NO DATA')
   rule='CREATE RULE __weft_privilege_rule AS ON INSERT TO pg_temp.__weft_privilege_event DO INSTEAD ('+typed+')'
   (out/'rule.sql').write_text(rule)
   c.execute(rule)
   tree=c.execute("SELECT ev_action::text FROM pg_catalog.pg_rewrite WHERE rulename='__weft_privilege_rule' AND ev_class='pg_temp.__weft_privilege_event'::regclass").fetchone()[0]
   (out/'analyzed-tree.txt').write_text(tree)
   check('native-tree-retains-update',':commandType 2' in tree)
   check('native-tree-retains-returning',':returningList (' in tree)
   check('native-tree-retains-typed-var-subqueries',':subselect {QUERY' in tree and ':vartype 25' in tree)
   check('original-business-row-unchanged',c.execute('TABLE orders').fetchall()==[('a','old')])
   check('event-and-input-empty',c.execute('SELECT (SELECT count(*) FROM pg_temp.__weft_privilege_event),(SELECT count(*) FROM pg_temp.__weft_privilege_inputs)').fetchone()==(0,0))
   check('nonsuper-nobypass',c.execute('SELECT rolsuper,rolbypassrls FROM pg_catalog.pg_roles WHERE rolname=current_user').fetchone()==(False,False))
  check('business-still-unchanged-after-rule-setup',c.execute('TABLE orders').fetchall()==[('a','old')])
finally:
 if started:run('stop',[pg/'pg_ctl','-D',out/'data','-m','fast','-w','stop'])
 (out/'result.json').write_text(json.dumps({'checks':checks,'serverStopped':not(out/'data/postmaster.pid').exists(),'temporal':'HELD','sourceOnlyPrototype':True,'enrolled':False},indent=2)+'\n')
print(json.dumps(checks))
