"""Native Python/Rust relationship transport parity; no native authority claim."""
import copy, hashlib, json, os, pathlib, subprocess
import weft
root=pathlib.Path(__file__).resolve().parents[1]
binroot=pathlib.Path(os.environ['WEFT_RELATIONSHIP_BINARIES'])
checks=[]
def record(name): checks.append(name)
def refused(fn, value):
    try: fn(value)
    except ValueError as exc:
        assert str(exc).startswith('WFT-'), str(exc)
        return
    raise AssertionError('malformed input admitted')
def rust(name,raw):
    p=subprocess.run([str(binroot/name)],input=raw.encode(),capture_output=True,check=True)
    assert not p.stderr
    return p.stdout.decode()
prepare=weft.prepare_truss_security_relationship_candidate_json
inspect=weft.inspect_truss_security_relationship_result_json
for name in ['compiler-request.json','compiler-request-observation-denied.json']:
    raw=(root/'crates/weft-postgresql/tests/security-public-relationship'/name).read_text()
    output=prepare(raw)
    assert output==rust('security_relationship_candidate',raw)
    wire=json.loads(output)
    assert wire['securityReady'] is False
    for duty in ['nativeAdmission','nativeCodecAdmission','nativePopulationAndSelection','releaseProtection']:
        assert wire[duty]=='required'
    assert 'value' not in next(p for p in wire['parameters'] if p['channel']=='subject')
    assert json.loads(weft.prepare_security_mapping_json(raw))['status']=='refused'
    record(name+': exact candidate parity, separate Subject and historical entrypoint refusal')
    operands={'version':'weft.security.operand-values/0.1-candidate','contextSha256':wire['queryContextSha256'],'scans':[{'scan':s['scan'],'rows':[]} for s in wire['operandReads']]}
    batch={'version':'weft.security.cells/0.1.0','resultContractSha256':wire['resultContractSha256'],'rows':[]}
    packet={'version':'weft.truss.security.relational-result/0.2-candidate','sourceRequestJson':raw,'sourceOutputJson':output,'operandBatchJson':json.dumps(operands),'batchJson':json.dumps(batch)}
    result=inspect(json.dumps(packet))
    assert result==rust('security_relationship_result',json.dumps(packet)).strip()
    assert json.loads(result)['securityReady'] is False
    record(name+': original-source empty-declaration result parity; no completeness claim')
    for key,value in [('sourceOutputJson',output.rstrip()),('sourceRequestJson',raw+' '),('nativeAdmission',True),('version','weft.truss.security.guarded-result/0.1-candidate')]:
        bad=copy.deepcopy(packet);bad[key]=value;refused(inspect,json.dumps(bad))
    altered=copy.deepcopy(wire);altered['executionSql']='SELECT private_fact';bad=copy.deepcopy(packet);bad['sourceOutputJson']=json.dumps(altered)+'\n';refused(inspect,json.dumps(bad))
    for removed in ['s0','t0']:
        bad=copy.deepcopy(packet);o=copy.deepcopy(operands);o['scans']=[s for s in o['scans'] if s['scan']!=removed];bad['operandBatchJson']=json.dumps(o);refused(inspect,json.dumps(bad))
    bad=copy.deepcopy(packet);b=copy.deepcopy(batch);b['resultContractSha256']='0'*64;bad['batchJson']=json.dumps(b);refused(inspect,json.dumps(bad))
    record(name+': SQL/newline/source/version/authority/scan/contract tamper refused')
    request=json.loads(raw)
    for member in ['subject','nativeAdmission','executionSql','backend']:
        bad=copy.deepcopy(request);bad[member]='caller supplied';refused(prepare,json.dumps(bad))
    record(name+': caller authority/SQL overrides refused')
for fn,limit in [(prepare,4_000_000),(inspect,32_000_000)]:
    for raw in ['{}','[]','{"version":1,"version":2}',' '* (limit+1)]: refused(fn,raw)
record('both boundaries refuse malformed, duplicate and oversized envelopes')
module=next(pathlib.Path(weft.__file__).parent.glob("*.so"))
print(json.dumps({'scope':'in-process original compiler correspondence only; no authentication/native population/current authority/publication qualification','pythonModuleSha256':hashlib.sha256(module.read_bytes()).hexdigest(),'checks':checks},indent=2))
