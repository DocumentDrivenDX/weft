#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Upload only missing exact assets to an existing GitHub draft release."""

import argparse
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path


class AssetMismatch(RuntimeError):
    pass


def plan_uploads(expected, remote):
    """Return missing asset names; refuse extras and byte mismatches first."""
    expected_names = set(expected)
    remote_names = set(remote)
    extra = sorted(remote_names - expected_names)
    if extra:
        raise AssetMismatch(f"unexpected draft release assets: {extra}")
    for name in sorted(expected_names & remote_names):
        if expected[name] != remote[name]:
            raise AssetMismatch(f"existing draft asset differs from expected bytes: {name}")
    return sorted(expected_names - remote_names)


def verify_exact_assets(expected, remote):
    """Require exact names and byte equality for the completed draft."""
    expected_names = set(expected)
    remote_names = set(remote)
    if remote_names != expected_names:
        raise AssetMismatch(
            f"draft asset set mismatch; missing={sorted(expected_names - remote_names)}, "
            f"unexpected={sorted(remote_names - expected_names)}"
        )
    for name in sorted(expected_names):
        if expected[name] != remote[name]:
            raise AssetMismatch(f"final draft asset differs from expected bytes: {name}")


def _run(args):
    return subprocess.run(args, check=True, capture_output=True, text=True).stdout


def _release_state(tag):
    value = json.loads(_run(["gh", "release", "view", tag, "--json", "isDraft,assets"]))
    if value.get("isDraft") is not True:
        raise AssetMismatch(f"release {tag} must remain a draft during asset reconciliation")
    assets = value.get("assets")
    if not isinstance(assets, list):
        raise AssetMismatch(f"release {tag} did not return its asset list")
    names = [item.get("name") for item in assets]
    if any(not isinstance(name, str) or not name or Path(name).name != name for name in names):
        raise AssetMismatch("release returned an invalid asset name")
    if len(names) != len(set(names)):
        raise AssetMismatch("release returned duplicate asset names")
    return set(names)


def _download(tag, name, root):
    destination = Path(tempfile.mkdtemp(prefix=hashlib.sha256(name.encode()).hexdigest()[:8] + "-", dir=root))
    _run(["gh", "release", "download", tag, "--pattern", name, "--dir", str(destination)])
    files = [path for path in destination.iterdir() if path.is_file()]
    if len(files) != 1 or files[0].name != name:
        raise AssetMismatch(f"could not download exactly one release asset named {name}")
    return files[0].read_bytes()


def reconcile(tag, asset_dir):
    entries = list(asset_dir.iterdir())
    paths = sorted(path for path in entries if path.is_file() and not path.is_symlink())
    if not paths or len(paths) != len(entries):
        raise AssetMismatch("expected asset directory must contain regular named files only")
    expected = {path.name: path.read_bytes() for path in paths}

    names = _release_state(tag)
    missing_from_expected = names - set(expected)
    if missing_from_expected:
        raise AssetMismatch(f"unexpected draft release assets: {sorted(missing_from_expected)}")
    with tempfile.TemporaryDirectory(prefix="weft-release-assets-") as temporary:
        root = Path(temporary)
        existing = {name: _download(tag, name, root) for name in sorted(names)}
        missing = plan_uploads(expected, existing)
        if missing:
            _run(["gh", "release", "upload", tag, *[str(asset_dir / name) for name in missing]])

        final_names = _release_state(tag)
        if final_names != set(expected):
            raise AssetMismatch(
                f"final draft asset set mismatch; missing={sorted(set(expected) - final_names)}, "
                f"unexpected={sorted(final_names - set(expected))}"
            )
        final = {name: _download(tag, name, root) for name in sorted(final_names)}
        verify_exact_assets(expected, final)
        if _release_state(tag) != set(expected):
            raise AssetMismatch("draft asset set changed after byte verification")

    report = {
        "status": "passed",
        "tag": tag,
        "assets": [
            {"name": name, "sha256": hashlib.sha256(expected[name]).hexdigest()}
            for name in sorted(expected)
        ],
        "limits": ["Assets were verified while the release was still a draft."],
    }
    print(json.dumps(report, indent=2))
    return report


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--asset-dir", type=Path, required=True)
    args = parser.parse_args()
    reconcile(args.tag, args.asset_dir)


if __name__ == "__main__":
    main()
