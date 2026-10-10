"""Qualify actual CLI transport against the retained 463-case public corpus.

This is compiler/transport evidence, not native execution or release authority.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

MAX = 16 * 1024 * 1024

def digest(data):
    return hashlib.sha256(data).hexdigest()

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    opening_binary = digest(binary.read_bytes())
    opening_harness = digest(Path(__file__).read_bytes())
    args.output.mkdir(parents=True, exist_ok=False)
    root = Path(__file__).resolve().parents[2]
    evidence = root / 'docs/helix/04-build/evidence'
    cases, sources = [], []
    def read(path):
        raw = path.read_bytes()
        sources.append({'path': str(path.relative_to(root)), 'sha256': digest(raw)})
        return raw
    for scope in ['columns-native', 'application-native', 'key-refusal', 'unsigned-columns', 'optional-native', 'relationship-native', 'compound-native', 'compound-boundaries-native', 'compound-application-native']:
        for line in read(evidence / f'B-006-{scope}/compile-artifacts.jsonl').splitlines():
            case = json.loads(line)
            case['id'] = scope + ':' + str(case['id'])
            cases.append(case)
    for scope in ['scalar-native', 'global-native']:
        for path in sorted((evidence / f'B-006-{scope}').glob('*-compile.json')):
            case = json.loads(read(path))
            case['id'] = scope + ':' + path.stem
            cases.append(case)
    case = json.loads(read(evidence / 'B-006-cross-module-native/compile.json'))
    case['id'] = 'cross-module'
    cases.append(case)
    assert len(cases) == 463, len(cases)
    assert len({case['id'] for case in cases}) == 463, 'duplicate case IDs'
    corrections = []
    migrations = []
    for case in cases:
        original_expected = json.dumps(case['response'], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
        if case['id'] == 'unsigned-columns:unsigned-64':
            assert case['response'] == {'diagnostics': [{'code': 'WFT-BINDING', 'message': 'Native column carrier cannot establish the original logical scalar domain', 'phase': 'binding', 'recoverability': 'correct-input', 'severity': 'error'}], 'interfaceVersion': 'weft-compile/0.1.0', 'status': 'blocked'}
            case['response']['diagnostics'][0]['phase'] = 'lower'
            migrations.append({'id': case['id'], 'path': '/diagnostics/0/phase', 'oldValue': 'binding', 'newValue': 'lower', 'sourceCommit': '39c35f5cfafc305a91d7c602b1775aa8a8d063d5', 'originalExpectedSha256': digest(original_expected), 'migratedExpectedSha256': digest(json.dumps(case['response'], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode())})
        for obligation in case['response'].get('obligations', []):
            if obligation['id'] == 'ashlar.candidate.publication':
                profile = obligation['parameters']['nativeProfile']
                if 'versionReported' in profile:
                    assert profile.pop('versionReported') == '4.2.0 zero build hash', case['id']
                    migrations.append({'id': case['id'], 'path': '/obligations/ashlar.candidate.publication/parameters/nativeProfile/versionReported', 'oldValue': '4.2.0 zero build hash', 'originalExpectedSha256': digest(original_expected), 'migratedExpectedSha256': digest(json.dumps(case['response'], ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode())})
        if case['response']['status'] != 'compiled':
            assert case['response']['status'] == 'blocked'
            continue
        for operation in case['response']['qualification']['operations']:
            declaration = operation['declaration']
            if declaration['id'] == 'value.presence':
                old = {'subset': 'optional scalar envelopes; absent or exact value; explicit native null refuses'}
                new = {'subset': 'optional scalar or compound envelopes; absent or exact value; explicit native null refuses'}
                assert declaration['logicalDomain'] in [old, new]
                if declaration['logicalDomain'] == old:
                    declaration['logicalDomain'] = new
                    corrections.append(case['id'])
    def run(data):
        assert digest(binary.read_bytes()) == opening_binary, 'compiler bytes changed before execution'
        result = subprocess.run([str(binary)], input=data, capture_output=True, timeout=30)
        assert digest(binary.read_bytes()) == opening_binary, 'compiler bytes changed after execution'
        return result
    reports = []
    mismatches = []
    for case in cases:
        request = json.dumps(case['request'], ensure_ascii=False, separators=(',', ':')).encode()
        result = run(request)
        row = {'id': case['id'], 'requestSha256': digest(request), 'exit': result.returncode,
               'stdoutSha256': digest(result.stdout), 'stderrSha256': digest(result.stderr)}
        with (args.output / 'cases.jsonl').open('ab') as output:
            output.write(json.dumps({**row, 'stdout': result.stdout.decode(), 'stderr': result.stderr.decode()}).encode() + b'\n')
        assert result.returncode == 0 and result.stderr == b'', row
        assert result.stdout.endswith(b'\n') and result.stdout.count(b'\n') == 1, row
        if json.loads(result.stdout) != case['response']:
            mismatches.append({'id': case['id'], 'expected': case['response'], 'actual': json.loads(result.stdout)})
        repeat = run(request)
        assert (repeat.returncode, repeat.stdout, repeat.stderr) == (0, result.stdout, b''), row
        reports.append(row)
    controls = []
    for name, data, code in [('oversize', b' ' * (MAX + 1), b'WEFT_CLI_INPUT_LIMIT\n'), ('invalid-utf8', b'\xff', b'WEFT_CLI_UTF8\n'), ('split-utf8-at-limit', b' ' * (MAX - 1) + b'\xc3', b'WEFT_CLI_UTF8\n')]:
        result = run(data)
        assert (result.returncode, result.stdout, result.stderr) == (2, b'', code), name
        controls.append({'id': name, 'inputBytes': len(data), 'exit': result.returncode, 'code': code.decode().strip()})
    for name, data in [('exact-limit', b' ' * MAX), ('exact-limit-multibyte', b' ' * (MAX - 2) + 'é'.encode()), ('malformed-json', b'{')]:
        result = run(data)
        assert result.returncode == 0 and result.stderr == b'' and result.stdout.endswith(b'\n'), name
        assert json.loads(result.stdout)['status'] == 'blocked', name
        controls.append({'id': name, 'inputBytes': len(data), 'exit': 0, 'status': 'blocked'})
    closing_binary = digest(binary.read_bytes())
    closing_harness = digest(Path(__file__).read_bytes())
    assert closing_binary == opening_binary and closing_harness == opening_harness, 'execution source changed'
    for source in sources:
        assert digest((root / source['path']).read_bytes()) == source['sha256'], source['path']
    summary = {'openingBinarySha256': opening_binary, 'closingBinarySha256': closing_binary,
               'openingHarnessSha256': opening_harness, 'closingHarnessSha256': closing_harness,
               'caseIdsUnique': True, 'sourceFilesUnchanged': True, 'scope': 'actual produced CLI compiler/transport only; no native execution or registration authority',
               'binarySha256': digest(binary.read_bytes()), 'harnessSha256': digest(Path(__file__).read_bytes()),
               'cases': len(reports), 'fullResponseParity': not mismatches, 'deterministic': True,
               'metadataCorrection': {'field': 'value.presence.logicalDomain', 'caseIds': corrections},
               'historicalObservationMigration': migrations, 'sourceFiles': sources, 'controls': controls}
    (args.output / 'mismatches.json').write_text(json.dumps(mismatches, ensure_ascii=False, indent=2) + '\n')
    (args.output / 'summary.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps({key: summary[key] for key in ['binarySha256', 'cases', 'fullResponseParity', 'scope']}))
    if mismatches:
        raise SystemExit(1)

if __name__ == '__main__':
    main()
