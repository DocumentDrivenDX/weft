"""Independent assertions and Rust/Python parity; no JS subprocess in the extension."""
import copy, hashlib, json, os, pathlib, platform, subprocess, sys
import weft_spike
ROOT=pathlib.Path(__file__).resolve().parents[2]
OUT=ROOT/'target'/'b001';OUT.mkdir(parents=True,exist_ok=True)
corpus=json.loads((ROOT/'docs/helix/03-test/fixtures/cases.json').read_text())
def adapt(req):
 r=copy.deepcopy(req);r['target']['backendId']='spike.synthetic';return r
cases=[{'id':c['id'],'request':json.dumps(adapt(c['request']),ensure_ascii=False),'expected':c['expected']} for c in corpus if c['category']=='numeric']
base=adapt(corpus[0]['request']);base['sql']='SELECT o.total FROM Orders o WHERE o.customer_id = 18446744073709551615 AND o.total = 9007199254740993.12'
def add(id,req,code=None,values=None):
 cases.append({'id':id,'request':req if isinstance(req,str) else json.dumps(req,ensure_ascii=False),'expected':{'status':'blocked' if code else 'compiled',**({'code':code} if code else {}),**({'values':values} if values is not None else {})}})
def modified(id,modify,code=None,values=None):
 r=copy.deepcopy(base);modify(r);add(id,r,code,values)
def with_model(req,modify):
 m=req['modules'][0];d=json.loads(m['documentJson']);modify(d);m['documentJson']=json.dumps(d,ensure_ascii=False);m['pin']['sha256']=hashlib.sha256(m['documentJson'].encode()).hexdigest()
add('exact-values',base,values=['18446744073709551615','9007199254740993.12'])
modified('unknown-huge-native-number',lambda r:with_model(r,lambda d:d['extensions']['future.vendor'].update({'exact':1234567890123456789012345678901234567890})))
modified('bad-pin',lambda r:r['modules'][0]['pin'].update(sha256='0'*64),'WFT-PIN')
modified('bad-binding-pin',lambda r:r['target'].update(bindingSha256='0'*64),'WFT-PIN')
modified('sql-limit',lambda r:r.update(sql=' '*65537),'WFT-LIMIT')
modified('unsupported-join',lambda r:r.update(sql=corpus[0]['request']['sql']),'WFT-UNSUPPORTED')
modified('unsupported-wildcard',lambda r:r.update(sql='SELECT * FROM Orders o'),'WFT-UNSUPPORTED')
modified('unsupported-comment',lambda r:r.update(sql='SELECT o.total FROM Orders o -- comment'),'WFT-UNSUPPORTED')
modified('unsupported-order',lambda r:r.update(sql='SELECT o.total FROM Orders o ORDER BY o.total'),'WFT-UNSUPPORTED')
modified('unknown-selected-type',lambda r:with_model(r,lambda d:d['modules'][0]['elements'][-1].update(scalarType='future.numeric')),'WFT-TYPE')
modified('unknown-selected-facet',lambda r:with_model(r,lambda d:d['modules'][0]['elements'][-1]['facets'].update(unknown=True)),'WFT-TYPE')
modified('wrong-core-version',lambda r:with_model(r,lambda d:d.update(umf='0.6.0')),'WFT-MODEL-VERSION')
modified('missing-field',lambda r:r.update(sql='SELECT o.missing FROM Orders o'),'WFT-NAME-MISSING')
modified('wrong-alias',lambda r:r.update(sql='SELECT x.total FROM Orders o'),'WFT-NAME-MISSING')
modified('candidate-required',lambda r:r['options'].update(allowCandidate=False),'WFT-CAPABILITY')
modified('wrong-interface',lambda r:r.update(interfaceVersion='future'),'WFT-VERSION')
modified('malformed-sql',lambda r:r.update(sql="SELECT o.total FROM Orders o WHERE o.total = '"),'WFT-SYNTAX')
add('duplicate-envelope','{"sql":"a","sql":"b"}','WFT-JSON-DUPLICATE')
modified('duplicate-model-key',lambda r:(r['modules'][0].update(documentJson='{"umf":"0.7.0","umf":"0.7.0"}'),r['modules'][0]['pin'].update(sha256=hashlib.sha256(r['modules'][0]['documentJson'].encode()).hexdigest())),'WFT-JSON-DUPLICATE')
add('malformed-json','{','WFT-INPUT')
add('surrogate-json','{"sql":"\\ud800"}','WFT-INPUT')
modified('string-safety',lambda r:r.update(sql="SELECT c.name FROM Customer c WHERE c.name = 'x''; DROP TABLE objects; -- 😀 é trail '"),values=["x'; DROP TABLE objects; -- 😀 é trail "])
responses=[]
for c in cases:
 response=weft_spike.compile_json(c['request'])
 native=subprocess.run([str(ROOT/'target/debug/report')],input=c['request'],text=True,capture_output=True,check=True).stdout.strip()
 assert response==native, ('Rust/Python mismatch',c['id'])
 d=json.loads(response);e=c['expected'];assert d['status']==e['status'],(c['id'],d,e)
 if e['status']=='blocked':
  assert d['code']==e['code'],(c['id'],d,e);assert 'sql' not in d and 'parameters' not in d
 else:
  req=json.loads(c['request']);assert d['retainedDocumentJson']==req['modules'][0]['documentJson'];assert d['retainedBindingJson']==req['target']['bindingJson']
  assert d['qualification']=='synthetic-candidate'
  assert d['coreSourceSha256']==hashlib.sha256((ROOT/'spikes/b001/core/src/lib.rs').read_bytes()).hexdigest()
  assert d['cargoLockSha256']==hashlib.sha256((ROOT/'Cargo.lock').read_bytes()).hexdigest()
  if 'values' in e:assert [p['value'] for p in d['parameters']]==e['values']
 responses.append(response)
(OUT/'cases.json').write_text(json.dumps(cases,ensure_ascii=False))
(OUT/'native-responses.json').write_text(json.dumps(responses,ensure_ascii=False))
summary={'cases':len(cases),'numericCases':575,'python':sys.version.split()[0],'platform':platform.platform(),'extension':sys.modules[weft_spike.compile_json.__module__].__file__,'extensionBytes':pathlib.Path(sys.modules[weft_spike.compile_json.__module__].__file__).stat().st_size,'nativePythonOnlySmoke':'run separately by python_only.py'}
(OUT/'native-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary,indent=2))
