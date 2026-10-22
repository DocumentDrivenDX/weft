# SPDX-License-Identifier: MIT OR Apache-2.0
"""Check independently expected column/fact/profile outcomes in an installed wheel."""
import argparse
import json
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
        columns=artifact['columns'];visible=len(columns) if case['pageSize'] is None else result['page']['visibleOutputs']
        assert [c['outputName'] for c in columns[:visible]]==expected['visibleColumns'],(case['id'],columns)
        assert [c['position'] for c in columns]==list(range(1,len(columns)+1))
        assert [c['representation']['metadata'] for c in columns if c['representation']['kind']=='nativeMetadata']==expected['facts']
        assert any(c['representation']['kind']=='completeRelatedKeys' for c in columns)==expected['automaticRelationships']
        if case['pageSize'] is not None:
            assert result['page']['keyOutputs']==list(range(visible,len(columns)))
            assert result['page']['pageSize']==case['pageSize']
    else:assert 'sql' not in result
    reports.append({'id':case['id'],'raw':raw})
args.reports.write_text(json.dumps(reports)+'\n')
print(json.dumps({'cases':len(cases),'version':weft.__version__,'status':'passed'}))
