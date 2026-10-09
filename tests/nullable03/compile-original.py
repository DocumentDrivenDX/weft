"""Explicit per-Field native-null request opt-in; original SQL/modules stay exact."""
import argparse,hashlib,json,subprocess
from pathlib import Path
FIELDS={'archaeology-sample':{'samples.parent_id'},'ecology-censor':{'observations.value','observations.threshold'},'ecology-effort':{'effort.amount'},'ecology-zero':{'effort.amount'}}
def encode(value):return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(',',':'))
def main():
 p=argparse.ArgumentParser(description=__doc__)
 for name in ('requests','compiler','output'):p.add_argument('--'+name,type=Path,required=True)
 a=p.parse_args();a.output.mkdir(exist_ok=False);cases=[]
 for case,selected in FIELDS.items():
  original=(a.requests/(case+'-request.json')).read_bytes();request=json.loads(original);binding=json.loads(request['target']['bindingJson']);homes=[]
  for record in binding['records']:
   for property in record['properties']:
    if property['logical']['element']in selected:
     property['home']['encoding']='ashlar-weft-json-native-null/0.1-candidate';homes.append(property['logical'])
  if len(homes)!=len(selected):raise ValueError('Explicit original property identity inventory differs')
  raw=encode(binding);request['target']['bindingJson']=raw;request['target']['bindingSha256']=hashlib.sha256(raw.encode()).hexdigest()
  artifact=subprocess.run([str(a.compiler)],input=encode(request).encode(),capture_output=True,timeout=30)
  (a.output/(case+'-original-request.json')).write_bytes(original);(a.output/(case+'-request.json')).write_bytes((encode(request)+'\n').encode());(a.output/(case+'-artifact.json')).write_bytes(artifact.stdout);(a.output/(case+'-stderr.txt')).write_bytes(artifact.stderr)
  response=json.loads(artifact.stdout);cases.append({'case':case,'status':response['status'],'originalSqlUnchanged':request['sql']==json.loads(original)['sql'],'originalModulesUnchanged':request['modules']==json.loads(original)['modules'],'explicitNativeNullHomes':homes,'diagnostics':response.get('diagnostics')})
 (a.output/'summary.json').write_text(json.dumps({'compilerSha256':hashlib.sha256(a.compiler.read_bytes()).hexdigest(),'cases':cases},indent=2)+'\n')
if __name__=='__main__':main()
