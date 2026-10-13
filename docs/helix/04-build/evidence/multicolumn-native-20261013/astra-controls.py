from pathlib import Path
import copy,json,hashlib,subprocess
out=Path(__file__).parent;binary=Path('/private/tmp/weft-042-release-target-a/debug/examples/security_predicate_spike');legacy=json.loads((out/'baseline-input.json').read_bytes())
def ref(n):return {'documentId':'domain','moduleId':'m','elementId':n}
def spec(field,name):return {'target':ref('Resource'),'field':ref(field),'outputName':name}
p=copy.deepcopy(legacy);p['request']['sql']='SELECT r.resourceId AS id,r.salary AS amount FROM Resource r';old=p['resultProbe'];p['resultProbe']={'columns':[spec('resourceId','id'),spec('salary','amount')],'rows':[[{'outcomeId':'original','disposition':'original','value':{'string':'r1'}},{'outcomeId':'withheld','disposition':'withheld'}]],'factCut':old['factCut'],'scopedFacts':old['scopedFacts']}
results=[]
def invoke(name,p,positive=True,facts=True,raw=None):
 r=subprocess.run([str(binary)],input=raw or json.dumps(p,separators=(',',':'),ensure_ascii=False).encode(),capture_output=True,timeout=45)
 assert r.returncode==(0 if positive else 1),(name,r.returncode,r.stderr[:1500])
 if positive:
  v=json.loads(r.stdout);assert v['resultCellInspectionPassed'] is True and v['securityReady'] is False;assert v.get('conditionalFactSelectionPassed',False)==facts
 else:assert r.stdout==b'';v=json.loads(r.stderr)
 results.append({'case':name,'passed':True,'exit':r.returncode,'diagnostic':v.get('code')});return v
invoke('pairedColumnsAndFacts',p)
q=copy.deepcopy(p);q['resultProbe'].pop('factCut');q['resultProbe'].pop('scopedFacts');invoke('pairedColumnsCellsOnly',q,facts=False)
one=copy.deepcopy(legacy);one['resultProbe']={'columns':[spec('salary','amount')],'rows':one['resultProbe']['rows'],'factCut':one['resultProbe']['factCut'],'scopedFacts':one['resultProbe']['scopedFacts']};invoke('oneColumnArrayAndFacts',one)
for name,change in [
 ('emptyColumns',lambda x:x['resultProbe'].update(columns=[])),
 ('257Columns',lambda x:x['resultProbe'].update(columns=[spec('resourceId','id')]*257)),
 ('columnsObject',lambda x:x['resultProbe'].update(columns={})),
 ('columnsNull',lambda x:x['resultProbe'].update(columns=None)),
 ('specExtra',lambda x:x['resultProbe']['columns'][0].update(extra=True)),
 ('specMissing',lambda x:x['resultProbe']['columns'][0].pop('target')),
 ('specPositional',lambda x:x['resultProbe']['columns'].__setitem__(0,list(x['resultProbe']['columns'][0].values()))),
 ('specNull',lambda x:x['resultProbe']['columns'].__setitem__(0,None)),
 ('specNameNumber',lambda x:x['resultProbe']['columns'][0].update(outputName=1)),
 ('specReferenceExtra',lambda x:x['resultProbe']['columns'][0]['field'].update(extra=True)),
 ('mixedLegacyTarget',lambda x:x['resultProbe'].update(target=ref('Resource'))),
 ('mixedLegacyField',lambda x:x['resultProbe'].update(field=ref('resourceId'))),
 ('probeExtra',lambda x:x['resultProbe'].update(extra=True)),
 ('missingRows',lambda x:x['resultProbe'].pop('rows')),
 ('missingCut',lambda x:x['resultProbe'].pop('factCut')),
 ('missingScopedFacts',lambda x:x['resultProbe'].pop('scopedFacts')),
 ('nullCut',lambda x:x['resultProbe'].update(factCut=None)),
 ('swappedCells',lambda x:x['resultProbe']['rows'][0].reverse()),
 ('swappedColumns',lambda x:x['resultProbe']['columns'].reverse()),
 ('missingCell',lambda x:x['resultProbe']['rows'][0].pop()),
 ('extraCell',lambda x:x['resultProbe']['rows'][0].append(copy.deepcopy(x['resultProbe']['rows'][0][0]))),
 ('wrongName',lambda x:x['resultProbe']['columns'][1].update(outputName='other')),
 ('wrongField',lambda x:x['resultProbe']['columns'][1].update(field=ref('resourceId'))),
 ('withheldCarriesValue',lambda x:x['resultProbe']['rows'][0][1].update(value=None)),
 ('firstOriginalForgery',lambda x:x['resultProbe']['rows'][0][0].update(value={'string':'wrong'})),
 ]:
 q=copy.deepcopy(p);change(q);invoke(name,q,False)
