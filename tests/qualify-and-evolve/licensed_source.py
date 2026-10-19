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
            raise ValueError('Historical source differs beyond one leading SPDX annotation')
        adjustment = 'one-leading-SPDX-annotation'
    return {'currentSha256':digest,'historicalSha256':historical_digest,'adjustment':adjustment}
