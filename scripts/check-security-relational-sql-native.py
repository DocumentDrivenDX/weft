# SPDX-License-Identifier: MIT OR Apache-2.0
import json,hashlib,subprocess,tempfile,os,time,argparse
from pathlib import Path
parser=argparse.ArgumentParser(description='Controlled SQL semantics only, no native security admission');parser.add_argument('--compiler',type=Path,required=True);parser.add_argument('--operand-inspector',type=Path,required=True);parser.add_argument('--guarded-producer',type=Path,required=True);parser.add_argument('--result-inspector',type=Path,required=True);parser.add_argument('--postgres-bin',type=Path,required=True);parser.add_argument('--truss-scalar-sql',type=Path,required=True);parser.add_argument('--evidence-dir',type=Path,required=True);args=parser.parse_args();root=Path(__file__).resolve().parents[1];f=root/'docs/helix/03-test/fixtures/security-graph-source';binary=args.compiler.resolve();pg=args.postgres_bin.resolve();ev=args.evidence_dir.resolve();ev.mkdir(parents=True,exist_ok=True)
sha=lambda s:hashlib.sha256(s.encode()).hexdigest()
def request(sql,mode=None,grant=True,parameters=None,optional=False,original_disclosure=False):
 r=json.loads((f/'mapping-request.json').read_text());r['sql']=sql;r['parameters']=parameters or {};physical=(f/'adoc.json').read_text();o=json.loads(r['ontologyJson']);p=json.loads(r['policyJson']);profile=json.loads(r['queryProfileJson']);resource={'documentId':'domain','moduleId':'m','elementId':'Resource'};salary={'documentId':'domain','moduleId':'m','elementId':'salary'}
 p['rules']=[{'id':'read','effect':'permit','actions':['read'],'target':[resource],'condition':{'op':'literal','value':True},'disclosure':[{'field':salary,'disposition':{'kind':'withheld'}}]}]
 if mode:
  entity=next(e for e in o['entities'] if e['type']==resource);field=next(e for e in entity['fields'] if e['ref']==salary);field['queryUse']={op:mode for op in ['predicate','order','group','join','aggregate']}
 if mode=='original-authorized':
  o['actions'].append('query-original');profile['bindings']=[{'target':resource,'field':salary,'operator':op,'originalAction':'query-original'} for op in ['predicate','order','group','join','aggregate']]
  if grant:p['rules'].append({'id':'original','effect':'permit','actions':['query-original'],'target':[resource],'condition':{'op':'literal','value':True}})
 if original_disclosure:p['rules'][0]['disclosure'][0]['disposition']={'kind':'original'}
 inventory=(f/'native-inventory.json').read_text()
 if optional:
  model=json.loads(r['modules'][0]['documentJson']);next(e for e in model['modules'][0]['elements'] if e['id']=='salary')['nullability']='absent-allowed';r['modules'][0]['documentJson']=json.dumps(model);r['modules'][0]['pin']['sha256']=sha(r['modules'][0]['documentJson']);profile['modelPins']=[m['pin'] for m in r['modules']]
  native=json.loads(physical);next(e for e in native['modules'][0]['elements'] if e['id']=='salary')['nullability']='absent-allowed';physical=json.dumps(native)
  captured=json.loads(inventory);next(e for e in captured['inventory']['properties'] if e['type_id']==5 and e['prop_id']==7)['nullability']='absent-allowed';inventory=json.dumps(captured)
 r['ontologyJson']=json.dumps(o);r['policyJson']=json.dumps(p);resolution=json.loads((f/'resolution.json').read_text());resolution['ontology']=o;
 if optional:resolution['documents'][0]['document']=model
 resolution=json.dumps(resolution);c=json.loads((f/'binding-correspondence.json').read_text());c['logicalSourceDigests']['policy']=sha(r['policyJson']);
 if optional:c['logicalSourceDigests']['document']=sha(r['modules'][0]['documentJson']);c['physicalSourceSha256']=sha(physical);c['inventorySha256']=sha(inventory)
 c['logicalSourceDigests']['resolution']=sha(resolution)
 source={'version':'weft.truss.security.graph-source/0.1-candidate','nativeSchemaRevision':'1','physicalModule':{'documentJson':physical,'pin':{'documentId':'adoc','revision':'r1','umfVersion':'0.8.0','sha256':sha(physical)},'selectedModuleIds':['m']},'inventoryJson':inventory,'correspondenceJson':json.dumps(c),'logicalResolutionJson':resolution,'logicalTransportJson':(f/'transport.json').read_text()}
 r['bindingJson']=json.dumps(source);profile['ontologySha256']=sha(r['ontologyJson']);profile['policySha256']=sha(r['policyJson']);profile['binding']['sha256']=sha(r['bindingJson']);r['queryProfileJson']=json.dumps(profile)
 raw=json.dumps(r);out=subprocess.run([str(binary)],input=raw,text=True,capture_output=True,timeout=20)
 if out.returncode:raise RuntimeError(out.stderr[:1000])
 result=json.loads(out.stdout);result["_originalRequest"]=raw;return result
