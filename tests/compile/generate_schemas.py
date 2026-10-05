"""Finalize draft public response schemas around the versioned registered boundary."""
import copy,json
from pathlib import Path
D=Path(__file__).resolve().parents[2]/'docs/helix/02-design/contracts'
BASE='https://github.com/DocumentDrivenDX/weft/raw/main/docs/helix/02-design/contracts/'
response=json.loads((D/'compile-response.schema.json').read_text())
manifest=json.loads((D/'backend-manifest-v0.2.schema.json').read_text())
compiled=response['oneOf'][1]['properties']
compiled['backend']['properties']['interfaceVersion']={'const':'weft-backend/0.2.0'}
compiled['targetContext']=manifest['properties']['targetProfiles']['items']
assessment={'type':'object','properties':{'id':{'type':'string','minLength':1},'status':{'enum':['supported','candidate','unsupported']},'evidence':{'type':'array','items':{'type':'string'},'uniqueItems':True},'obligations':compiled['obligations']},'required':['id','status','evidence','obligations'],'additionalProperties':False}
compiled['qualification']['properties']['operations']={'type':'array','items':{'type':'object','properties':{'assessment':assessment,'declaration':manifest['properties']['capabilities']['items']},'required':['assessment','declaration'],'additionalProperties':False},'minItems':1}
response02=copy.deepcopy(response);response02['$id']=BASE+'compile-response-v0.2.schema.json'
for branch in response02['oneOf']:branch['properties']['interfaceVersion']={'const':'weft-compile/0.2.0'}
c=response02['oneOf'][1]['properties'];c['dialect']={'const':'weft-sql/0.2.0'};c['logicalPlan']={'$ref':BASE+'logical-plan-v0.2.schema.json'}
plan=json.loads((D/'logical-plan-v0.2.schema.json').read_text());id_schema=plan['$defs']['identity'];type_schema=plan['$defs']['type']
scalar={'type':'object','properties':{'kind':{'const':'scalar'},'logicalType':type_schema,'carrier':{'enum':['text','boolean']},'decoder':{'enum':['text','boolean','exact-integer','exact-decimal']}},'required':['kind','logicalType','carrier','decoder'],'additionalProperties':False}
value={'type':'object','properties':{'kind':{'const':'value'},'descriptor':id_schema,'nativeNull':{'type':'boolean'}},'required':['kind','descriptor','nativeNull'],'additionalProperties':False}
related={'type':'object','properties':{'kind':{'const':'relatedKeys'},'relationship':plan['$defs']['relationshipIdentity'],'key':{'type':'object','properties':{'id':{'type':'string','minLength':1},'fields':{'type':'array','items':id_schema,'minItems':1},'types':{'type':'array','items':type_schema,'minItems':1}},'required':['id','fields','types'],'additionalProperties':False},'bound':{'type':'integer','minimum':1,'maximum':1000}},'required':['kind','relationship','key','bound'],'additionalProperties':False}
c['columns']={'type':'array','minItems':1,'maxItems':256,'items':{'type':'object','properties':{'position':{'type':'integer','minimum':1},'outputName':{'type':'string','minLength':1},'representation':{'oneOf':[scalar,value,related]},'sourceIdentities':{'type':'array','items':id_schema,'minItems':1},'nullable':{'type':'boolean'}},'required':['position','outputName','representation','sourceIdentities','nullable'],'additionalProperties':False}}
for name,v in [('compile-response.schema.json',response),('compile-response-v0.2.schema.json',response02)]:(D/name).write_text(json.dumps(v,indent=2)+'\n')
