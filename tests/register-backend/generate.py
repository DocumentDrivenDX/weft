"""Author independent third-backend cases; no compiler output is used."""
import copy,hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
original=json.loads((ROOT/'docs/helix/03-test/fixtures/cases.json').read_text())
modules=original[0]['request']['modules']
rows=['Alice','Alice','é','é','trail','trail ',"", "'; DROP TABLE fixture_customers; --",'😀']
cases=[]
def add(id,sql='SELECT c.name AS label FROM Customer c',code=None,**options):
    r=dict(sql=sql,dialect='weft-sql/0.1.0',modules=copy.deepcopy(modules))
    r.update(options)
    cases.append(dict(id=id,request=r,expected=dict(status='blocked' if code else 'emitted',**({'code':code}if code else {'rows':[[s] for s in rows]}))))
add('required-string-01')
add('required-string-02',dialect='weft-sql/0.2.0')
add('quoted-label','SELECT c.name AS "😀label" FROM Customer c')
add('injection-label','SELECT c.name AS "label""; DROP TABLE x; --" FROM Customer c')
add('candidate-opt-in',candidate=True,allowCandidate=True)
add('candidate-default-refusal',candidate=True,code='WFT-CAPABILITY')
add('missing-backend',backendId='not.registered',code='WFT-BACKEND-MISSING')
add('stale-backend',backendVersion='0.0.0',code='WFT-BACKEND-VERSION')
add('missing-target-profile',targetProfile='unknown',code='WFT-BACKEND-VERSION')
for behavior,code in [('missing-coverage','WFT-BINDING'),('missing-assessment','WFT-CAPABILITY'),('empty-sql','WFT-EMIT'),('wrong-label','WFT-EMIT'),('wrong-type','WFT-EMIT'),('wrong-carrier','WFT-EMIT'),('bad-slots','WFT-EMIT'),('parameter-lexical','WFT-EMIT'),('lower-failure','WFT-CAPABILITY')]:add(behavior,behavior=behavior,code=code)
add('unsupported-filter',"SELECT c.name FROM Customer c WHERE c.name='Alice'",code='WFT-CAPABILITY')
pin=modules[0]['pin']
def identity(element):return dict(documentId=pin['documentId'],revision=pin['revision'],module='sales',element=element)
def binding():return dict(pins=[m['pin'] for m in modules],record=dict(identity=identity('customer'),table='fixture_customers'),field=dict(identity=identity('customer-name'),column='display_name'),extensions={'unknown':'retained'})
def with_binding(id,edit,code=None,sql='SELECT c.name AS label FROM Customer c'):
    b=binding();edit(b);raw=json.dumps(b,ensure_ascii=False,separators=(',',':'))
    add(id,sql=sql,code=code,bindingJson=raw,bindingSha256=hashlib.sha256(raw.encode()).hexdigest())
with_binding('unsupported-numeric',lambda b:b['field'].update(identity=identity('customer-id'),column='numeric_id'),code='WFT-CAPABILITY',sql='SELECT c.id FROM Customer c')
with_binding('alternate-home',lambda b:b['field'].update(column='renamed_name'))
with_binding('alternate-table',lambda b:b['record'].update(table='fixture_customers_v2'))
with_binding('selected-sql-fragment',lambda b:b['field'].update(sql='SELECT * FROM privileged'),code='WFT-BINDING')
with_binding('identifier-injection',lambda b:b['record'].update(table='fixture_customers; DROP TABLE x'),code='WFT-BINDING')
with_binding('stale-model-revision',lambda b:b['field']['identity'].update(revision='stale'),code='WFT-BINDING')
add('bad-binding-digest',bindingJson=json.dumps(binding()),bindingSha256='0'*64,code='WFT-PIN')
OUT=Path(__file__).parent/'fixtures';OUT.mkdir(exist_ok=True)
(OUT/'cases.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
(OUT/'rows.json').write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
print('Authored third-backend cases:',len(cases))