def run(cmd,**kw):
 p=subprocess.run(cmd,text=True,capture_output=True,timeout=kw.pop('timeout',30),**kw)
 if p.returncode:raise RuntimeError(p.stderr[-2000:])
 return p.stdout
home=Path(tempfile.mkdtemp(prefix='weft-relational-native-',dir='/private/tmp'));data=home/'data';sock=home/'socket';sock.mkdir();started=False;results=[];begin=time.monotonic()
try:
 run([str(pg/'initdb'),'-D',str(data),'-U','postgres','--auth=trust','--encoding=UTF8'])
 run([str(pg/'pg_ctl'),'-D',str(data),'-l',str(home/'server.log'),'-w','-o',f"-h '' -k {sock}",'start']);started=True
 def sql(text):return run([str(pg/'psql'),'-h',str(sock),'-U','postgres','-d','postgres','-Atq','-v','ON_ERROR_STOP=1'],input=text).strip()
 helpers=args.truss_scalar_sql.read_text()
 sql('CREATE SCHEMA truss;CREATE TABLE truss.object(id bigint PRIMARY KEY,type_id bigint,props jsonb);CREATE TABLE truss.edge(rel_type_id bigint,source_type bigint,target_type bigint,source_id bigint,target_id bigint);'+helpers)
 rows=[(1,6,{'8':{'kind':'string','text':'S'}}),(2,5,{'6':{'kind':'string','text':'RA'},'7':{'kind':'integer','text':'100'}}),(3,5,{'6':{'kind':'string','text':'RB'},'7':{'kind':'integer','text':'200'}}),(4,5,{'6':{'kind':'string','text':'RC'},'7':{'kind':'integer','text':'100'}})]
 for id,t,props in rows:sql("INSERT INTO truss.object VALUES(%s,%s,'%s'::jsonb)"%(id,t,json.dumps(props).replace("'","''")))
 assert sql("SELECT pg_catalog.bool_and(truss.valid_scalar_property(props->'7','integer',FALSE,'{\"integerWidth\":{\"bits\":64,\"signed\":true}}')) FROM truss.object WHERE type_id=5")=='t'
 def execute(program,statement):
  params=sorted(program['parameters'],key=lambda p:p['ordinal']);types=','.join(p['physicalType'] for p in params);values=[]
  for p in params:
   v=1 if p['channel']=='subject' else p['value'];values.append('TRUE' if v is True else 'FALSE' if v is False else str(v) if isinstance(v,int) else "'"+str(v).replace("'","''")+"'")
  return sql('PREPARE probe ('+types+') AS '+statement+';EXECUTE probe ('+','.join(values)+');DEALLOCATE probe;')
 def check(name,query,mode=None,grant=True,parameters=None,expected=None,refused=False,operand_refused=False,optional=False,original_disclosure=False):
  program=request(query,mode,grant,parameters,optional,original_disclosure);assert all(c['stage'] in ['beforeOperandCapture','afterOperandValidation'] for c in program['checks']);violations=[int(execute(program,c['sql'])) for c in program['checks'] if c['stage']=='beforeOperandCapture']
  if refused:assert any(violations),name;results.append({'case':name,'refusedBeforeExecution':True,'violations':violations});return
  assert not any(violations),(name,violations)
  captures=[]
  for read in program['operandReads']:
   captured=execute(program,read['sql']);captures.append({'scan':read['scan'],'rows':[json.loads(line) for line in captured.splitlines()] if captured else []})
  batch={'version':'weft.security.operand-values/0.1-candidate','contextSha256':program['contextSha256'],'scans':captures}
  inspected=subprocess.run([str(args.operand_inspector)],input=json.dumps({'requestJson':program['_originalRequest'],'batchJson':json.dumps(batch)}),text=True,capture_output=True,timeout=20)
  if operand_refused:
   assert inspected.returncode!=0,name
   results.append({'case':name,'refusedBeforeExecution':True,'phase':'original_operand_values','diagnostic':json.loads(inspected.stderr)})
   return
  if inspected.returncode:raise RuntimeError(inspected.stderr[:1000])
  for c in program['checks']:
   if c['stage']=='afterOperandValidation':assert int(execute(program,c['sql']))==0,name
  output=execute(program,program['sql']);rows=[json.loads(line) for line in output.splitlines()] if output else [];assert rows==expected,(name,rows,expected)
  producer=subprocess.run([str(args.guarded_producer)],input=program['_originalRequest'],text=True,capture_output=True,timeout=20)
  if producer.returncode:raise RuntimeError(producer.stderr[:1000])
  artifact=json.loads(producer.stdout)
  assert artifact['queryContextSha256']==program['contextSha256']
  result={'version':artifact['resultContract']['encoding'],'resultContractSha256':artifact['resultContractSha256'],'rows':rows}
  input={'version':'weft.truss.security.relational-result/0.2-candidate','sourceRequestJson':program['_originalRequest'],'sourceOutputJson':producer.stdout,'operandBatchJson':json.dumps(batch),'batchJson':json.dumps(result)}
  checked_result=subprocess.run([str(args.result_inspector)],input=json.dumps(input),text=True,capture_output=True,timeout=20)
  if checked_result.returncode:raise RuntimeError(checked_result.stderr[:1000])
  results.append({'case':name,'rows':rows,'guards':len(program['checks']),'operandCheck':json.loads(inspected.stdout),'resultCheck':json.loads(checked_result.stdout)})
 computed=lambda n:{'outcomeId':'computed','disposition':'computed','value':{'integerToken':str(n)}}
 original=lambda s:{'outcomeId':'original','disposition':'original','value':{'string':s}}
 check('count','SELECT COUNT(*) FROM Resource r',expected=[[computed(3)]])
 check('sum','SELECT SUM(r.salary) FROM Resource r','original-authorized',expected=[[computed(400)]])
 check('all-operators','SELECT r.salary,SUM(s.salary),COUNT(*) FROM Resource r JOIN Resource s ON r.salary=s.salary WHERE r.salary=100 GROUP BY r.salary ORDER BY r.salary LIMIT 1','original-authorized',expected=[[{'outcomeId':'withheld','disposition':'withheld'},computed(400),computed(4)]])
 check('keyset','SELECT r.resourceId FROM Resource r WHERE (r.resourceId)>(\'RA\') ORDER BY r.resourceId LIMIT 1',expected=[[original('RB')]])
 check('named-value','SELECT r.resourceId FROM Resource r WHERE r.resourceId=:id',parameters={'id':{'family':'string','value':"RA';SELECT secret;--"}},expected=[])
 check('withheld-before-empty-filter','SELECT COUNT(*) FROM Resource r WHERE r.salary=999','disclosed',refused=True)
 check('missing-original-grant','SELECT COUNT(*) FROM Resource r WHERE r.salary=999','original-authorized',False,refused=True)
 sql("UPDATE truss.object SET props=pg_catalog.jsonb_set(props,'{7}','{\"kind\":\"integer\",\"text\":\"not-a-number\"}'::jsonb) WHERE id=4")
 check('invalid-later-operand-before-empty-filter','SELECT COUNT(*) FROM Resource r WHERE r.salary=999','original-authorized',operand_refused=True)
 sql("DELETE FROM truss.object WHERE type_id=5")
 for id,key,cell in [(2,'RA',None),(3,'RB',{'kind':'null'})]:
  props={'6':{'kind':'string','text':key}}
  if cell is not None:props['7']=cell
  sql("INSERT INTO truss.object VALUES(%s,5,'%s'::jsonb)"%(id,json.dumps(props).replace("'","''")))
 null_sum={'outcomeId':'computed','disposition':'computed','value':None}
 check('nullable-grouped-sum-original','SELECT r.resourceId,SUM(r.salary) FROM Resource r GROUP BY r.resourceId ORDER BY r.resourceId','original-authorized',expected=[[original('RA'),null_sum],[original('RB'),null_sum]],optional=True)
 check('absence-null-distinct-group-keys','SELECT COUNT(*) FROM Resource r GROUP BY r.salary','original-authorized',expected=[[computed(1)],[computed(1)]],optional=True)
 check('nullable-disclosed-grouped-sum','SELECT SUM(r.salary) FROM Resource r GROUP BY r.resourceId','disclosed',expected=[[null_sum],[null_sum]],optional=True,original_disclosure=True)
 report={'version':'weft.relational.sql-controlled-probe/0.1','engine':run([str(pg/'postgres'),'--version']).strip(),'securityReady':False,'qualification':'Excluded administrator, partial synthetic graph layout, original scalar helper bodies; no installed source/caller/current-authority or guarded release proof; no native acceptance case promotion','helperSha256':sha(helpers),'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'durationSeconds':round(time.monotonic()-begin,3),'controls':results}
 (ev/'native-sql-controls.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'passed':len(results),'durationSeconds':report['durationSeconds']}))
finally:
 if started:run([str(pg/'pg_ctl'),'-D',str(data),'-w','-m','immediate','stop'])
