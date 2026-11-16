#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Fail unless a release tag matches every version field in the repository.

Usage: check-release-version.py vX.Y.Z
       check-release-version.py --shared-crates   (only the shared-crate check; CI runs it on every PR)

Checks that the tag, both pyproject.toml files and the Cargo workspace version
carry the same version, that no Cargo.toml pins UMF by commit (`rev =`), that
CHANGELOG.md has a section for the version, and that each shared compiler crate
(SHARED_CRATES, published to crates.io) inherits the workspace version, is
publishable only to crates.io, and is pinned at exactly that version in
[workspace.dependencies].
"""
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
# Crates other compilers depend on, published to crates.io at the workspace
# version by the release workflow, in this (dependency) order.
SHARED_CRATES = ("weft-model", "weft-expr", "weft-backend", "weft-postgresql-emit")


def workspace() -> dict:
    return tomllib.loads((ROOT / "Cargo.toml").read_text())["workspace"]


def shared_crate_errors() -> list[str]:
    errors = []
    root = workspace()
    version = root["package"]["version"]
    pins = root.get("dependencies", {})
    for name in SHARED_CRATES:
        package = tomllib.loads((ROOT / "crates" / name / "Cargo.toml").read_text())["package"]
        if package.get("version") != {"workspace": True}:
            errors.append(f"crates/{name}/Cargo.toml must use version.workspace = true")
        if package.get("publish") != ["crates-io"]:
            errors.append(f"crates/{name}/Cargo.toml must set publish = [\"crates-io\"]")
        pin = pins.get(name, {})
        if pin.get("path") != f"crates/{name}" or pin.get("version") != f"={version}":
            errors.append(f"Cargo.toml [workspace.dependencies] {name} must be path crates/{name}, version ={version}")
    return errors


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
        "Cargo.toml [workspace.package]": workspace()["package"]["version"],
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
    errors.extend(shared_crate_errors())
    changelog = (ROOT / "CHANGELOG.md").read_text()
    if not re.search(rf"^## {re.escape(version)}(\s|$)", changelog, re.M):
        errors.append(f"CHANGELOG.md has no '## {version}' section")
    for error in errors:
        print(f"error: {error}", file=sys.stderr)
    if not errors:
        print(f"release {version}: tag, manifests and changelog agree")
    return 1 if errors else 0


def shared_only() -> int:
    errors = shared_crate_errors()
    for error in errors:
        print(f"error: {error}", file=sys.stderr)
    if not errors:
        print("shared crates: " + ", ".join(SHARED_CRATES) + " inherit and pin the workspace version")
    return 1 if errors else 0


if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit(__doc__)
    sys.exit(shared_only() if sys.argv[1] == "--shared-crates" else main(sys.argv[1]))
