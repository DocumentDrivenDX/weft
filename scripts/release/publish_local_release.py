#!/usr/bin/env python3
"""Offline assembly for the explicit Weft 0.4.56 local release profile.

The command does not upload, create tags/releases, invoke Actions, or read
credentials.  Public writes are deliberately separate from this assembler.
"""

from __future__ import annotations

import argparse
import base64
import csv
import hashlib
import io
import json
import os
import re
import selectors
import signal
import subprocess
import shutil
import stat
import sys
import time
import tempfile
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from email.parser import BytesParser
from pathlib import Path, PurePosixPath
from typing import Any

from verify_local_release_profile import (
    COMMIT,
    MAX_EVIDENCE_FILE,
    MAX_EVIDENCE_TOTAL,
    MAX_PACKET,
    MAX_RAW_LOGS,
    MAX_SELECTED_SOFTWARE,
    MAX_WHEEL,
    TABLE012_ROOT,
    TABLE012_SOFTWARE_ROOT,
    TABLE012_SELECTED,
    TABLE012_RECEIPT_SHA256,
    TRUSS011_ROOT,
    TRUSS011_SOFTWARE_ROOT,
    TRUSS011_SELECTED,
    TRUSS011_RECEIPT_SHA256,
    ProfileError,
    _closed,
    _read_bounded,
    _sha,
    _under,
    validate_packet,
)

MAX_ASSEMBLY = 70 * 1024 * 1024
MAX_EVIDENCE_ZIP = 16 * 1024 * 1024
MAX_ENTRY = 16 * 1024 * 1024
MAX_TOOL_CLOSURE = 16 * 1024 * 1024
SHA256_RE = re.compile(r"[0-9a-f]{64}\Z")
AGGREGATE_SECONDS = 1800
CHILD_SECONDS = 180
TERMINAL_RESERVE_SECONDS = 60
TERMINATION_GRACE_SECONDS = 10
KILL_DRAIN_SECONDS = 1
_LAST_CAPTURE: dict[str, Any] = {}


def sha(data: bytes) -> str:
    return _sha(data)


def _write_new(path: Path, data: bytes) -> None:
    with path.open("xb") as stream:
        stream.write(data)
        stream.flush()
        os.fsync(stream.fileno())


def _durable_result(path: Path, value: dict[str, Any]) -> None:
    raw = (json.dumps(value, sort_keys=True, indent=2) + "\n").encode()
    _need(len(raw) <= MAX_EVIDENCE_TOTAL, "transaction receipt exceeds evidence cap")
    _need(not path.exists() and not path.is_symlink(), "transaction receipt path must be new")
    path.parent.mkdir(parents=True, exist_ok=True)
    _write_new(path, raw)


