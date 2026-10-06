"""Freeze an explicitly selected owner proposal closure; never fetch resources."""
import json,hashlib
from pathlib import Path
SOURCE=Path('/private/tmp/claude-501/truss-spec-wt/docs/helix/02-design/contracts')
DEST=Path(__file__).parent/'upstream'
resources={}
for f in SOURCE.glob('*.schema.json'):
    s=json.loads(f.read_text())
    if '$id' in s:resources[s['$id']]=(f,s)
roots=['urn:truss:draft:value-definition:0.1.0']
needed={};pending=list(roots)
def refs(x):
    if isinstance(x,dict):
        if '$ref' in x:yield x['$ref'].split('#')[0]
        for v in x.values():yield from refs(v)
    elif isinstance(x,list):
        for v in x:yield from refs(v)
while pending:
    uri=pending.pop()
    if not uri or uri in needed:continue
    f,s=resources[uri];needed[uri]=(f,s)
    pending+=list(refs(s))
keys={uri:'resource'+str(i) for i,uri in enumerate(sorted(needed))}
def rewrite(x,base):
    if isinstance(x,list):return [rewrite(v,base) for v in x]
    if not isinstance(x,dict):return x
    out={k:rewrite(v,base) for k,v in x.items() if k not in ['$id','$schema']}
    if '$ref' in x:
        uri,_,fragment=x['$ref'].partition('#');uri=uri or base
        out['$ref']='#/$defs/'+keys[uri]+fragment
    return out
bundle={'$schema':'https://json-schema.org/draft/2020-12/schema','$defs':{keys[u]:rewrite(s,u) for u,(f,s) in needed.items()},'oneOf':[{'$ref':'#/$defs/'+keys[u]} for u in roots]}
DEST.mkdir(exist_ok=True)
(DEST/'value-definition-schema-bundle.json').write_text(json.dumps(bundle,indent=2)+'\n')
(DEST/'value-definition-source-pins.json').write_text(json.dumps([{'id':u,'file':f.name,'sha256':hashlib.sha256(f.read_bytes()).hexdigest()} for u,(f,s) in sorted(needed.items())],indent=2)+'\n')
print('Frozen owner schema resources:',len(needed))
