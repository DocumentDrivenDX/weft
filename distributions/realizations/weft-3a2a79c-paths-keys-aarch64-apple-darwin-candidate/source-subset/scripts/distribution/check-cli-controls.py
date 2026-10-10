"""Actual candidate CLI refusal and fresh-binding controls, without native claims."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import uuid


def sha(data):
    return hashlib.sha256(data).hexdigest()


def encode(value):
    return json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', required=True, type=Path)
    parser.add_argument('--output', required=True, type=Path)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=False)
    root = Path(__file__).resolve().parents[2]
    source = root / 'docs/helix/04-build/evidence/B-006-cross-module-native/compile.json'
    source_raw = source.read_bytes()
    fixture = json.loads(source_raw)
    original = fixture['request']
    opening = sha(binary.read_bytes())
    harness_opening = sha(Path(__file__).read_bytes())
    records = []

    def execute(case_id, request, expected_code=None, binding=None):
        raw = request if isinstance(request, bytes) else encode(request)
        assert sha(binary.read_bytes()) == opening, 'compiler changed before execution'
        result = subprocess.run([str(binary)], input=raw, capture_output=True, timeout=30)
        assert sha(binary.read_bytes()) == opening, 'compiler changed after execution'
        record = {'id': case_id, 'requestSha256': sha(raw), 'requestJson': raw.decode(),
                  'stdout': result.stdout.decode(), 'stderr': result.stderr.decode(),
                  'exit': result.returncode, 'expectedCode': expected_code}
        records.append(record)
        # Failed observations survive without a success report.
        with (args.output / 'cases.jsonl').open('ab') as output:
            output.write(encode(record) + b'\n')
        assert result.returncode == 0 and result.stderr == b'', case_id
        assert result.stdout.endswith(b'\n') and result.stdout.count(b'\n') == 1, case_id
        response = json.loads(result.stdout)
        if expected_code:
            assert set(response) == {'interfaceVersion', 'status', 'diagnostics'}, case_id
            assert response['status'] == 'blocked' and len(response['diagnostics']) == 1, case_id
            assert response['diagnostics'][0]['code'] == expected_code, (case_id, expected_code, response['diagnostics'][0]['code'])
        else:
            assert response['status'] == 'compiled', (case_id, response['status'])
            assert response['modelPins'] == fixture['response']['modelPins'], case_id
            assert response['logicalPlan'] == fixture['response']['logicalPlan'], case_id
            assert response['columns'] == fixture['response']['columns'], case_id
            assert response['parameters'] == fixture['response']['parameters'], case_id
            assert response['bindingSha256'] == request['target']['bindingSha256'], case_id
            publication = next(o for o in response['obligations'] if o['id'] == 'ashlar.candidate.publication')
            assert publication['owner'] == 'host' and publication['parameters']['publication'] == binding['publication'], case_id
            table = binding['publication']['tables'][0]
            reference = '.'.join('`' + part.replace('`', '``') + '`' for part in table['name']) + ' VERSION AS OF ' + str(table['version'])
            assert reference in response['sql'], case_id
            checks = next(o for o in response['obligations'] if o['id'] == 'ashlar.candidate.scalarIntegrity')['parameters']['checks']
            assert len(checks) == 3 and all(reference in check['sql'] for check in checks), case_id
        return response

    def negative(case_id, change, code):
        request = copy.deepcopy(original)
        change(request)
        execute(case_id, request, code)

    negative('candidate-opt-out', lambda r: r['options'].update(allowCandidate=False), 'WFT-CAPABILITY')
    negative('unknown-backend', lambda r: r['target'].update(backendId='unregistered.distribution'), 'WFT-BACKEND-MISSING')
    negative('wrong-backend-version', lambda r: r['target'].update(backendVersion='99.0.0'), 'WFT-BACKEND-VERSION')
    negative('wrong-profile', lambda r: r['target'].update(targetProfile='unregistered-profile'), 'WFT-BACKEND-VERSION')
    negative('unknown-envelope-member', lambda r: r.update(unregisteredOption=True), 'WFT-INPUT')
    negative('mismatched-interface-dialect', lambda r: r.update(dialect='weft-sql/0.2.0'), 'WFT-VERSION')
    negative('binding-digest-mismatch', lambda r: r['target'].update(bindingSha256='0' * 64), 'WFT-PIN')
    negative('model-digest-mismatch', lambda r: r['modules'][0]['pin'].update(sha256='0' * 64), 'WFT-PIN')
    negative('sql-byte-limit', lambda r: r.update(sql=' ' * 65537), 'WFT-LIMIT')
    negative('binding-byte-limit', lambda r: r['target'].update(bindingJson=' ' * (4 * 1024 * 1024 + 1)), 'WFT-LIMIT')
    execute('duplicate-envelope-member', b'{"interfaceVersion":"weft-compile/0.1.0",' + encode(original)[1:], 'WFT-JSON-DUPLICATE')

    def binding_case(case_id, change, code=None):
        request = copy.deepcopy(original)
        binding = json.loads(request['target']['bindingJson'])
        change(binding)
        raw = encode(binding)
        request['target']['bindingJson'] = raw.decode()
        request['target']['bindingSha256'] = sha(raw)
        return execute(case_id, request, code, binding)

    binding_case('missing-publication', lambda b: b['publication'].update(id=''), 'WFT-BINDING')
    binding_case('negative-table-version', lambda b: b['publication']['tables'][0].update(version=-1), 'WFT-BINDING')
    binding_case('duplicate-table-uuid', lambda b: b['publication']['tables'].append({**b['publication']['tables'][0], 'name': ['local', 'fresh', 'other']}), 'WFT-BINDING')
    binding_case('missing-table-mapping', lambda b: b['records'][0].update(table=123), 'WFT-BINDING')
    binding_case('inconsistent-model-pin', lambda b: b['modelPins'][0].update(revision='stale-unmatched-revision'), 'WFT-BINDING')
    for index, version in enumerate([0, 7, 57]):
        def fresh(binding, index=index, version=version):
            publication = binding['publication']
            publication['id'] = f'fresh-distribution-publication-{index}'
            publication['manifestUuid'] = str(uuid.uuid5(uuid.NAMESPACE_URL, f'weft-distribution-manifest-{index}'))
            publication['tables'][0].update(name=['local_fixture', 'new_distribution', f'table_{index}'], uuid=str(uuid.uuid5(uuid.NAMESPACE_URL, f'weft-distribution-table-{index}')), version=version)
        binding_case(f'fresh-binding-{index}', fresh)
    assert len(records) == 19 and len({r['id'] for r in records}) == 19
    assert sha(binary.read_bytes()) == opening and sha(Path(__file__).read_bytes()) == harness_opening
    assert source.read_bytes() == source_raw
    summary = {'scope': 'actual CLI static refusal and fresh-binding compilation only; no native data, authorization or freshness claim', 'binarySha256': opening, 'harnessSha256': harness_opening, 'sourceSha256': sha(source_raw), 'cases': len(records), 'blockedCases': 16, 'compiledCases': 3, 'custodyClosed': True}
    (args.output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary))


if __name__ == '__main__':
    main()
