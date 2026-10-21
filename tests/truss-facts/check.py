# SPDX-License-Identifier: MIT OR Apache-2.0
"""Emit actual installed native reports for the shared browser parity runner."""
import argparse
import json
from pathlib import Path
import weft

parser = argparse.ArgumentParser()
parser.add_argument('--reports', type=Path, required=True)
args = parser.parse_args()
cases = json.loads(Path(__file__).with_name('cases.json').read_text())
reports = []
for case in cases:
    raw = weft.compile_truss_runtime_json(json.dumps(case['request']))
    result = json.loads(raw)
    expected = 'blocked' if case['id'] == 'wrong-binding-pin' else 'compiled'
    assert result['status'] == expected, (case['id'], result)
    if case['id'].startswith('complete-facts'):
        sql = result['sql']
        for helper in ('query_object_attribution', 'query_object_times', 'query_object_load_id'):
            assert sql.count('"' + helper + '"(') == 1, (case['id'], sql)
        assert sql.count(' AS MATERIALIZED ') == 3
    reports.append({'id': case['id'], 'raw': raw})
args.reports.write_text(json.dumps(reports) + '\n')
print(json.dumps({'cases': len(cases), 'version': weft.__version__, 'status': 'passed'}))
