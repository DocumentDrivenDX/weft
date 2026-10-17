import json,pathlib,hashlib
s=pathlib.Path('/private/tmp/weft-r22-historical-golden-baseline.log').read_text().split('  left: ',1)[1].split('\n right: ',1)[0];i=0;decoder=json.JSONDecoder()
def ws():
 global i
 while i<len(s)and s[i].isspace():i+=1
def take(t):
 global i
 ws();assert s.startswith(t,i),(i,t,s[i:i+20]);i+=len(t)
def string():
 global i
 ws();v,n=decoder.raw_decode(s[i:]);assert isinstance(v,str);i+=n;return v
def value():
 global i
 ws()
 if s.startswith('Object {',i):
  take('Object {');out={};ws()
  while s[i]!='}':
   k=string();take(':');out[k]=value();ws()
   if s[i]!=',':break
   take(',')
  take('}');return out
 if s.startswith('Array [',i):
  take('Array [');out=[];ws()
  while s[i]!=']':
   out.append(value());ws()
   if s[i]!=',':break
   take(',')
  take(']');return out
 if s.startswith('String(',i):take('String(');out=string();take(')');return out
 if s.startswith('Number(',i)or s.startswith('Bool(',i):
  take('Number('if s.startswith('Number(',i)else'Bool(');end=s.index(')',i);out=json.loads(s[i:end]);i=end;take(')');return out
 take('Null');return None
out=value();ws();assert i==len(s)
root=pathlib.Path('/private/tmp/weft-sdk-engine-successor-20261017');case=json.loads((root/'docs/helix/04-build/evidence/B-006-cross-module-native/compile.json').read_text());r=json.dumps(case['request'],sort_keys=True,separators=(',',':'));packet={'scope':'source-produced current R22 baseline; historical native B006 receipt retained unchanged; no wheel/native qualification transfer','sourceCommit':'5605d358650468d651f69ff29c40e986a0359a5b','features':['raw-postgresql-reads','ashlar-databricks-candidate'],'provenanceLogSha256':hashlib.sha256(pathlib.Path('/private/tmp/weft-r22-historical-golden-baseline.log').read_bytes()).hexdigest(),'actualRustDebugSha256':hashlib.sha256(s.encode()).hexdigest(),'requestSha256':hashlib.sha256(r.encode()).hexdigest(),'response':out};p=root/'tests/engine-catalog/B006-current-r22-baseline.json';p.write_text(json.dumps(packet,indent=2)+'\n');print(p,hashlib.sha256(p.read_bytes()).hexdigest())
