#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Fail if a tracked file is larger than 1 MiB outside the allowlist, or the tracked tree is too big.

Run output belongs in CI artifacts, never in git.
"""
import fnmatch
import os
import subprocess
import sys

MAX_FILE = 1 << 20
MAX_TREE = 160 << 20
ALLOW = (
    "tests/*/fixtures/*",
    "tests/fixtures/*",
    "crates/weft-core/tests/*.json",
    "docs/helix/03-test/fixtures/*",
)


def main() -> int:
    root = subprocess.check_output(["git", "rev-parse", "--show-toplevel"], text=True).strip()
    names = subprocess.check_output(["git", "ls-files", "-z"], cwd=root).decode().split("\0")
    total, bad = 0, []
    for name in filter(None, names):
        path = os.path.join(root, name)
        if not os.path.isfile(path):
            continue  # deleted in the working tree, or not checked out (sparse)
        size = os.path.getsize(path)
        total += size
        if size > MAX_FILE and not any(fnmatch.fnmatchcase(name, pat) for pat in ALLOW):
            bad.append((size, name))
    for size, name in sorted(bad, reverse=True):
        print(f"too large ({size / 1048576:.1f} MiB), not allowlisted: {name}")
    if total > MAX_TREE:
        print(f"tracked tree is {total / 1048576:.1f} MiB, over the {MAX_TREE >> 20} MiB budget")
    print(f"tracked tree: {total / 1048576:.1f} MiB")
    return 1 if bad or total > MAX_TREE else 0


if __name__ == "__main__":
    sys.exit(main())
