"""Actual PathsKeys release transport corpus; no native SQL qualification.

This finite corpus retains all historical profile requests and exact responses,
plus independently reviewed 0.4.1 semantic outputs. The enclosing bounded launcher
owns process deadlines, environment, opening/closing executable/source custody.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
from types import SimpleNamespace
import sys


def main():
    parser = argparse.ArgumentParser()
    for name in ('executable', 'historical', 'new-cases', 'parity-output', 'receipt'):
        parser.add_argument('--' + name, type=Path, required=True)
    args = parser.parse_args()
    if not all(p.is_absolute() for p in vars(args).values()):
        raise ValueError('absolute-config')
    # Reuse the owning bounded regular-file read, never import compiler semantics.
    spec = importlib.util.spec_from_file_location('embedding_io', Path(__file__).with_name('check-paths-embedding-python.py'))
    io = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(io)
    transport_spec = importlib.util.spec_from_file_location('paths_transport', Path(__file__).with_name('check-paths-cli.py'))
    transport = importlib.util.module_from_spec(transport_spec)
    sys.modules[transport_spec.name] = transport
    transport_spec.loader.exec_module(transport)
    limits = SimpleNamespace(timeout_seconds=15, maximum_response_bytes=4 * 1024 * 1024,
                             process_group_owner='transport')
    old = json.loads(io.read(args.historical, io.CORPUS_LIMIT))
    new = json.loads(io.read(args.new_cases, io.CORPUS_LIMIT))
    selected = [('namespace:' + c['id'], c) for c in old['namespaceFences']]
    selected += [('paths:' + c['id'], c) for c in old['paths']]
    selected += [('controls:' + c['id'], c) for c in old['controls']]
    if len(selected) != 74 or len({i for i, _ in selected}) != 74:
        raise ValueError('complete-historical-inventory')
    cases = []
    for identity, case in selected:
        request = bytes.fromhex(case['requestHex'])
        expected = bytes.fromhex(case['responseHex'])
        cases.append((identity, request, expected, None))
    additions = [c for c in new['checks'] if 'request' in c]
    expected_ids = ('original-replay', 'joined-filtered-full-bag', 'expansion-count-owner',
                    'old-dialect-new-interface', 'new-dialect-old-interface',
                    'new-language-old-backend', 'old-language-new-backend', 'candidate-optout',
                    'unprojected-count', 'no-groups', 'output-alias-not-source-field')
    if tuple(c['case'] for c in additions) != expected_ids:
        raise ValueError('complete-new-semantic-inventory')
    for case in additions:
        request = json.dumps(case['request'], ensure_ascii=False, separators=(',', ':')).encode('utf8')
        cases.append(('having:' + case['case'], request, None, case['response']))
    rows = []
    for identity, request, expected, semantic in cases:
        if len(request) > 16 * 1024 * 1024:
            raise ValueError('request-bound')
        outputs = []
        for _ in range(2):
            # The owning transport bounds every pipe and shares one deadline;
            # inherited child environment is the enclosing launcher's closed map.
            code, stdout, stderr = transport.transport(args.executable, request, 'normal', limits)
            if code != 0 or stderr:
                raise ValueError('cli-protocol')
            outputs.append(stdout)
        if outputs[0] != outputs[1] or not outputs[0].endswith(b'\n'):
            raise ValueError('deterministic-framing')
        raw = outputs[0]
        if expected is not None and raw != expected:
            raise ValueError('historical-byte-parity')
        if semantic is not None and json.loads(raw) != semantic:
            raise ValueError('independent-semantic-parity')
        rows.append({'id': identity, 'request': request.decode('utf8'), 'expected': raw[:-1].decode('utf8')})
    raw = (json.dumps(rows, ensure_ascii=False, separators=(',', ':')) + '\n').encode('utf8')
    if len(raw) > io.CORPUS_LIMIT:
        raise ValueError('corpus-bound')
    with args.parity_output.open('xb') as stream:
        stream.write(raw)
    receipt = {'format': 'weft-paths-release-corpus/0.1', 'protocolCases': len(rows),
               'historicalCases': 74, 'newSemanticCases': 11,
               'casesSha256': hashlib.sha256(raw).hexdigest(),
               'casesBytes': len(raw), 'qualification': 'Actual compiler transport only; no native SQL or host obligation discharge.'}
    with args.receipt.open('x', encoding='utf8') as stream:
        json.dump(receipt, stream, indent=2)
        stream.write('\n')
    print(json.dumps({'state': 'release-corpus-passed', 'cases': len(rows)}))

if __name__ == '__main__':
    main()
