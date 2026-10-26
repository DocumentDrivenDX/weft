#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Compare the exact installed Weft wheel against frozen Rust responses."""
import argparse
import hashlib
import importlib.metadata
import json
import pathlib
import sys

import weft
import weft.weft


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('--cases', type=pathlib.Path, required=True)
    parser.add_argument('--reports', type=pathlib.Path, required=True)
    parser.add_argument('--output', type=pathlib.Path, required=True)
    parser.add_argument('--source-root', type=pathlib.Path, required=True)
    parser.add_argument('--version', required=True)
    parser.add_argument('--distribution', required=True)
    args = parser.parse_args()
    if importlib.metadata.version(args.distribution) != args.version or pathlib.Path(weft.__file__).resolve().is_relative_to(args.source_root.resolve()):
        raise SystemExit('installed wheel identity check failed')
    cases = json.loads(args.cases.read_bytes())
    reports = json.loads(args.reports.read_bytes())
    if len(cases) != 9 or len(reports) != len(cases):
        raise SystemExit('the frozen nine-vector profile is incomplete')
    actual = []
    for case, report in zip(cases, reports, strict=True):
        request = json.dumps(case['request'], ensure_ascii=False, separators=(',', ':'))
        result = weft.compile_json(request)
        if result != report['raw']:
            raise SystemExit('installed-wheel/Rust byte mismatch: ' + case['id'])
        actual.append({
            'id': case['id'],
            'requestSha256': hashlib.sha256(request.encode()).hexdigest(),
            'rustResponseSha256': hashlib.sha256(report['raw'].encode()).hexdigest(),
            'pythonResponseSha256': hashlib.sha256(result.encode()).hexdigest(),
        })
    module = pathlib.Path(weft.weft.__file__).resolve()
    args.output.write_text(json.dumps({
        'status': 'passed', 'cases': actual, 'python': sys.version,
        'distribution': args.distribution,
        'distributionVersion': importlib.metadata.version(args.distribution),
        'nativeModule': str(module), 'nativeModuleSha256': hashlib.sha256(module.read_bytes()).hexdigest(),
        'scope': 'Installed exact wheel compiler parity for the frozen refusal-response profile; no native database execution.'
    }, indent=2) + '\n')


if __name__ == '__main__':
    main()
