from __future__ import annotations

import hashlib
import io
import json
import sys
import tarfile
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).parent))
import publish_pypi_release as publisher


def wheel(metadata: bytes) -> bytes:
    result = io.BytesIO()
    with zipfile.ZipFile(result, "w") as archive:
        archive.writestr("truss_toolkit-0.0.50.dist-info/METADATA", metadata)
    return result.getvalue()


def sdist(metadata: bytes) -> bytes:
    result = io.BytesIO()
    with tarfile.open(fileobj=result, mode="w:gz") as archive:
        for name in (
            "truss_toolkit-0.0.50/PKG-INFO",
            "truss_toolkit-0.0.50/src/truss_toolkit.egg-info/PKG-INFO",
        ):
            info = tarfile.TarInfo(name)
            info.size = len(metadata)
            archive.addfile(info, io.BytesIO(metadata))
    return result.getvalue()


class PublishPypiReleaseTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.root = Path(self.temp.name)
        self.assets = self.root / "assets"
        self.assets.mkdir()
        self.output = self.root / "dist"
        self.whl_name = "truss_toolkit-0.0.50-py3-none-any.whl"
        self.sdist_name = "truss_toolkit-0.0.50.tar.gz"
        metadata = b"Metadata-Version: 2.4\nName: truss-toolkit\nVersion: 0.0.50\nRequires-Dist: jsonschema==4.26.0\n\n"
        self.payloads = {
            self.whl_name: wheel(metadata),
            self.sdist_name: sdist(metadata),
        }
        for name, payload in self.payloads.items():
            (self.assets / name).write_bytes(payload)
        self.sums = "".join(
            f"{hashlib.sha256(data).hexdigest()}  {name}\n"
            for name, data in self.payloads.items()
        ).encode()
        self.pins = json.dumps(
            {
                "tag": "v0.0.50",
                "commit": "a" * 40,
                "version": "0.0.50",
                "distribution": "truss-toolkit",
            }
        ).encode()
        (self.assets / "SHA256SUMS").write_bytes(self.sums)
        (self.assets / "release-pins.json").write_bytes(self.pins)
        self.release = {
            "tag_name": "v0.0.50",
            "draft": False,
            "prerelease": False,
            "created_at": "2026-10-26T00:01:00Z",
            "published_at": "2026-10-26T00:02:00Z",
            "assets": [
                {"name": name, "digest": "sha256:" + hashlib.sha256(data).hexdigest()}
                for name, data in {
                    "SHA256SUMS": self.sums,
                    "release-pins.json": self.pins,
                    **self.payloads,
                }.items()
            ],
        }
        self.ref = {
            "ref": "refs/tags/v0.0.50",
            "object": {"sha": "a" * 40, "type": "commit"},
        }

    def tearDown(self) -> None:
        self.temp.cleanup()

    def prepare(self, **overrides: object) -> dict[str, object]:
        values: dict[str, object] = {
            "repository": "DocumentDrivenDX/weft",
            "release": self.release,
            "tag_ref": self.ref,
            "tag_object": None,
            "assets_dir": self.assets,
            "output_dir": self.output,
            "tag": "v0.0.50",
            "source_sha": "a" * 40,
            "expected_sums_sha256": hashlib.sha256(self.sums).hexdigest(),
            "distribution": "truss-toolkit",
            "wheel_prefix": "truss_toolkit-0.0.50-",
            "wheel_count": 1,
            "require_sdist": True,
        }
        values.update(overrides)
        return publisher.prepare_release(**values)  # type: ignore[arg-type]

    def test_prepare_binds_public_assets_tag_source_and_package_metadata(self) -> None:
        receipt = self.prepare()
        self.assertEqual(receipt["status"], "verified")
        self.assertEqual(receipt["source_sha"], "a" * 40)
        self.assertEqual(set(receipt["files"]), set(self.payloads))  # type: ignore[arg-type]
        self.assertEqual(
            (self.output / self.whl_name).read_bytes(), self.payloads[self.whl_name]
        )

    def test_selected_release_pins_bind_metadata_source_and_candidate_archives(
        self,
    ) -> None:
        selected_pins = json.dumps(
            {
                "schema": "weft-selected-release-pins/1",
                "tag": "v0.0.50",
                "sourceCommit": "b" * 40,
                "releaseMetadataCommit": "a" * 40,
                "candidateAssets": [
                    {"name": name, "sha256": hashlib.sha256(data).hexdigest()}
                    for name, data in self.payloads.items()
                ],
            }
        ).encode()
        selected_name = "release-pins-0.0.50.json"
        (self.assets / "release-pins.json").unlink()
        (self.assets / selected_name).write_bytes(selected_pins)
        self.release["assets"] = [
            {"name": item["name"], "digest": item["digest"]}
            for item in self.release["assets"]
            if item["name"] != "release-pins.json"
        ] + [
            {
                "name": selected_name,
                "digest": "sha256:" + hashlib.sha256(selected_pins).hexdigest(),
            }
        ]

        receipt = self.prepare()
        self.assertEqual(receipt["source_sha"], "a" * 40)
        self.assertEqual(receipt["compiler_source_commit"], "b" * 40)
        self.assertEqual(receipt["release_metadata_commit"], "a" * 40)
        self.assertEqual(receipt["release_pins_schema"], "weft-selected-release-pins/1")

    def test_selected_release_pins_refuse_candidate_digest_mismatch(self) -> None:
        selected_pins = json.dumps(
            {
                "schema": "weft-selected-release-pins/1",
                "tag": "v0.0.50",
                "sourceCommit": "b" * 40,
                "releaseMetadataCommit": "a" * 40,
                "candidateAssets": [{"name": self.whl_name, "sha256": "0" * 64}],
            }
        ).encode()
        selected_name = "release-pins-0.0.50.json"
        (self.assets / "release-pins.json").unlink()
        (self.assets / selected_name).write_bytes(selected_pins)
        self.release["assets"] = [
            {"name": item["name"], "digest": item["digest"]}
            for item in self.release["assets"]
            if item["name"] != "release-pins.json"
        ] + [
            {
                "name": selected_name,
                "digest": "sha256:" + hashlib.sha256(selected_pins).hexdigest(),
            }
        ]

        with self.assertRaisesRegex(
            publisher.ReleaseError, "selected release pin differs"
        ):
            self.prepare()

    def test_prepare_refuses_source_movement_and_github_digest_mismatch(self) -> None:
        with self.assertRaisesRegex(publisher.ReleaseError, "pinned source"):
            self.prepare(source_sha="b" * 40)
        self.output = self.root / "dist2"
        changed = dict(self.release)
        changed["assets"] = list(self.release["assets"])
        changed["assets"][0] = {**changed["assets"][0], "digest": "sha256:" + "0" * 64}
        with self.assertRaisesRegex(publisher.ReleaseError, "GitHub asset digest"):
            self.prepare(release=changed, output_dir=self.output)

    def test_producer_run_must_be_successful_and_from_allowed_release_workflow(
        self,
    ) -> None:
        producer = {
            "id": 123,
            "path": ".github/workflows/release-selected-publish-0.0.50.yml",
            "conclusion": "success",
            "event": "workflow_dispatch",
            "head_sha": "a" * 40,
            "created_at": "2026-10-26T00:00:00Z",
            "updated_at": "2026-10-26T00:10:00Z",
            "repository": {"full_name": "DocumentDrivenDX/weft"},
        }
        receipt = self.prepare(
            producer_run=producer,
            allowed_workflow_prefixes=(".github/workflows/release-selected-publish-",),
        )
        self.assertEqual(receipt["producer_run_id"], 123)
        self.output = self.root / "dist2"
        with self.assertRaisesRegex(publisher.ReleaseError, "not allowed"):
            self.prepare(
                output_dir=self.output,
                producer_run={**producer, "path": ".github/workflows/other.yml"},
                allowed_workflow_prefixes=(
                    ".github/workflows/release-selected-publish-",
                ),
            )
        self.output = self.root / "dist3"
        with self.assertRaisesRegex(publisher.ReleaseError, "did not succeed"):
            self.prepare(
                output_dir=self.output,
                producer_run={**producer, "conclusion": "failure"},
            )
        self.output = self.root / "dist4"
        with self.assertRaisesRegex(publisher.ReleaseError, "repository differs"):
            self.prepare(
                output_dir=self.output,
                producer_run={**producer, "repository": {"full_name": "other/repo"}},
            )
        self.output = self.root / "dist5"
        with self.assertRaisesRegex(publisher.ReleaseError, "source commit differs"):
            self.prepare(
                output_dir=self.output, producer_run={**producer, "head_sha": "b" * 40}
            )
        self.output = self.root / "dist6"
        with self.assertRaisesRegex(
            publisher.ReleaseError, "outside its successful workflow run"
        ):
            self.prepare(
                output_dir=self.output,
                producer_run={**producer, "created_at": "2026-10-26T00:03:00Z"},
                allowed_workflow_prefixes=(
                    ".github/workflows/release-selected-publish-",
                ),
            )

    def test_prepare_refuses_direct_url_metadata_and_wrong_distribution(self) -> None:
        self.output = self.root / "dist3"
        raw = wheel(
            b"Metadata-Version: 2.4\nName: truss-toolkit\nVersion: 0.0.50\nRequires-Dist: umf-core @ https://example.invalid/core.whl\n\n"
        )
        (self.assets / self.whl_name).write_bytes(raw)
        changed = dict(self.release)
        changed["assets"] = [
            {**item, "digest": "sha256:" + hashlib.sha256(raw).hexdigest()}
            if item["name"] == self.whl_name
            else item
            for item in self.release["assets"]
        ]
        sums = "".join(
            f"{hashlib.sha256(raw if name == self.whl_name else data).hexdigest()}  {name}\n"
            for name, data in self.payloads.items()
        ).encode()
        (self.assets / "SHA256SUMS").write_bytes(sums)
        changed["assets"] = [
            {**item, "digest": "sha256:" + hashlib.sha256(sums).hexdigest()}
            if item["name"] == "SHA256SUMS"
            else item
            for item in changed["assets"]
        ]
        with self.assertRaisesRegex(publisher.ReleaseError, "direct URL"):
            self.prepare(
                release=changed,
                output_dir=self.output,
                expected_sums_sha256=hashlib.sha256(sums).hexdigest(),
            )
        self.output = self.root / "dist4"
        with self.assertRaisesRegex(
            publisher.ReleaseError, "distribution name differs"
        ):
            publisher.validate_archive(
                self.assets / self.sdist_name, "other-package", "0.0.50"
            )

    def test_sdist_validates_nested_package_metadata_as_well_as_root(self) -> None:
        bad_metadata = b"Metadata-Version: 2.4\nName: truss-toolkit\nVersion: 0.0.50\nRequires-Dist: umf-core @ file:///tmp/core.whl\n\n"
        path = self.root / self.sdist_name
        path.write_bytes(sdist(bad_metadata))
        with self.assertRaisesRegex(publisher.ReleaseError, "direct URL"):
            publisher.validate_archive(path, "truss-toolkit", "0.0.50")

    def test_registry_check_accepts_only_exact_public_file_bytes(self) -> None:
        payload = self.payloads[self.whl_name]
        digest = hashlib.sha256(payload).hexdigest()
        metadata = json.dumps(
            {
                "info": {"name": "truss-toolkit", "version": "0.0.50"},
                "urls": [
                    {
                        "filename": self.whl_name,
                        "digests": {"sha256": digest},
                        "url": "https://files.pythonhosted.org/packages/truss.whl",
                    }
                ],
            }
        ).encode()

        class Response:
            def __init__(self, body: bytes):
                self.body = body

            def __enter__(self) -> "Response":
                return self

            def __exit__(self, *_args: object) -> None:
                return None

            def read(self) -> bytes:
                return self.body

        with patch.object(
            publisher.urllib.request,
            "urlopen",
            side_effect=[Response(metadata), Response(payload)],
        ):
            self.assertEqual(
                publisher.check_pypi(
                    "truss-toolkit", "0.0.50", {self.whl_name: digest}
                ),
                "matching",
            )

        conflict = json.loads(metadata)
        conflict["urls"][0]["digests"]["sha256"] = "0" * 64
        with patch.object(
            publisher.urllib.request,
            "urlopen",
            return_value=Response(json.dumps(conflict).encode()),
        ):
            with self.assertRaisesRegex(publisher.ReleaseError, "version collision"):
                publisher.check_pypi("truss-toolkit", "0.0.50", {self.whl_name: digest})

    def test_visibility_wait_retries_absence_but_propagates_errors(self) -> None:
        with (
            patch.object(publisher, "check_pypi", side_effect=["absent", "matching"]),
            patch.object(publisher.time, "monotonic", side_effect=[0.0, 0.0, 0.0]),
            patch.object(publisher.time, "sleep") as sleep,
        ):
            self.assertEqual(
                publisher.wait_for_pypi("truss-toolkit", "0.0.50", {}, timeout=300),
                "matching",
            )
            sleep.assert_called_once()
        with patch.object(
            publisher, "check_pypi", side_effect=publisher.ReleaseError("HTTP 401")
        ):
            with self.assertRaisesRegex(publisher.ReleaseError, "HTTP 401"):
                publisher.wait_for_pypi("truss-toolkit", "0.0.50", {}, timeout=300)


class ExistingWorkflowBoundsTests(unittest.TestCase):
    def test_browser_mirror_fix_keeps_existing_job_and_stage_bounds(self) -> None:
        workflow = (
            Path(__file__).parents[2] / ".github/workflows/table-operations.yml"
        ).read_text()
        self.assertIn("timeout-minutes: 45", workflow)
        self.assertIn('budget-gate" browser 10', workflow)
        self.assertEqual(workflow.count("timeout-minutes: ${{ fromJSON(steps."), 12)
        self.assertIn(
            "apt-sources 30 sudo python3 .github/scripts/configure_ubuntu_apt_sources.py --apply",
            workflow,
        )
        self.assertIn("apt-refresh 90 sudo apt-get update", workflow)
        self.assertNotIn("install --with-deps chromium", workflow)


if __name__ == "__main__":
    unittest.main()