def _replace_result(path: Path, value: dict[str, Any]) -> None:
    raw = (json.dumps(value, sort_keys=True, indent=2) + "\n").encode()
    _need(len(raw) <= MAX_EVIDENCE_TOTAL, "transaction receipt exceeds evidence cap")
    temporary = path.with_name(path.name + ".tmp")
    _need(not temporary.exists() and not temporary.is_symlink(),
          "transaction receipt temporary path already exists")
    _write_new(temporary, raw)
    os.replace(temporary, path)
    descriptor = os.open(path.parent, os.O_RDONLY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def _seed_result(path: Path, phase: str, assembly_sha: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    _durable_result(path, {"profile": f"weft-local-{phase}-transaction/1",
                           "status": "FAIL_OR_UNKNOWN", "phase": phase,
                           "assemblySha256": assembly_sha,
                           "errorCode": "transaction-in-progress",
                           "error": "durable failure-only seed; replaced only after full closure"})


class _CappedBytesIO(io.BytesIO):
    def __init__(self, cap: int):
        super().__init__()
        self.cap = cap

    def write(self, data: bytes) -> int:
        position = self.tell()
        current_size = self.getbuffer().nbytes
        _need(max(current_size, position + len(data)) <= self.cap,
              "compressed evidence archive exceeds its cap while writing")
        return super().write(data)


def assemble(packet_path: Path, notes_path: Path, output: Path, *,
             metadata_review_sha256: str) -> dict[str, Any]:
    # Refuse an oversized notes file before the verifier reads any wheel or
    # evidence payload. The bounded reader repeats this check at consumption.
    _need(not notes_path.is_symlink(), "release notes must be a regular non-symlink file")
    try:
        notes_stat = notes_path.stat(follow_symlinks=False)
    except OSError as exc:
        raise ProfileError("release notes cannot be statted") from exc
    _need(notes_stat.st_size <= MAX_ENTRY, "release notes exceed the metadata cap")
    notes_data = _read_bounded(notes_path, MAX_ENTRY, "release notes")
    _need(notes_path.name == "v0.4.56.md", "versioned release notes path differs")
    notes = notes_data
    _need(0 < len(notes) <= MAX_ENTRY, "release notes exceed the metadata cap")
    verified = validate_packet(packet_path,
                               expected_metadata_review_sha256=metadata_review_sha256)
    notes_source = verified["metadataFiles"].get(
        "docs/helix/05-deploy/releases/v0.4.56.md")
    _need(isinstance(notes_source, dict), "versioned notes are absent from reviewed M")
    reviewed_notes = _read_bounded(notes_source["physicalPath"], MAX_ENTRY,
                                   "reviewed M release notes")
    _need(notes == reviewed_notes,
          "consumed release notes differ from the exact reviewed M source bytes")
    _need(not output.exists() and not output.is_symlink(), "assembly output must be a new path")
    wheel_facts = [{"name": w["filename"], "sha256": w["sha256"]} for w in verified["wheels"]]
    pin = {
        "schema": "weft-selected-release-pins/1", "tag": "v0.4.56",
        "sourceCommit": verified["producer"]["commit"],
        "releaseMetadataCommit": verified["metadata"]["commit"],
        "candidateAssets": wheel_facts,
        "localProfile": {"profile": verified["profile"],
                         "packetSha256": verified["packetSha256"],
                         "qualification": verified["qualification"]},
    }
    pin_bytes = (json.dumps(pin, indent=2, sort_keys=True) + "\n").encode()
    qualification = {
        "profile": verified["profile"], "status": "VERIFIED_FOR_ASSEMBLY",
        "release": verified["release"], "producer": verified["producer"],
        "metadata": verified["metadata"], "sourceComparison": verified["sourceComparison"],
        "packetSha256": verified["packetSha256"],
        "wheels": [{k: wheel[k] for k in ("role", "filename", "bytes", "sha256")}
                   for wheel in verified["wheels"]],
        "gates": verified["gates"], "reviews": verified["reviews"],
        "externalSelectedSoftware": verified["externalSelectedSoftware"],
        "qualification": verified["qualification"],
    }
    qualification_bytes = (json.dumps(qualification, indent=2, sort_keys=True) + "\n").encode()
    evidence_index_size = (1024 + sum(len(row["path"]) + 128
                                       for row in verified["evidenceInventory"])
                           + sum(len(row["sourcePath"]) + 256
                                 for row in verified["externalSelectedSoftware"]))
    evidence_expanded = verified["evidenceBytes"] + evidence_index_size
    _need(evidence_expanded <= MAX_EVIDENCE_TOTAL,
          "preflight packet/source/gate/log evidence plus index exceeds 16 MiB")
    predicted = (sum(wheel["bytes"] for wheel in verified["wheels"]) + len(notes)
                 + len(pin_bytes) + len(qualification_bytes) + evidence_expanded + 1024 * 1024)
    _need(predicted <= MAX_ASSEMBLY,
          "preflight complete release family exceeds the original 70 MiB cap")
    _need(len(pin_bytes) <= MAX_ENTRY and len(qualification_bytes) <= MAX_ENTRY,
          "preflight generated metadata exceeds the per-entry cap")
    output.mkdir(parents=True)
    root = packet_path.resolve().parent
    copied: dict[str, str] = {}
    sizes: dict[str, int] = {}

    try:
        for wheel in verified["wheels"]:
            data = _read_bounded(wheel["path"], MAX_WHEEL, f"assembly wheel {wheel['role']}")
            _need(sha(data) == wheel["sha256"], "wheel changed after verification")
            _write_new(output / wheel["filename"], data)
            copied[wheel["filename"]] = wheel["sha256"]
            sizes[wheel["filename"]] = len(data)

        _write_new(output / "release-pins-0.4.56.json", pin_bytes)
        copied["release-pins-0.4.56.json"] = sha(pin_bytes)
        sizes["release-pins-0.4.56.json"] = len(pin_bytes)

        _write_new(output / "qualification-0.4.56.json", qualification_bytes)
        copied["qualification-0.4.56.json"] = sha(qualification_bytes)
        sizes["qualification-0.4.56.json"] = len(qualification_bytes)

        # Evidence ZIP contains bounded source receipts/reviews, never software wheels.
        evidence_names: dict[str, tuple[Path, str, int]] = {}
        rows = list(verified["evidenceRefs"])
        packet_path = verified["packetPath"]
        packet_bytes = _read_bounded(packet_path, MAX_PACKET, "packet during assembly")
        _need(len(packet_bytes) <= MAX_PACKET and sha(packet_bytes) == verified["packetSha256"],
              "packet changed after verification")
        evidence_names["packet.json"] = (packet_path, verified["packetSha256"], len(packet_bytes))
        selected_paths = {"packet.json"}
        for row in rows:
            raw_path = row["path"]
            path = _under(root, raw_path, "assembly evidence")
            relative = PurePosixPath(raw_path).as_posix()
            _need(relative not in selected_paths, "evidence paths must be unique")
            selected_paths.add(relative)
            data = _read_bounded(path, MAX_ENTRY, "assembly evidence input")
            size = len(data)
            _need(size == row["sizeBytes"] and sha(data) == row["sha256"],
                  "evidence changed after packet verification")
            evidence_names[relative] = (path, row["sha256"], size)
        evidence_manifest = {
            "profile": "weft-local-release-evidence-index/1",
            "files": [{"path": name, "bytes": size, "sha256": digest}
                      for name, (_, digest, size) in sorted(evidence_names.items())],
            "externalSelectedSoftware": verified["externalSelectedSoftware"],
            "indexExcludedFromItsOwnRoster": True,
        }
        evidence_index = (json.dumps(evidence_manifest, indent=2, sort_keys=True) + "\n").encode()
        _need(sum(size for _, _, size in evidence_names.values()) + len(evidence_index)
              <= MAX_EVIDENCE_TOTAL, "evidence archive expanded size exceeds cap")
        evidence_path = output / "qualification-evidence-0.4.56.zip"
        compressed_evidence = _CappedBytesIO(MAX_EVIDENCE_ZIP)
        with zipfile.ZipFile(compressed_evidence, "w", compression=zipfile.ZIP_DEFLATED,
                             compresslevel=9) as archive:
            archive.writestr("evidence-index.json", evidence_index)
            for name, (path, digest, size) in sorted(evidence_names.items()):
                data = _read_bounded(path, MAX_ENTRY, "evidence archive source")
                _need(len(data) == size and sha(data) == digest,
                      "evidence changed during archive write")
                archive.writestr(name, data)
        evidence_bytes = compressed_evidence.getvalue()
        evidence_size = len(evidence_bytes)
        _need(evidence_size <= MAX_EVIDENCE_ZIP, "compressed evidence archive exceeds cap")
        expected_zip_names = ["evidence-index.json", *sorted(evidence_names)]
        try:
            with zipfile.ZipFile(io.BytesIO(evidence_bytes)) as check:
                entries = check.infolist()
                _need([entry.filename for entry in entries] == expected_zip_names
                      and all(not entry.is_dir() and entry.file_size <= MAX_ENTRY
                              for entry in entries),
                      "written evidence ZIP roster/member bounds differ")
                index_info = check.getinfo("evidence-index.json")
                _need(index_info.file_size == len(evidence_index)
                      and check.read(index_info) == evidence_index,
                      "written evidence ZIP index differs from its closed roster")
                for name, (_, expected_digest, expected_size) in evidence_names.items():
                    info = check.getinfo(name)
                    _need(info.file_size == expected_size and sha(check.read(info)) == expected_digest,
                          f"written evidence ZIP member differs: {name}")
                _verify_external_software_excluded(check, verified["externalSelectedSoftware"])
        except (OSError, zipfile.BadZipFile, KeyError) as exc:
            raise ProfileError("written evidence ZIP could not be physically reclosed") from exc
        _write_new(evidence_path, evidence_bytes)
        evidence_digest = sha(evidence_bytes)
        copied[evidence_path.name] = evidence_digest
        sizes[evidence_path.name] = evidence_size

        _need(len(notes) <= MAX_ENTRY, "release notes exceed cap")
        _write_new(output / "v0.4.56.md", notes)
        copied["v0.4.56.md"] = sha(notes)
        sizes["v0.4.56.md"] = len(notes)

        _need(sum(sizes.values()) <= MAX_ASSEMBLY, "release software/metadata family exceeds cap")
        inventory = {
            "profile": "weft-local-release-asset-inventory/1",
            "release": "v0.4.56",
            "status": "ASSEMBLED_LOCAL_UNSIGNED",
            "assets": [{"name": name, "bytes": sizes[name], "sha256": copied[name]}
                       for name in sorted(copied)],
            "excludedFromInventory": ["asset-inventory.json", "SHA256SUMS"],
        }
        inventory_bytes = (json.dumps(inventory, indent=2, sort_keys=True) + "\n").encode()
        _write_new(output / "asset-inventory.json", inventory_bytes)
        sums_rows = dict(copied)
        sums_rows["asset-inventory.json"] = sha(inventory_bytes)
        sums_bytes = "".join(f"{sums_rows[name]}  {name}\n" for name in sorted(sums_rows)).encode("ascii")
        _write_new(output / "SHA256SUMS", sums_bytes)
        final_files = sorted(output.iterdir(), key=lambda p: p.name)
        _need(len(final_files) == 9 and all(p.is_file() and not p.is_symlink() for p in final_files),
              "final assembly asset roster differs")
        _need(sum(p.stat().st_size for p in final_files) <= MAX_ASSEMBLY,
              "complete assembly exceeds cap")
        for name, expected in sums_rows.items():
            actual = _read_bounded(output / name, MAX_ASSEMBLY, "final assembly asset")
            _need(sha(actual) == expected, f"final assembled asset changed: {name}")
        return {"status": "ASSEMBLED_LOCAL_UNSIGNED", "output": str(output),
                "files": len(final_files), "bytes": sum(p.stat().st_size for p in final_files),
                "sha256sums": sha(sums_bytes), "pinsSha256": sha(pin_bytes),
                "qualificationSha256": sha(qualification_bytes),
                "evidenceSha256": evidence_digest}
    except BaseException:
        # Preserve partial output for forensic review; never silently overwrite it.
        raise


def _need(ok: bool, message: str) -> None:
    if not ok:
        raise ProfileError(message)


def _verify_external_software_excluded(archive: zipfile.ZipFile,
                                       witnesses: Any) -> list[dict[str, Any]]:
    """Rehash the finite selected software witnesses and prove payload exclusion."""
    expected: dict[tuple[str, str], tuple[Path, int, str, str, str]] = {}
    for rel, (size, digest) in TABLE012_SELECTED.items():
        expected[(str(TABLE012_SOFTWARE_ROOT), rel)] = (
            TABLE012_SOFTWARE_ROOT, size, digest, TABLE012_RECEIPT_SHA256,
            "table012-selected-artifact")
    for rel, (size, digest) in TRUSS011_SELECTED.items():
        expected[(str(TRUSS011_SOFTWARE_ROOT), rel)] = (
            TRUSS011_SOFTWARE_ROOT, size, digest, TRUSS011_RECEIPT_SHA256,
            "truss011-selected-runtime-software")
    _need(isinstance(witnesses, list) and len(witnesses) == len(expected),
          "external selected-software witness roster is incomplete")
    seen: set[tuple[str, str]] = set()
    member_names = {info.filename for info in archive.infolist()}
    for raw in witnesses:
        item = _closed(raw, {"scope", "kind", "sourceRoot", "sourcePath", "receiptSha256",
                             "sizeBytes", "sha256"}, "external selected-software witness")
        key = (item["sourceRoot"], item["sourcePath"])
        _need(key in expected and key not in seen, "external software witness is unknown or duplicated")
        seen.add(key)
        root, size, digest, receipt_sha, kind = expected[key]
        _need(item == {"scope": "external selected runtime software", "kind": kind,
                       "sourceRoot": str(root), "sourcePath": key[1],
                       "receiptSha256": receipt_sha, "sizeBytes": size, "sha256": digest}
              and 0 <= size <= MAX_SELECTED_SOFTWARE,
              "external software witness differs from its original selected roster")
        source_root = root.resolve(strict=True)
        path = root / key[1]
        current = root
        for part in Path(key[1]).parts:
            current = current / part
            _need(not current.is_symlink(),
                  "external selected software path contains a symlink")
        _need(not root.is_symlink() and path.resolve(strict=True).is_relative_to(source_root),
              "external selected software escapes its pinned source root")
        before = path.stat(follow_symlinks=False)
        before_id = (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns)
        first = _read_bounded(path, MAX_SELECTED_SOFTWARE, "external selected software")
        middle = path.stat(follow_symlinks=False)
        second = _read_bounded(path, MAX_SELECTED_SOFTWARE, "external selected software recheck")
        after = path.stat(follow_symlinks=False)
        after_id = (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns)
        _need(stat.S_ISREG(after.st_mode) and before_id == after_id
              == (middle.st_dev, middle.st_ino, middle.st_size, middle.st_mtime_ns)
              and len(first) == len(second) == size and sha(first) == sha(second) == digest,
              "external selected software changed or has a different hash")
        _need(key[1] not in member_names,
              "external selected software path appears as a ZIP evidence member")
        for info in archive.infolist():
            if info.is_dir() or info.file_size != size:
                continue
            _need(sha(archive.read(info)) != digest,
                  "external selected software payload was included in the evidence ZIP")
    _need(seen == set(expected), "external software witness roster has omitted rows")
    return list(witnesses)


def _bounded_capture(argv: list[str], *, timeout: float, env: dict[str, str] | None = None,
                    capture_dir: Path | None = None,
                    raw_cap: int = MAX_RAW_LOGS) -> dict[str, Any]:
    _need(0 < timeout <= CHILD_SECONDS, "public child deadline is outside the local profile")
    _need(0 < raw_cap <= MAX_RAW_LOGS, "public child raw capture allowance is invalid")
    started = time.monotonic()
    proc = subprocess.Popen(argv, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE,
                            stderr=subprocess.PIPE, env=env, close_fds=True,
                            start_new_session=True)
    selector = selectors.DefaultSelector()
    streams = {proc.stdout: "stdout", proc.stderr: "stderr"}
    retained = {"stdout": bytearray(), "stderr": bytearray()}
    hasher = {"stdout": hashlib.sha256(), "stderr": hashlib.sha256()}
    counts = {"stdout": 0, "stderr": 0}
    for stream, name in streams.items():
        os.set_blocking(stream.fileno(), False)
        selector.register(stream, selectors.EVENT_READ, name)
    deadline = started + timeout
    term_deadline = None
    kill_deadline = None
    timed_out = output_capped = drain_expired = False
    secrets = [value.encode() for key in ("GITHUB_TOKEN", "TWINE_PASSWORD")
               if (value := (env or {}).get(key))]
    while selector.get_map() or proc.poll() is None:
        now = time.monotonic()
        if now >= deadline and term_deadline is None:
            timed_out = True
            _signal_process_group(proc.pid, signal.SIGTERM)
            term_deadline = now + TERMINATION_GRACE_SECONDS
        if term_deadline is not None and now >= term_deadline and kill_deadline is None:
            _signal_process_group(proc.pid, signal.SIGKILL)
            kill_deadline = now + KILL_DRAIN_SECONDS
        if kill_deadline is not None and now >= kill_deadline and (selector.get_map() or proc.poll() is None):
            drain_expired = True
            for key in list(selector.get_map().values()):
                try: selector.unregister(key.fileobj)
                except Exception: pass
                try: key.fileobj.close()
                except OSError: pass
            break
        wait_until = deadline if term_deadline is None else term_deadline
        if kill_deadline is not None: wait_until = kill_deadline
        events = selector.select(max(0.01, min(0.1, wait_until - now)))
        if not selector.get_map() and proc.poll() is None and not events:
            time.sleep(0.05)
        for key, _ in events:
            name = key.data
            try:
                chunk = os.read(key.fileobj.fileno(), 65536)
            except BlockingIOError:
                continue
            if not chunk:
                selector.unregister(key.fileobj)
                key.fileobj.close()
                continue
            counts[name] += len(chunk)
            hasher[name].update(chunk)
            room = max(0, raw_cap // 2 - len(retained[name]))
            if room:
                retained[name].extend(chunk[:room])
            if counts["stdout"] + counts["stderr"] > raw_cap or counts[name] > raw_cap // 2:
                output_capped = True
                if term_deadline is None:
                    _signal_process_group(proc.pid, signal.SIGTERM)
                    term_deadline = time.monotonic() + TERMINATION_GRACE_SECONDS
    try:
        exit_code = proc.wait(timeout=TERMINATION_GRACE_SECONDS)
    except subprocess.TimeoutExpired:
        _signal_process_group(proc.pid, signal.SIGKILL)
        try: exit_code = proc.wait(timeout=KILL_DRAIN_SECONDS)
        except subprocess.TimeoutExpired: exit_code = None
    redacted = {}
    for name in ("stdout", "stderr"):
        data = bytes(retained[name])
        for secret in secrets:
            if secret: data = data.replace(secret, b"[REDACTED]")
        redacted[name] = data
    files = []
    if capture_dir is not None:
        capture_dir.mkdir(parents=True, exist_ok=False)
        for name in ("stdout", "stderr"):
            path = capture_dir / f"{name}.log"
            _write_new(path, redacted[name])
            files.append({"stream": name, "path": str(path), "sizeBytes": len(redacted[name]),
                          "sha256": sha(redacted[name]), "rawBytes": counts[name],
                          "rawSha256": hasher[name].hexdigest(),
                          "truncated": counts[name] > len(retained[name])})
    result = {"exitCode": exit_code, "durationSeconds": time.monotonic() - started,
              "streams": {name: {"rawBytes": counts[name], "rawSha256": hasher[name].hexdigest(),
                                  "retainedBytes": len(retained[name]),
                                  "truncated": counts[name] > len(retained[name])}
                          for name in ("stdout", "stderr")},
              "captureFiles": files, "timedOut": timed_out, "outputCapped": output_capped,
              "drainExpired": drain_expired}
    _LAST_CAPTURE.clear(); _LAST_CAPTURE.update(result)
    _need(not timed_out and not output_capped and not drain_expired and exit_code == 0,
          "bounded public child failed, timed out, or exceeded capture limits")
    return result


def _signal_process_group(pid: int, signum: int) -> None:
    """Bound the uploader and any inherited-pipe descendants as one child."""
    try:
        os.killpg(pid, signum)
    except ProcessLookupError:
        pass


def _assembly_assets(directory: Path, expected_sha: str, review_path: Path,
                     expected_review_sha256: str) -> tuple[dict[str, Path], dict[str, Any]]:
    _need(directory.is_dir() and not directory.is_symlink(), "assembly directory is unsafe")
    files = sorted(directory.iterdir(), key=lambda p: p.name)
    _need(len(files) == 9 and all(p.is_file() and not p.is_symlink() for p in files),
          "assembly must contain exactly the nine frozen assets")
    preflight_sizes = {}
    preflight_total = 0
    for path in files:
        try:
            st = path.stat(follow_symlinks=False)
        except OSError as exc:
            raise ProfileError("assembly asset cannot be statted") from exc
        _need(stat.S_ISREG(st.st_mode) and 0 <= st.st_size <= MAX_ASSEMBLY,
              "assembly contains an unsafe or oversized asset")
        preflight_sizes[path.name] = st.st_size
        preflight_total += st.st_size
    _need(preflight_total <= MAX_ASSEMBLY,
          "complete assembly exceeds the 70 MiB family cap before reads")
    inv = directory / "asset-inventory.json"
    inv_data = _read_bounded(inv, MAX_ENTRY, "asset inventory")
    sums_data = _read_bounded(directory / "SHA256SUMS", MAX_ENTRY, "SHA256SUMS")
    _need(sha(sums_data) == expected_sha, "supplied digest is not the exact assembly SHA256SUMS")
    assets = json.loads(_read_bounded(inv, MAX_ENTRY, "asset inventory"))
    _need(assets.get("status") == "ASSEMBLED_LOCAL_UNSIGNED"
          and assets.get("profile") == "weft-local-release-asset-inventory/1"
          and assets.get("release") == "v0.4.56"
          and assets.get("excludedFromInventory") == ["asset-inventory.json", "SHA256SUMS"],
          "assembly inventory is not an unsigned local assembly")
    declared = {row["name"]: row for row in assets.get("assets", [])}
    _need(len(declared) == len(assets.get("assets", [])) and set(declared) ==
          {p.name for p in files} - {"asset-inventory.json", "SHA256SUMS"},
          "asset inventory roster differs")
    sums = {}
    try:
        sum_lines = sums_data.decode("ascii").splitlines()
    except UnicodeDecodeError as exc:
        raise ProfileError("SHA256SUMS is not ASCII") from exc
    for line in sum_lines:
        match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9_.+-]+)", line)
        _need(match is not None, "SHA256SUMS entry is malformed")
        _need(match.group(2) not in sums, "SHA256SUMS has duplicate asset names")
        sums[match.group(2)] = match.group(1)
    _need(set(sums) == {p.name for p in files} - {"SHA256SUMS"},
          "SHA256SUMS roster differs")
    total = 0
    for name, row in declared.items():
        path = directory / name
        data = _read_bounded(path, MAX_ASSEMBLY, "assembly asset")
        size, digest = len(data), sha(data)
        _need(size == preflight_sizes[name], f"assembly asset changed after preflight: {name}")
        _need(row == {"name": name, "bytes": size, "sha256": digest}
              and sums.get(name) == digest, f"assembly asset differs: {name}")
        total += size
    inventory_digest = sha(inv_data)
    _need(sums.get("asset-inventory.json") == inventory_digest and total <= MAX_ASSEMBLY,
          "assembly checksum/70 MiB family differs")
    try:
        qualification = json.loads(_read_bounded(directory / "qualification-0.4.56.json",
                                                  MAX_ENTRY, "qualification facts"))
        _need(isinstance(qualification, dict), "qualification facts must be a JSON object")
        evidence_raw = _read_bounded(directory / "qualification-evidence-0.4.56.zip",
                                     MAX_EVIDENCE_ZIP, "qualification evidence ZIP")
        with zipfile.ZipFile(io.BytesIO(evidence_raw)) as archive:
            members = archive.infolist()
            _need(members and members[0].filename == "evidence-index.json"
                  and len({item.filename for item in members}) == len(members),
                  "evidence ZIP member roster is malformed")
            index_raw = archive.read("evidence-index.json")
            index = json.loads(index_raw)
            _need(isinstance(index, dict)
                  and set(index) == {"profile", "files", "externalSelectedSoftware",
                                     "indexExcludedFromItsOwnRoster"}
                  and index.get("profile") == "weft-local-release-evidence-index/1"
                  and index.get("indexExcludedFromItsOwnRoster") is True
                  and index.get("externalSelectedSoftware") == qualification.get("externalSelectedSoftware"),
                  "evidence index does not bind qualification's external software witnesses")
            rows = index.get("files")
            _need(isinstance(rows, list), "evidence ZIP index file roster is absent")
            names = [row.get("path") for row in rows if isinstance(row, dict)]
            _need(len(names) == len(rows) and names == sorted(set(names))
                  and [item.filename for item in members] == ["evidence-index.json", *names],
                  "evidence ZIP member list differs from its sorted complete index")
            expanded = len(index_raw)
            for row in rows:
                _need(set(row) == {"path", "bytes", "sha256"}
                      and isinstance(row["path"], str) and type(row["bytes"]) is int
                      and 0 <= row["bytes"] <= MAX_ENTRY
                      and isinstance(row["sha256"], str) and SHA256_RE.fullmatch(row["sha256"]),
                      "evidence ZIP index row is malformed")
                payload = archive.read(row["path"])
                _need(len(payload) == row["bytes"] and sha(payload) == row["sha256"],
                      "evidence ZIP payload differs from its index")
                expanded += len(payload)
            _need(expanded <= MAX_EVIDENCE_TOTAL,
                  "evidence ZIP expanded metadata exceeds unchanged 16 MiB cap")
            _verify_external_software_excluded(archive, index["externalSelectedSoftware"])
    except (OSError, UnicodeError, json.JSONDecodeError, zipfile.BadZipFile, KeyError) as exc:
        raise ProfileError("qualification evidence ZIP could not be reclosed") from exc
    _need(qualification.get("externalSelectedSoftware") == index["externalSelectedSoftware"],
          "qualification facts omit or change external selected software witnesses")
    review_raw = _read_bounded(review_path, MAX_EVIDENCE_FILE, "post-assembly review")
    _need(isinstance(expected_review_sha256, str)
          and SHA256_RE.fullmatch(expected_review_sha256)
          and sha(review_raw) == expected_review_sha256,
          "post-assembly review differs from the independently supplied outside-archive digest")
    review = json.loads(review_raw)
    _need(set(review) == {"profile", "status", "assemblySha256", "inventorySha256",
                         "finalReviewExcludedFromEvidenceZip"},
          "final assembly review field set is open or incomplete")
    _need(review.get("profile") == "weft-local-final-assembly-review/1"
          and review.get("status") == "ACCEPTED"
          and review.get("assemblySha256") == expected_sha
          and review.get("inventorySha256") == inventory_digest
          and review.get("finalReviewExcludedFromEvidenceZip") is True,
          "independent final assembly review is not bound to exact output")
    return {p.name: p for p in files}, review


def _request_json(url: str, method: str, token: str | None, payload: dict[str, Any] | None = None,
                  *, timeout: float = CHILD_SECONDS,
                  deadline: float | None = None) -> tuple[int, dict[str, Any], bytes]:
    _need(0 < timeout <= CHILD_SECONDS, "public request child limit is invalid")
    now = time.monotonic()
    deadline = min(now + timeout, deadline if deadline is not None else now + timeout)
    remaining = deadline - time.monotonic()
    _need(remaining > 0, "public request reached its absolute deadline")
    headers = {"Accept": "application/vnd.github+json", "X-GitHub-Api-Version": "2022-11-28"}
    body = None if payload is None else json.dumps(payload, separators=(",", ":")).encode()
    if body is not None:
        headers["Content-Type"] = "application/json"
    request = urllib.request.Request(url, data=body, headers=headers, method=method)
    if token:
        request.add_unredirected_header("Authorization", f"Bearer {token}")
    try:
        with urllib.request.urlopen(request, timeout=remaining) as response:
            raw = _bounded_response_read(response, MAX_ENTRY, deadline)
            _need(len(raw) <= MAX_ENTRY, "public API response exceeds metadata cap")
            decoded = json.loads(raw) if raw else {}
            _need(isinstance(decoded, dict), "public API response is not an object")
            return response.status, decoded, raw
    except urllib.error.HTTPError as exc:
        try:
            body = _bounded_response_read(exc, MAX_ENTRY, deadline)
            _need(len(body) <= MAX_ENTRY, "public API error response exceeds cap")
            try:
                decoded = json.loads(body) if body else {}
            except json.JSONDecodeError:
                decoded = {"message": "non-JSON error body", "bodySha256": sha(body)}
        finally:
            exc.close()
        return exc.code, decoded, body


def _bounded_response_read(response: Any, cap: int, deadline: float) -> bytes:
    """Read only through a socket-time-bounded single-read response interface."""
    chunks: list[bytes] = []
    total = 0
    # urllib.error.HTTPError is a wrapper around the response object.  Its
    # .fp is the HTTPResponse which owns read1() and the buffered raw socket.
    # Unwrap only this documented stdlib wrapper shape; retain the same single
    # reader, socket timeout, deadline, and byte cap for normal/error bodies.
    bounded_response = response.fp if isinstance(response, urllib.error.HTTPError) else response
    reader = getattr(bounded_response, "read1", None)
    sock = getattr(getattr(getattr(bounded_response, "fp", None), "raw", None), "_sock", None)
    _need(callable(reader) and sock is not None and callable(getattr(sock, "settimeout", None)),
          "HTTP response lacks the bounded read1/socket shape")
    while True:
        remaining = deadline - time.monotonic()
        _need(remaining > 0, "network response exceeded its absolute child deadline")
        sock.settimeout(max(0.001, remaining))
        chunk = reader(min(64 * 1024, cap + 1 - total))
        _need(time.monotonic() <= deadline,
              "network response exceeded its absolute child deadline")
        if not chunk:
            break
        total += len(chunk)
        _need(total <= cap, "network response exceeds its byte cap")
        chunks.append(chunk)
    _need(time.monotonic() <= deadline, "network response exceeded its absolute child deadline")
    return b"".join(chunks)


def _github_asset_bytes(base: str, asset_id: Any, token: str | None,
                        deadline: float) -> tuple[int, str, bytes]:
    _need(type(asset_id) is int and asset_id > 0,
          "GitHub asset identity is not a numeric server identifier")
    request_started = time.monotonic()
    operation_deadline = min(deadline, request_started + CHILD_SECONDS)
    remaining = operation_deadline - time.monotonic()
    _need(remaining > 0, "GitHub asset readback reached terminal reserve")
    url = base + f"/releases/assets/{asset_id}"
    request = urllib.request.Request(url, headers={
        "Accept": "application/octet-stream",
        "X-GitHub-Api-Version": "2022-11-28"})
    if token:
        request.add_unredirected_header("Authorization", f"Bearer {token}")
    with urllib.request.urlopen(request, timeout=remaining) as response:
        data = _bounded_response_read(response, MAX_ASSEMBLY, operation_deadline)
        _need(len(data) <= MAX_ASSEMBLY,
              "GitHub asset byte readback exceeds the 70 MiB family cap")
        # Never retain a redirected signed blob URL or query credential.
        return response.status, url, data


def _verify_metadata_commit(assembly: Path, metadata_commit: str) -> dict[str, Any]:
    _need(COMMIT.fullmatch(metadata_commit) is not None,
          "metadata commit must be a full Git object identity")
    pins = json.loads(_read_bounded(assembly / "release-pins-0.4.56.json",
                                    MAX_ENTRY, "release pins"))
    qualification = json.loads(_read_bounded(assembly / "qualification-0.4.56.json",
                                             MAX_ENTRY, "qualification record"))
    _need(pins.get("schema") == "weft-selected-release-pins/1"
          and pins.get("tag") == "v0.4.56"
          and pins.get("releaseMetadataCommit") == metadata_commit
          and qualification.get("metadata", {}).get("commit") == metadata_commit
          and qualification.get("sourceComparison", {}).get("selectedBuildInputsEqual") is True
          and qualification.get("status") == "VERIFIED_FOR_ASSEMBLY",
          "requested M commit differs from the assembled, independently reviewed metadata source")
    return pins


def _unique_remote_assets(rows: Any, expected_names: set[str]) -> dict[str, dict[str, Any]]:
    _need(isinstance(rows, list), "remote asset inventory is absent")
    result: dict[str, dict[str, Any]] = {}
    ids: set[int] = set()
    for row in rows:
        _need(isinstance(row, dict) and isinstance(row.get("name"), str)
              and type(row.get("id")) is int and row["id"] > 0
              and type(row.get("size")) is int and row["size"] >= 0,
              "remote asset identity is malformed")
        _need(row["name"] not in result and row["id"] not in ids,
              "remote asset names and numeric IDs must be unique")
        result[row["name"]] = row
        ids.add(row["id"])
    _need(set(result) == expected_names, "remote asset roster differs from exact local release")
    return result


def publish_github(assembly: Path, assembly_sha: str, review_path: Path, *,
                   repository: str, metadata_commit: str, expected_review_sha256: str,
                   tag: str = "v0.4.56") -> dict[str, Any]:
    started = time.monotonic()
    deadline = started + AGGREGATE_SECONDS - TERMINAL_RESERVE_SECONDS
    token = os.environ.get("GITHUB_TOKEN")
    _need(bool(token), "GITHUB_TOKEN is required for the explicit GitHub transaction")
    _need(re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository) is not None
          and COMMIT.fullmatch(metadata_commit) and tag == "v0.4.56",
          "repository, metadata commit or tag is outside the selected release")
    _verify_metadata_commit(assembly, metadata_commit)
    assets, review = _assembly_assets(assembly, assembly_sha, review_path,
                                      expected_review_sha256)
    base = f"https://api.github.com/repos/{repository}"
    network_steps: list[dict[str, Any]] = []

    def api(url: str, method: str, payload: dict[str, Any] | None = None) -> tuple[int, dict[str, Any], bytes]:
        remaining = deadline - time.monotonic()
        _need(remaining > 0, "GitHub transaction reached terminal reserve")
        request_started = time.monotonic()
        try:
            result = _request_json(url, method, token, payload,
                                   timeout=min(CHILD_SECONDS, remaining), deadline=deadline)
        except Exception as exc:
            network_steps.append({"method": method, "url": url,
                                  "transportError": type(exc).__name__,
                                  "durationSeconds": time.monotonic() - request_started})
            raise
        code, value, raw = result
        network_steps.append({"method": method, "url": url, "status": code,
                              "responseBytes": len(raw), "responseSha256": sha(raw),
                              "durationSeconds": time.monotonic() - request_started})
        return result

    status, perms, _ = api(base + "/actions/permissions", "GET")
    _need(status == 200 and perms.get("enabled") is False,
          "Actions must be disabled before the no-Actions GitHub transaction")
    tag_status, tag_ref, _ = api(base + f"/git/ref/tags/{tag}", "GET")
    _need(tag_status == 404, "release tag already exists or tag absence is unknown")
    releases_status, releases, _ = api(base + f"/releases/tags/{tag}", "GET")
    _need(releases_status == 404, "release already exists or release absence is unknown")
    # Create the exact metadata tag only after both absence observations.
    try:
        code, created_tag, _ = api(base + "/git/refs", "POST",
                                   {"ref": f"refs/tags/{tag}", "sha": metadata_commit})
    except Exception:
        code, created_tag = 0, {}
    tag_reconciled = False
    if code != 201:
        # A timed-out/ambiguous create is never blindly replayed. Exact ref
        # readback may establish that the requested immutable tag already landed.
        observed_code, observed_ref, _ = api(base + f"/git/ref/tags/{tag}", "GET")
        _need(observed_code == 200
              and observed_ref.get("object", {}).get("type") == "commit"
              and observed_ref.get("object", {}).get("sha") == metadata_commit,
              "tag creation outcome is unknown; exact ref readback did not close")
        created_tag = observed_ref
        tag_reconciled = True
    _need((code == 201 or tag_reconciled)
          and created_tag.get("object", {}).get("type") == "commit"
          and created_tag.get("object", {}).get("sha") == metadata_commit,
          "tag creation outcome is unknown; reconcile by exact API readback")
    draft_reconciled = False
    try:
        code, draft, _ = api(base + "/releases", "POST",
                             {"tag_name": tag, "target_commitish": metadata_commit,
                              "name": tag, "draft": True, "prerelease": False})
    except Exception:
        code, draft = 0, {}
    if code != 201:
        # Reconcile only by tag and metadata commit; never create a second draft.
        observed_code, observed_release, _ = api(base + f"/releases/tags/{tag}", "GET")
        _need(observed_code == 200 and observed_release.get("tag_name") == tag
              and observed_release.get("draft") is True
              and type(observed_release.get("id")) is int,
              "draft creation outcome is unknown; exact release readback did not close")
        draft = observed_release
        draft_reconciled = True
    _need((code == 201 or draft_reconciled) and type(draft.get("id")) is int
          and draft.get("draft") is True
          and draft.get("tag_name") == tag, "numeric GitHub draft creation failed")
    release_id = draft["id"]
    upload_base = draft.get("upload_url", "").split("{", 1)[0]
    _need(upload_base.startswith("https://uploads.github.com/"), "upload endpoint is unexpected")
    upload_facts = []
    for name, path in sorted(assets.items()):
        _need(time.monotonic() < deadline, "GitHub transaction reached terminal reserve")
        data = _read_bounded(path, MAX_ASSEMBLY, "GitHub asset upload")
        req = urllib.request.Request(upload_base + "?name=" + urllib.parse.quote(name), data=data,
            headers={"Content-Type": "application/octet-stream",
                     "Accept": "application/vnd.github+json"}, method="POST")
        req.add_unredirected_header("Authorization", f"Bearer {token}")
        upload_verified = False
        upload_error = None
        try:
            upload_started = time.monotonic()
            upload_deadline = min(deadline, upload_started + CHILD_SECONDS)
            with urllib.request.urlopen(req, timeout=max(0.001, upload_deadline-time.monotonic())) as resp:
                response_raw = _bounded_response_read(resp, MAX_ENTRY, upload_deadline)
                _need(len(response_raw) <= MAX_ENTRY, "GitHub upload response exceeds metadata cap")
                response = json.loads(response_raw)
                _need(time.monotonic() <= upload_deadline,
                      "GitHub upload response exceeded its absolute child deadline")
                network_steps.append({"method": "POST", "url": upload_base + "?name=" + urllib.parse.quote(name),
                                      "status": resp.status, "responseBytes": len(response_raw),
                                      "responseSha256": sha(response_raw)})
                upload_verified = (resp.status == 201 and response.get("name") == name
                                   and response.get("size") == len(data))
                if not upload_verified:
                    upload_error = "upload response did not close"
        except urllib.error.HTTPError as exc:
            try:
                upload_error = f"HTTP {exc.code}"
                network_steps.append({"method": "POST", "url": upload_base + "?name=" + urllib.parse.quote(name),
                                      "status": exc.code, "responseBytes": 0,
                                      "responseSha256": sha(b"")})
            finally:
                exc.close()
        except Exception as exc:
            upload_error = type(exc).__name__
            network_steps.append({"method": "POST", "url": upload_base + "?name=" + urllib.parse.quote(name),
                                  "transportError": upload_error})
        if not upload_verified:
            code, observed_upload, _ = api(base + f"/releases/{release_id}", "GET")
            rows = observed_upload.get("assets", [])
            matches = [row for row in rows if isinstance(row, dict) and row.get("name") == name]
            _need(code == 200 and observed_upload.get("draft") is True and len(matches) == 1
                  and matches[0].get("size") == len(data)
                  and type(matches[0].get("id")) is int,
                  f"GitHub upload outcome is unknown and numeric readback did not close: {upload_error}")
            read_status, read_url, recovered = _github_asset_bytes(
                base, matches[0]["id"], token, deadline)
            _need(read_status == 200 and len(recovered) == len(data) and sha(recovered) == sha(data),
                  f"GitHub upload readback differs after unknown outcome: {name}")
            network_steps.append({"method": "GET-upload-reconciliation", "url": read_url,
                                  "responseBytes": len(recovered), "responseSha256": sha(recovered)})
        upload_facts.append({"name": name, "bytes": len(data), "sha256": sha(data),
                             "reconciled": not upload_verified})
    code, observed, _ = api(base + f"/releases/{release_id}", "GET")
    _need(code == 200 and observed.get("draft") is True, "numeric draft readback differs")
    remote = _unique_remote_assets(observed.get("assets"), set(assets))
    for name, row in remote.items():
        _need(row.get("size") == assets[name].stat().st_size and type(row.get("id")) is int,
              f"draft asset size/id differs: {name}")
        _need(time.monotonic() < deadline, "GitHub transaction reached terminal reserve")
        read_status, read_url, raw = _github_asset_bytes(base, row["id"], token, deadline)
        network_steps.append({"method": "GET-draft-asset-bytes", "url": read_url,
                              "status": read_status, "responseBytes": len(raw),
                              "responseSha256": sha(raw)})
        _need(read_status == 200 and len(raw) <= MAX_ASSEMBLY and sha(raw) == sha(_read_bounded(assets[name], MAX_ASSEMBLY,
                                                                         "GitHub readback source")),
              f"draft served bytes differ: {name}")
    _need(time.monotonic() < deadline, "GitHub transaction reached terminal reserve before publish")
    try:
        code, published, _ = api(base + f"/releases/{release_id}", "PATCH", {"draft": False})
    except Exception:
        code, published = 0, {}
    if code != 200 or published.get("draft") is not False:
        # A lost PATCH response is reconciled through the same numeric release.
        code, published, _ = api(base + f"/releases/{release_id}", "GET")
    _need(code == 200 and published.get("id") == release_id and published.get("draft") is False,
          "draft publication outcome is unknown; reconcile by numeric readback")
    _need(published.get("immutable") is True,
          "GitHub did not report the published release as immutable")
    code, final_readback, _ = api(base + f"/releases/{release_id}", "GET")
    _need(code == 200 and final_readback.get("draft") is False
          and final_readback.get("immutable") is True
          and final_readback.get("id") == release_id,
          "published immutable release did not close by numeric readback")
    final_assets = _unique_remote_assets(final_readback.get("assets"), set(assets))
    for name, row in final_assets.items():
        _need(row.get("size") == assets[name].stat().st_size and type(row.get("id")) is int,
              f"published asset size/id differs: {name}")
        _need(time.monotonic() < deadline, "GitHub transaction reached terminal reserve")
        read_status, read_url, data = _github_asset_bytes(base, row["id"], token, deadline)
        network_steps.append({"method": "GET-published-asset-bytes",
                              "url": read_url, "status": read_status,
                              "responseBytes": len(data), "responseSha256": sha(data)})
        _need(read_status == 200 and len(data) <= MAX_ASSEMBLY
              and sha(data) == sha(_read_bounded(assets[name], MAX_ASSEMBLY,
                                                  "published GitHub readback source")),
              f"published served bytes differ: {name}")
    return {"profile": "weft-local-github-transaction/1", "status": "PUBLISHED_IMMUTABLE",
            "repository": repository, "tag": tag, "metadataCommit": metadata_commit,
            "releaseId": release_id, "assemblySha256": assembly_sha,
            "reviewSha256": sha(_read_bounded(review_path, MAX_EVIDENCE_FILE, "final review")),
            "assets": upload_facts, "networkSteps": network_steps, "immutable": True}


