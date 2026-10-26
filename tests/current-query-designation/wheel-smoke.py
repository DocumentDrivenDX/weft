# SPDX-License-Identifier: MIT OR Apache-2.0
"""Gate exact standard wheels on real current-query producers/conditional ABI2/3/4.
No database connection, native registration, permission or security qualification.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import argparse,hashlib,importlib.metadata,importlib.machinery,json,pathlib,subprocess,sys,zipfile,tomllib
import weft
p=argparse.ArgumentParser();p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
root=pathlib.Path(__file__).resolve().parents[2];fixture=root/'tests/current-query-designation/fixtures';checks=[]
def digest(data):return hashlib.sha256(data).hexdigest()
def pins():
 modules=[module for name,module in sys.modules.items() if name.startswith('weft') and any(getattr(module,'__file__','').endswith(suffix) for suffix in importlib.machinery.EXTENSION_SUFFIXES)]
 assert len(modules)==1;native=pathlib.Path(modules[0].__file__);shim=pathlib.Path(weft.__file__)
 with zipfile.ZipFile(a.wheel) as archive:
  names=[name for name in archive.namelist() if name.endswith(('.so','.pyd'))];assert len(names)==1
  assert digest(native.read_bytes())==digest(archive.read(names[0])) and shim.read_bytes()==archive.read('weft/__init__.py')
 return {'wheelSha256':digest(a.wheel.read_bytes()),'nativeModuleSha256':digest(native.read_bytes()),'shimSha256':digest(shim.read_bytes())}
before=pins();version=importlib.metadata.version('umf-weft');expected_version=tomllib.loads((root/'crates/weft-python/pyproject.toml').read_text())['project']['version'];assert version==weft.__version__==expected_version
assert callable(weft.compile_truss_runtime_json) and callable(weft.compile_truss_runtime_page_json)
def identity(element):return {'documentId':'domain','revision':'query-value-core-r1','module':'m','element':element}
def binding(child):
 packet=json.loads(child.native_request_json());values=[]
 for operation in packet['operations']:
  for operand in operation['operands']:
   v=operand['input']
   if v['kind']!='field':continue
   action=v['originalAction'];values.append({'path':operand['path'],'scan':v['scan'],'field':v['identity'],'column':v['identity']['element'],'domain':v['identity'],'comparisonScope':'a'*64,'mode':'original-authorized' if action else 'disclosed','originalAction':action,'exposure':None})
 for projection in packet['projections']:
  field=projection['field'];salary=field['element']=='salary'
  values.append({'path':projection['path'],'scan':projection['scan'],'field':field,'column':'salary_public' if salary else field['element'],'domain':identity('salaryMaskLabel') if salary else field,'comparisonScope':'a'*64,'mode':None,'originalAction':None,'exposure':'transformed' if salary else 'original'})
 scans=[{'scan':s['scan'],'target':s['record'],'relation':{'schema':'private_stage','table':'current_values','eligibleColumn':'eligible','rowKeyColumn':'private_key'}} for s in packet['scans']]
 return packet,{'version':'weft.postgresql.current-values/0.2-candidate','sourceRequestSha256':packet['sourceRequestSha256'],'designationSha256':packet['designationSha256'],'sourceBindingSha256':packet['binding']['sha256'],'scans':scans,'values':values}
def produce(request,version,method):
 raw=json.dumps(request,separators=(',',':'));parent=weft.prepare_application_lineage(raw);child=parent.prepare_current_query();assert child.source_preparation() is parent
 packet,declaration=binding(child);declaration['version']='weft.postgresql.current-values/'+version+'-candidate'
 compiled=json.loads(getattr(child,method)(json.dumps(declaration,separators=(',',':'))));assert compiled['authority'] is False and compiled['securityReady'] is False and compiled['nativeAdmission']=='required'
 assert compiled['sourceRequestSha256']==digest(raw.encode()) and compiled['designationSha256']==digest(child.designation_json().encode())
 return child,packet,declaration,compiled
request=json.loads((fixture/'mixed-original-uses.request.json').read_text());child,packet,declaration,compiled=produce(request,'0.2','lower_postgresql_current_rows_candidate_json')
assert type(child) is weft.CurrentQueryPreparation and packet['version']=='weft.current-query.native-input/0.1-candidate'
assert compiled['version']=='weft.postgresql.current-scalar/0.2-candidate' and len(compiled['resultColumns'])==2 and compiled['privateResultColumns'][0]['position']==2
assert compiled['resultColumns'][0]['logicalType']['family']=='string' and compiled['publicCellSources'][0]['field']['element']=='salary'
assert 'salary_public' in compiled['sql'] and '$1::pg_catalog.int8' in compiled['sql'] and compiled['parameters'][0]['value']=='100'
checks.append({'id':'original-source-custody-and-abi2-masked-read-raw-predicate','compiledSha256':digest(json.dumps(compiled,sort_keys=True).encode())})
request['sql']='SELECT COUNT(*) AS n FROM Resource r';child,packet,declaration,compiled=produce(request,'0.3','lower_postgresql_current_contributors_candidate_json')
assert compiled['version']=='weft.postgresql.current-scalar/0.3-candidate' and compiled['publicCellSources'][0]['kind']=='count' and compiled['resultColumns'][0]['logicalType']['facets']=={}
checks.append({'id':'abi3-complete-count-bag','compiledSha256':digest(json.dumps(compiled,sort_keys=True).encode())})
request=json.loads((fixture/'live-original-integer-sum.request.json').read_text());child,packet,declaration,compiled=produce(request,'0.4','lower_postgresql_current_sum_candidate_json')
assert packet['version']=='weft.current-query.native-input/0.2-candidate' and packet['capabilities']==['live-original-integer-sum'] and packet['routes'][0]['required']==['nullity']
assert compiled['version']=='weft.postgresql.current-scalar/0.4-candidate' and compiled['publicCellSources'][0]['kind']=='sum' and compiled['resultColumns'][0]['encoding']=='native-numeric-integer-nullable-text'
checks.append({'id':'explicit-abi4-original-sum','compiledSha256':digest(json.dumps(compiled,sort_keys=True).encode())})
# Exercise current UMF source through the same public native child and lowerer.
current=json.loads((fixture/'mixed-original-uses.request.json').read_text())
for module in current['modules']:
 document=json.loads(module['documentJson']);document['umf']='0.10.0'
 module['documentJson']=json.dumps(document,separators=(',',':'))
 module['pin']['umfVersion']='0.10.0';module['pin']['sha256']=digest(module['documentJson'].encode())
for source in current['opaqueSources']:
 if source['role']=='query-designation':
  designation=json.loads(source['utf8']);designation['modelPins']=[module['pin'] for module in current['modules']]
  source['utf8']=json.dumps(designation,separators=(',',':'));source['sha256']=digest(source['utf8'].encode())
current_child,current_packet,current_declaration,current_compiled=produce(current,'0.2','lower_postgresql_current_rows_candidate_json')
assert current_compiled['resultColumns'][0]['logicalType']['family']=='string'
checks.append({'id':'core010-exact-source-current-query-and-abi2','compiledSha256':digest(json.dumps(current_compiled,sort_keys=True).encode())})
presence=json.loads((root/'tests/current-query-presence/fixtures/unadmitted-context.request.json').read_text())
for module in presence['modules']:
 document=json.loads(module['documentJson']);document['umf']='0.10.0'
 module['documentJson']=json.dumps(document,separators=(',',':'))
 module['pin']['umfVersion']='0.10.0';module['pin']['sha256']=digest(module['documentJson'].encode())
for source in presence['opaqueSources']:
 if source['role']=='presence-designation':
  designation=json.loads(source['utf8']);designation['modelPins']=[module['pin'] for module in presence['modules']]
  source['utf8']=json.dumps(designation,separators=(',',':'));source['sha256']=digest(source['utf8'].encode())
presence_raw=json.dumps(presence,separators=(',',':'));presence_parent=weft.prepare_application_lineage(presence_raw)
presence_child=presence_parent.prepare_current_presence();graph=json.loads(presence_child.graph_requirements_json())
assert graph['sourceRequestJson']==presence_raw and graph['sourceRequestSha256']==digest(presence_raw.encode())
checks.append({'id':'core010-exact-source-presence-graph','graphSha256':digest(presence_child.graph_requirements_json().encode())})
declaration['version']='weft.postgresql.current-values/0.2-candidate'
try:child.lower_postgresql_current_rows_candidate_json(json.dumps(declaration))
except ValueError as error:assert str(error).startswith('WFT-CURRENT-SCALAR-SQL:')
else:raise AssertionError('New context downgraded to ABI2')
checks.append({'id':'no-context-downgrade','refused':True})
assert pins()==before
try:source_commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True,stderr=subprocess.DEVNULL).strip()
except subprocess.CalledProcessError:source_commit=None
receipt={'scope':'Exact installed standard wheel current-query source/conditional SQL APIs only; no database/native permission or security qualification','sourceCommit':source_commit,'version':version,'wheel':a.wheel.name,'pinsBeforeAfter':before,'python':sys.version,'checks':checks,'nativeAdmission':False,'securityReady':False}
a.output.write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'status':'passed','version':version,'checks':len(checks)}))
