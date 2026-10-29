"""Test-only assembly and transport protocol controls; never a publication record."""
from __future__ import annotations

import hashlib
import io
import json
import os
import tempfile
import time
import unittest
import zipfile
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

import publish_local_release as publisher
from verify_local_release_profile import ProfileError

COMMIT = "c" * 40
WHEELS = (
    "umf_weft-0.4.56-cp39-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl",
    "umf_weft-0.4.56-cp39-abi3-manylinux_2_17_aarch64.manylinux2014_aarch64.whl",
    "umf_weft-0.4.56-cp39-abi3-macosx_11_0_arm64.whl",
)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def tool_manifest_fixture(root: Path, payload_sizes: list[int]) -> Path:
    files_root = root / "tool-files"
    files_root.mkdir(parents=True)
    entries = []
    for index, size in enumerate(payload_sizes):
        path = files_root / f"member-{index:04d}.bin"
        with path.open("wb") as stream:
            stream.truncate(size)
        entries.append({"path": f"tool-files/{path.name}", "sizeBytes": size,
                        "sha256": sha(path.read_bytes())})
    manifest = {"profile": "weft-local-pypi-uploader-tool-closure/1",
                "uploader": "twine", "version": "7.0.0", "pinnedVersion": "7.0.0",
                "pythonExecutable": str(Path(publisher.sys.executable).resolve()),
                "files": entries}
    manifest_path = root / "tool-manifest.json"
    manifest_path.write_text(json.dumps(manifest, sort_keys=True) + "\n")
    return manifest_path


def test_transport_assembly(root: Path):
    """Build transport assets with ten explicit, tiny external witnesses."""
    root.mkdir(parents=True, exist_ok=False)
    assets = root / "assets"
    assets.mkdir()
    payloads = {}
    for name in WHEELS:
        wheel = io.BytesIO()
        with zipfile.ZipFile(wheel, "w", zipfile.ZIP_DEFLATED) as archive:
            archive.writestr("umf_weft-0.4.56.dist-info/METADATA",
                "Metadata-Version: 2.1\nName: umf-weft\nVersion: 0.4.56\n\n")
        payloads[name] = wheel.getvalue()
    table_root, truss_root = root / "table-software", root / "truss-software"
    table_selected, truss_selected, witnesses = {}, {}, []
    for label, software_root, selected, kind, receipt_sha in (
            ("table", table_root, table_selected, "table012-selected-artifact", "a" * 64),
            ("truss", truss_root, truss_selected, "truss011-selected-runtime-software", "b" * 64)):
        for index in range(5):
            relative = f"{label}/selected-{index}.bin"
            content = f"external {label} software {index}\n".encode()
            path = software_root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(content)
            digest = sha(content)
            selected[relative] = (len(content), digest)
            witnesses.append({"scope": "external selected runtime software", "kind": kind,
                "sourceRoot": str(software_root), "sourcePath": relative,
                "receiptSha256": receipt_sha, "sizeBytes": len(content), "sha256": digest})
    # These assignments are test-local fixture pins. Production defaults remain immutable.
    publisher.TABLE012_ROOT = table_root
    publisher.TABLE012_SOFTWARE_ROOT = table_root
    publisher.TABLE012_SELECTED = table_selected
    publisher.TABLE012_RECEIPT_SHA256 = "a" * 64
    publisher.TRUSS011_ROOT = truss_root
    publisher.TRUSS011_SOFTWARE_ROOT = truss_root
    publisher.TRUSS011_SELECTED = truss_selected
    publisher.TRUSS011_RECEIPT_SHA256 = "b" * 64
    payloads.update({
        "release-pins-0.4.56.json": json.dumps({
            "schema": "weft-selected-release-pins/1", "tag": "v0.4.56",
            "releaseMetadataCommit": COMMIT}).encode(),
        "qualification-0.4.56.json": json.dumps({
            "profile": "synthetic-transport-fixture/1",
            "status": "VERIFIED_FOR_ASSEMBLY", "metadata": {"commit": COMMIT},
            "sourceComparison": {"selectedBuildInputsEqual": True},
            "externalSelectedSoftware": witnesses}).encode(),
        "v0.4.56.md": b"test-only local transport protocol fixture\n",
    })
    evidence_index = json.dumps({
        "profile": "weft-local-release-evidence-index/1", "files": [],
        "externalSelectedSoftware": witnesses, "indexExcludedFromItsOwnRoster": True,
    }, sort_keys=True).encode()
    evidence_buffer = io.BytesIO()
    with zipfile.ZipFile(evidence_buffer, "w", zipfile.ZIP_DEFLATED) as archive:
        archive.writestr("evidence-index.json", evidence_index)
    payloads["qualification-evidence-0.4.56.zip"] = evidence_buffer.getvalue()
    sums = {}
    for name, data in payloads.items():
        (assets / name).write_bytes(data)
        sums[name] = sha(data)
    inventory = {"profile": "weft-local-release-asset-inventory/1",
                 "release": "v0.4.56", "status": "ASSEMBLED_LOCAL_UNSIGNED",
                 "assets": [{"name": n, "bytes": len(payloads[n]), "sha256": sums[n]}
                            for n in sorted(payloads)],
                 "excludedFromInventory": ["asset-inventory.json", "SHA256SUMS"]}
    inventory_bytes = (json.dumps(inventory, sort_keys=True) + "\n").encode()
    (assets / "asset-inventory.json").write_bytes(inventory_bytes)
    sums["asset-inventory.json"] = sha(inventory_bytes)
    sums_bytes = "".join(f"{sums[n]}  {n}\n" for n in sorted(sums)).encode()
    (assets / "SHA256SUMS").write_bytes(sums_bytes)
    sums_sha = sha(sums_bytes)
    review = root / "final-review.json"
    review.write_text(json.dumps({"profile": "weft-local-final-assembly-review/1",
        "status": "ACCEPTED", "assemblySha256": sums_sha,
        "inventorySha256": sha(inventory_bytes),
        "finalReviewExcludedFromEvidenceZip": True}))
    return assets, review, sums_sha


