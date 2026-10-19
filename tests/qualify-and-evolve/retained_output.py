# SPDX-License-Identifier: MIT OR Apache-2.0
"""Keep replay products separate from immutable historical evidence inputs."""
import os
from pathlib import Path

EVIDENCE = Path(__file__).resolve().parents[2]/'docs/helix/04-build/evidence'


def replay_output(requested: Path) -> Path:
    destination = os.environ.get('WEFT_REPLAY_OUTPUT_ROOT')
    if destination is None or not requested.is_relative_to(EVIDENCE):
        return requested
    root = Path(destination)
    if not root.is_absolute():
        raise ValueError('Replay output root must be absolute')
    return root/requested.relative_to(EVIDENCE)
