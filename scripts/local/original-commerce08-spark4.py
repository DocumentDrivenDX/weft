"""Original commerce0.8 unchanged compiler SQL, independent local Spark4 profile.
No production publication/profile admission; no Databricks calls.
"""
import argparse,hashlib,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
def sha(b):return hashlib.sha256(b).hexdigest()
def save(p,v):p.write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n')
def main():
 p=argparse.ArgumentParser();p.add_argument('--compiler',type=Path,required=True);p.add_argument('--output',type=Path,required=True);p.add_argument('--jars',type=Path,required=True);a=p.parse_args()
 if a.output.exists():raise ValueError('Fresh output required')
 from pyspark.sql import SparkSession
 import pyspark
 if pyspark.__version__!='4.0.1':raise ValueError('Explicit independent Spark4.0.1 profile required')
 model=(ROOT/'tests/fixtures/original-commerce-0.8/ontology.json').read_bytes();graph=json.loads((ROOT/'tests/fixtures/original-commerce-0.8/graph.json').read_bytes());d=json.loads(model)
 a.output.mkdir(parents=True);save(a.output/'qualification.json',{'scope':'Independent local compatibility; original0.8 source. No qualified Databricks profile, publication/authorization or ACK admission. Partial scalar queries; unbounded integers remain required outstanding.','compilerSha256':sha(a.compiler.read_bytes()),'sourceSha256':sha(model)})
 pin={'documentId':d['id'],'revision':'original-1','umfVersion':'0.8.0','sha256':sha(model)}
 preflight={'interfaceVersion':'weft-compile/0.2.0','dialect':'weft-sql/0.2.0','sql':'SELECT o.quantity FROM order_lines o','modules':[{'documentJson':model.decode(),'pin':pin,'selectedModuleIds':['domain']}],'target':{'backendId':'ashlar.databricks','backendVersion':'0.1.0-candidate','targetProfile':'dbsql-candidate','bindingJson':'{}','bindingSha256':sha(b'{}')},'options':{'allowCandidate':True}}
 refusal=json.loads(subprocess.run([str(a.compiler)],input=json.dumps(preflight),text=True,capture_output=True,check=True).stdout)
 if refusal['status']!='blocked'or refusal['diagnostics'][0]['code']!='WFT-TYPE':raise ValueError('Unbounded integer must refuse before local runtime acquisition')
 save(a.output/'integer-refusal-request.json',preflight);save(a.output/'integer-refusal-artifact.json',refusal)
 jars=[a.jars/'delta-spark_2.13-4.0.0.jar',a.jars/'delta-storage-4.0.0.jar'];expected=['538511702aae0ef6973a6a70af3d4543c9009f8edbed786a00737e2d3cd7f04e','9bdb9fb450f1e119eba53feb427f331b0d09072d26485b8273883ad72c9a2e1d']
 if [sha(j.read_bytes())for j in jars]!=expected:raise ValueError('Delta4 jar byte identity differs')
 spark=(SparkSession.builder.master('local[1]').appName('Original commerce0.8 Weft bounded compatibility').config('spark.driver.memory','512m').config('spark.ui.enabled','false').config('spark.sql.shuffle.partitions','1').config('spark.databricks.delta.snapshotPartitions','1').config('spark.sql.ansi.enabled','true').config('spark.jars',','.join(map(str,jars))).config('spark.sql.extensions','io.delta.sql.DeltaSparkSessionExtension').config('spark.sql.catalog.spark_catalog','org.apache.spark.sql.delta.catalog.DeltaCatalog').config('spark.sql.warehouse.dir',str(a.output/'warehouse')).getOrCreate());spark.sparkContext.setLogLevel('ERROR')
 try:
  name='spark_catalog.default.commerce_current';spark.sql(f"CREATE TABLE {name} (source_system STRING,type_id BIGINT,id BIGINT,schema_revision STRING,props_json STRING) USING DELTA LOCATION '{a.output/'delta'}'")
  # Explicit development mapping, not source keys converted to storage IDs.
  records={'products':(1,{'products.id':'1','products.supplier_id':'2','products.sku':'3','products.unit_price':'4'}),'suppliers':(2,{'suppliers.id':'5','suppliers.name':'6'})}
  native=[]
  for i,o in enumerate(graph['objects'],1):
   rec=o['type']['element']
   if rec not in records:continue
   tid,fields=records[rec];parts=[]
   for f,pid in fields.items():parts.append(json.dumps(pid)+':'+(o['values'][f] if f=='products.unit_price' else json.dumps(o['values'][f],ensure_ascii=False)))
   props='{'+','.join(parts)+'}';native.append({'id':i,'record':rec,'props_json':props,'original':o});spark.sql(f'INSERT INTO {name} VALUES (:s,:t,:i,:r,:p)',args={'s':'independent-commerce08','t':tid,'i':i,'r':'original-1','p':props})
  detail=spark.sql('DESCRIBE DETAIL '+name).first().asDict();version=int(spark.sql('DESCRIBE HISTORY '+name).first()['version']);table={'name':name.split('.'),'uuid':detail['id'],'version':version};save(a.output/'materialization.json',{'table':table,'rows':native,'manifestAuthority':False})
  pin={'documentId':d['id'],'revision':'original-1','umfVersion':'0.8.0','sha256':sha(model)}
  logical=lambda f:{'documentId':d['id'],'revision':'original-1','module':'domain','element':f}
  binding={'profile':'ashlar-databricks-candidate/0.1.0','layoutRevision':'ashlar-delta/0.3','layoutSha256':'ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e','modelPins':[pin],'publication':{'id':'independent-local-only','manifestUuid':table['uuid'],'tables':[table]},'records':[{'logical':logical(rec),'table':0,'kind':'object','sourceSystem':'independent-commerce08','typeId':str(tid),'schemaRevision':'original-1','properties':[{'logical':logical(f),'home':{'kind':'props','propertyId':pid}}for f,pid in fields.items()]}for rec,(tid,fields)in records.items()]};bt=json.dumps(binding,ensure_ascii=False,separators=(',',':'),sort_keys=True)
  def compile(sql):
   request={'interfaceVersion':'weft-compile/0.2.0','dialect':'weft-sql/0.2.0','sql':sql,'modules':[{'documentJson':model.decode(),'pin':pin,'selectedModuleIds':['domain']}],'target':{'backendId':'ashlar.databricks','backendVersion':'0.1.0-candidate','targetProfile':'dbsql-candidate','bindingJson':bt,'bindingSha256':sha(bt.encode())},'options':{'allowCandidate':True}}
   return request,json.loads(subprocess.run([str(a.compiler)],input=json.dumps(request),text=True,capture_output=True,check=True).stdout)
  product=next(o for o in graph['objects']if o['type']['element']=='products');supplier=next(o for o in graph['objects']if o['type']['element']=='suppliers')
  queries=[('select','SELECT p.id, p.unit_price FROM products p',[{'id':product['values']['products.id'],'unit_price':'12.50'}]),('join','SELECT p.id AS product_id, s.name AS supplier_name FROM products p JOIN suppliers s ON p.supplier_id = s.id',[{'product_id':product['values']['products.id'],'supplier_name':supplier['values']['suppliers.name']}]),('count','SELECT COUNT(*) AS n FROM products p',[{'n':'1'}])];results=[]
  for label,sql,expectedrows in queries:
   request,artifact=compile(sql);save(a.output/(label+'-request.json'),request);save(a.output/(label+'-artifact.json'),artifact)
   if artifact['status']!='compiled':raise ValueError(artifact)
   if artifact['modelPins']!=[pin]:raise ValueError('Original model pin lost')
   ids={o['id']for o in artifact['obligations']};known={'ashlar.candidate.publication','ashlar.candidate.scalarIntegrity'}
   if not ids.issubset(known):raise ValueError('Unadmitted obligation '+str(ids-known))
   params={'p'+str(p['position']):p['value']for p in artifact['parameters']};checks=[]
   for o in artifact['obligations']:
    if o['id']=='ashlar.candidate.scalarIntegrity':
     for c in o['parameters']['checks']:
      rows=[r.asDict()for r in spark.sql(c['sql'],args=params).collect()]
      if rows!=[{'violations':'0'}]:raise ValueError('Original scalar guard failed')
      checks.append({'check':c,'rows':rows})
   rows=[r.asDict()for r in spark.sql(artifact['sql'],args=params).collect()]
   if rows!=expectedrows:raise ValueError((rows,expectedrows))
   results.append({'name':label,'rows':rows,'checks':checks,'productionAdmission':False})
  save(a.output/'report.json',{'results':results,'integerRefused':True,'publicationAdmitted':False,'partialOnly':True,'sparkVersion':spark.version});print(json.dumps({'passed':len(results),'output':str(a.output)}))
 finally:spark.stop()
if __name__=='__main__':main()
