from __future__ import annotations

import subprocess
import sys
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).parent))
import publish_hosted_release as hosted


class HostedReleasePublisherControls(unittest.TestCase):
    def test_producer_intent_matches_canonical_main_dispatch_pin_fields(self) -> None:
        intent = hosted.producer_intent("a" * 40, "a" * 40, 123456)
        self.assertEqual(
            intent,
            {"commit": "a" * 40, "workflowCommit": "a" * 40, "workflowRunId": 123456},
        )

    def test_numeric_draft_readback_requires_exact_tag_and_draft_state(self) -> None:
        release = {
            "id": 123,
            "tag_name": "v0.4.55",
            "draft": True,
            "assets": [{"name": "wheel.whl", "digest": "sha256:" + "b" * 64}],
        }
        assets = hosted.verify_release_api(
            release,
            release_id=123,
            tag="v0.4.55",
            draft=True,
            expected_digests={"wheel.whl": "b" * 64},
        )
        self.assertEqual(len(assets), 1)
        for change, message in [
            ({"id": 124}, "numeric release ID"),
            ({"tag_name": "v0.4.54"}, "tag differs"),
            ({"draft": False}, "draft state differs"),
            ({"assets": [{"name": "wheel.whl", "digest": "sha256:" + "0" * 64}]}, "digest differs"),
            ({"assets": []}, "asset set differs"),
        ]:
            with self.subTest(change=change), self.assertRaisesRegex(hosted.ReleaseError, message):
                hosted.verify_release_api(
                    {**release, **change},
                    release_id=123,
                    tag="v0.4.55",
                    draft=True,
                    expected_digests={"wheel.whl": "b" * 64},
                )

    def test_published_readback_requires_immutable_release(self) -> None:
        published = {"id": 123, "tag_name": "v0.4.55", "draft": False, "immutable": True, "assets": []}
        hosted.verify_release_api(published, release_id=123, tag="v0.4.55", draft=False, immutable=True)
        with self.assertRaisesRegex(hosted.ReleaseError, "immutable state differs"):
            hosted.verify_release_api(
                {**published, "immutable": False},
                release_id=123,
                tag="v0.4.55",
                draft=False,
                immutable=True,
            )

    def test_gh_subprocess_calls_have_a_hard_timeout(self) -> None:
        with patch.object(hosted.subprocess, "run", side_effect=subprocess.TimeoutExpired(["gh"], 2)):
            with self.assertRaisesRegex(hosted.ReleaseError, "timed out after 300s"):
                hosted.command(["gh", "api", "repos/example/releases"])


if __name__ == "__main__":
    unittest.main()
