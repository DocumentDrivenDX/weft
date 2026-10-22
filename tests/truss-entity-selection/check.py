# SPDX-License-Identifier: MIT OR Apache-2.0
"""Check independently expected column/fact/profile outcomes in an installed wheel."""
import argparse
import json
import hashlib
from pathlib import Path
import weft

p=argparse.ArgumentParser();p.add_argument('--reports',type=Path,required=True);args=p.parse_args()
cases=json.loads(Path(__file__).with_name('cases.json').read_text());reports=[]
for case in cases:
    raw_request=json.dumps(case['request'])
    raw=(weft.compile_truss_runtime_json(raw_request) if case['pageSize'] is None else
         weft.compile_truss_runtime_page_json(raw_request,case['pageSize'],case.get('after')))
    result=json.loads(raw);expected=case['expected']
    assert result['status']==expected['status'],(case['id'],result)
    if result['status']=='compiled':
        artifact=result if case['pageSize'] is None else result['artifact']
        if 'sourceFunction' in expected:
            assert '"'+expected['sourceFunction']+'"(' in artifact['sql'],(case['id'],artifact)
        # Source custody is independent of selected read semantics: compile the
        # original extension-bearing bytes and retain their exact admitted pins.
        assert artifact['modelPins']==[m['pin'] for m in case['request']['modules']]
        for module in case['request']['modules']:
            assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
        columns=artifact['columns'];visible=len(columns) if case['pageSize'] is None else result['page']['visibleOutputs']
        assert [c['outputName'] for c in columns[:visible]]==expected['visibleColumns'],(case['id'],columns)
        assert [c['position'] for c in columns]==list(range(1,len(columns)+1))
        assert [c['representation']['metadata'] for c in columns if c['representation']['kind']=='nativeMetadata']==expected['facts']
        assert any(c['representation']['kind']=='completeRelatedKeys' for c in columns)==expected['automaticRelationships']
        if case['pageSize'] is not None:
            assert result['page']['keyOutputs']==list(range(visible,len(columns)))
            assert result['page']['pageSize']==case['pageSize']
    else:
        assert 'sql' not in result
        if 'diagnosticCode' in expected:assert result['diagnostics'][0]['code']==expected['diagnosticCode'],(case['id'],result)
    reports.append({'id':case['id'],'raw':raw})
args.reports.write_text(json.dumps(reports)+'\n')
print(json.dumps({'cases':len(cases),'version':weft.__version__,'status':'passed'}))
