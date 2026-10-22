# SPDX-License-Identifier: MIT OR Apache-2.0
"""Prove exact historical source custody across one leading license annotation.

Historical receipts stay immutable. No AST normalization or comment stripping:
every byte after the one explicit SPDX line must match the original digest.
"""
import hashlib
from pathlib import Path

PREFIX = b'# SPDX-License-Identifier: MIT OR Apache-2.0\n'
SLASH_PREFIX = b'// SPDX-License-Identifier: MIT OR Apache-2.0\n'


def source_custody(path: Path, historical_digest: str) -> dict[str, str]:
    current = path.read_bytes()
    digest = hashlib.sha256(current).hexdigest()
    adjustment = 'none'
    if digest != historical_digest:
        prefix = PREFIX if path.suffix == '.py' else SLASH_PREFIX if path.suffix in ('.rs','.ts','.tsx','.js') else None
        if prefix is None or not current.startswith(prefix) or hashlib.sha256(current[len(prefix):]).hexdigest() != historical_digest:
            raise ValueError('Historical source differs beyond one leading SPDX annotation: '+str(path))
        adjustment = 'one-leading-SPDX-annotation'
    return {'currentSha256':digest,'historicalSha256':historical_digest,'adjustment':adjustment}


def retained_truss_application_inputs(root: Path) -> tuple[Path, Path]:
    """Original producer/corpus, pinned by the immutable native run summary."""
    import json
    base = root / 'docs/helix/04-build/evidence/B-007-truss-application-native'
    summary = json.loads((base / 'summary.json').read_bytes())
    harness = base / 'harness.py'
    source_custody(harness, summary['harnessSha256'])
    corpus = root / 'docs/helix/04-build/evidence/truss-selective-sources-20261022/retained-input-repair/application-cases.original.json'
    if hashlib.sha256(corpus.read_bytes()).hexdigest() != summary['corpusSha256']:
        raise ValueError('Original retained Truss corpus custody mismatch')
    return harness, corpus
