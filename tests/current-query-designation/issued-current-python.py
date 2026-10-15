"""Actual frozen same-parent handoff; no native permission or SQL.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import copy,gc,hashlib,json,pathlib,pickle
import weft
root=pathlib.Path(__file__).resolve().parent;cls=weft.CurrentQueryPreparation;checks=[]
def refuses(name,call):
 try:call()
 except (TypeError,AttributeError,ValueError) as error:
  checks.append({'id':name,'refused':True});return str(error)
 raise AssertionError(name)
counts={'disclosed-token-equality':1,'disclosed-token-selfjoin':2,'original-token-order':1,'read-dto-hidden-original':1,'mixed-original-uses':2,'prohibited-salary-order-empty':1}
for name,count in counts.items():
 raw=(root/'fixtures'/f'{name}.request.json').read_text();parent=weft.prepare_application_lineage(raw);before=parent.json();child=parent.prepare_current_query()
 assert type(child) is cls and child.source_preparation() is parent and child.operand_count()==count
 assert child.designation_json().encode()==(root/'fixtures'/f'{name}.report.json').read_bytes()
 report=json.loads(child.designation_json());assert report['authority'] is False and report['securityReady'] is False
 assert parent.json()==before and parent.source_request_json()==raw
 checks.append({'id':name,'sameParent':True,'operandCount':count,'designationSha256':hashlib.sha256(child.designation_json().encode()).hexdigest()})
parent=weft.prepare_application_lineage((root/'fixtures/mixed-original-uses.request.json').read_text());child=parent.prepare_current_query();expected=child.designation_json()
for i,args in enumerate([(),(expected,),({'source':parent},),(parent,) ]):refuses('no-child-constructor:'+str(i),lambda args=args:cls(*args))
refuses('no-object-new',lambda:object.__new__(cls));refuses('no-subclass',lambda:type('ForgedCurrentQuery',(cls,),{}))
for name,value in [('original',parent),('inner',{}),('designation_json',lambda:'{}'),('operand_count',lambda:0)]:refuses('no-instance-replacement:'+name,lambda name=name,value=value:setattr(child,name,value))
refuses('no-instance-dictionary',lambda:getattr(child,'__dict__'))
for name,fn in [('copy',copy.copy),('deepcopy',copy.deepcopy),('pickle',pickle.dumps)]:refuses('no-'+name,lambda fn=fn:fn(child))
for value in [expected,{},parent,object()]:
 for name in ['source_preparation','designation_json','operand_count']:
  refuses('native-receiver:'+name+':'+type(value).__name__,lambda name=name,value=value:getattr(cls,name)(value))
for value in [None,expected,{},lambda:expected]:refuses('no-factory-input:'+type(value).__name__,lambda value=value:parent.prepare_current_query(value))
refuses('no-factory-mode',lambda:parent.prepare_current_query(mode='disclosed'))
refuses('no-child-action',lambda:child.designation_json(originalAction='read'))
refuses('no-parent-override',lambda:child.source_preparation(parent))
refuses('legacy-compiler-rejects-object',lambda:weft.compile_json(child))
legacy_result=json.loads(weft.compile_json(child.designation_json()));assert legacy_result.get('status')!='compiled' and legacy_result.get('sql') is None
checks.append({'id':'legacy-compiler-refuses-diagnostic-report','passed':True})
other=weft.prepare_application_lineage(parent.source_request_json());other_child=other.prepare_current_query()
assert other_child.designation_json()==expected and other_child.source_preparation() is other and other_child.source_preparation() is not parent
checks.append({'id':'identical-source-parent-not-object-substitution','passed':True})
for name in ['unsupported-salary-sum']:
 p=weft.prepare_application_lineage((root/'fixtures'/f'{name}.request.json').read_text());before=p.json();error=refuses('unsupported-source:'+name,p.prepare_current_query);assert error.startswith('WFT-CURRENT-QUERY-OPERANDS:') and p.json()==before
legacy=root.parent/'application-lineage/fixtures/token-equality.request.json';p=weft.prepare_application_lineage(legacy.read_text());error=refuses('legacy-source-refused',p.prepare_current_query);assert error.startswith('WFT-CURRENT-QUERY-DESIGNATION:')
# Saved native descriptors use actual private fields even if trusted host code can
# monkeypatch a class method. Native consumers must never trust Python overrides.
native=cls.designation_json
try:setattr(cls,'designation_json',lambda self:'caller report')
except (TypeError,AttributeError):checks.append({'id':'class-method-replacement-refused','passed':True})
else:
 try:assert native(child)==expected;checks.append({'id':'native-descriptor-ignores-python-replacement','passed':True})
 finally:setattr(cls,'designation_json',native)
original_id=id(parent);del parent;gc.collect()
assert id(child.source_preparation())==original_id and child.designation_json()==expected and child.operand_count()==2
checks.append({'id':'original-parent-retained-after-gc','passed':True})
print(json.dumps({'scope':'Actual source-object custody and Rust consumer hook through native Python getters only; no native source/current authority or SQL','controls':len(checks),'checks':checks},indent=2))