def readback_public(assembly: Path, assembly_sha: str, review_path: Path, *,
                    expected_review_sha256: str, repository: str,
                    metadata_commit: str) -> dict[str, Any]:
    """Unauthenticated readback of actual public tag, release roster and served bytes."""
    started = time.monotonic()
    deadline = started + AGGREGATE_SECONDS - TERMINAL_RESERVE_SECONDS
    _need(re.fullmatch(r"[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+", repository) is not None,
          "repository is outside the selected public readback profile")
    _verify_metadata_commit(assembly, metadata_commit)
    assets, review = _assembly_assets(assembly, assembly_sha, review_path,
                                      expected_review_sha256)
    base = f"https://api.github.com/repos/{repository}"
    status, release, _ = _request_json(
        base + "/releases/tags/v0.4.56", "GET", None,
        timeout=min(CHILD_SECONDS, deadline-time.monotonic()), deadline=deadline)
    _need(status == 200 and release.get("tag_name") == "v0.4.56"
          and release.get("draft") is False and release.get("prerelease") is False
          and release.get("immutable") is True and type(release.get("id")) is int,
          "public release readback is not the exact published immutable release")
    status, ref, _ = _request_json(
        base + "/git/ref/tags/v0.4.56", "GET", None,
        timeout=min(CHILD_SECONDS, deadline-time.monotonic()), deadline=deadline)
    _need(status == 200 and isinstance(ref.get("object"), dict),
          "public tag readback is absent")
    obj = ref["object"]
    if obj.get("type") == "tag":
        status, annotated, _ = _request_json(
            base + "/git/tags/" + str(obj.get("sha")), "GET", None,
            timeout=min(CHILD_SECONDS, deadline-time.monotonic()), deadline=deadline)
        _need(status == 200 and annotated.get("object", {}).get("type") == "commit"
              and annotated.get("object", {}).get("sha") == metadata_commit,
              "public annotated tag does not resolve to verified M")
    else:
        _need(obj.get("type") == "commit" and obj.get("sha") == metadata_commit,
              "public tag does not resolve to verified M")
    remote = _unique_remote_assets(release.get("assets"), set(assets))
    served = []
    for name, row in sorted(remote.items()):
        _need(row.get("size") == assets[name].stat(follow_symlinks=False).st_size
              and time.monotonic() < deadline,
              f"public asset identity differs or readback deadline is exhausted: {name}")
        code, endpoint, body = _github_asset_bytes(base, row["id"], None, deadline)
        expected = _read_bounded(assets[name], MAX_ASSEMBLY, "public readback source")
        _need(code == 200 and len(body) == len(expected) and sha(body) == sha(expected),
              f"public served asset bytes differ: {name}")
        served.append({"name": name, "id": row["id"], "bytes": len(body),
                       "sha256": sha(body), "endpoint": endpoint})
    return {"profile": "weft-local-public-readback/1",
            "status": "PUBLIC_IMMUTABLE_RELEASE_BYTES_VERIFIED",
            "repository": repository, "tag": "v0.4.56", "metadataCommit": metadata_commit,
            "releaseId": release["id"], "assemblySha256": assembly_sha,
            "reviewSha256": sha(_read_bounded(review_path, MAX_EVIDENCE_FILE,
                                               "final assembly review")),
            "assets": served, "authenticationUsed": False,
            "elapsedSeconds": time.monotonic() - started,
            "limits": {"aggregateSeconds": AGGREGATE_SECONDS,
                       "childSeconds": CHILD_SECONDS,
                       "terminalReserveSeconds": TERMINAL_RESERVE_SECONDS}}