q=copy.deepcopy(p);q['resultProbe']['rows']*=2;q['resultProbe']['scopedFacts']['rows']*=2;invoke('twoMatchingRows',q)
q=copy.deepcopy(q);q['resultProbe']['rows'][1]=copy.deepcopy(q['resultProbe']['rows'][1]);q['resultProbe']['rows'][1][0]['value']={'string':'late-forgery'};invoke('lateSecondRowForgery',q,False)
# Repeated columns under distinct explicit aliases are legitimate original query occurrences.
def repeated(count):
 q=copy.deepcopy(p);q['request']['sql']='SELECT '+','.join('r.resourceId AS c'+str(i) for i in range(count))+' FROM Resource r';q['resultProbe']={'columns':[spec('resourceId','c'+str(i)) for i in range(count)],'rows':[[{'outcomeId':'original','disposition':'original','value':{'string':'r1'}} for _ in range(count)]]};return q
invoke('256OriginalColumns',repeated(256),facts=False)
# Exact source-copy reservation uses serialized owner input/output sizes, before per-column construction.
q=repeated(8);no_probe={k:v for k,v in q.items() if k!='resultProbe'}
r=subprocess.run([str(binary)],input=json.dumps(no_probe).encode(),capture_output=True,timeout=30);assert r.returncode==0,r.stderr;rendered=json.loads(r.stdout)
def size(v):return len(json.dumps(v,separators=(',',':'),ensure_ascii=False).encode())
base_bytes=size(q['request'])+size(rendered);pad=2000000-base_bytes;assert pad>0
for delta,positive in [(0,True),(1,False)]:
 v=copy.deepcopy(q);v['request']['bindingJson']+=' '*(pad+delta);profile=json.loads(v['request']['queryProfileJson']);profile['binding']['sha256']=hashlib.sha256(v['request']['bindingJson'].encode()).hexdigest();v['request']['queryProfileJson']=json.dumps(profile,separators=(',',':'))
 # Original queryProfileJson formatting affects serialized source size; adjust only binding whitespace until exact.
 current=size(v['request'])+size(rendered);correction=2000000+delta-current
 if correction>=0:v['request']['bindingJson']+=' '*correction
 else:v['request']['bindingJson']=v['request']['bindingJson'][:correction]
 profile['binding']['sha256']=hashlib.sha256(v['request']['bindingJson'].encode()).hexdigest();v['request']['queryProfileJson']=json.dumps(profile,separators=(',',':'))
 assert (size(v['request'])+size(rendered))*8==16000000+delta*8
 invoke('sourceCopyBudget'+str(16000000+delta*8),v,positive,facts=False)
raw=json.dumps(p,separators=(',',':')).replace('"columns":[','"columns":[],"columns":[',1).encode();invoke('duplicateColumns',p,False,raw=raw)
result={'scope':'Actual current diagnostic with original source/owner checkers; synthetic declared facts. No native SQL execution by reviewer or protected release qualification.','passedControls':len(results),'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cases':results}
(out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passedControls':len(results),'path':str(out/'results.json')}))
