"""Public API corpus: preserve frontend cases; author backend availability and transport outcomes."""
import copy,json,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
original=json.loads((ROOT/'docs/helix/03-test/fixtures/cases.json').read_text())
application=json.loads((ROOT/'tests/application/fixtures/cases.json').read_text())
third=json.loads((ROOT/'tests/register-backend/fixtures/cases.json').read_text())
cases=[]
for c in original:
    code='WFT-INPUT' if c['setup'].get('schemaNegative') else ('WFT-BACKEND-MISSING' if c['expected']['status']=='compiled' else c['expected']['code'])
    cases.append(dict(id='original-'+c['id'],request=c['request'],expected=dict(status='blocked',code=code)))
for c in application:
    r=copy.deepcopy(c['request']);r.update(interfaceVersion='weft-compile/0.2.0',target=copy.deepcopy(original[0]['request']['target']))
    code='WFT-BACKEND-MISSING' if c['expected']['status']=='resolved' else c['expected']['code']
    cases.append(dict(id='application-'+c['id'],request=r,expected=dict(status='blocked',code=code)))
# Only normal/candidate context cases enter the public transport. Fault modes stay in B-003's test probe.
for c in third:
    src=c['request']
    if src.get('behavior'):continue
    r={k:copy.deepcopy(src[k]) for k in ['sql','dialect','modules']}
    r['interfaceVersion']='weft-compile/0.2.0' if r['dialect']=='weft-sql/0.2.0' else 'weft-compile/0.1.0'
    pin=r['modules'][0]['pin']
    identity=lambda id:dict(documentId=pin['documentId'],revision=pin['revision'],module='sales',element=id)
    binding=dict(pins=[m['pin'] for m in r['modules']],record=dict(identity=identity('customer'),table='fixture_customers'),field=dict(identity=identity('customer-name'),column='display_name'),extensions={'unknown':'retained'})
    raw=src.get('bindingJson',json.dumps(binding,ensure_ascii=False,separators=(',',':')))
    r['target']=dict(backendId=src.get('backendId','test.third.candidate' if src.get('candidate') else 'test.third'),backendVersion=src.get('backendVersion','0.1.0'),targetProfile=src.get('targetProfile','fixture-only'),bindingJson=raw,bindingSha256=src.get('bindingSha256',hashlib.sha256(raw.encode()).hexdigest()))
    if src.get('allowCandidate'):r['options']={'allowCandidate':True}
    expected=dict(c['expected']);expected['status']='compiled' if expected['status']=='emitted' else 'blocked'
    cases.append(dict(id='third-'+c['id'],request=r,expected=expected))
OUT=Path(__file__).parent/'fixtures';OUT.mkdir(exist_ok=True)
(OUT/'cases.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
print('Public API cases:',len(cases))