def publish_pypi(assembly: Path, assembly_sha: str, review_path: Path, *,
                 tool_manifest: Path, tool_manifest_sha256: str,
                 repository: str, metadata_commit: str, expected_review_sha256: str,
                 capture_dir: Path | None = None) -> dict[str, Any]:
    deadline = time.monotonic() + AGGREGATE_SECONDS - TERMINAL_RESERVE_SECONDS
    username = os.environ.get("TWINE_USERNAME")
    password = os.environ.get("TWINE_PASSWORD")
    _need(username == "__token__" and bool(password),
          "authorized TWINE token credential is required for PyPI upload")
    manifest = _read_bounded(tool_manifest, MAX_EVIDENCE_FILE, "pinned PyPI tool manifest")
    _need(sha(manifest) == tool_manifest_sha256,
          "PyPI uploader tool-closure manifest digest differs")
    tools = json.loads(manifest)
    _need(set(tools) == {"profile", "uploader", "version", "pinnedVersion",
                         "pythonExecutable", "files"}
          and tools.get("profile") == "weft-local-pypi-uploader-tool-closure/1"
          and tools.get("uploader") == "twine" and tools.get("version") == tools.get("pinnedVersion")
          and tools.get("pythonExecutable") == str(Path(sys.executable).resolve())
          and isinstance(tools.get("files"), list) and tools["files"],
          "PyPI uploader closure is not explicitly pinned")
    _need(0 < len(tools["files"]) <= 1000,
          "PyPI uploader closure roster is outside its bound")
    tool_paths: set[str] = set()
    tool_preflight: list[tuple[dict[str, Any], Path, int]] = []
    tool_total = 0
    for row in tools["files"]:
        item = _closed(row, {"path", "sizeBytes", "sha256"}, "PyPI tool closure member")
        _need(item["path"] not in tool_paths, "PyPI tool closure has duplicate paths")
        tool_paths.add(item["path"])
        member_path = _under(tool_manifest.parent, item["path"], "PyPI tool closure member")
        try:
            member_stat = member_path.stat(follow_symlinks=False)
        except OSError as exc:
            raise ProfileError("PyPI tool closure member cannot be statted") from exc
        _need(stat.S_ISREG(member_stat.st_mode) and type(item["sizeBytes"]) is int
              and member_stat.st_size == item["sizeBytes"]
              and 0 <= member_stat.st_size <= MAX_EVIDENCE_FILE
              and isinstance(item["sha256"], str) and SHA256_RE.fullmatch(item["sha256"]),
              "PyPI tool closure member is not bounded regular input")
        tool_total += member_stat.st_size
        tool_preflight.append((item, member_path, member_stat.st_size))
    _need(tool_total <= MAX_TOOL_CLOSURE,
          "PyPI uploader closure exceeds its complete 16 MiB bound")
    for item, member_path, member_size in tool_preflight:
        data = _read_bounded(member_path, MAX_EVIDENCE_FILE, "PyPI tool closure member")
        digest = sha(data)
        _need(digest == item["sha256"] and len(data) == member_size,
              "PyPI uploader tool closure member differs")
    _need(capture_dir is not None and not capture_dir.exists() and not capture_dir.is_symlink(),
          "PyPI phase requires a fresh bounded capture directory")
    capture_dir.mkdir(parents=True)
    _verify_metadata_commit(assembly, metadata_commit)
    assets, review = _assembly_assets(assembly, assembly_sha, review_path,
                                      expected_review_sha256)
    wheels = {name: path for name, path in assets.items() if name.endswith(".whl")}
    _need(len(wheels) == 3 and not any(name.endswith(".tar.gz") for name in assets),
          "PyPI selected release must contain exactly three wheels and no sdist")
    for path in wheels.values():
        data = _read_bounded(path, MAX_WHEEL, "PyPI wheel")
        with zipfile.ZipFile(io.BytesIO(data)) as zf:
            metadata = BytesParser().parsebytes(zf.read("umf_weft-0.4.56.dist-info/METADATA"))
            _need(metadata.get("Name") == "umf-weft" and metadata.get("Version") == "0.4.56",
                  "PyPI wheel metadata differs")
    expected = {name: sha(_read_bounded(path, MAX_WHEEL, "PyPI selected wheel"))
                for name, path in wheels.items()}
    from publish_pypi_release import prepare_release

    base = f"https://api.github.com/repos/{repository}"
    gh_token = os.environ.get("GITHUB_TOKEN")
    code, release, _ = _request_json(
        base + "/releases/tags/v0.4.56", "GET", gh_token,
        timeout=min(CHILD_SECONDS, deadline-time.monotonic()), deadline=deadline)
    _need(code == 200 and release.get("draft") is False and release.get("prerelease") is False,
          "PyPI prepare requires the exact published GitHub release")
    code, tag_ref, _ = _request_json(base + "/git/ref/tags/v0.4.56", "GET", gh_token,
                                    timeout=min(CHILD_SECONDS, deadline-time.monotonic()),
                                    deadline=deadline)
    _need(code == 200 and isinstance(tag_ref.get("object"), dict),
          "PyPI prepare cannot verify the release tag")
    tag_object = None
    if tag_ref["object"].get("type") == "tag":
        code, tag_object, _ = _request_json(base + "/git/tags/" + tag_ref["object"]["sha"],
                                            "GET", gh_token,
                                            timeout=min(CHILD_SECONDS, deadline-time.monotonic()),
                                            deadline=deadline)
        _need(code == 200 and tag_object.get("object", {}).get("type") == "commit"
              and tag_object.get("object", {}).get("sha") == metadata_commit,
              "annotated release tag does not resolve to exact metadata M")
    else:
        _need(tag_ref["object"].get("type") == "commit"
              and tag_ref["object"].get("sha") == metadata_commit,
              "release tag does not resolve to exact metadata M")
    sums_digest = sha(_read_bounded(assembly / "SHA256SUMS", MAX_ENTRY, "assembly SHA256SUMS"))
    with tempfile.TemporaryDirectory(prefix="weft-pypi-prepare-") as temporary:
        prepared = prepare_release(repository=repository, release=release, tag_ref=tag_ref,
            tag_object=tag_object, assets_dir=assembly, output_dir=Path(temporary) / "verified",
            tag="v0.4.56", source_sha=metadata_commit, expected_sums_sha256=sums_digest,
            distribution="umf-weft", wheel_prefix="umf_weft-0.4.56-", wheel_count=3,
            require_sdist=False, producer_run=None)
        _need(set(prepared.get("files", {})) == set(expected),
              "existing selected-release prepare path did not select the exact wheel set")
    state = _pypi_readback(expected, wait_seconds=CHILD_SECONDS, deadline=deadline,
                           wait_for_match=False)
    if state == "matching":
        return {"profile": "weft-local-pypi-transaction/1", "status": "ALREADY_PUBLISHED_EXACT",
                "version": "0.4.56", "assemblySha256": assembly_sha,
                "files": [{"name": n, "sha256": h} for n, h in sorted(expected.items())],
                "credentialValueRecorded": False}
    _need(state == "absent", "PyPI version is present or conflicts; upload is forbidden")
    _need(time.monotonic() < deadline, "PyPI transaction reached terminal reserve")
    version_env = dict(os.environ)
    version_env.pop("TWINE_USERNAME", None)
    version_env.pop("TWINE_PASSWORD", None)
    version_env.pop("GITHUB_TOKEN", None)
    version_capture = _bounded_capture(
        [sys.executable, "-m", "twine", "--version"],
        timeout=min(CHILD_SECONDS, deadline-time.monotonic()), env=version_env,
        capture_dir=capture_dir / "version")
    version_stdout = next((Path(row["path"]) for row in version_capture["captureFiles"]
                          if row.get("stream") == "stdout"), None)
    _need(version_capture["exitCode"] == 0 and version_stdout is not None,
          "pinned Twine version command did not close")
    observed_version = _read_bounded(version_stdout, MAX_RAW_LOGS // 2,
                                     "Twine version output").decode("utf-8", "strict").strip()
    _need(re.fullmatch(r"twine version " + re.escape(tools["version"]) + r"(?:\s+\([^\r\n]*\))?",
                       observed_version) is not None,
          "installed Twine version differs from the pinned uploader closure")
    version_raw_bytes = sum(row["rawBytes"] for row in version_capture["streams"].values())
    remaining_capture_bytes = MAX_RAW_LOGS - version_raw_bytes
    _need(remaining_capture_bytes > 0,
          "Twine version capture exhausted the complete PyPI log allowance")
    env = dict(os.environ)
    env["TWINE_USERNAME"] = username
    env["TWINE_PASSWORD"] = password
    argv = [sys.executable, "-m", "twine", "upload", "--non-interactive",
            "--repository-url", "https://upload.pypi.org/legacy/"] + [str(p) for _, p in sorted(wheels.items())]
    capture = _bounded_capture(argv, timeout=min(CHILD_SECONDS, deadline - time.monotonic()),
                               env=env, capture_dir=capture_dir / "upload",
                               raw_cap=remaining_capture_bytes)
    _need(capture["exitCode"] == 0, "PyPI upload failed; do not retry automatically")
    _need(_pypi_readback(expected, wait_seconds=CHILD_SECONDS, deadline=deadline,
                         wait_for_match=True) == "matching",
          "bounded PyPI readback did not close")
    return {"profile": "weft-local-pypi-transaction/1", "status": "PUBLISHED_EXACT_READBACK",
            "version": "0.4.56", "assemblySha256": assembly_sha,
            "reviewSha256": sha(_read_bounded(review_path, MAX_EVIDENCE_FILE, "final review")),
            "uploaderToolManifestSha256": tool_manifest_sha256,
            "credentialValueRecorded": False, "versionCapture": version_capture,
            "commandCapture": capture}


