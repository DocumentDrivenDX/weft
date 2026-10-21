#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Focused guard for the selected-only 0.4.49 release route."""
from pathlib import Path

legacy = Path('.github/workflows/release.yml').read_text()
build = Path('.github/workflows/release-selected-linux-0.4.49.yml').read_text()
publish = Path('.github/workflows/release-selected-publish-0.4.49.yml').read_text()

assert "github.ref_name != 'v0.4.49'" in legacy
assert "inputs.tag != 'v0.4.49'" in legacy
assert 'matrix:' in legacy  # Legacy tags retain their existing platform behavior.
assert 'workflow_dispatch:' in build and 'push:' not in build
assert 'x86_64-unknown-linux-gnu' in build and "manylinux: '2014'" in build
assert 'aarch64-unknown-linux-gnu' not in build and 'windows-2022' not in build
assert 'contents: read' in build and 'upload-artifact@v4' in build
assert 'BUILD_RUN_ID' in build and 'build-run-id' in build and 'WORKFLOW_SHA' in build
assert 'workflow_dispatch:' in publish and 'maturin-action' not in publish
assert 'download-artifact@v4' in publish
assert 'verify-selected-assets-0.4.49.py' in publish
assert 'gh release upload' in publish and '--draft=false' in publish
assert 'gh run view' in publish and 'workflowName,conclusion,event,headSha' in publish
assert 'selected/linux -maxdepth 1 -name \'*.whl\' | wc -l' in publish
assert 'selected/macos -maxdepth 1 -name \'*.whl\' | wc -l' in publish
for asset in ('managed-current-uc-qualification-v0.4.49.json',
              'managed-current-uc-case-backend03-grouped-having-v0.4.49.json',
              'managed-current-uc-case-backend03-negative-having-v0.4.49.json',
              'managed-current-uc-case-backend02-page1-v0.4.49.json',
              'managed-current-uc-case-backend02-continuation-v0.4.49.json',
              'managed-current-uc-program-parity-v0.4.49.json'):
    assert asset in publish
mac = Path('scripts/release/build-selected-macos-0.4.49.sh').read_text()
assert 'git status --porcelain' in mac and 'git symbolic-ref --short -q HEAD' in mac
assert "-name 'weft_sql-0.4.49-*-macosx_*_arm64.whl' | wc -l" in mac
verifier = Path('scripts/release/verify-selected-assets-0.4.49.py').read_text()
assert "backend03-grouped-having" in verifier and "backend03-negative-having" in verifier and "backend02-continuation" in verifier
assert 'BUILD_RUN_ID: ${{ github.run_id }}' in build and 'WORKFLOW_NAME: ${{ github.workflow }}' in build[build.index('Check exact wheel'):]
assert '--prior-grouped-gzip' in publish and '--prior-negative-gzip' in publish and '--prior-pages-gzip' in publish
assert "requestJson" in verifier and "artifactJson" in verifier
assert "expectedRows" in verifier and "orderedParameterSha256" in verifier
assert "weft-local-program-parity/0.2" in verifier and "prior_artifact.get('sql')" in verifier and "localWheel" in verifier
assert '0.4.49' in Path('crates/weft-python/Cargo.toml').read_text()
assert '0.4.49' in Path('crates/weft-python/pyproject.toml').read_text()
lock = Path('Cargo.lock').read_text().split('name = "weft-python"\n', 1)[1].split('\n\n', 1)[0]
assert 'version = "0.4.49"' in lock
print('selected 0.4.49 release gates passed')
