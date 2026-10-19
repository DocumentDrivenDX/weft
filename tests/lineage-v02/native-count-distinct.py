# SPDX-License-Identifier: MIT OR Apache-2.0
"""PG18 count primitives over VALUES only; no compiler/native security qualification."""
import argparse,hashlib,json,pathlib,sys
p=argparse.ArgumentParser();p.add_argument('--truss',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
sys.path.insert(0,str(a.truss/'packages/python/src'))
from truss import LocalPostgres
import psycopg
checks=[]
with LocalPostgres(a.output/'postgres') as runtime:
 with psycopg.connect(runtime.info.connection_uri,autocommit=True) as c:
  c.execute('SET default_transaction_read_only=on');version=c.execute('SHOW server_version_num').fetchone()[0];assert version=='180006',version
  cases=[
   ('empty-input','SELECT COUNT(DISTINCT v) FROM (SELECT NULL::text v WHERE FALSE) s',[],[(0,)]),
   ('all-null','SELECT COUNT(DISTINCT v) FROM (VALUES(NULL::text),(NULL::text)) s(v)',[],[(0,)]),
   ('text-duplicates-null','SELECT COUNT(DISTINCT v COLLATE "C") FROM (VALUES(%s::text),(%s::text),(%s::text),(%s::text)) s(v)',['token','token',None,'other'],[(2,)]),
   ('unicode-exact','SELECT COUNT(DISTINCT v COLLATE "C") FROM (VALUES(%s::text),(%s::text),(%s::text)) s(v)',['é','e\u0301','é'],[(2,)]),
   ('exact-decimal-equality','SELECT COUNT(DISTINCT v) FROM (VALUES(%s::numeric),(%s::numeric),(%s::numeric),(%s::numeric)) s(v)',['1.0','1e0','2.00',None],[(2,)]),
   ('boolean-duplicates-null','SELECT COUNT(DISTINCT v) FROM (VALUES(FALSE),(FALSE),(TRUE),(NULL::bool)) s(v)',[],[(2,)]),
   ('where-false-count-row','SELECT COUNT(DISTINCT v) FROM (VALUES(1),(1),(2)) s(v) WHERE FALSE',[],[(0,)]),
   ('limit-zero-removes-count-row','SELECT COUNT(DISTINCT v) FROM (VALUES(1),(1),(2)) s(v) LIMIT 0',[],[]),
   ('bag-count-is-not-distinct-count','SELECT COUNT(*),COUNT(DISTINCT v) FROM (VALUES(1),(1),(NULL::int)) s(v)',[],[(3,1)]),
   ('null-sentinel-changes-count','SELECT COUNT(DISTINCT v),COUNT(DISTINCT COALESCE(v,\'sentinel\')) FROM (VALUES(\'token\'::text),(NULL::text)) s(v)',[],[(1,2)]),
   ('lossy-comparison-changes-count','SELECT COUNT(DISTINCT v COLLATE "C"),COUNT(DISTINCT lower(v COLLATE "C")) FROM (VALUES(\'A\'::text),(\'a\'::text)) s(v)',[],[(2,1)])
  ]
  for name,sql,params,expected in cases:
   cur=c.execute(sql,params);rows=cur.fetchall();assert rows==expected,(name,rows,expected);assert all(d.type_code==20 for d in cur.description)
   checks.append({'id':name,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest(),'rows':rows,'countCarrier':'int8','passed':True})
assert runtime._server is None and runtime._lease is None
report={'scope':'Fixed PG18.6 count/NULL/equality primitives and counterexamples only; no admitted nullable/absent UMF Field, current authority, compiled SQL refinement, protected dispatch/publication or backend case qualification. Disposable empty server; original SQL contains only authored VALUES.','postgres':version,'psycopg':psycopg.__version__,'serverStopped':True,'checks':checks}
(a.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
