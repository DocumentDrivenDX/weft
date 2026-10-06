"""Run all authored application decisions through the actual candidate compiler.
Frontend refusals keep their authored diagnostics; backend refusals of resolved
inputs are reported as unresolved coverage, never relabeled successes.
"""
import copy,hashlib,importlib.util,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('physical',HERE/'generate-compiler-fixtures.py')
physical=importlib.util.module_from_spec(spec);spec.loader.exec_module(physical)
source=json.loads((ROOT/'tests/application/fixtures/cases.json').read_text())
reports=[];coverage=[]
for case in source:
    for home in ['props','row']:
        request=copy.deepcopy(case['request'])
        # Deliberately invalid models can never reach mapping use. Preserve their
        # original input and pin a custody bundle without inventing selected IDs.
        try:binding,_=physical.make(request,home)
        except (KeyError,ValueError,TypeError):
            assert case['expected']['status']=='blocked',case['id']
            binding=copy.deepcopy(physical.BASE)
            binding['basis']['modelBundle']=physical.artifact(request.get('modules',[]))
        raw=json.dumps(binding,ensure_ascii=False,separators=(',',':'))
        request['interfaceVersion']='weft-compile/0.2.0'
        request['target']=dict(backendId='truss.postgresql',backendVersion='0.1.0-candidate',targetProfile='pg17.9-candidate',bindingJson=raw,bindingSha256=hashlib.sha256(raw.encode()).hexdigest())
        request['options']={'allowCandidate':True}
        result=subprocess.check_output([ROOT/'target/debug/examples/compile_probe'],input=json.dumps(request,ensure_ascii=False).encode()).decode().strip()
        response=json.loads(result)
        if case['expected']['status']=='blocked':
            assert response['status']=='blocked' and response['diagnostics'][0]['code']==case['expected']['code'],(case['id'],home,response)
        elif response['status']!='compiled':coverage.append(dict(id=case['id'],home=home,diagnostics=response['diagnostics']))
        reports.append(dict(id=case['id']+'-'+home,request=request,raw=result,response=response))
OUT=ROOT/'target/b005';OUT.mkdir(exist_ok=True)
(OUT/'full-corpus-reports.json').write_text(json.dumps(reports,ensure_ascii=False))
(OUT/'full-corpus-coverage.json').write_text(json.dumps(coverage,indent=2))
print(f'{len(reports)} compiler decisions checked; {len(coverage)} resolved inputs remain backend refusals.')