def _pypi_readback(expected: dict[str, str], *, wait_seconds: int,
                   deadline: float | None = None, wait_for_match: bool = False) -> str:
    _need(0 < wait_seconds <= CHILD_SECONDS,
          "PyPI readback must fit one bounded child interval")
    deadline = min(time.monotonic() + wait_seconds,
                   deadline if deadline is not None else time.monotonic() + wait_seconds)
    while True:
        _need(time.monotonic() < deadline,
              "PyPI readback reached its absolute transaction deadline")
        request = urllib.request.Request("https://pypi.org/pypi/umf-weft/0.4.56/json",
                                         headers={"Accept": "application/json"})
        try:
            request_started = time.monotonic()
            request_deadline = min(deadline, request_started + CHILD_SECONDS)
            with urllib.request.urlopen(request, timeout=max(0.001, request_deadline-time.monotonic())) as response:
                raw = _bounded_response_read(response, MAX_ENTRY, request_deadline)
            _need(time.monotonic() <= request_deadline,
                  "PyPI JSON response exceeded its absolute child deadline")
            _need(len(raw) <= MAX_ENTRY, "PyPI JSON response exceeds metadata cap")
            value = json.loads(raw)
        except urllib.error.HTTPError as exc:
            try:
                if exc.code == 404:
                    if not wait_for_match or time.monotonic() >= deadline:
                        return "absent"
                    time.sleep(min(3, max(0, deadline-time.monotonic())))
                    continue
                raise ProfileError(f"PyPI readback failed with HTTP {exc.code}") from exc
            finally:
                exc.close()
        except (urllib.error.URLError, TimeoutError, json.JSONDecodeError) as exc:
            raise ProfileError("PyPI exact readback failed") from exc
        _need(value.get("info", {}).get("name") == "umf-weft"
              and value.get("info", {}).get("version") == "0.4.56",
              "PyPI readback identifies a different project/version")
        rows = value.get("urls")
        _need(isinstance(rows, list), "PyPI file inventory is absent")
        observed: dict[str, tuple[str, str]] = {}
        for row in rows:
            _need(isinstance(row, dict) and isinstance(row.get("filename"), str)
                  and isinstance(row.get("url"), str)
                  and row["url"].startswith("https://files.pythonhosted.org/"),
                  "PyPI file row is malformed")
            name = row["filename"]
            digest = row.get("digests", {}).get("sha256")
            _need(name not in observed and isinstance(digest, str) and SHA256_RE.fullmatch(digest),
                  "PyPI duplicate or malformed file digest")
            observed[name] = (digest, row["url"])
        _need(set(observed) == set(expected), "PyPI file set conflicts with exact local assets")
        for name, (digest, url) in observed.items():
            _need(digest == expected[name], f"PyPI index digest conflicts: {name}")
            _need(time.monotonic() < deadline,
                  "PyPI wheel readback reached its absolute deadline")
            request_started = time.monotonic()
            request_deadline = min(deadline, request_started + CHILD_SECONDS)
            with urllib.request.urlopen(urllib.request.Request(url),
                                        timeout=max(0.001, request_deadline-time.monotonic())) as response:
                data = _bounded_response_read(response, MAX_WHEEL, request_deadline)
            _need(time.monotonic() <= request_deadline,
                  "PyPI wheel response exceeded its absolute child deadline")
            _need(len(data) <= MAX_WHEEL and sha(data) == digest,
                  f"PyPI served bytes differ: {name}")
        return "matching"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("assemble", "publish-github", "publish-pypi", "readback"))
    parser.add_argument("--packet", type=Path)
    parser.add_argument("--notes", type=Path)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--assembly-sha256")
    parser.add_argument("--review", type=Path)
    parser.add_argument("--review-sha256")
    parser.add_argument("--metadata-review-sha256")
    parser.add_argument("--repository")
    parser.add_argument("--metadata-commit")
    parser.add_argument("--tool-manifest", type=Path)
    parser.add_argument("--tool-manifest-sha256")
    parser.add_argument("--receipt", type=Path)
    args = parser.parse_args()
    if args.phase == "assemble":
        _need(args.packet is not None and args.notes is not None and args.output is not None
              and args.metadata_review_sha256 is not None
              and SHA256_RE.fullmatch(args.metadata_review_sha256),
              "assemble requires packet, release notes, outside-Q M review digest, and new output")
        result = assemble(args.packet, args.notes, args.output,
                          metadata_review_sha256=args.metadata_review_sha256)
        print(json.dumps(result, sort_keys=True))
        return 0
    _need(args.assembly_sha256 is not None and SHA256_RE.fullmatch(args.assembly_sha256),
          "public phases require the exact reviewed assembly digest")
    _need(args.output is not None and args.review is not None,
          "public phases require the exact assembled directory and external final-review receipt")
    _need(args.receipt is not None, "public phases require a separate durable transaction receipt path")
    _need(args.review_sha256 is not None and SHA256_RE.fullmatch(args.review_sha256),
          "public phases require the independent final-review digest outside the assembly ZIP")
    started = time.monotonic()
    _seed_result(args.receipt, args.phase, args.assembly_sha256)
    try:
        if args.phase == "publish-github":
            _need(args.repository is not None and args.metadata_commit is not None,
                  "GitHub phase requires repository and actual metadata M commit")
            result = publish_github(args.output, args.assembly_sha256, args.review,
                                    repository=args.repository, metadata_commit=args.metadata_commit,
                                    expected_review_sha256=args.review_sha256)
        elif args.phase == "publish-pypi":
            _need(args.tool_manifest is not None and args.tool_manifest_sha256 is not None
                  and args.repository is not None and args.metadata_commit is not None,
                  "PyPI phase requires uploader closure and actual GitHub M source")
            result = publish_pypi(args.output, args.assembly_sha256, args.review,
                                  tool_manifest=args.tool_manifest,
                                  tool_manifest_sha256=args.tool_manifest_sha256,
                                  repository=args.repository, metadata_commit=args.metadata_commit,
                                  expected_review_sha256=args.review_sha256,
                                  capture_dir=args.receipt.parent / "publish-pypi-capture")
        else:
            _need(args.repository is not None and args.metadata_commit is not None,
                  "public readback requires repository and verified metadata M commit")
            result = readback_public(args.output, args.assembly_sha256, args.review,
                                     expected_review_sha256=args.review_sha256,
                                     repository=args.repository,
                                     metadata_commit=args.metadata_commit)
    except BaseException as exc:
        secret_values = [os.environ.get("GITHUB_TOKEN", ""), os.environ.get("TWINE_PASSWORD", "")]
        message = str(exc)
        for secret in secret_values:
            if secret:
                message = message.replace(secret, "[REDACTED]")
        failure = {"profile": f"weft-local-{args.phase}-transaction/1",
                   "status": "FAIL_OR_UNKNOWN", "phase": args.phase,
                   "assemblySha256": args.assembly_sha256,
                   "errorCode": type(exc).__name__, "error": message[:2048],
                   "lastCapture": _LAST_CAPTURE or None,
                   "elapsedSeconds": time.monotonic() - started,
                   "limits": {"aggregateSeconds": AGGREGATE_SECONDS,
                              "childSeconds": CHILD_SECONDS,
                              "terminalReserveSeconds": TERMINAL_RESERVE_SECONDS,
                              "terminationGraceSeconds": TERMINATION_GRACE_SECONDS,
                              "killDrainSeconds": KILL_DRAIN_SECONDS,
                              "rawLogBytes": MAX_RAW_LOGS, "metadataBytes": MAX_EVIDENCE_TOTAL}}
        _replace_result(args.receipt, failure)
        raise
    result["elapsedSeconds"] = time.monotonic() - started
    _need(result["elapsedSeconds"] < AGGREGATE_SECONDS,
          "public transaction exceeded its aggregate deadline before durable receipt")
    result["limits"] = {"aggregateSeconds": AGGREGATE_SECONDS,
                        "childSeconds": CHILD_SECONDS,
                        "terminalReserveSeconds": TERMINAL_RESERVE_SECONDS,
                        "terminationGraceSeconds": TERMINATION_GRACE_SECONDS,
                        "killDrainSeconds": KILL_DRAIN_SECONDS,
                        "rawLogBytes": MAX_RAW_LOGS, "metadataBytes": MAX_EVIDENCE_TOTAL}
    _replace_result(args.receipt, result)
    _need(time.monotonic() - started < AGGREGATE_SECONDS,
          "public transaction exceeded its aggregate deadline during receipt finalization")
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (ProfileError, OSError, zipfile.BadZipFile, KeyError, TypeError) as exc:
        print(f"local release held: {exc}", file=sys.stderr)
        raise SystemExit(1) from exc

