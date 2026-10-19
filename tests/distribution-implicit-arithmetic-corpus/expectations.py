# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent source-label/typed-operand checks; does not parse SQL or execute it."""
import json
NEWCAP='project.implicitArithmeticLabels'
def check(case,response):
 if 'expectedResponse' in case:
  if response!=case['expectedResponse']:raise ValueError('historical expected correspondence')
  return
 e=case['expectedSemantics'];q=json.loads(bytes.fromhex(case['requestHex']));raw=q['sql'].encode()
 if response['interfaceVersion']!='weft-compile/0.4.2' or response['status']!=e['status']:raise ValueError('protocol/status')
 if e['status']=='blocked':
  if [d['code']for d in response['diagnostics']]!=[e['diagnosticCode']] or 'sql'in response:raise ValueError('refusal')
  return
 if response['dialect']!='weft-sql/0.4.2' or response['backend']['backendVersion']!='0.4.2-implicit-arithmetic-labels-candidate':raise ValueError('selectedprofile')
 p=response['logicalPlan'];cols=response['columns'];outputs=p['outputs'];caps=p['requiredCapabilities']
 if p['irVersion']!='weft-ir/0.4.2' or (NEWCAP in caps)!=e['newCapability']:raise ValueError('selectedcapability')
 if [o['name']for o in outputs]!=e['outputNames'] or [c['outputName']for c in cols]!=e['outputNames'] or len(cols)!=len(outputs):raise ValueError('completeorderedoutputs')
 carriers=[c.get('carrierName',c['outputName'])for c in cols]
 if 'project.positionedOutputs' in caps and any(c.get('carrierName') is None for c in cols):raise ValueError('positionedcarrierrequired')
 if len(set(carriers))!=len(carriers) or any(type(x)is not str or not x for x in carriers) or any(type(c['position']) is not int for c in cols) or [c['position']for c in cols]!=list(range(1,len(cols)+1)):raise ValueError('ordinalcarriers')
 for exp in e.get('expressionLabels',[]):
  o=outputs[exp['position']];expr=o['expression']
  if expr['op']!='arithmetic':raise ValueError('arithmetic')
  elements=[];operators=[];identities=[]
  def walk(n):
   span=n['span'];a,b=span['start'],span['end']
   if type(a)is not int or type(b)is not int or not 0<=a<b<=len(raw):raise ValueError('spanbounds')
   raw[:a].decode();raw[a:b].decode();raw[b:].decode()
   if n['domain']!={'family':'integer'}:raise ValueError('exact originaldomain')
   k=n['kind']
   if k['op']=='binary':
    walk(k['left']);walk(k['right']);operators.append(k['operator'])
   elif k['op']=='field':
    ident=k['field']['identity']
    if ident['documentId']!='urn:umf:domain:commerce' or ident['module']!='domain':raise ValueError('originalfieldowner')
    if ident not in cols[exp['position']]['sourceIdentities']:raise ValueError('completecarrierlineage')
    if not any(m['documentId']==ident['documentId'] and m['revision']==ident['revision'] for m in response['modelPins']):raise ValueError('originalrevision')
    identities.append(ident);elements.append(ident['element'])
   else:raise ValueError('originalexpressionkind')
  node=expr['expression'];walk(node);span=node['span']
  if raw[span['start']:span['end']].decode()!=exp['text'] or o['name']!=exp['text']:raise ValueError('literalcompleteexpressionlabel')
  if elements!=exp['sourceElements'] or operators!=exp['operators']:raise ValueError('originaltypedoperand/order')
  distinct={json.dumps(i,sort_keys=True,separators=(',',':')) for i in identities}
  lineage=cols[exp['position']]['sourceIdentities']
  encoded=[json.dumps(i,sort_keys=True,separators=(',',':')) for i in lineage]
  if len(encoded)!=len(set(encoded)) or set(encoded)!=distinct:raise ValueError('exactcompletecarrierlineage')
