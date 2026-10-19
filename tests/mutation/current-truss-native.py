# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original Python compiler -> current native selection -> public Truss update.

Disposable original-owner installation only. Administrative raw selection is an
excluded test assessor, not protected RR/caller/action/seal qualification.
"""
import argparse,hashlib,json,sys
from pathlib import Path

def main():
 p=argparse.ArgumentParser();p.add_argument('--truss',type=Path,required=True);p.add_argument('--model',type=Path,required=True);p.add_argument('--output',type=Path,required=True);args=p.parse_args()
 args.output.mkdir(parents=True,exist_ok=False)
 sys.path.insert(0,str(args.truss/'packages/python/src'));sys.path.insert(0,str(args.truss/'packages/python/examples'))
 import weft,psycopg
 from adopter_workflow import transaction,success,expect
 from truss import ObjectFound
 from truss.groups import UpdateObject,StoredObjectReference,AuthoredValue,GroupSuccess
 from truss.contracts import MapValue,TextValue
 packet=json.loads(args.model.read_text());module=Path(weft.__file__);native=module if module.suffix=='.so' else next(module.parent.glob('*.so'));report={"pythonVersion":sys.version,"pythonModuleSha256":hashlib.sha256(native.read_bytes()).hexdigest(),"scope":"PG18.6 same-store original Python lowering/native preflight/selection and public SDK effect correspondence; excluded administrative selector; no RR/bulk seal/action authority qualification","checks":[]}
 def artifact(raw):return {'mediaType':'application/json','utf8':raw,'sha256':hashlib.sha256(raw.encode()).hexdigest()}
 def exercise(runtime,person,admin,documents,identities,bundle_bytes):
  bundle=json.loads(bundle_bytes);binding=artifact(bundle['bindingJson']);compiler=weft.MutationCompiler(json.dumps(packet['configuration']))
  configuration={'model':packet['request']['model'],'modelPin':packet['request']['modelPin'],'binding':binding,'evidence':['Original current Truss native exporter/same installed store; scoped selection E2E']}
  handle=compiler.register_truss_selection(json.dumps(configuration))
  def lower(sql,max_items=1):
   request=dict(packet['request'],sql=sql);compiled=json.loads(compiler.compile_json(json.dumps(request)));assert compiled['status']=='compiled',compiled
   plan=json.loads(compiled['plan']['utf8']);r={'selection':plan['selection']['artifact'],'model':request['model'],'binding':binding,'maxItems':max_items}
   result=json.loads(compiler.lower_selection_json(handle,json.dumps(r)));assert result['status']=='lowered',result
   return plan,result
  def values(parameters):
   result=[]
   for p in parameters:
    v=p['value'];assert len(v)==1
    if 'string' in v:result.append(v['string'])
    elif 'boolean' in v:result.append(v['boolean'])
    else:raise AssertionError('Unqualified native parameter domain')
   return result
  def select(program):
   # One caller-owned native cut; execute original statements with RawCursor's
   # dollar placeholders unchanged. Native source authorization is excluded here.
   with admin.transaction():
    admin.execute('LOCK TABLE truss.object, truss.edge IN SHARE MODE')
    with psycopg.RawCursor(admin) as cursor:
     for check in program['preflight']:
      cursor.execute(check['sql'],values(check['parameters']));rows=cursor.fetchall();assert rows==[('0',)],rows
     cursor.execute(program['sql'],values(program['parameters']));rows=cursor.fetchall()
     if len(rows)>program['bounds']['maxItems']:raise OverflowError('Original maxItems+1 sentinel refused without trimming')
     return rows
  plan,program=lower(packet['request']['sql']);rows=select(program);assert rows==[(str(identities[0].id),'1','bar')],rows
  report['checks'].append('exact original Python action plan and same-store native identity/version/key selection')
  with transaction(person,read_only=True) as (assembly,tx):
   denied=person.execute('SELECT has_table_privilege(current_user,\'truss.object\',\'SELECT\')').fetchone()[0];assert denied is False
  report['checks'].append('ordinary module reader has no raw object SELECT; excluded assessor remains explicit')
  _,overflow=lower("UPDATE products SET name='Foo' WHERE id <> 'never'")
  try:select(overflow)
  except OverflowError:pass
  else:raise AssertionError('Overflow silently trimmed')
  report['checks'].append('maxItems+1 overflow refuses before effects')
  bad=dict(configuration,modelPin=dict(configuration['modelPin'],revision='demo1'))
  try:compiler.register_truss_selection(json.dumps(bad))
  except ValueError:pass
  else:raise AssertionError('Old illustrative revision silently relabeled')
  bad=dict(configuration,binding=dict(binding,sha256='0'*64))
  try:compiler.register_truss_selection(json.dumps(bad))
  except ValueError:pass
  else:raise AssertionError('Binding hash substitution admitted')
  report['checks'].append('stale revision and changed binding pins refused')
  # A hidden-by-predicate malformed key must fail complete-population preflight.
  original=admin.execute('SELECT props FROM truss.object WHERE id=%s',(int(identities[1].id),)).fetchone()[0]
  record=bundle['binding']['records'][0];key_prop=record['keys'][0]['propertyIds'][0]
  corrupt=dict(original);corrupt[key_prop]={'kind':'string','text':17}
  admin.execute('UPDATE truss.object SET props=%s::jsonb WHERE id=%s',(json.dumps(corrupt),int(identities[1].id)))
  try:
   try:select(program)
   except AssertionError:pass
   else:raise AssertionError('Malformed filtered-out key escaped preflight')
  finally:admin.execute('UPDATE truss.object SET props=%s::jsonb WHERE id=%s',(json.dumps(original),int(identities[1].id)))
  report['checks'].append('malformed filtered-out property refuses before native selection')
  assert select(program)==rows
  assignments=plan['inputs']['assignments'];assert len(assignments)==1 and assignments[0]['field']=={'module':'sales','element':'product-name'}
  value=assignments[0]['value']['string'];assert value=='Foo'
  with transaction(person) as (assembly,tx):
   context=success(assembly.group_context_in_transaction(tx));op=UpdateObject(target=StoredObjectReference(identity=identities[0]),set=(AuthoredValue(name='name',value=TextValue(kind='string',text=value)),),unset=(),expected_version=rows[0][1])
   expect(success(assembly.apply_in_transaction(tx,context.input((op,),MapValue(entries=())),context.request('compiler-selected-name'),documents=documents,bun_executable='/opt/homebrew/bin/bun')),GroupSuccess)
  with transaction(person,read_only=True) as (assembly,tx):
   for identity,expected in zip(identities,('Foo','keep'),strict=True):
    found=expect(success(assembly.get_object_in_transaction(tx,document_id='product-action-demo',module_id='sales',record_id='products',object_id=int(identity.id))),ObjectFound);assert dict(found.object.values)['name']==expected
  report['checks'].append('public typed Truss update applies compiled assignment only to selected version; other product unchanged')
  report['postgres']=admin.execute('SHOW server_version').fetchone()[0];assert report['postgres'].startswith('18.6')
  report['bindingSha256']=binding['sha256'];report['modelSha256']=packet['request']['model']['sha256'];report['installationId']=bundle['binding']['installationId']
  (args.output/'lowering.json').write_text(json.dumps(program,indent=2)+'\n');(args.output/'compiled-plan.json').write_text(json.dumps(plan,indent=2)+'\n');(args.output/'binding.json').write_text(binding['utf8']);(args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n')
 source=args.truss/'scripts/export-products-native-binding.py';raw=source.read_text();needle="                    bundle_bytes=success(assembly._host._run_operation(tx,export))";assert raw.count(needle)==1
 # Execute only the pinned original exporter with one explicit test callback.
 # It preserves original public installation/acceptance/seed/readback and cleanup.
 patched=raw.replace(needle,needle+"\n                exercise(runtime,person,admin,documents,identities,bundle_bytes)")
 scope={'__name__':'original_exporter_test','__file__':str(source),'exercise':exercise};exec(compile(patched,str(source),'exec'),scope)
 report['exporterSha256']=hashlib.sha256(source.read_bytes()).hexdigest();scope['run'](args.model,args.output/'original-export','/opt/homebrew/bin/bun');(args.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
if __name__=='__main__':main()
