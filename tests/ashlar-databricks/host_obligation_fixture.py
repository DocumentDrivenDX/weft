"""Test-only buffered host executor; callbacks supply independently trusted custody.
No production resolver, policy service, database connection or implicit fallback.
"""
import json
class Refused(Exception):pass
def require(condition):
 if not condition:raise Refused('WFT-OBLIGATION')
REQUIREMENTS=[
 'authenticate effective caller and establish complete authorized input view',
 'verify one complete immutable manifest with trusted UUID and all schema revisions/projection dependencies',
 'verify every consumed native schema, table UUID, version and retained data-file custody',
 'verify source/type/id uniqueness, typed endpoint integrity and complete serving projection coverage at this publication',
 'check policy and pin custody before execution and again before buffered result publication',
 'refuse unknown obligations or unavailable evidence; no latest version or broader principal fallback',
 'validate original mapping correspondence independently; identifiers and physical IDs are not logical identity']
PAYLOAD=['exact validated JSON text with duplicate-key refusal','well-formed Unicode scalar strings with no NUL','registered fixed-base-ten numeric carrier; exponent/DOUBLE representations refuse','no rounding, coercion, missing/null substitution or hidden corrupt inputs']
PROFILE=dict(versionReported='4.2.0 zero build hash',warehouseRelease='unqualified',comparison='UTF8_BINARY',arithmetic='ANSI exact-or-error')
FLAGS=['completeVisibility','manifestVerified','schemasVerified','retainedDataFiles','sourceIdentityVerified','endpointIntegrityVerified','projectionCoverageVerified','payloadValidated','mappingVerified','lifecycleVerified']
def strict_json(raw):
 def pairs(items):
  value={}
  for k,v in items:require(k not in value);value[k]=v
  return value
 def forbidden(value):raise Refused('invalid JSON constant')
 value=json.loads(raw,object_pairs_hook=pairs,parse_constant=forbidden)
 def unicode(v):
  if isinstance(v,str):require('\0' not in v and all(not 0xd800<=ord(c)<=0xdfff for c in v))
  elif isinstance(v,list):
   for x in v:unicode(x)
  elif isinstance(v,dict):
   for k,x in v.items():unicode(k);unicode(x)
 unicode(value);return value
def execute(artifact,snapshot,query,session):
 try:
  require(artifact['status']=='compiled' and artifact['qualification']['status']=='candidate')
  require(set(session)=={'caller','policyRevision'} and all(isinstance(v,str) and v for v in session.values()))
  guards=[];publication=None;seen=set()
  for o in artifact['obligations']:
   require(set(o)=={'id','owner','parameters','failureCode'} and o['owner']=='host' and o['id'] not in seen);seen.add(o['id'])
   p=o['parameters'];id=o['id']
   if id=='ashlar.candidate.publication':
    require(o['failureCode']=='WFT-OBLIGATION')
    require(set(p)=={'publication','modelPins','layoutRevision','layoutSha256','requirements','nativeProfile','payloadValidation','visibility','publicationPhase'})
    require(p['requirements']==REQUIREMENTS and p['payloadValidation']==PAYLOAD and p['nativeProfile']==PROFILE)
    require(p['visibility']=='hidden input is not evidence of absence or integrity' and p['publicationPhase']=='after-complete-buffer-and-context-recheck')
    require(p['modelPins']==artifact['modelPins']);publication=p;continue
   fixed=dict(phase='before-user-query',success='one exact STRING count equal to 0 per check',samePublicationRequired=True)
   if id=='ashlar.candidate.scalarIntegrity':fixed.update(parameters='same emitted ordered slots; values never interpolated',noPartialPublication=True);code='WFT-NUMERIC-DOMAIN';extra={'field','record'}
   elif id=='ashlar.candidate.compoundIntegrity':fixed.update(encoding='ashlar-weft-json-value/0.1-candidate',nativeNull=False,numericLeaves='exact strings',limits=dict(depthExclusive=128,nodesPerValue=100000),execution='native recursion/resource errors refuse atomically; no partial values');code='WFT-OBLIGATION';extra={'field','limits','encoding'}
   elif id=='ashlar.candidate.relationshipIntegrity':fixed.update(policy='complete authorized source and target inputs; inverse traversal cannot broaden authority',multiplicity='parallel edges retained; min/max checked in authored orientation',lifecycle='host verifies the authored lifecycle and projection coverage against original publication');code='WFT-BINDING';extra={'relationship'}
   elif id=='ashlar.candidate.keyIntegrity':
    fixed=dict(phase='before-user-query',success='one exact STRING count equal to 0',continuation='same immutable publication, original binding and effective policy across pages; ORDER BY alone proves no continuity',key=p['key']);code='WFT-BINDING';extra=set()
   else:raise Refused('unknown obligation')
   require(o['failureCode']==code and set(p)==set(fixed)|{'checks'} and all(p[k]==v for k,v in fixed.items()))
   require(isinstance(p['checks'],list))
   for check in p['checks']:
    require({'sql','failureCode'}<=set(check)<=extra|{'sql','failureCode'})
    require(check['failureCode'] in ['WFT-BINDING','WFT-OBLIGATION','WFT-NUMERIC-DOMAIN'] and isinstance(check['sql'],str) and check['sql'])
    if 'limits' in check:require(check['limits']==dict(depthExclusive=128,nodes=100000))
    if 'encoding' in check:require(check['encoding']=='ashlar-weft-json-value/0.1-candidate')
    guards.append(check)
  require(publication is not None and 'ashlar.candidate.scalarIntegrity' in seen)
  def validate(v):
   require(set(v)==set(FLAGS)|{'effectiveCaller','authenticatedCaller','policyRevision','publication','modelPins','bindingSha256','layoutRevision','layoutSha256','targetContext'})
   require(all(v[k] is True for k in FLAGS))
   require(v['effectiveCaller']==v['authenticatedCaller']==session['caller'] and v['policyRevision']==session['policyRevision'])
   for k in ['publication','modelPins','layoutRevision','layoutSha256']:require(v[k]==publication[k])
   require(v['bindingSha256']==artifact['bindingSha256'] and v['targetContext']==artifact['targetContext'])
  initial=snapshot();validate(initial)
  def fetch(sql,columns):
   r=query(sql,artifact['parameters']);require(set(r)=={'rows','types','complete','truncated'} and r['complete'] is True and r['truncated'] is False)
   require(r['types']==['STRING']*columns and isinstance(r['rows'],list) and all(isinstance(row,list) and len(row)==columns for row in r['rows']))
   return r['rows']
  for guard in guards:require(fetch(guard['sql'],1)==[['0']])
  rows=fetch(artifact['sql'],len(artifact['columns']))
  for row in rows:
   for value,col in zip(row,artifact['columns']):
    require(value is None and col['nullable'] or isinstance(value,str))
    if artifact['interfaceVersion']=='weft-compile/0.2.0':
     require('representation' in col);kind=col['representation']['kind']
    else:
     require(artifact['interfaceVersion']=='weft-compile/0.1.0' and 'logicalType' in col and 'decoder' in col);kind='scalar'
    if value is not None and kind in ['value','entity','relatedKeys']:strict_json(value)
  current=snapshot();validate(current);require(current==initial)
  return rows
 except Refused:raise
 except Exception as error:raise Refused('host refused execution/publication') from error
