#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Fail unless a release tag matches every version field in the repository.

Usage: check-release-version.py vX.Y.Z

Checks that the tag, both pyproject.toml files and crates/weft-python/Cargo.toml
carry the same version, that no Cargo.toml pins UMF by commit (`rev =`), and
that CHANGELOG.md has a section for the version.
"""
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def main(tag: str) -> int:
    errors = []
    if not re.fullmatch(r"v\d+\.\d+\.\d+", tag):
        errors.append(f"tag {tag!r} is not of the form vX.Y.Z")
    version = tag.removeprefix("v")
    fields = {
        "pyproject.toml": tomllib.loads((ROOT / "pyproject.toml").read_text())["project"]["version"],
        "crates/weft-python/pyproject.toml": tomllib.loads(
            (ROOT / "crates/weft-python/pyproject.toml").read_text()
        )["project"]["version"],
        "crates/weft-python/Cargo.toml": tomllib.loads(
            (ROOT / "crates/weft-python/Cargo.toml").read_text()
        )["package"]["version"],
    }
    for name, value in fields.items():
        if value != version:
            errors.append(f"{name} has version {value}, tag is {version}")
    for manifest in sorted(ROOT.glob("**/Cargo.toml")):
        if any(part in {"target", "node_modules", "vendor"} for part in manifest.parts):
            continue
        for number, line in enumerate(manifest.read_text().splitlines(), 1):
            if "umf.git" in line and re.search(r'\brev\s*=\s*"', line):
                errors.append(f"{manifest.relative_to(ROOT)}:{number} pins UMF by rev; use tag = \"umf-vX.Y.Z\"")
    changelog = (ROOT / "CHANGELOG.md").read_text()
    if not re.search(rf"^## {re.escape(version)}(\s|$)", changelog, re.M):
        errors.append(f"CHANGELOG.md has no '## {version}' section")
    for error in errors:
        print(f"error: {error}", file=sys.stderr)
    if not errors:
        print(f"release {version}: tag, manifests and changelog agree")
    return 1 if errors else 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    sys.exit(main(sys.argv[1]))
