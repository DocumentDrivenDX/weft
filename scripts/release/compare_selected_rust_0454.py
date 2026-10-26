#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Run the frozen nine requests through the hosted Rust comparator."""
import argparse
import json
import pathlib
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--cases', type=pathlib.Path, required=True)
    parser.add_argument('--executable', type=pathlib.Path, required=True)
    parser.add_argument('--output', type=pathlib.Path, required=True)
    args = parser.parse_args()
    cases = json.loads(args.cases.read_bytes())
    payload = ''.join(json.dumps(case['request'], ensure_ascii=False, separators=(',', ':')) + '\n' for case in cases).encode()
    result = subprocess.run([args.executable], input=payload, capture_output=True, check=True, timeout=25)
    lines = result.stdout.decode().splitlines()
    if len(lines) != len(cases):
        raise SystemExit('Rust comparator output count mismatch')
    rows = [{'id': case['id'], 'raw': line} for case, line in zip(cases, lines, strict=True)]
    args.output.write_text(json.dumps(rows, ensure_ascii=False) + '\n')


if __name__ == '__main__':
    main()
