"""Exact native68 owner binding compilation, not native execution/authority."""
import hashlib,json,pathlib,weft
root=pathlib.Path(__file__).parent
checks=[]
def sha(b):return hashlib.sha256(b).hexdigest()
def request(bundle,sql,version):
 raw=bundle['bindingJson']
 return {'interfaceVersion':'weft-compile/0.2.6','dialect':'weft-sql/0.2.0','timestampComparisonProfile':'umf-rfc3339-exact-instant/1.0.0','modules':bundle['modules'],'sql':sql,'parameters':{},'exactParameters':{},'options':{'allowCandidate':True},'target':{'backendId':'truss.postgresql.runtime','backendVersion':version,'targetProfile':'truss-runtime-pg18.6/0.4','bindingJson':raw,'bindingSha256':sha(raw.encode())}}
for directory in [root,root/'core08']:
 bundle=json.loads((directory/'native-query-bundle.json').read_text());manifest=(directory/'native-runtime-manifest.json').read_bytes();assert sha(manifest)==bundle['binding']['releaseSha256']=='3588cf030cb46c7d12368557e351bd767c10b4dd2e1644f7f100cec54602a2fc'
 out=json.loads(weft.compile_truss_runtime_json(json.dumps(request(bundle,'SELECT COUNT(*) FROM "A" a','0.4.37'))));assert out['status']=='blocked' and out['diagnostics'][0]['code']=='WFT-NAME-MISSING';checks.append({'id':directory.name+':no-invented-record-name','status':'blocked'})
bundle=json.loads((root/'products-native-query-bundle.json').read_text())
for query in ['SELECT p.name FROM products p','SELECT COUNT(*) FROM products p']:
 for version,expected in [('0.4.37','compiled'),('0.4.36','blocked'),('0.4.35','blocked')]:
  out=json.loads(weft.compile_truss_runtime_json(json.dumps(request(bundle,query,version))));assert out['status']==expected,(version,out);assert ('sql' in out)==(expected=='compiled');checks.append({'id':'actual-public-products-binding:'+query+':'+version,'status':out['status']})
fixture=json.loads((root.parent/'mutation/fixtures/truss066-configuration.json').read_text());compiler=weft.MutationCompiler(fixture['utf8'])
bundle=json.loads((root/'products-native-query-bundle.json').read_text());source=bundle['modules'][0]
def artifact(utf8):return {'mediaType':'application/json','utf8':utf8,'sha256':sha(utf8.encode())}
model=artifact(source['documentJson']);binding=artifact(bundle['bindingJson'])
selection=artifact(json.dumps({'protocol':'weft.update-selection/1','record':{'module':'sales','element':'products'},'key':{'module':'sales','element':'products','key':'pk'},'predicate':{'compare':{'field':{'module':'sales','element':'product-id'},'op':'eq','value':{'string':'bar'}}}},separators=(',',':')))
for digest,expected in [('3588cf030cb46c7d12368557e351bd767c10b4dd2e1644f7f100cec54602a2fc','lowered'),('0'*64,'blocked')]:
 changed=dict(bundle['binding'],releaseSha256=digest);b=artifact(json.dumps(changed,separators=(',',':')))
 handle=compiler.register_truss_selection(json.dumps({'model':model,'modelPin':source['pin'],'binding':b,'evidence':['Original owner-exported native68 Core08 model/binding compiler-only smoke']}))
 out=json.loads(compiler.lower_selection_json(handle,json.dumps({'selection':selection,'model':model,'binding':b,'maxItems':2})));assert out['status']==expected,out
 if expected=='lowered':assert 'runtime0.68' in out['obligations'][0] and out['parameters'][0]['value']=={'string':'bar'}
 checks.append({'id':'native68-mutation:'+expected,'status':out['status']})
print(json.dumps({'scope':'Original native68 owner-exported Core07/Core08 bindings through actual wheel compiler; no native query/action execution or caller authority','checks':checks,'controls':len(checks)},indent=2))
