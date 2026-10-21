#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Add source/target and exact RECORD digest to a checked wheel report."""
import argparse
import hashlib
import json
import zipfile
from pathlib import Path

p = argparse.ArgumentParser()
p.add_argument('--wheel', type=Path, required=True)
p.add_argument('--packaging-report', type=Path, required=True)
p.add_argument('--build-report', type=Path, required=True)
p.add_argument('--source-commit', required=True)
p.add_argument('--target', required=True)
p.add_argument('--version', default='0.4.49')
p.add_argument('--build-run-id')
p.add_argument('--workflow-name')
a = p.parse_args()
report = json.loads(a.packaging_report.read_text())
with zipfile.ZipFile(a.wheel) as archive:
    record_name = next(name for name in archive.namelist() if name.endswith('.dist-info/RECORD'))
    record_sha = hashlib.sha256(archive.read(record_name)).hexdigest()
requirement = report.get('status') == 'passed' and report.get('version') == a.version
if not requirement or report.get('wheel') != a.wheel.name:
    raise SystemExit(f'selected wheel packaging report does not match {a.version} wheel')
report['recordSha256'] = record_sha
a.packaging_report.write_text(json.dumps(report, indent=2) + '\n')
build = dict(report)
build.update({'sourceCommit': a.source_commit, 'target': a.target, 'functionalQualification': 'not-claimed'})
if a.target == 'x86_64-unknown-linux-gnu':
    build['manylinux'] = '2014'
    if not a.build_run_id or a.workflow_name != f'Build selected Weft {a.version} Linux wheel':
        raise SystemExit('Linux selected build must bind its exact successful workflow run')
    build.update({'buildRunId': a.build_run_id, 'buildWorkflowName': a.workflow_name})
a.build_report.write_text(json.dumps(build, indent=2) + '\n')
