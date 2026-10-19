# SPDX-License-Identifier: MIT OR Apache-2.0
"""Exact wheel canonical-date compilation; native execution remains Truss-owned."""
import argparse,hashlib,importlib,json,pathlib,subprocess,tomllib,zipfile
import weft
p=argparse.ArgumentParser();p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
root=pathlib.Path(__file__).resolve().parents[2]
def sha(value):return hashlib.sha256(value).hexdigest()
def text(value):return json.dumps(value,separators=(',',':'))
native=pathlib.Path(importlib.import_module('weft.weft').__file__)
with zipfile.ZipFile(a.wheel) as archive:
    names=[n for n in archive.namelist() if n.endswith(('.so','.pyd'))];assert len(names)==1
    assert native.read_bytes()==archive.read(names[0]) and pathlib.Path(weft.__file__).read_bytes()==archive.read('weft/__init__.py')
pins=dict(wheelSha256=sha(a.wheel.read_bytes()),nativeModuleSha256=sha(native.read_bytes()),shimSha256=sha(pathlib.Path(weft.__file__).read_bytes()))
assert weft.__version__==tomllib.loads((root/'crates/weft-python/pyproject.toml').read_text())['project']['version']
bundle=json.loads((root/'tests/truss-date/native-query-bundle.json').read_bytes())
abi_raw=(root/'tests/truss-date/query-abi1.1.json').read_bytes();abi=json.loads(abi_raw)
abi_pin=dict(identity=abi['identity'],version=abi['version'],sha256=sha(abi_raw))
assert abi_pin==bundle['binding']['queryAbi'] and abi_pin['version']=='1.1'
assert json.loads(bundle['bindingJson'])==bundle['binding']
def request(sql):
    raw=bundle['bindingJson']
    return dict(interfaceVersion='weft-compile/0.2.7',dialect='weft-sql/0.2.0',timestampComparisonProfile='umf-rfc3339-exact-instant/1.0.0',modules=bundle['modules'],sql=sql,parameters={},exactParameters={},options=dict(allowCandidate=True),target=dict(backendId='truss.postgresql.runtime',backendVersion='0.4.41',targetProfile='truss-runtime-pg18.6/0.5',bindingJson=raw,bindingSha256=sha(raw.encode())))
checks=[]
def run(label,request,expected='compiled'):
    out=json.loads(weft.compile_truss_runtime_json(text(request)))
    assert out['status']==expected,(label,out)
    if expected=='compiled':
        assert out['interfaceVersion']==request['interfaceVersion'] and out['dialect']==request['dialect']
        assert out['backend']==dict(backendId=request['target']['backendId'],backendVersion=request['target']['backendVersion'],targetProfile=request['target']['targetProfile'],interfaceVersion='weft-backend/0.2.7')
        assert out['logicalPlan']['irVersion']=='weft-ir/0.2.7'
        assert out['bindingSha256']==request['target']['bindingSha256'] and out['modelPins']==[m['pin'] for m in request['modules']]
        assert any(p['value']==request['target']['bindingJson'] for p in out['parameters'])
        assert 'COLLATE "C"' in out['sql']
    else:assert 'sql' not in out and 'parameters' not in out
    checks.append(dict(id=label,status=expected));return out
out=run('literal',request("SELECT a.day FROM A a WHERE a.day='2000-02-29'"))
assert any(p['value']=='2000-02-29' and p['logicalType']==dict(kind='exactScalar',scalarType='date') for p in out['parameters'])
r=request('SELECT a.day FROM A a WHERE a.day=:day');r['exactParameters']['day']=dict(scalarType='date',value='0001-01-01')
out=run('typed-parameter',r);assert any(p['value']=='0001-01-01' and p['logicalType']==dict(kind='exactScalar',scalarType='date') for p in out['parameters'])
run('field-join',request('SELECT a.day FROM A a INNER JOIN A b ON a.day=b.day'))
out=run('invalid-date',request("SELECT a.day FROM A a WHERE a.day='1900-02-29'"),'blocked');assert out['diagnostics'][0]['code']=='WFT-DATE-DOMAIN'
r=request('SELECT a.day FROM A a');r['interfaceVersion']='weft-compile/0.2.6';run('old-envelope',r,'blocked')
r=request('SELECT a.day FROM A a');r['target']['backendVersion']='0.4.39';run('old-backend',r,'blocked')
r=request('SELECT a.day FROM A a');binding=json.loads(r['target']['bindingJson']);old=json.loads((root/'tests/truss-query-abi/fixtures/query-abi.json').read_bytes());binding['queryAbi']=dict(identity=old['identity'],version=old['version'],sha256=sha((root/'tests/truss-query-abi/fixtures/query-abi.json').read_bytes()));raw=text(binding);r['target']['bindingJson']=raw;r['target']['bindingSha256']=sha(raw.encode());out=run('old-abi',r,'blocked');assert out['diagnostics'][0]['code']=='WFT-BINDING'
assert pins==dict(wheelSha256=sha(a.wheel.read_bytes()),nativeModuleSha256=sha(native.read_bytes()),shimSha256=sha(pathlib.Path(weft.__file__).read_bytes()))
report=dict(version=weft.__version__,sourceCommit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),wheel=a.wheel.name,pinsBeforeAfter=pins,controls=len(checks),checks=checks,queryAbi=abi_pin,nativeAdmission=False,securityReady=False,scope='Exact-wheel compiler checks over genuine paired-candidate Truss source; no native action/security/performance qualification')
a.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'controls':len(checks),'nativeAdmission':False}))