class FakeSocket:
    def settimeout(self, timeout):
        self.timeout = timeout


class FakeResponse(io.BytesIO):
    def __init__(self, raw: bytes, status: int = 200):
        super().__init__(raw)
        self.status = status
        self.fp = SimpleNamespace(raw=SimpleNamespace(_sock=FakeSocket()))

    def __enter__(self): return self
    def __exit__(self, *args): self.close()
    def geturl(self): return "https://objects.example.invalid/signed?token=redacted-test-value"


class LocalAssemblyTests(unittest.TestCase):
    def test_pypi_tool_manifest_file_count_exact_boundary(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = tool_manifest_fixture(root, [0] * publisher.MAX_TOOL_FILES)
            checked = publisher._validate_pypi_tool_manifest(manifest, sha(manifest.read_bytes()))
            self.assertEqual(len(checked["files"]), 2048)
            extra = root / "tool-files" / "member-extra.bin"
            extra.write_bytes(b"")
            data = json.loads(manifest.read_text())
            data["files"].append({"path": "tool-files/member-extra.bin", "sizeBytes": 0,
                                  "sha256": sha(b"")})
            manifest.write_text(json.dumps(data, sort_keys=True) + "\n")
            with self.assertRaisesRegex(ProfileError, "roster is outside its bound"):
                publisher._validate_pypi_tool_manifest(manifest, sha(manifest.read_bytes()))

    def test_pypi_tool_manifest_total_bytes_exact_boundary(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            manifest = tool_manifest_fixture(root, [16 * 1024 * 1024, 16 * 1024 * 1024, 0])
            checked = publisher._validate_pypi_tool_manifest(manifest, sha(manifest.read_bytes()))
            self.assertEqual(len(checked["files"]), 3)
            extra = root / "tool-files" / "member-0002.bin"
            extra.write_bytes(b"x")
            data = json.loads(manifest.read_text())
            data["files"][2]["sizeBytes"] = 1
            data["files"][2]["sha256"] = sha(b"x")
            manifest.write_text(json.dumps(data, sort_keys=True) + "\n")
            with self.assertRaisesRegex(ProfileError, "exceeds its complete 32 MiB bound"):
                publisher._validate_pypi_tool_manifest(manifest, sha(manifest.read_bytes()))

    def test_external_runtime_witnesses_are_rehashed_and_zip_payloads_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            table_root = root / "table"
            truss_root = root / "truss"
            table_file = table_root / "one.bin"
            truss_file = truss_root / "two.bin"
            table_file.parent.mkdir(parents=True)
            truss_file.parent.mkdir(parents=True)
            table_file.write_bytes(b"table")
            truss_file.write_bytes(b"truss")
            table_digest, truss_digest = sha(b"table"), sha(b"truss")
            witnesses = [
                {"scope": "external selected runtime software",
                 "kind": "table012-selected-artifact", "sourceRoot": str(table_root),
                 "sourcePath": "one.bin", "receiptSha256": "a" * 64,
                 "sizeBytes": 5, "sha256": table_digest},
                {"scope": "external selected runtime software",
                 "kind": "truss011-selected-runtime-software", "sourceRoot": str(truss_root),
                 "sourcePath": "two.bin", "receiptSha256": "b" * 64,
                 "sizeBytes": 5, "sha256": truss_digest},
            ]
            with patch.multiple(publisher, TABLE012_ROOT=table_root,
                                TABLE012_SOFTWARE_ROOT=table_root, TABLE012_SELECTED={"one.bin": (5, table_digest)},
                                TABLE012_RECEIPT_SHA256="a" * 64,
                                TRUSS011_ROOT=truss_root, TRUSS011_SOFTWARE_ROOT=truss_root,
                                TRUSS011_SELECTED={"two.bin": (5, truss_digest)},
                                TRUSS011_RECEIPT_SHA256="b" * 64):
                clean = io.BytesIO()
                with zipfile.ZipFile(clean, "w") as archive:
                    archive.writestr("metadata/report.json", b"{}")
                with zipfile.ZipFile(io.BytesIO(clean.getvalue())) as archive:
                    self.assertEqual(publisher._verify_external_software_excluded(
                        archive, witnesses), witnesses)
                contaminated = io.BytesIO()
                with zipfile.ZipFile(contaminated, "w") as archive:
                    archive.writestr("metadata/report.json", b"{}")
                    archive.writestr("payloads/one.bin", b"table")
                with zipfile.ZipFile(io.BytesIO(contaminated.getvalue())) as archive:
                    with self.assertRaisesRegex(ProfileError, "payload was included"):
                        publisher._verify_external_software_excluded(archive, witnesses)

    def test_exact_asset_inventory_and_external_review_close(self):
        with tempfile.TemporaryDirectory() as temp:
            assets, review, sums = test_transport_assembly(Path(temp) / "fixture")
            actual, _review = publisher._assembly_assets(assets, sums, review, sha(review.read_bytes()))
            self.assertEqual(len(actual), 9)
            qualification = json.loads((assets / "qualification-0.4.56.json").read_text())
            self.assertEqual(len(qualification["externalSelectedSoftware"]), 10)
            with zipfile.ZipFile(actual["qualification-evidence-0.4.56.zip"]) as archive:
                index = json.loads(archive.read("evidence-index.json"))
                self.assertEqual(len(index["externalSelectedSoftware"]), 10)
                with self.assertRaisesRegex(ProfileError, "witness roster is incomplete"):
                    publisher._verify_external_software_excluded(
                        archive, index["externalSelectedSoftware"][:-1])

    def test_oversized_notes_refuse_before_packet_read_or_output_creation(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            packet = root / "intentionally-absent.json"
            notes = root / "v0.4.56.md"
            notes.write_bytes(b"x" * (publisher.MAX_ENTRY + 1))
            output = root / "new-output"
            with self.assertRaisesRegex(ProfileError, "notes exceed"):
                publisher.assemble(packet, notes, output, metadata_review_sha256="0" * 64)
            self.assertFalse(output.exists())

    def test_missing_pypi_credential_stops_before_transport(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); assets, review, sums = test_transport_assembly(root / "fixture")
            with patch.dict(os.environ, {}, clear=True), patch.object(publisher, "_request_json") as api:
                with self.assertRaisesRegex(ProfileError, "TWINE token"):
                    publisher.publish_pypi(assets, sums, review, tool_manifest=root / "absent.json",
                        tool_manifest_sha256="0" * 64, repository="org/repo",
                        metadata_commit=COMMIT, expected_review_sha256=sha(review.read_bytes()))
                api.assert_not_called()

    def test_asset_readback_uses_per_request_child_deadline(self):
        observed = {}
        def fake_urlopen(request, timeout=0):
            observed["timeout"] = timeout
            return FakeResponse(b"asset")
        def bounded(response, cap, deadline):
            observed["deadline"] = deadline
            return b"asset"
        with patch.object(publisher.time, "monotonic", return_value=100.0), \
             patch("urllib.request.urlopen", side_effect=fake_urlopen), \
             patch.object(publisher, "_bounded_response_read", side_effect=bounded):
            status, _, payload = publisher._github_asset_bytes(
                "https://api.github.com/repos/org/repo", 19, None, 1000.0)
        self.assertEqual((status, payload), (200, b"asset"))
        self.assertEqual(observed["timeout"], publisher.CHILD_SECONDS)
        self.assertEqual(observed["deadline"], 100.0 + publisher.CHILD_SECONDS)

    def test_response_capture_uses_single_read_and_enforces_wall_deadline(self):
        clock = {"now": 0.0}
        class SlowResponse(FakeResponse):
            def read1(self, size=-1):
                chunk = super().read1(size)
                clock["now"] += 2.0
                return chunk
        response = SlowResponse(b"chunk")
        with patch.object(publisher.time, "monotonic", side_effect=lambda: clock["now"]):
            with self.assertRaisesRegex(ProfileError, "absolute child deadline"):
                publisher._bounded_response_read(response, 64, 1.0)

    def test_response_capture_fails_closed_without_bounded_socket_shape(self):
        class Unbounded:
            def read1(self, size=-1):
                return b""
        with self.assertRaisesRegex(ProfileError, "bounded read1/socket shape"):
            publisher._bounded_response_read(Unbounded(), 64, time.monotonic() + 1)

    def test_request_json_reads_real_stdlib_http_error_wrapper_bounded_and_closes(self):
        cases = (
            (b'{"message":"Not Found"}', {"message": "Not Found"}, False, None),
            (b"", {}, False, None),
            (b"too long", None, False, "byte cap"),
            (b"deadline", None, True, "absolute child deadline"),
        )
        for body, expected, slow, refusal in cases:
            with self.subTest(body=body, slow=slow):
                clock = {"now": 0.0}

                class ErrorResponse(FakeResponse):
                    def read1(self, size=-1):
                        chunk = super().read1(size)
                        if slow:
                            clock["now"] += 2.0
                        return chunk

                inner = ErrorResponse(body)
                error = publisher.urllib.error.HTTPError(
                    "https://api.github.com/repos/org/repo/git/ref/tags/v0.4.56",
                    404, "Not Found", None, inner)
                with patch("urllib.request.urlopen", side_effect=error):
                    if slow:
                        with patch.object(publisher.time, "monotonic", side_effect=lambda: clock["now"]):
                            with self.assertRaisesRegex(ProfileError, refusal):
                                publisher._request_json("https://api.github.com/repos/org/repo/git/ref/tags/v0.4.56",
                                                        "GET", "test-token", timeout=1.0)
                    elif refusal:
                        with patch.object(publisher, "MAX_ENTRY", 4):
                            with self.assertRaisesRegex(ProfileError, refusal):
                                publisher._request_json("https://api.github.com/repos/org/repo/git/ref/tags/v0.4.56",
                                                        "GET", "test-token")
                    else:
                        status, value, raw = publisher._request_json(
                            "https://api.github.com/repos/org/repo/git/ref/tags/v0.4.56",
                            "GET", "test-token")
                        self.assertEqual((status, value, raw), (404, expected, body))
                self.assertTrue(inner.closed)

        class MissingSocket:
            def __init__(self):
                self.closed = False
            def read1(self, size=-1):
                return b""
            def close(self):
                self.closed = True

        inner = MissingSocket()
        error = publisher.urllib.error.HTTPError("https://api.github.com/", 404,
                                                  "Not Found", None, inner)
        with patch("urllib.request.urlopen", side_effect=error):
            with self.assertRaisesRegex(ProfileError, "bounded read1/socket shape"):
                publisher._request_json("https://api.github.com/", "GET", None)
        self.assertTrue(inner.closed)

    def test_github_auth_stays_unredirected_and_endpoint_is_not_redirect_url(self):
        request_seen = []
        def fake_urlopen(request, timeout=0):
            request_seen.append(request)
            return FakeResponse(b"asset bytes")
        with patch("urllib.request.urlopen", side_effect=fake_urlopen):
            status, endpoint, body = publisher._github_asset_bytes(
                "https://api.github.com/repos/org/repo", 19, "test-secret", time.monotonic() + 5)
        self.assertEqual((status, body), (200, b"asset bytes"))
        self.assertEqual(endpoint, "https://api.github.com/repos/org/repo/releases/assets/19")
        self.assertNotIn("Authorization", request_seen[0].headers)
        self.assertEqual(request_seen[0].unredirected_hdrs["Authorization"], "Bearer test-secret")
        redirected = publisher.urllib.request.HTTPRedirectHandler().redirect_request(
            request_seen[0], FakeResponse(b""), 302, "Found", {},
            "https://objects.example.invalid/signed?token=redacted-test-value")
        self.assertIsNotNone(redirected)
        self.assertNotIn("Authorization", redirected.headers)
        self.assertNotIn("Authorization", redirected.unredirected_hdrs)

    def test_github_draft_upload_readback_and_immutable_promotion(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); assets, review, sums = test_transport_assembly(root / "fixture")
            local = {p.name: p for p in assets.iterdir()}
            by_id = {index + 100: name for index, name in enumerate(sorted(local))}
            remote = [{"name": name, "id": ident, "size": local[name].stat().st_size}
                      for ident, name in by_id.items()]
            calls = []
            published = False
            def api(url, method, token, payload=None, **kwargs):
                nonlocal published
                calls.append((method, url))
                if url.endswith("/actions/permissions"): return 200, {"enabled": True}, b"{}"
                if "/actions/workflows/" in url:
                    ident = int(url.rsplit("/", 1)[1])
                    path = dict(publisher.GITHUB_AUTOMATIC_WORKFLOWS)[ident]
                    return 200, {"id": ident, "path": path, "state": "disabled_manually"}, b"{}"
                if "/git/ref/tags/" in url or "/releases/tags/" in url: return 404, {}, b"{}"
                if url.endswith("/git/refs"): return 201, {"object": {"type": "commit", "sha": COMMIT}}, b"{}"
                if url.endswith("/releases") and method == "POST":
                    return 201, {"id": 777, "draft": True, "tag_name": "v0.4.56",
                        "upload_url": "https://uploads.github.com/repos/org/repo/releases/777/assets{?name,label}"}, b"{}"
                if url.endswith("/releases/777") and method == "GET":
                    return 200, {"id": 777, "draft": not published,
                                 "immutable": published, "assets": remote}, b"{}"
                if url.endswith("/releases/777") and method == "PATCH":
                    published = True
                    return 200, {"id": 777, "draft": False, "immutable": True, "assets": remote}, b"{}"
                raise AssertionError((method, url))
            def urlopen(request, timeout=0):
                if request.get_method() == "POST":
                    import urllib.parse
                    name = urllib.parse.unquote(request.full_url.split("name=", 1)[1])
                    return FakeResponse(json.dumps({"name": name,
                        "size": local[name].stat().st_size}).encode(), 201)
                ident = int(request.full_url.rsplit("/", 1)[1])
                return FakeResponse(local[by_id[ident]].read_bytes())
            with patch.dict(os.environ, {"GITHUB_TOKEN": "test-secret"}), \
                 patch.object(publisher, "_request_json", side_effect=api), \
                 patch("urllib.request.urlopen", side_effect=urlopen):
                result = publisher.publish_github(assets, sums, review,
                    repository="org/repo", metadata_commit=COMMIT,
                    expected_review_sha256=sha(review.read_bytes()))
            self.assertEqual(result["status"], "PUBLISHED_IMMUTABLE")
            self.assertTrue(result["immutable"])
            self.assertGreaterEqual(len(calls), 7)

    def test_github_refuses_unapproved_automatic_workflow_state_before_mutation(self):
        ids = publisher.GITHUB_AUTOMATIC_WORKFLOWS
        cases = ("active", "missing", "unknown", "wrong-id", "wrong-path")
        for case in cases:
            with self.subTest(case=case), tempfile.TemporaryDirectory() as temp:
                rows = {ident: {"id": ident, "path": path, "state": "disabled_manually"}
                        for ident, path in ids}
                target = ids[0][0]
                if case == "active": rows[target]["state"] = "active"
                elif case == "unknown": rows[target]["state"] = "disabled_inactivity"
                elif case == "wrong-id": rows[target]["id"] = str(target)
                elif case == "wrong-path": rows[target]["path"] += ".changed"
                calls = []
                def api(url, method, token, payload=None, **kwargs):
                    calls.append((method, url))
                    if url.endswith("/actions/permissions"): return 200, {"enabled": True}, b"{}"
                    if "/actions/workflows/" in url:
                        ident = int(url.rsplit("/", 1)[1])
                        if case == "missing" and ident == target: return 404, {}, b"{}"
                        return 200, rows[ident], b"{}"
                    raise AssertionError((method, url))
                with patch.dict(os.environ, {"GITHUB_TOKEN": "test-secret"}), \
                     patch.object(publisher, "_verify_metadata_commit"), \
                     patch.object(publisher, "_assembly_assets", return_value=({}, {})), \
                     patch.object(publisher, "_request_json", side_effect=api):
                    with self.assertRaisesRegex(ProfileError, "automatic workflow identity or disabled state"):
                        publisher.publish_github(Path(temp), "0" * 64, Path(temp)/"review.json",
                            repository="org/repo", metadata_commit=COMMIT,
                            expected_review_sha256="0" * 64)
                self.assertTrue(calls)
                self.assertFalse(any(method != "GET" for method, _ in calls))
                self.assertFalse(any("/git/refs" in url or "/releases" in url for _, url in calls))

    def test_pypi_conflict_does_not_launch_uploader(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); assets, review, sums = test_transport_assembly(root / "fixture")
            tool = root / "tool"; tool.mkdir()
            member = tool / "twine.py"; member.write_text("test-only uploader closure\n")
            manifest = {"profile": "weft-local-pypi-uploader-tool-closure/1",
                        "uploader": "twine", "version": "1.0", "pinnedVersion": "1.0",
                        "pythonExecutable": str(Path(publisher.sys.executable).resolve()),
                        "files": [{"path": "twine.py", "sizeBytes": member.stat().st_size,
                                   "sha256": sha(member.read_bytes())}]}
            manifest_path = tool / "manifest.json"
            manifest_path.write_text(json.dumps(manifest))
            with patch.dict(os.environ, {"TWINE_USERNAME": "__token__", "TWINE_PASSWORD": "test-secret"}), \
                 patch.object(publisher, "_request_json", side_effect=[
                     (200, {"draft": False, "prerelease": False, "tag_name": "v0.4.56",
                            "assets": []}, b"{}"),
                     (200, {"object": {"type": "commit", "sha": COMMIT}}, b"{}")]), \
                 patch.object(publisher, "_pypi_readback", return_value="conflict"), \
                 patch("publish_pypi_release.prepare_release", return_value={"files": {
                     name: Path(name) for name in WHEELS}}), \
                 patch.object(publisher, "_bounded_capture") as uploader:
                with self.assertRaisesRegex(ProfileError, "present or conflicts"):
                    publisher.publish_pypi(assets, sums, review,
                        tool_manifest=manifest_path,
                        tool_manifest_sha256=sha(manifest_path.read_bytes()),
                        repository="org/repo", metadata_commit=COMMIT,
                        expected_review_sha256=sha(review.read_bytes()), capture_dir=root / "capture")
                uploader.assert_not_called()

    def test_pypi_absent_version_uploads_once_then_reads_back_exact_files(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); assets, review, sums = test_transport_assembly(root / "fixture")
            tool = root / "tool"; tool.mkdir()
            member = tool / "twine.py"; member.write_text("test-only uploader closure\n")
            manifest = {"profile": "weft-local-pypi-uploader-tool-closure/1",
                        "uploader": "twine", "version": "1.0", "pinnedVersion": "1.0",
                        "pythonExecutable": str(Path(publisher.sys.executable).resolve()),
                        "files": [{"path": "twine.py", "sizeBytes": member.stat().st_size,
                                   "sha256": sha(member.read_bytes())}]}
            manifest_path = tool / "manifest.json"
            manifest_path.write_text(json.dumps(manifest))
            api_calls = []
            def api(url, method, token, payload=None, **kwargs):
                api_calls.append((url, method))
                if url.endswith("/releases/tags/v0.4.56"):
                    return 200, {"draft": False, "prerelease": False,
                                 "tag_name": "v0.4.56", "assets": []}, b"{}"
                if url.endswith("/git/ref/tags/v0.4.56"):
                    return 200, {"object": {"type": "commit", "sha": COMMIT}}, b"{}"
                raise AssertionError((url, method))
            capture_calls = []
            def capture(argv, *, timeout, env, capture_dir, raw_cap=publisher.MAX_RAW_LOGS):
                capture_calls.append((argv, env, raw_cap))
                capture_dir.mkdir(parents=True)
                stdout = capture_dir / "stdout.log"
                stderr = capture_dir / "stderr.log"
                stdout_data = b"twine version 1.0\n" if "--version" in argv else b"uploaded\n"
                stdout.write_bytes(stdout_data); stderr.write_bytes(b"")
                return {"exitCode": 0, "streams": {
                    "stdout": {"rawBytes": len(stdout_data)}, "stderr": {"rawBytes": 0}},
                    "captureFiles": [{"stream": "stdout", "path": str(stdout)},
                                     {"stream": "stderr", "path": str(stderr)}]}
            with patch.dict(os.environ, {"TWINE_USERNAME": "__token__",
                                         "TWINE_PASSWORD": "test-secret"}, clear=True), \
                 patch.object(publisher, "_request_json", side_effect=api), \
                 patch.object(publisher, "_pypi_readback", side_effect=["absent", "matching"]), \
                 patch("publish_pypi_release.prepare_release", return_value={"files": {
                     name: Path(name) for name in WHEELS}}), \
                 patch.object(publisher, "_bounded_capture", side_effect=capture):
                result = publisher.publish_pypi(assets, sums, review,
                    tool_manifest=manifest_path,
                    tool_manifest_sha256=sha(manifest_path.read_bytes()),
                    repository="org/repo", metadata_commit=COMMIT,
                    expected_review_sha256=sha(review.read_bytes()),
                    capture_dir=root / "capture")
            self.assertEqual(result["status"], "PUBLISHED_EXACT_READBACK")
            self.assertEqual(len(capture_calls), 2)
            upload_argv, upload_env, _ = capture_calls[1]
            self.assertEqual(upload_argv[-3:], [str(assets / name) for name in sorted(WHEELS)])
            self.assertNotIn("test-secret", " ".join(upload_argv))
            self.assertEqual(upload_env["TWINE_PASSWORD"], "test-secret")
            self.assertEqual(len(api_calls), 2)

    def test_assembly_rejects_duplicate_checksum_rows_and_changed_public_review(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); assets, review, sums = test_transport_assembly(root / "fixture")
            checksums = assets / "SHA256SUMS"
            checksums.write_bytes(checksums.read_bytes() + checksums.read_bytes().splitlines(keepends=True)[0])
            with self.assertRaises(ProfileError):
                publisher._assembly_assets(assets, sha(checksums.read_bytes()), review,
                                           sha(review.read_bytes()))
            with tempfile.TemporaryDirectory() as second:
                assets2, review2, sums2 = test_transport_assembly(Path(second) / "fixture")
                outside_review_sha = sha(review2.read_bytes())
                changed_review = json.loads(review2.read_text())
                changed_review["status"] = "CHANGED"
                review2.write_text(json.dumps(changed_review, sort_keys=True) + "\n")
                with self.assertRaisesRegex(ProfileError, "outside-archive digest"):
                    publisher._assembly_assets(assets2, sums2, review2, outside_review_sha)


if __name__ == "__main__":
    unittest.main()
