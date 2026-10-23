#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Inert unit coverage for resumable selected-release asset reconciliation."""

import json
import unittest
from pathlib import Path

from sync_release_assets import AssetMismatch, plan_uploads, verify_exact_assets


FIXTURES = Path(__file__).parent / "fixtures" / "sync-release-assets"


def load_fixture(name):
    value = json.loads((FIXTURES / name).read_text())
    return {
        key: {asset: content.encode() for asset, content in entries.items()}
        if isinstance(entries, dict)
        else entries
        for key, entries in value.items()
    }


class SyncReleaseAssetsTests(unittest.TestCase):
    def test_identical_staged_macos_collision_is_skipped(self):
        fixture = load_fixture("staged-macos-collision.json")
        self.assertEqual(plan_uploads(fixture["expected"], fixture["remote"]), fixture["uploads"])

    def test_partial_upload_can_resume_without_reuploading_identical_assets(self):
        fixture = load_fixture("partial-retry.json")
        self.assertEqual(
            plan_uploads(fixture["expected"], fixture["first_remote"]),
            fixture["first_uploads"],
        )
        after_partial_upload = dict(fixture["first_remote"])
        after_partial_upload.update({
            name: fixture["expected"][name] for name in fixture["partial_successes"]
        })
        self.assertEqual(
            plan_uploads(fixture["expected"], after_partial_upload),
            fixture["retry_uploads"],
        )
        after_retry = dict(after_partial_upload)
        after_retry.update({
            name: fixture["expected"][name] for name in fixture["retry_uploads"]
        })
        verify_exact_assets(fixture["expected"], after_retry)

    def test_mismatched_staged_macos_asset_refuses_before_any_upload(self):
        fixture = load_fixture("mismatch-refusal.json")
        uploads = []
        with self.assertRaisesRegex(AssetMismatch, "existing draft asset differs"):
            uploads.extend(plan_uploads(fixture["expected"], fixture["remote"]))
        self.assertEqual(uploads, fixture["uploads_before_refusal"])


if __name__ == "__main__":
    unittest.main()
