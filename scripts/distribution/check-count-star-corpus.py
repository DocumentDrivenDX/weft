"""Bounded actual CLI corpus/witness evidence; no release index or native engine."""
import argparse,gzip,hashlib,importlib.util,json,sys,types,zlib
from pathlib import Path
from jsonschema import Draft202012Validator
from referencing import Registry,Resource
p=argparse.ArgumentParser()
for name in ('binary','cases','backend','schemas','output'):p.add_argument('--'+name,type=Path,required=True)
a=p.parse_args()
if not all(path.is_absolute() for path in vars(a).values()):raise ValueError('absolute-config')
root=Path(__file__).parent
s=importlib.util.spec_from_file_location('bounded_io',root/'check-paths-embedding-python.py');io=importlib.util.module_from_spec(s);s.loader.exec_module(io)
s=importlib.util.spec_from_file_location('bounded_transport',root/'check-paths-cli.py');transport=importlib.util.module_from_spec(s);sys.modules[s.name]=transport;s.loader.exec_module(transport)
raw=io.read(a.cases,1024*1024);decoder=zlib.decompressobj(16+zlib.MAX_WBITS);decoded=decoder.decompress(raw,32*1024*1024+1)
if len(decoded)>32*1024*1024 or not decoder.eof or decoder.unused_data or decoder.unconsumed_tail:raise ValueError('corpus-compression')
bundle=json.loads(decoded);backend=json.loads(io.read(a.backend,1024*1024))
caps={c['id'] for c in backend['capabilities']}
if len(caps)!=49 or len(backend['capabilities'])!=49 or set(bundle['coverage'])!=caps or backend['backendVersion']!='0.4.1-count-star-having-candidate' or backend['languageProfiles']!=[{'dialectProfile':'weft-sql/0.4.1','irVersion':'weft-ir/0.4.1'}]:raise ValueError('exact49profile')
if len(bundle['cases'])!=80 or len({c['id'] for c in bundle['cases']})!=80:raise ValueError('complete-case-inventory')
schemas={}
for path in sorted(a.schemas.glob('*.schema.json')):
 value=json.loads(io.read(path,1024*1024));schemas[path.name]=value
# Registry is closed: an unregistered reference raises instead of network fetch.
registry=Registry().with_resources((v['$id'],Resource.from_contents(v)) for v in schemas.values())
validators={name:Draft202012Validator(value,registry=registry) for name,value in schemas.items()}
limits=types.SimpleNamespace(timeout_seconds=15,maximum_response_bytes=4*1024*1024,process_group_owner='transport')
records=[];observed={}
observation_path=Path(str(a.output)+'.observations.jsonl.gz')
with observation_path.open('xb'):pass
observation_bytes=0;compressed_bytes=0
def retain_observation(value):
 global observation_bytes,compressed_bytes
 line=(json.dumps(value,separators=(',',':'))+'\n').encode()
 observation_bytes+=len(line)
 if len(line)>40*1024*1024 or observation_bytes>96*1024*1024:raise ValueError('observation-decoded-bound')
 member=gzip.compress(line,mtime=0);compressed_bytes+=len(member)
 if compressed_bytes>16*1024*1024:raise ValueError('observation-compressed-bound')
 # Each observation is a complete gzip member, closed before comparison.
 with observation_path.open('ab') as f:f.write(member)

for c in bundle['cases']:
 request=bytes.fromhex(c['requestHex'])
 if len(request)>16*1024*1024:raise ValueError('request-limit')
 outputs=[]
 for _ in range(2):
  code,out,err=transport.transport(a.binary,request,'normal',limits)
  retain_observation({'id':c['id'],'repeat':_,'requestHex':request.hex(),'responseHex':out.hex(),'exit':code,'stderrHex':err.hex()})
  if code or err or not out.endswith(b'\n'):raise ValueError('transport-framing')
  outputs.append(out)
 if outputs[0]!=outputs[1]:raise ValueError('deterministic-bytes')
 response=json.loads(outputs[0])
 if c['expectation']=='exact-migrated-bytes':
  if outputs[0]!=bytes.fromhex(c['responseHex']):raise ValueError('exact-migrated-response:'+c['id'])
 elif response!=c['expectedResponse']:raise ValueError('independent-response:'+c['id'])
 version=response.get('interfaceVersion')
 name='compile-response-v0.4.1.schema.json' if version=='weft-compile/0.4.1' else 'compile-response-v0.4.schema.json'
 validators[name].validate(response)
 required=response.get('logicalPlan',{}).get('requiredCapabilities',[])
 codes=[d['code'] for d in response.get('diagnostics',[])]
 if response['status']=='compiled':
  if response.get('backend',{}).get('backendVersion')!='0.4.1-count-star-having-candidate' or response.get('logicalPlan',{}).get('irVersion')!='weft-ir/0.4.1':raise ValueError('compiled-new-profile')
  validators['compile-request-v0.4.1.schema.json'].validate(json.loads(request))
 observed[c['id']]={'status':response['status'],'requiredCapabilities':required,'diagnosticCodes':codes}
 records.append({'id':c['id'],'requestHex':request.hex(),'responseHex':outputs[0].hex(),'responseSha256':hashlib.sha256(outputs[0]).hexdigest(),**observed[c['id']]})
coverage={}
for cap,item in bundle['coverage'].items():
 for identity in item['accepted']:
  if observed[identity]['status']!='compiled' or cap not in observed[identity]['requiredCapabilities']:raise ValueError('accepted-capability-witness')
 for identity in item['refused']:
  if observed[identity]['status']!='blocked':raise ValueError('refused-case-witness')
 coverage[cap]={'accepted':item['accepted'],'refused':item['refused'],'notExercised':not(item['accepted'] or item['refused']),'scope':item['scope']}
receipt={'format':'weft-count-star-cli-corpus-evidence/0.1','sourceCommit':bundle['sourceCommit'],'casesInputSha256':hashlib.sha256(raw).hexdigest(),'backendInputSha256':hashlib.sha256(io.read(a.backend,1024*1024)).hexdigest(),'binarySha256':hashlib.sha256(io.read(a.binary,32*1024*1024)).hexdigest(),'cases':records,'coverage':coverage,'protocolCases':80,'declaredCapabilities':49,'qualification':'Actual CLI full authored 80-case correspondence with49 capability accepted/refused witnesses; closed offline response schemas and compiled-request schemas. No native SQL, full transport boundary suite, installed package or index admission.'}
output=(json.dumps(receipt,ensure_ascii=False,separators=(',',':'))+'\n').encode()
if len(output)>64*1024*1024:raise ValueError('receipt-limit')
with a.output.open('xb') as f:f.write(gzip.compress(output,mtime=0))
print(json.dumps({'state':'count-star-full-corpus-passed','cases':80,'capabilities':49,'receiptDecodedBytes':len(output)}))
