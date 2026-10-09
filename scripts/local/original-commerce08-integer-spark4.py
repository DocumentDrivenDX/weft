"""Original commerce0.8 unchanged compiler SQL, independent local Spark4 profile.
No production publication/profile admission; no Databricks calls.
"""
import argparse,hashlib,json,subprocess
from exact_integer_result import decode_rows
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(b):return hashlib.sha256(b).hexdigest()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def main():
 p=argparse.ArgumentParser();p.add_argument('--compiler',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--jars',type=Path,required=True);p.add_argument('--umf',type=Path,required=True);a=p.parse_args()
 if a.output.exists():raise ValueError('Fresh output required')
 from pyspark.sql import SparkSession
 import pyspark
 if pyspark.__version__!='4.0.1':raise ValueError('Explicit independent Spark4.0.1 profile required')
 model=(ROOT/'tests/fixtures/original-commerce-0.8/ontology.json').read_bytes();graph=json.loads((ROOT/'tests/fixtures/original-commerce-0.8/graph.json').read_bytes());d=json.loads(model)
 a.output.mkdir(parents=True);save(a.output/'qualification.json',{'scope':'Independent local compatibility; original0.8 source. No qualified Databricks profile, publication/authorization or ACK admission. Finite exact backend representation of original mathematical integers; no general arbitrary precision claim.','compilerSha256':sha(a.compiler.read_bytes()),'sourceSha256':sha(model)})
 pin={'documentId':d['id'],'revision':'original-1','umfVersion':'0.8.0','sha256':sha(model)}
 preflight={'interfaceVersion':'weft-compile/0.2.0','dialect':'weft-sql/0.2.0','sql':'SELECT o.quantity FROM order_lines o','modules':[{'documentJson':model.decode(),'pin':pin,'selectedModuleIds':['domain']}],'target':{'backendId':'ashlar.databricks','backendVersion':'0.1.0-candidate','targetProfile':'dbsql-candidate','bindingJson':'{"profile":"ashlar-databricks-candidate/0.1.0"}','bindingSha256':sha(b'{"profile":"ashlar-databricks-candidate/0.1.0"}')},'options':{'allowCandidate':True}}
 refusal=json.loads(subprocess.run([str(a.compiler)],input=json.dumps(preflight),text=True,capture_output=True,check=True).stdout)
 if refusal['status']!='blocked'or refusal['diagnostics'][0]['code']!='WFT-CAPABILITY':raise ValueError('Unbounded integer must refuse before local runtime acquisition')
 save(a.output/'integer-refusal-request.json',preflight);save(a.output/'integer-refusal-artifact.json',refusal)
 subprocess.run(['bun',str(ROOT/'scripts/local/original-commerce08-values.ts'),str(a.umf),str(a.output/'public-umf-values.json')],check=True)
 source_receipt=json.loads((a.output/'public-umf-values.json').read_bytes())
 if source_receipt['sourceText']!=model.decode()or[o['original']for o in source_receipt['records']]!=graph['objects']:raise ValueError('Original public UMF source/record custody differs')
 jars=[a.jars/'delta-spark_2.13-4.0.0.jar',a.jars/'delta-storage-4.0.0.jar'];expected=['538511702aae0ef6973a6a70af3d4543c9009f8edbed786a00737e2d3cd7f04e','9bdb9fb450f1e119eba53feb427f331b0d09072d26485b8273883ad72c9a2e1d']
 if [sha(j.read_bytes())for j in jars]!=expected:raise ValueError('Delta4 jar byte identity differs')
 spark=(SparkSession.builder.master('local[1]').appName('Original commerce0.8 Weft bounded compatibility').config('spark.driver.memory','512m').config('spark.ui.enabled','false').config('spark.sql.shuffle.partitions','1').config('spark.databricks.delta.snapshotPartitions','1').config('spark.sql.ansi.enabled','true').config('spark.jars',','.join(map(str,jars))).config('spark.sql.extensions','io.delta.sql.DeltaSparkSessionExtension').config('spark.sql.catalog.spark_catalog','org.apache.spark.sql.delta.catalog.DeltaCatalog').config('spark.sql.warehouse.dir',str(a.output/'warehouse')).getOrCreate());spark.sparkContext.setLogLevel('ERROR')
 try:
  name='spark_catalog.default.commerce_current';spark.sql(f"CREATE TABLE {name} (source_system STRING,type_id BIGINT,id BIGINT,schema_revision STRING,props_json STRING) USING DELTA LOCATION '{a.output/'delta'}'")
  # Explicit development mapping, not source keys converted to storage IDs.
  elements={e['id']:e for m in d['modules']for e in m['elements']};records={};pid=0
  for tid,record in enumerate([e for e in elements.values()if e.get('kind')=='record'],1):
   fields={}
   for ref in record['members']:pid+=1;fields[ref['element']]=str(pid)
   records[record['id']]=(tid,fields)
  native=[]
  for i,o in enumerate(graph['objects'],1):
   rec=o['type']['element']
   if rec not in records:continue
   tid,fields=records[rec];parts=[]
   for f,pid in fields.items():parts.append(json.dumps(pid)+':'+(o['values'][f] if elements[f]['scalarType']in ['integer','decimal'] else json.dumps(o['values'][f],ensure_ascii=False)))
   props='{'+','.join(parts)+'}';native.append({'id':i,'record':rec,'props_json':props,'original':o})
  spark.createDataFrame([('independent-commerce08',records[r['record']][0],r['id'],'original-1',r['props_json'])for r in native],schema='source_system STRING,type_id BIGINT,id BIGINT,schema_revision STRING,props_json STRING').write.format('delta').mode('append').saveAsTable(name)
  detail=spark.sql('DESCRIBE DETAIL '+name).first().asDict();version=int(spark.sql('DESCRIBE HISTORY '+name).first()['version']);table={'name':name.split('.'),'uuid':detail['id'],'version':version};save(a.output/'materialization.json',{'table':table,'rows':native,'manifestAuthority':False})
  pin={'documentId':d['id'],'revision':'original-1','umfVersion':'0.8.0','sha256':sha(model)}
  logical=lambda f:{'documentId':d['id'],'revision':'original-1','module':'domain','element':f}
  binding={'profile':'ashlar-databricks-candidate/0.1.0','layoutRevision':'ashlar-delta/0.3','layoutSha256':'ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e','modelPins':[pin],'publication':{'id':'independent-local-only','manifestUuid':table['uuid'],'tables':[table]},'records':[{'logical':logical(rec),'table':0,'kind':'object','sourceSystem':'independent-commerce08','typeId':str(tid),'schemaRevision':'original-1','properties':[{'logical':logical(f),'home':{'kind':'props','propertyId':pid}}for f,pid in fields.items()]}for rec,(tid,fields)in records.items()]};bt=json.dumps(binding,ensure_ascii=False,separators=(',',':'),sort_keys=True)
  def compile(sql):
   bt=json.dumps(binding,ensure_ascii=False,separators=(',',':'),sort_keys=True)
   request={'interfaceVersion':'weft-compile/0.2.0','dialect':'weft-sql/0.2.0','sql':sql,'modules':[{'documentJson':model.decode(),'pin':pin,'selectedModuleIds':['domain']}],'target':{'backendId':'ashlar.databricks.mathematical-integer','backendVersion':'0.1.0-candidate','targetProfile':'dbsql-mathematical-integer-candidate','bindingJson':bt,'bindingSha256':sha(bt.encode())},'options':{'allowCandidate':True}}
   return request,json.loads(subprocess.run([str(a.compiler)],input=json.dumps(request),text=True,capture_output=True,check=True).stdout)
  receipt_index=0
  def source_guard(artifact,params):
   nonlocal receipt_index
   checks=[]
   for obligation in artifact['obligations']:
    if obligation['id']=='ashlar.mathematicalInteger.publicSourceValidity':
     for check in obligation['parameters']['checks']:
      checks.append({'check':check,'rows':[r.asDict()for r in spark.sql(check['sql'],args=params).collect()]})
   if not checks:return None
   receipt_index+=1;request_path=a.output/('source-'+str(receipt_index)+'-request.json');receipt_path=a.output/('source-'+str(receipt_index)+'-receipt.json')
   save(request_path,{'sourceText':model.decode(),'modelPins':artifact['modelPins'],'bindingSha256':artifact['bindingSha256'],'checks':checks})
   subprocess.run(['bun',str(ROOT/'scripts/local/public_integer_source.ts'),str(a.umf),str(request_path),str(receipt_path)],check=True)
   return json.loads(receipt_path.read_bytes())
  line=next(o for o in graph['objects']if o['type']['element']=='order_lines');quantity=line['values']['order_lines.quantity']
  queries=[('quantity','SELECT o.quantity FROM order_lines o',[{'quantity':quantity}]),('filter','SELECT o.quantity FROM order_lines o WHERE o.quantity = '+quantity,[{'quantity':quantity}]),('join','SELECT o.quantity AS left_quantity, q.quantity AS right_quantity FROM order_lines o JOIN order_lines q ON o.quantity = q.quantity',[{'left_quantity':quantity,'right_quantity':quantity}]),('count','SELECT COUNT(*) AS n FROM order_lines o',[{'n':'1'}]),('sum','SELECT SUM(o.quantity) AS total FROM order_lines o',[{'total':quantity}])];results=[]
  for label,sql,expectedrows in queries:
   request,artifact=compile(sql);save(a.output/(label+'-request.json'),request);save(a.output/(label+'-artifact.json'),artifact)
   if artifact['status']!='compiled':raise ValueError(artifact)
   if artifact['modelPins']!=[pin]:raise ValueError('Original model pin lost')
   ids={o['id']for o in artifact['obligations']};known={'ashlar.candidate.publication','ashlar.candidate.scalarIntegrity','ashlar.mathematicalInteger.representability','ashlar.mathematicalInteger.exactResult','ashlar.mathematicalInteger.publicSourceValidity'}
   if not ids.issubset(known):raise ValueError('Unadmitted obligation '+str(ids-known))
   params={'p'+str(p['position']):p['value']for p in artifact['parameters']};checks=[]
   public=source_guard(artifact,params)
   if public is not None and public['admitted']is not True:raise ValueError('Original public source validity failed')
   for o in artifact['obligations']:
    if o['id']in ['ashlar.candidate.scalarIntegrity','ashlar.mathematicalInteger.representability']:
     for c in o['parameters']['checks']:
      rows=[r.asDict()for r in spark.sql(c['sql'],args=params).collect()]
      if rows!=[{'violations':'0'}]:raise ValueError('Original scalar guard failed')
      checks.append({'check':c,'rows':rows})
   rows=[r.asDict()for r in spark.sql(artifact['sql'],args=params).collect()]
   decoded=decode_rows(artifact,rows)
   if rows!=expectedrows:raise ValueError((rows,expectedrows))
   results.append({'name':label,'rows':rows,'decoded':decoded,'checks':checks,'publicSourceReceipt':public,'productionAdmission':False})
  # Independently authored backend-capability controls, not original graph rows.
  controls=[];tid,fields=records['order_lines'];quantity_pid=fields['order_lines.quantity'];id_pid=fields['order_lines.id'];native_line=next(r for r in native if r['record']=='order_lines')
  def set_quantity(token):
   props=native_line['props_json'];marker=json.dumps(quantity_pid)+':'+quantity
   if marker not in props:raise ValueError('Original raw integer carrier missing')
   return props.replace(marker,json.dumps(quantity_pid)+':'+token)
  def current():table['version']=int(spark.sql('DESCRIBE HISTORY '+name).first()['version'])
  def guard(artifact):
   params={'p'+str(p['position']):p['value']for p in artifact['parameters']};observations=[]
   public=source_guard(artifact,params)
   if public is not None and public['admitted']is not True:raise ValueError('Original public source validity failed')
   if public is not None:observations.append({'obligation':'ashlar.mathematicalInteger.publicSourceValidity','receipt':public})
   for obligation in artifact['obligations']:
    if obligation['id']in ['ashlar.candidate.scalarIntegrity','ashlar.mathematicalInteger.representability']:
     for check in obligation['parameters']['checks']:
      rows=[r.asDict()for r in spark.sql(check['sql'],args=params).collect()];observations.append({'obligation':obligation['id'],'check':check,'rows':rows})
   return params,observations
  for label,token,expected in [('beyond-safe-integer','9007199254740993','9007199254740993'),('representation-maximum','9'*38,'9'*38),('unrepresentable-source','1'+'0'*38,None)]:
   props=set_quantity(token);spark.sql('UPDATE '+name+' SET props_json=:p WHERE id=:i',args={'p':props,'i':native_line['id']});current();request,artifact=compile('SELECT o.quantity FROM order_lines o');save(a.output/(label+'-request.json'),request);save(a.output/(label+'-artifact.json'),artifact)
   if artifact['status']!='compiled':raise ValueError(artifact)
   params,checks=guard(artifact);wanted='1'if expected is None else'0'
   if any(c['rows']!=[{'violations':wanted if c['obligation']=='ashlar.mathematicalInteger.representability'else '0'}]for c in checks if 'rows'in c):raise ValueError('Capability guard classification differs')
   rows=None
   if expected is not None:
    rows=[r.asDict()for r in spark.sql(artifact['sql'],args=params).collect()]
    decoded=decode_rows(artifact,rows)
    if rows!=[{'quantity':expected}]:raise ValueError('Exact integer carrier was rounded')
   observed=spark.sql('SELECT props_json FROM '+name+' WHERE id=:i',args={'i':native_line['id']}).first()['props_json']
   if observed!=props:raise ValueError('Original numeric token custody lost')
   controls.append({'name':label,'token':token,'sourcePublicValidation':next(c['result']for c in source_receipt['controls']if c['token']==token),'table':dict(table),'props_json':props,'checks':checks,'rows':rows,'userSQLExecuted':expected is not None,'failureClass':'backend-capability'if expected is None else None})
  for label,token in [('boolean','true'),('null','null'),('quoted-numeric','"123"'),('fraction','12.5'),('exponent','1e3'),('duplicate','10,"'+quantity_pid+'":11')]:
   props=set_quantity(token);spark.sql('UPDATE '+name+' SET props_json=:p WHERE id=:i',args={'p':props,'i':native_line['id']});current();request,artifact=compile('SELECT o.quantity FROM order_lines o');save(a.output/('invalid-'+label+'-request.json'),request);save(a.output/('invalid-'+label+'-artifact.json'),artifact)
   params={'p'+str(p['position']):p['value']for p in artifact['parameters']}
   if label=='duplicate':
    # Native parse_json refuses duplicate JSON keys before returning source rows.
    try:public=source_guard(artifact,params)
    except Exception as error:public={'admitted':False,'nativeIntegrityError':str(error)}
   else:public=source_guard(artifact,params)
   if public is None or public['admitted']is not False:raise ValueError('Invalid source misclassified as capacity failure')
   controls.append({'name':'invalid-'+label,'props_json':props,'publicSourceReceipt':public,'failureClass':'backend-capability'if label=='exponent'else'source-integrity','userSQLExecuted':False})
  props=set_quantity('10');spark.sql('UPDATE '+name+' SET props_json=:p, schema_revision=:r WHERE id=:i',args={'p':props,'r':'wrong-revision','i':native_line['id']});current();request,artifact=compile('SELECT o.quantity FROM order_lines o');save(a.output/'wrong-revision-request.json',request);save(a.output/'wrong-revision-artifact.json',artifact);params,checks=guard(artifact)
  for check in checks:
   if 'rows'in check and check['rows']!=[{'violations':'1'if check['obligation']=='ashlar.candidate.scalarIntegrity'else'0'}]:raise ValueError('Revision integrity was conflated with capacity')
  controls.append({'name':'wrong-revision','checks':checks,'failureClass':'source-integrity','userSQLExecuted':False})
  spark.sql('UPDATE '+name+' SET schema_revision=:r WHERE id=:i',args={'r':'original-1','i':native_line['id']});current()
  request,artifact=compile('SELECT o.quantity FROM order_lines o WHERE o.quantity = '+('1'+'0'*38));save(a.output/'unrepresentable-literal-request.json',request);save(a.output/'unrepresentable-literal-artifact.json',artifact)
  if artifact['status']!='blocked'or artifact['diagnostics'][0]['code']!='WFT-CAPABILITY'or'sql'in artifact:raise ValueError('Out-of-capability literal was narrowed or published')
  # Both operands are source-valid and individually representable; their SUM is not.
  maximum='9'*38;props=set_quantity(maximum);spark.sql('UPDATE '+name+' SET props_json=:p WHERE id=:i',args={'p':props,'i':native_line['id']})
  extra=props.replace(json.dumps(id_pid)+':'+json.dumps(line['values']['order_lines.id']),json.dumps(id_pid)+':'+json.dumps('independently-authored-overflow-row'))
  spark.sql('INSERT INTO '+name+' VALUES (:s,:t,100,:r,:p)',args={'s':'independent-commerce08','t':tid,'r':'original-1','p':extra});current();request,artifact=compile('SELECT SUM(o.quantity) AS total FROM order_lines o');save(a.output/'aggregate-overflow-request.json',request);save(a.output/'aggregate-overflow-artifact.json',artifact);params,checks=guard(artifact)
  if any(c['rows']!=[{'violations':'0'}]for c in checks if 'rows'in c):raise ValueError('Individually representable operands refused')
  try:spark.sql(artifact['sql'],args=params).collect()
  except Exception as error:
   if'WFT-CAPABILITY'not in str(error):raise
   controls.append({'name':'aggregate-overflow','table':dict(table),'checks':checks,'error':str(error),'partialRowsPublished':False,'failureClass':'backend-capability'})
  else:raise ValueError('Aggregate overflow silently rounded/null/wrapped')
  # Cancellation must be exact or explicitly fail; safe operands do not prove safe intermediates.
  negative=set_quantity('-'+maximum).replace(json.dumps(id_pid)+':'+json.dumps(line['values']['order_lines.id']),json.dumps(id_pid)+':'+json.dumps('independently-authored-cancellation-row'))
  spark.sql('INSERT INTO '+name+' VALUES (:s,:t,101,:r,:p)',args={'s':'independent-commerce08','t':tid,'r':'original-1','p':negative});current();request,artifact=compile('SELECT SUM(o.quantity) AS total FROM order_lines o');save(a.output/'cancellation-request.json',request);save(a.output/'cancellation-artifact.json',artifact);params,checks=guard(artifact)
  if any(c['rows']!=[{'violations':'0'}]for c in checks if 'rows'in c):raise ValueError('Cancellation input representation failed')
  try:
   rows=[r.asDict()for r in spark.sql(artifact['sql'],args=params).collect()]
   decoded=decode_rows(artifact,rows)
   if rows!=[{'total':maximum}]:raise ValueError('Intermediate overflow silently changed exact sum')
   controls.append({'name':'cancellation','checks':checks,'rows':rows,'outcome':'exact'})
  except Exception as error:
   if'WFT-CAPABILITY'not in str(error):raise
   controls.append({'name':'cancellation','checks':checks,'error':str(error),'outcome':'explicit-capability-refusal','partialRowsPublished':False})
  save(a.output/'report.json',{'results':results,'controls':controls,'legacyProfileIntegerRefusedBeforeRuntime':True,'publicationAdmitted':False,'generalArbitraryPrecisionAdmitted':False,'sparkVersion':spark.version});print(json.dumps({'passed':len(results),'output':str(a.output)}))
 finally:spark.stop()
if __name__=='__main__':main()
