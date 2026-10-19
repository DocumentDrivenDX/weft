# SPDX-License-Identifier: MIT OR Apache-2.0
"""Compile the current-format 1,000-type successor; test closed capacity bounds."""
import copy,gzip,hashlib,json,time
from pathlib import Path
import weft
fixture_path=Path(__file__).with_name('genuine-thousand-type-current010-request.json.gz')
fixture_bytes=fixture_path.read_bytes()
assert hashlib.sha256(fixture_bytes).hexdigest()=='07d2dcd76b9a7651cf9da405d97abcbc7ae26a022bd6fc21afa0e2300105605a'
request=json.loads(gzip.decompress(fixture_bytes))
assert request['modules'][0]['pin']['umfVersion']=='0.10.0'
document_json=json.loads(request['modules'][0]['documentJson'])
assert document_json['umf']=='0.10.0'
native_binding=json.loads(request['target']['bindingJson'])
assert len(native_binding['records'])==1000
assert all(len(record['fields'])==12 for record in native_binding['records'])
assert native_binding['documents'][0]['sha256']==request['modules'][0]['pin']['sha256']
assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256']

def compile_request(value):
    return json.loads(weft.compile_truss_runtime_json(json.dumps(value,separators=(',',':'))))

def refused(value,code):
    result=compile_request(value)
    assert result['status']=='blocked' and result['diagnostics'][0]['code']==code,result
    assert 'sql' not in result

def binding(value,raw):
    value['target']['bindingJson']=raw
    value['target']['bindingSha256']=hashlib.sha256(raw.encode()).hexdigest()

def document(value,source):
    raw=json.dumps(source,separators=(',',':'))
    digest=hashlib.sha256(raw.encode()).hexdigest()
    value['modules'][0]['documentJson']=raw
    value['modules'][0]['pin']['sha256']=digest
    native=json.loads(value['target']['bindingJson']);native['documents'][0]['sha256']=digest
    binding(value,json.dumps(native,separators=(',',':')))
    return raw

started=time.monotonic();result=compile_request(request);elapsed=time.monotonic()-started
assert result['status']=='compiled',result
# The presence/Graph release adds one explicitly qualified engine-navigation
# language profile to each operation declaration. Assert that addition, then
# compare every other result byte against the reviewed current-0.10 successor hash.
baseline=copy.deepcopy(result)
operations=baseline['qualification']['operations']
assert len(operations)==15
for operation in operations:
    profiles=operation['declaration']['languageProfiles']
    added={'dialectProfile':'weft-sql/0.2.0','irVersion':'weft-ir/engine-navigation/0.1'}
    assert profiles.count(added)==1,profiles
    profiles.remove(added)
assert hashlib.sha256(json.dumps(baseline,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()=='166b8cb8ca0c697e97a16a85a4fe14897a1d29ca3e13cca8d0ee0e1fb1860bbb'
assert result['bindingSha256']==request['target']['bindingSha256']
assert result['modelPins']==[item['pin'] for item in request['modules']]
assert any(p.get('value')==request['target']['bindingJson'] for p in result['parameters'])
# Existing registrations keep their old parse budget and their refusal.
old=copy.deepcopy(request);old['target']['backendVersion']='0.4.40';old['target']['targetProfile']='truss-runtime-pg18.6/0.4'
refused(old,'WFT-LIMIT')
# Larger bounded inputs must still reject duplicate JSON members before lowering.
duplicate=copy.deepcopy(request)
binding(duplicate,request['target']['bindingJson'][:-1]+',"schema":"truss"}')
refused(duplicate,'WFT-JSON-DUPLICATE')
# A compact model can reach the value limit while remaining inside four MiB.
over=copy.deepcopy(request);source=json.loads(over['modules'][0]['documentJson'])
source['extensions']['capacity-probe']=[0]*821_000
raw=document(over,source);assert len(raw.encode())<=4*1024*1024
refused(over,'WFT-LIMIT')
# Four MiB remains a byte bound, including UTF-8 whitespace.
over=copy.deepcopy(request);binding(over,request['target']['bindingJson']+' '*(4*1024*1024+1-len(request['target']['bindingJson'].encode())))
refused(over,'WFT-LIMIT')
print(json.dumps({'status':'passed','compiler':weft.__version__,'records':1000,'fieldsPerRecord':12,'compileSeconds':elapsed,'controls':5}))
