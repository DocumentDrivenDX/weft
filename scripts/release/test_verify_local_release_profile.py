"""Synthetic validator/assembler controls; never actual release evidence."""
from __future__ import annotations

import base64
import csv
import hashlib
import io
import json
import struct
import tempfile
import unittest
import zipfile
from pathlib import Path
from unittest.mock import patch

import verify_local_release_profile as validator
import publish_local_release as publisher


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def ref(root: Path, name: str, data: bytes) -> dict[str, object]:
    path = root / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    return {"path": name, "sizeBytes": len(data), "sha256": sha(data)}


def json_ref(root: Path, name: str, value: dict) -> dict[str, object]:
    return ref(root, name, (json.dumps(value, sort_keys=True) + "\n").encode())


def rewrite_json_ref(root: Path, ref_value: dict, value: dict) -> None:
    raw = (json.dumps(value, sort_keys=True) + "\n").encode()
    (root / ref_value["path"]).write_bytes(raw)
    ref_value.update(sizeBytes=len(raw), sha256=sha(raw))


def git_blob_sha(data: bytes) -> str:
    return hashlib.sha1(b"blob " + str(len(data)).encode("ascii") + b"\0" + data).hexdigest()


def padded_json_bytes(value: dict, target: int) -> bytes:
    padding = ""
    for _ in range(32):
        value["_syntheticPadding"] = padding
        raw = (json.dumps(value, sort_keys=True) + "\n").encode()
        delta = target - len(raw)
        if delta == 0:
            return raw
        padding = (padding + "x" * delta) if delta > 0 else padding[:delta]
    raise AssertionError("could not close test-only JSON to source-pinned byte count")


def source_map(root: Path, role: str, commit: str, tree: str, changed: bool = False,
               metadata_contents: dict[str, bytes] | None = None) -> tuple[dict, dict]:
    paths = [f"src/input-{i:04d}" for i in range(1459)]
    required = ["docs/helix/05-deploy/README.md", "docs/helix/05-deploy/releases/v0.4.56.md",
                "docs/helix/05-deploy/local-release-runbook-0.4.56.md",
                "scripts/release/verify_local_release_profile.py",
                "scripts/release/publish_local_release.py",
                "scripts/release/test_verify_local_release_profile.py",
                "scripts/release/test_publish_local_release.py"]
    paths += required
    paths += [f"docs/helix/05-deploy/meta-{i:04d}.md" for i in range(5901 - len(paths))]
    paths.sort()
    stage, tracked, selected = [], [], []
    metadata_contents = metadata_contents or {}
    for path in paths:
        content = metadata_contents.get(path, path.encode())
        if changed and path == "docs/helix/05-deploy/README.md":
            content = b"metadata-only"
        blob = git_blob_sha(content)
        mode = "100644"
        stage.append(f"{mode} {blob} 0\t{path}")
        tracked.append({"path": path, "mode": mode, "gitBlobSha1": blob})
        if path.startswith("src/input-"):
            raw = path.encode()
            selected.append({"path": path, "bytes": len(raw), "sha256": sha(raw)})
    if changed:
        ix = paths.index("docs/helix/05-deploy/README.md")
        old = tracked[ix]["gitBlobSha1"]
        new = sha(b"metadata-only").zfill(40)[:40]
        tracked[ix]["gitBlobSha1"] = new
        stage[ix] = f"100644 {new} 0\t{paths[ix]}"
    selected_digest = sha((json.dumps(selected, sort_keys=True) + "\n").encode())
    path_digest = sha(("\n".join(paths) + "\n").encode())
    stage_digest = sha("\n".join(stage).encode())
    source = {"commit": commit, "tree": tree, "status": "", "trackedCount": len(paths),
              "trackedPathSha256": path_digest, "gitStageRosterSha256": stage_digest,
              "selectedInputCount": len(selected), "selectedInputRosterSha256": selected_digest,
              "version": "0.4.56"}
    value = {"profile": "weft-local-release-source-map/1", "source": source,
             "trackedPaths": paths, "stageRows": stage, "trackedFiles": tracked,
             "selectedBuildInputs": selected}
    return json_ref(root, f"receipts/{role}-source-map.json", value), value


class FullPacket:
    def __init__(self, root: Path):
        self.root = root
        self.patchers = []

    def _historical_files(self, root: Path, name: str) -> list[dict]:
        logs = root / "logs"
        logs.mkdir(parents=True, exist_ok=True)
        path = logs / name
        path.write_bytes(("test-only producer log " + name).encode())
        # Keep the producer-root copy and its receipt-relative counterpart.
        # The validator intentionally resolves files[] under the receipt root.
        receipt_copy = self.root / "receipts" / "logs" / name
        receipt_copy.parent.mkdir(parents=True, exist_ok=True)
        receipt_copy.write_bytes(path.read_bytes())
        return [{"path": "logs/" + name, "bytes": path.stat().st_size,
                 "sha256": sha(path.read_bytes())}]

    def _mac_producer_records(self, roots, commit, tree, filename, wheel_sha,
                              wheel_bytes, report, licenses, source_facts):
        build_root, audit_root = roots["macBuild"], roots["macAudit"]
        build_operator_root = roots["macBuildOperator"]
        audit_operator_root = roots["macAuditOperator"]
        source_root, custody_root = roots["macSource"], roots["macCustody"]
        py_path = roots["macPython"]
        source_identity = {"commit": commit, "tree": tree, "version": "0.4.56", "status": "",
            "trackedCount": source_facts["trackedCount"],
            "gitStageRosterSha256": source_facts["gitStageRosterSha256"],
            "selectedInputCount": source_facts["selectedInputCount"],
            "selectedInputRosterSha256": source_facts["selectedInputRosterSha256"]}
        build_operator_root.mkdir(parents=True, exist_ok=True)
        audit_operator_root.mkdir(parents=True, exist_ok=True)
        runner = build_operator_root / "run_build_static.py"; runner.write_bytes(b"test Mac build runner")
        build_audit = build_operator_root / "static_audit.py"; build_audit.write_bytes(b"test Mac build audit helper")
        audit_runner = audit_operator_root / "run_static_audit.py"; audit_runner.write_bytes(b"test Mac audit runner".ljust(22917, b" "))
        audit_helper = audit_operator_root / "static_audit.py"; audit_helper.write_bytes(b"test Mac audit helper")
        runner_sha, helper_sha = sha(runner.read_bytes()), sha(audit_runner.read_bytes())
        audit_helper_sha = sha(audit_helper.read_bytes())
        build_logs = {}
        for name, data in (("setup.json", b"test setup\n"),
                           ("logs/candidate-build.stdout.log", b""),
                           ("logs/candidate-build.stderr.log", b"build stderr\n"),
                           ("logs/static-audit.stdout.log", b""),
                           ("logs/static-audit.stderr.log", b"audit stderr\n")):
            path = build_root / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(data)
            build_logs[name] = {"path": name, "bytes": len(data), "sha256": sha(data)}
        stages = []
        for label, stdout_name, stderr_name, exit_code in (
                ("candidate-build", "logs/candidate-build.stdout.log", "logs/candidate-build.stderr.log", 0),
                ("static-audit", "logs/static-audit.stdout.log", "logs/static-audit.stderr.log", 1)):
            physical = []
            for name in (stdout_name, stderr_name):
                row = build_logs[name]
                physical.append({"path": str(build_root / name), "bytes": row["bytes"], "sha256": row["sha256"]})
            stages.append({"name": label, "exitCode": exit_code, "physical": physical})
        build = {"profile": validator.MAC_ORIGINAL_BUILD_PROFILE,
            "status": validator.MAC_ORIGINAL_BUILD_STATUS, "executionKind": "local-build-only",
            "hosted": False, "installed": False, "candidate": None,
            "source": source_identity, "postRunSource": source_identity,
            "operator": {"acceptedPlanSha256": "cafc0eb9b3ed1346210e2e1b85e437df8fb85beb7bd70e5e454f992de5b5ac53",
                "measurementRepairPlanSha256": "fba4b31855acb8ab74a864ef4e5b84d080a52987d5e4a95ffe493c1d045b0ff4",
                "runnerPath": str(runner), "runnerBytes": runner.stat().st_size, "runnerSha256": runner_sha,
                "staticAuditPath": str(build_audit), "staticAuditBytes": build_audit.stat().st_size,
                "staticAuditSha256": sha(build_audit.read_bytes())},
            "postRunOperator": {"runnerBytes": runner.stat().st_size, "runnerSha256": runner_sha,
                "staticAuditBytes": build_audit.stat().st_size, "staticAuditSha256": sha(build_audit.read_bytes())},
            "selectedEvidenceFiles": list(build_logs.values()),
            "selectedEvidenceBytes": sum(row["bytes"] for row in build_logs.values()),
            "stages": stages, "cleanupErrors": []}
        # Build receipt's absolute historical result is copied after it is encoded by caller.
        audit_root.mkdir(parents=True, exist_ok=True)
        custody_root.mkdir(parents=True, exist_ok=True)
        custody = {}
        for name, content in (("custody.json", b"test custody"),
                              ("git-stage-roster.txt", b"test stage roster"),
                              ("source-inputs.rows.json", b"test source inputs")):
            path = custody_root / name; path.write_bytes(content)
            custody[name] = {"bytes": len(content), "path": str(path), "sha256": sha(content)}
        inputs = {"count": source_identity["selectedInputCount"],
                  "rosterSha256": source_identity["selectedInputRosterSha256"], "totalBytes": 12345}
        original_source = {"commit": commit, "tree": tree,
            "selectedInputCount": source_identity["selectedInputCount"],
            "selectedInputRosterSha256": source_identity["selectedInputRosterSha256"],
            "sourceInputsBefore": inputs, "sourceInputsAfter": inputs,
            "custodyAssetsBefore": custody, "custodyAssetsAfter": custody}
        audit_logs = {}
        for name, content in (("logs/static-audit.stdout.log", b"static audit stdout\n"),
                              ("logs/static-audit.stderr.log", b"")):
            path = audit_root / name; path.parent.mkdir(parents=True, exist_ok=True); path.write_bytes(content)
            audit_logs[name] = {"path": name, "bytes": len(content), "sha256": sha(content)}
        report_raw = (json.dumps(report, sort_keys=True) + "\n").encode()
        static_path = audit_root / "static-audit.json"; static_path.write_bytes(report_raw)
        static_fact = {"path": "static-audit.json", "bytes": len(report_raw), "sha256": sha(report_raw)}
        selected = [static_fact, *audit_logs.values()]
        capture_physical = [{"path": str(audit_root / row["path"]), "bytes": row["bytes"],
                             "sha256": row["sha256"]} for row in audit_logs.values()]
        wheel_path = build_root / "dist" / filename
        capture = {"argv": [str(py_path), str(audit_helper), str(wheel_path), str(static_path), str(source_root)],
            "cwd": str(source_root), "limitSeconds": 120, "effectiveDeadlineSeconds": 1.0,
            "exitCode": 0, "captureVerified": True, "timedOut": False, "drainExpired": False,
            "captureErrors": [], "paths": [row["path"] for row in capture_physical],
            "rawSha256": [row["sha256"] for row in capture_physical],
            "retainedSha256": [row["sha256"] for row in capture_physical],
            "retainedBytes": [row["bytes"] for row in capture_physical],
            "truncated": [False, False], "physical": capture_physical}
        audit = {"profile": validator.MAC_AUDIT_PROFILE,
            "status": "LOCAL_MACOS_STATIC_AUDIT_ONLY_PASS", "executionKind": "local-static-audit-only",
            "hosted": False, "installed": False, "imported": False, "nativeExecuted": False,
            "browserParity": False, "compilerParity": False, "functionalQualification": "NOT_RUN",
            "failure": None, "cleanupErrors": [], "closureErrors": [],
            "operator": {"runnerPath": str(audit_runner),
                "runnerBefore": [audit_runner.stat().st_size, helper_sha],
                "runnerAfter": [audit_runner.stat().st_size, helper_sha],
                "helper": {"path": str(audit_helper), "bytes": audit_helper.stat().st_size,
                           "sha256": audit_helper_sha},
                "helperAfter": [audit_helper.stat().st_size, audit_helper_sha],
                "pythonPath": str(py_path), "pythonBefore": [py_path.stat().st_size, sha(py_path.read_bytes())],
                "pythonAfter": [py_path.stat().st_size, sha(py_path.read_bytes())]},
            "originalInputs": {"result": {"path": str(build_root / "result.json"),
                "bytes": 0, "sha256": "0"*64, "before": [0, "0"*64], "after": [0, "0"*64]},
                "source": original_source,
                "wheel": {"path": str(wheel_path), "bytes": wheel_bytes, "sha256": wheel_sha}},
            "wheelBefore": [wheel_bytes, wheel_sha], "wheelAfter": [wheel_bytes, wheel_sha],
            "staticReport": {"path": str(static_path), "bytes": len(report_raw), "sha256": sha(report_raw)},
            "capture": capture, "limits": {"aggregateSeconds": 180, "auditSeconds": 120,
                "terminalReserveSeconds": 60, "aggregateLogBytes": validator.MAX_RAW_LOGS,
                "metadataBytes": validator.MAX_EVIDENCE_TOTAL},
            "selectedEvidenceFiles": selected,
            "selectedEvidenceBytes": sum(row["bytes"] for row in selected)}
        return build, audit

    @staticmethod
    def _pad_receipt(root: Path, ref_value: dict, target_bytes: int) -> dict:
        path = root / ref_value["path"]
        value = json.loads(path.read_text())
        raw = padded_json_bytes(value, target_bytes)
        path.write_bytes(raw)
        ref_value.update(sizeBytes=len(raw), sha256=sha(raw))
        return value

    @staticmethod
    def _copy_mac_audit_outputs(roots, receipt_parent: Path, report):
        for name in ("static-audit.json", "logs/static-audit.stdout.log", "logs/static-audit.stderr.log"):
            source = roots["macAudit"] / name
            target = receipt_parent / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(source.read_bytes())

    def build(self) -> tuple[Path, list]:
        p_commit, p_tree, m_commit, m_tree = "a" * 40, "b" * 40, "c" * 40, "d" * 40
        metadata_base = {
            path: (f"test-only metadata bytes for {path}\n".encode())
            for path in validator.METADATA_ONLY_PATHS}
        metadata_m = dict(metadata_base)
        metadata_m["docs/helix/05-deploy/README.md"] = b"metadata-only"
        p_ref, pmap = source_map(self.root, "P", p_commit, p_tree, metadata_contents=metadata_base)
        m_ref, mmap = source_map(self.root, "M", m_commit, m_tree, metadata_contents=metadata_m)
        commands = [
            {"name": "merge-base", "argv": ["git", "merge-base", "--is-ancestor", p_commit, m_commit],
             "exitCode": 0, "durationSeconds": 0.01, "timedOut": False, "outputCapped": False,
             "stdout": "", "stderr": ""},
            {"name": "metadata-head", "argv": ["git", "rev-parse", "HEAD"], "exitCode": 0,
             "durationSeconds": 0.01, "timedOut": False, "outputCapped": False,
             "stdout": m_commit + "\n", "stderr": ""},
            {"name": "metadata-tree", "argv": ["git", "rev-parse", "HEAD^{tree}"], "exitCode": 0,
             "durationSeconds": 0.01, "timedOut": False, "outputCapped": False,
             "stdout": m_tree + "\n", "stderr": ""},
            {"name": "clean-status", "argv": ["git", "status", "--porcelain"], "exitCode": 0,
             "durationSeconds": 0.01, "timedOut": False, "outputCapped": False,
             "stdout": "", "stderr": ""},
        ]
        ancestry = json_ref(self.root, "receipts/ancestry.json", {
            "profile": "weft-local-git-ancestry-observation/1", "sourceCommit": p_commit,
            "metadataCommit": m_commit, "metadataTree": m_tree, "commands": commands})
        x64_wrapper = b"# test-only source correspondence\n".ljust(99, b" ")
        roots = {"x64Build": self.root / "historical/x64-build",
                 "x64Audit": self.root / "historical/x64-audit",
                 "armBuild": self.root / "historical/arm-build",
                 "macBuild": self.root / "historical/mac-build",
                 "macAudit": self.root / "historical/mac-audit",
                 "macBuildOperator": self.root / "historical/mac-build-operator",
                 "macAuditOperator": self.root / "historical/mac-audit-operator",
                 "macSource": self.root / "historical/mac-source",
                 "macCustody": self.root / "historical/mac-custody",
                 "macPython": self.root / "historical/python3.9"}
        for directory in roots.values():
            if directory.suffix != ".9":
                directory.mkdir(parents=True, exist_ok=True)
        roots["macPython"].write_bytes(b"synthetic Python identity")
        gate_profiles = dict(validator.GATE_PROFILES)
        gate_profiles["linux-x86_64-final-native-campaign"] = (
            "umf.test-linux-x86_64-final-native-campaign", "PASS")
        for gate_name, spec in list(gate_profiles.items()):
            if spec is None:
                gate_profiles[gate_name] = ("umf.test-" + gate_name, "PASS_LOCAL_FULL14" if "native-campaign" in gate_name else "PASS")
        gate_refs, gate_receipt_digests, gate_review_digests = {}, {}, {}
        for name, (profile, status) in gate_profiles.items():
            if name == "linux-x86_64-final-installed-admission":
                guest = {"producer": {"commit": validator.UMF_GATE_SOURCE,
                                      "tree": validator.UMF_GATE_TREE, "mount": "read-only"},
                         "cohort": {"selectedLockSha256": "cfe589472a5340446e5b1ddf5e39dbf00e6b6d83370fbe066122f52139fe2274",
                                    "aggregateIndexSha256": "78026c05d3824613f0e2a3bf30cf24bd32af24de1d560f0b7fa555a1e75f8d60",
                                    "custodySha256": "95e9493f06adefb323193f9d8fbe5577ebfa335140a9841f549d2caf098df8ed",
                                    "mirrorManifestSha256": "809a418b9f6f97e403e3fbcc7066d23657347d1323efb6baf2d4d3bb3f17bd36",
                                    "requirementsLockSha256": "4847944f3cafa5fcc9813890f85a4db24a064ed49b479c890e3f4363cb97de51",
                                    "weftSourceCommit": p_commit,
                                    "weftSourceTree": p_tree,
                                    "weftWheelSha256": "0"*64,
                                    "weftWheelSizeBytes": 0},
                         "status": "PASS_INSTALLED_SNAPSHOT_AND_COMPILER_ADMISSION_ONLY",
                         "admissionCases": [{"caseId": case, "status": x, "leaseAttempts": 0,
                                             "programSha256": "c8be43e133c5869b2ac01c520e92177d620b558c326cc8d6aa6b087454ecaf5e" if i == 0 else None}
                                            for i, (case, x) in enumerate(zip([
                                                "baseline-absent-envelope-01", "matching-logical5-envelope-02",
                                                "matching-logical5-legacy-envelope-01", "native-only-set5-envelope-02",
                                                "logical9-native5-mismatch-envelope-02"],
                                                ["ACCEPTED", "ACCEPTED", "REFUSED", "ACCEPTED", "REFUSED"]))]}
                receipt = {"profile": profile, "status": status, "guestResult": guest,
                           "cleanup": {"verified": True}, "hosted": False, "workflowRunId": None,
                           "executionKind":"local_linux_diagnostic", "platform":"linux/amd64",
                           "image":"sha256:7e44ba6b5c483f84c0d630180ba4cd17a82f1bba950577bf1dc9200e7f26e556",
                           "producerVolume":"umf-migration-source-267675510",
                           "limits":{"aggregateSeconds":600,"commandSeconds":180,
                               "evidenceBytes":validator.MAX_EVIDENCE_TOTAL,"logsBytes":validator.MAX_RAW_LOGS,
                               "reserveSeconds":60,"stopGraceSeconds":10},"elapsedSeconds":1.0}
            elif name == "linux-aarch64-build-static":
                receipt = {"profile": profile, "status": status,
                           "source": {"head": p_commit, "tree": p_tree},
                           "candidateWheel": {"memberCount": 9}, "cleanupVerified": True,
                           "hosted": False, "workflowRunId": None}
            elif name == "macos-arm64-build-static":
                receipt = {"profile": profile, "status": status, "candidateInstall": False,
                           "candidateImport": False, "candidateExecution": False,
                           "cleanupVerified": True, "hosted": False, "workflowRunId": None}
            elif name == "linux-source-package":
                log_bytes = b"test-only complete captured source output\n"
                log_path = self.root / "historical/source-run/logs/source-command.stdout.bin"
                physical_log_path = self.root / "receipts/logs/source-command.stdout.bin"
                physical_log_path.parent.mkdir(parents=True, exist_ok=True)
                physical_log_path.write_bytes(log_bytes)
                empty_sha = sha(b"")
                processes = []
                for index in range(40):
                    if index < 20:
                        phase = "source-command"
                    elif index == 20:
                        phase = "release-metadata"
                    elif index == 37:
                        phase = "container-list-before-remove"
                    elif index == 38:
                        phase = "remove-owned-container"
                    elif index == 39:
                        phase = "verify-container-absence"
                    else:
                        phase = "operator-control"
                    stdout_size = len(log_bytes) if index == 0 else 0
                    stdout_sha = sha(log_bytes) if index == 0 else empty_sha
                    processes.append({"phase": phase, "exitCode": 0, "timedOut": False,
                        "drainExpired": False, "rawOverflow": False,
                        "streamCaptureComplete": True, "spawnOrCaptureError": None,
                        "launchError": None, "streamsEOF": {"stdout": True, "stderr": True},
                        "effectiveDeadlineSeconds": 30.0, "durationSeconds": 0.01,
                        "startedMonotonic": float(index),
                        "stdout": {"rawBytes": stdout_size, "sha256": stdout_sha,
                                   "logPath": str(log_path.resolve()) if index == 0 else None},
                        "stderr": {"rawBytes": 0, "sha256": empty_sha, "logPath": None}})
                receipt = {"profile": profile, "status": status, "failure": None,
                    "hosted": False, "workflowRunId": None,
                    "source": {"equalityStatus": "VERIFIED",
                        "identity": {"head": p_commit, "tree": p_tree, "trackedCount": 5901},
                        "producerCheckoutTouched": False, "containerAbsenceVerified": True,
                        "ownedCopyAfter": {"files": []}},
                    "runtime": {"image": "sha256:1f983eafda3fd12f069837f7e972179ec3b8d74587190751db85b555b3430c10",
                        "imageId": "sha256:1f983eafda3fd12f069837f7e972179ec3b8d74587190751db85b555b3430c10",
                        "platform": "linux/amd64",
                        "pythonQualification": "3.11.17 explicit replacement for requested 3.12; narrower evidence",
                        "toolchain": {"PYTHON_VERSION":"Python 3.11.17","BUN_VERSION":"1.4.3",
                            "RUSTC_VERSION":"rustc 1.90.0 (test)","CARGO_VERSION":"cargo 1.90.0 (test)"}},
                    "javascriptDependencies": {"before": {"sourceBunLockSha256":
                        "28b740f61429640ea5c005837d37b5bf7abf67b414c8139659cccc0daee6071a",
                        "sourcePackageJsonSha256":"dd22198e0de80eaab805fae19db057668fcf3c180a84c142d74318c541ad6455",
                        "inventory":{"fileBytes":44271120,"fileCount":1081,
                            "rosterSha256":"b4f2331b8c4d85dadaa2d361d2b1dd89a27a0ebfdee7885f94c94ba6af5fd5f7"}},
                        "after": {"sourceBunLockSha256":
                        "28b740f61429640ea5c005837d37b5bf7abf67b414c8139659cccc0daee6071a",
                        "sourcePackageJsonSha256":"dd22198e0de80eaab805fae19db057668fcf3c180a84c142d74318c541ad6455",
                        "inventory":{"fileBytes":44271120,"fileCount":1081,
                            "rosterSha256":"b4f2331b8c4d85dadaa2d361d2b1dd89a27a0ebfdee7885f94c94ba6af5fd5f7"}}},
                    "releaseMetadata":{"version":"0.4.56","cargoVersion":"0.4.56",
                        "projectName":"umf-weft",
                        "publicationTagRequirement":"v0.4.56 (not checked by source-only guard)"},
                    "processes":processes,
                    "terminalArtifacts":[{"type":"file","path":"logs/source-command.stdout.bin",
                        "bytes":len(log_bytes),"sha256":sha(log_bytes)}],
                    "rawOutputBytes":{"limit":4194304,"total":len(log_bytes),
                        "work":len(log_bytes),"workLimit":3145728},
                    "retainedLogBytes":len(log_bytes),"inclusiveEvidenceBytes":200000,
                    "cleanup":{"errors":[],"volumeAbsenceVerified":{
                        "g03-source-cargo-5cfb93d1-07c":True,
                        "g03-source-target-5cfb93d1-07c":True}}}
                selected_by_path = {row["path"]: row for row in pmap["selectedBuildInputs"]}
                receipt["source"]["ownedCopyAfter"]["files"] = [
                    {"path": path, "bytes": selected_by_path[path]["bytes"]
                        if path in selected_by_path else len(path.encode()),
                     "sha256": selected_by_path[path]["sha256"]
                        if path in selected_by_path else sha(path.encode()),
                     "mode": 0o100644, "linkCount": 1}
                    for path in pmap["trackedPaths"]]
            else:
                receipt = {"profile": profile, "status": status,
                           "commands": [{"exitCode": 0, "timedOut": False, "outputCapped": False}],
                           "cleanupVerified": True, "hosted": False, "workflowRunId": None}
            output_root=self.root/"receipts"
            guest_root=output_root/"guest"/name
            outer_root=output_root/"outer"/name
            guest_root.mkdir(parents=True,exist_ok=True)
            outer_root.mkdir(parents=True,exist_ok=True)
            outer_rows=[]
            output_files=[]
            for stream in ("stdout","stderr"):
                log=outer_root/f"outer.{stream}.log"; log.write_bytes(b"")
                fact={"path":str(log.resolve()),"sizeBytes":0,"sha256":sha(b"")}
                outer_rows.append(fact)
                output_files.append({"path":f"outer/{name}/outer.{stream}.log",
                                     "sizeBytes":0,"sha256":sha(b"")})
            if name == "linux-x86_64-final-installed-admission":
                guest=receipt["guestResult"]
                snap_doc={"synthetic":"test-only canonical snapshot"}
                snap_canonical=json.dumps(snap_doc,sort_keys=True,separators=(",", ":")).encode()
                snap_bytes=snap_canonical+b"\n"
                snap_sha=sha(snap_canonical); snap_file_sha=sha(snap_bytes)
                guest["profile"]="umf.g03-final-installed-prep-guest/1"
                guest["admissionStatus"]="PASS"
                guest["failure"]=None
                guest["admissionSource"]={"path":"/probe/admission_controls.py",
                                           "sha256":"8"*64}
                guest["runtimeVersions"]={"python":"3.11.17","bun":"1.4.3"}
                guest["sourceSeal"]={"manifestSha256":"2"*64,"runnerSha256":"7"*64}
                guest["installedSnapshot"]={"profile":"umf.g03-installed-cohort-snapshot/1",
                    "canonical":True,"runnerSha256":"7"*64,"snapshotSha256":snap_sha,
                    "fileSha256":snap_file_sha,"fileSizeBytes":len(snap_bytes),
                    "snapshotBytes":len(snap_canonical)}
                guest["wheelMemberProof"]={"wheelSha256":"0"*64,
                    "wheelSizeBytes":0,
                    "memberCount":2,"memberRosterExact":True,"members":[
                        {"wheelPath":"weft/__init__.py","sizeBytes":99,"sha256":sha(x64_wrapper)},
                        {"wheelPath":"weft/weft.abi3.so","sizeBytes":0,"sha256":"0"*64}]}
                guest["commandResults"]=[]
                guest_outputs=[]
                for command_name in ["runtime-python","runtime-bun","source-commit","source-tree",
                        "source-clean-status","offline-install-and-pip-check","installed-cohort-snapshot",
                        "fixture-generation","five-compiler-admission-controls"]:
                    streams=[]
                    for stream_name in ("stdout","stderr"):
                        logical=f"logs/{command_name}.{stream_name}.log"
                        physical=guest_root/logical
                        physical.parent.mkdir(parents=True,exist_ok=True); physical.write_bytes(b"")
                        file_ref={"path":f"guest/{name}/{logical}","sizeBytes":0,"sha256":sha(b"")}
                        output_files.append(file_ref); guest_outputs.append({"path":f"{name}/logs/{command_name}.{stream_name}.log",
                            "sizeBytes":0,"sha256":sha(b"")})
                        streams.append({"path":f"{name}/logs/{command_name}.{stream_name}.log",
                            "physicalClosure":True,"physicalSha256":sha(b""),"rawBytes":0,
                            "rawSha256":sha(b""),"retainedBytes":0,"truncated":False})
                    guest["commandResults"].append({"name":command_name,"argv":[command_name],
                        "exitCode":0,"elapsedSeconds":0.01,"timeoutSeconds":180,
                        "timedOut":False,"failure":None,"streams":streams})
                for rel, content in [("installed-cohort-snapshot.json",snap_bytes),
                                     ("g03-defaults-native-fixture.json",b"{}\n"),
                                     ("g03-admission-report.json",b"{}\n")]:
                    physical=guest_root/rel; physical.parent.mkdir(parents=True,exist_ok=True)
                    physical.write_bytes(content)
                    output_files.append({"path":f"guest/{name}/{rel}",
                        "sizeBytes":len(content),"sha256":sha(content)})
                    guest_outputs.append({"path":f"{name}/{rel}","sizeBytes":len(content),"sha256":sha(content)})
                    if rel == "installed-cohort-snapshot.json":
                        snapshot_alias=output_root/"guest"/rel
                        snapshot_alias.parent.mkdir(parents=True,exist_ok=True)
                        snapshot_alias.write_bytes(content)
                        output_files.append({"path":f"guest/{rel}","sizeBytes":len(content),
                            "sha256":sha(content)})
                guest["outputFiles"]=guest_outputs
                receipt["outputBytes"]=sum(row["sizeBytes"] for row in output_files)
                receipt["logBytes"]={"guest":0,"outer":0,"total":0,
                                     "limit":validator.MAX_RAW_LOGS}
            output_total = sum(row["sizeBytes"] for row in output_files)
            log_rows = [row for row in output_files
                        if Path(row["path"]).name.endswith((".log", ".stdout", ".stderr"))]
            guest_log_total = sum(row["sizeBytes"] for row in log_rows
                                  if Path(row["path"]).parts[0] == "guest")
            outer_log_total = sum(row["sizeBytes"] for row in log_rows
                                  if Path(row["path"]).name.startswith("outer."))
            receipt.update(outputFiles=output_files, outputBytes=output_total,
                logBytes={"guest":guest_log_total,"outer":outer_log_total,
                    "total":guest_log_total+outer_log_total,"limit":validator.MAX_RAW_LOGS},
                outerCapturePhysicalClosure={"verified":True,"errors":[],"files":outer_rows})
            rr = json_ref(self.root, f"receipts/{name}.json", receipt)
            if name == "linux-x86_64-final-native-campaign":
                native_reanalysis = json_ref(self.root, "receipts/native-reanalysis.json", {
                    "profile": "umf.test-native-reanalysis", "status": "PASS"})
            else:
                native_reanalysis = None
            if name == "linux-source-package":
                review_raw = ("# Test-only source guard acceptance\n\n"
                    "ACCEPT these actual bounded Linux source/package guard results and physical custody.\n"
                    f"Receipt SHA-256: {rr['sha256']}\n").encode()
                rv = ref(self.root, f"reviews/{name}.md", review_raw)
            else:
                rv = json_ref(self.root, f"reviews/{name}.json", {
                    "profile":"weft-local-gate-review/1", "status":"ACCEPTED",
                    "gate":name, "receiptSha256":rr["sha256"]})
            gate_refs[name] = {"receipt": rr, "review": rv}
            if native_reanalysis is not None:
                gate_refs[name]["reanalysis"] = native_reanalysis
            gate_receipt_digests[name] = rr["sha256"]
            gate_review_digests[name] = rv["sha256"]
        wheel_rows, static_refs, chain_refs, artifact_refs = [], [], [], []
        wheel_members_by_role = {}
        fake_constants = {"LICENSE.md": b"synthetic license\n", "LICENSE-MIT": b"mit\n",
                          "LICENSE-APACHE": b"apache\n", "THIRD-PARTY-LICENSES.txt": b"third party\n"}
        tags = {"linux-x86_64": "manylinux_2_17_x86_64.manylinux2014_x86_64",
                "linux-aarch64": "manylinux_2_17_aarch64.manylinux2014_aarch64",
                "macos-arm64": "macosx_11_0_arm64"}
        wheel_digests = {}
        for role, platform_tag in tags.items():
            filename = f"umf_weft-0.4.56-cp39-abi3-{platform_tag}.whl"
            wrapper = b"# test-only source correspondence\n".ljust(99, b" ")
            if role == "macos-arm64":
                mac_native = bytearray(400000)
                mac_native[:4] = b"\xcf\xfa\xed\xfe"
                struct.pack_into("<I", mac_native, 4, 16777228)
                struct.pack_into("<I", mac_native, 12, 6)
                native = bytes(mac_native)
            else:
                machine = 62 if role == "linux-x86_64" else 183
                native = bytearray(64); native[:6] = b"\x7fELF\x02\x01"; struct.pack_into("<H", native, 18, machine); native = bytes(native)
            metadata = ("Metadata-Version: 2.1\nName: umf-weft\nVersion: 0.4.56\n"
                        "Requires-Python: >=3.9\nLicense-Expression: MIT OR Apache-2.0\n" +
                        "".join(f"License-File: {n}\n" for n in fake_constants) + "\n").encode()
            wheel_text = ("Wheel-Version: 1.0\nGenerator: test-only\nRoot-Is-Purelib: false\n"
                          f"Tag: cp39-abi3-{platform_tag}\n").encode()
            members = {"umf_weft-0.4.56.dist-info/METADATA": metadata,
                       "umf_weft-0.4.56.dist-info/WHEEL": wheel_text,
                       "weft/__init__.py": wrapper, "weft/weft.abi3.so": native}
            for n, content in fake_constants.items():
                members[f"umf_weft-0.4.56.dist-info/licenses/{n}"] = content
            record = io.StringIO(newline="")
            writer = csv.writer(record, lineterminator="\n")
            for n, content in members.items():
                encoded = base64.urlsafe_b64encode(hashlib.sha256(content).digest()).rstrip(b"=").decode()
                writer.writerow([n, "sha256=" + encoded, str(len(content))])
            writer.writerow(["umf_weft-0.4.56.dist-info/RECORD", "", ""])
            members["umf_weft-0.4.56.dist-info/RECORD"] = record.getvalue().encode()
            wheel_members_by_role[role] = members
            wheel_path = self.root / "wheels" / filename
            wheel_path.parent.mkdir(parents=True, exist_ok=True)
            with zipfile.ZipFile(wheel_path, "w", compression=zipfile.ZIP_DEFLATED) as zf:
                for n, content in members.items(): zf.writestr(n, content)
            wheel = wheel_path.read_bytes(); wheel_sha = sha(wheel); wheel_digests[role] = wheel_sha
            if role == "macos-arm64":
                mac_wheel_path = roots["macBuild"] / "dist" / filename
                mac_wheel_path.parent.mkdir(parents=True, exist_ok=True)
                mac_wheel_path.write_bytes(wheel)
            member_rows = {n: {"bytes": len(b), "sha256": sha(b)} for n, b in members.items()}
            expected_tag = f"cp39-abi3-{platform_tag}"
            report = {"status": "STATIC_WHEEL_PASS" if role == "macos-arm64" else "STATIC_BASELINE_WHEEL_PASS",
                      "wheel": {"filename": filename, "bytes": len(wheel), "sha256": wheel_sha},
                      "members": member_rows, "nativeMember": "weft/weft.abi3.so",
                      "tags": [expected_tag], "recordExact": True, "version": "0.4.56",
                      "licenseExpression": "MIT OR Apache-2.0", "noCandidateImportOrInstallation": True,
                      "expandedBytes": sum(map(len, members.values()))}
            if role == "linux-x86_64":
                report.update(architecture="amd64", nativeMachine="Advanced Micro Devices X86-64",
                              glibcVersions=["2.2.5"], neededLibraries=["libc.so.6"])
            elif role == "linux-aarch64":
                report.update(architecture="arm64", nativeMachine="AArch64",
                              glibcVersions=["2.2.5"], neededLibraries=["libc.so.6"])
            else:
                report["wheel"]["path"] = str(roots["macBuild"] / "dist" / filename)
                report.update(profile=validator.MAC_AUDIT_PROFILE,
                              scope="static archive/Mach-O data inspection only; not installed, imported, or executed",
                              macho={"cpuType": 16777228, "fileType": 6, "minimumOS": [11, 0, 0],
                                     "sdk": [27, 0, 0],
                                     "dylibIds": [{"command": 13, "name": "@rpath/weft.abi3.so"}],
                                     "loadDependencies": [
                                         {"command": 12, "name": "/usr/lib/libiconv.2.dylib"},
                                         {"command": 12, "name": "/usr/lib/libSystem.B.dylib"}],
                                     "rpaths": [], "exports": {"names": ["_PyInit_weft"],
                                         "initializerAddress": 388144},
                                     "symbolTable": {"initializer": {"external": True, "section": 1}},
                                     "sections": [{"ordinal": 1, "segment": "__TEXT", "section": "__text",
                                         "initProtection": 5, "fileOffset": 380000, "size": 10000}]},
                              wrapper={"member": "weft/__init__.py", "sha256": sha(wrapper),
                                       "acceptedLinuxSameSourceSha256": sha(wrapper)},
                              producerLicenseSources={n: {"sha256": sha(v)} for n,v in fake_constants.items()})
                report_raw = padded_json_bytes(report, 9604)
                (roots["macAudit"] / "static-audit.json").parent.mkdir(parents=True, exist_ok=True)
                (roots["macAudit"] / "static-audit.json").write_bytes(report_raw)
            sr = json_ref(self.root, f"reports/static-{role}.json", report)
            static_refs.append(sr)
            if role == "linux-x86_64":
                build = {"profile":"umf.g03-final-local-baseline-build-static/1","status":"FAIL_OR_UNKNOWN",
                         "failure":"RuntimeError: static-audit failed or did not close","source":{"head":p_commit,"tree":p_tree},
                         "commands":[{"exitCode":0},{"exitCode":0},{"exitCode":1}],"cleanup":{"errors":[]}}
                build["files"] = self._historical_files(roots["x64Build"], "x64-build.log")
                audit = {"profile":"umf.g03-local-baseline-static-audit-only/1","status":"LOCAL_BASELINE_AUDIT_ONLY_PASS",
                         "source":{"head":p_commit,"tree":p_tree},"cleanup":{"errors":[]},
                         "files":self._historical_files(roots["x64Audit"], "x64-audit.log"),
                         "originalInputs":{"buildResultSha256":"0"*64,"buildCommandExitCode":0,
                                           "wheelSha256":wheel_sha,"producerCommit":p_commit}}
            elif role == "linux-aarch64":
                build = {"profile":"umf.g03-final-local-baseline-build-static/1","status":"LOCAL_FINAL_BASELINE_BUILD_STATIC_PASS",
                         "source":{"head":p_commit,"tree":p_tree},"cleanup":{"errors":[]},
                         "candidateWheel":{"sha256":wheel_sha,"memberCount":9,"expandedBytes":sum(map(len,members.values())),
                                           "inspectionPath":str(roots["armBuild"] / "wheel-inspection.json"),
                                           "inspectionSha256":sha(b"arm inspection")}}
                (roots["armBuild"] / "wheel-inspection.json").write_bytes(b"arm inspection")
                build["files"] = self._historical_files(roots["armBuild"], "arm-build.log")
                audit = report
            else:
                build, audit = self._mac_producer_records(
                    roots, p_commit, p_tree, filename, wheel_sha, len(wheel), report,
                    fake_constants, pmap["source"])
            br = json_ref(self.root, f"receipts/build-{role}.json", build)
            if role == "macos-arm64":
                build = self._pad_receipt(self.root, br, 376450)
                mac_original_result = roots["macBuild"] / "result.json"
                mac_original_result.write_bytes((self.root / br["path"]).read_bytes())
                audit["originalInputs"]["result"] = {
                    "path": str(mac_original_result), "bytes": 376450,
                    "sha256": br["sha256"], "before": [376450, br["sha256"]],
                    "after": [376450, br["sha256"]]}
            if role == "linux-x86_64":
                ar = json_ref(self.root, f"receipts/audit-{role}.json", audit)
            elif role == "macos-arm64":
                ar = json_ref(self.root, f"receipts/audit-{role}.json", audit)
                audit = self._pad_receipt(self.root, ar, 8051)
                data_review = {
                    "disposition": "ACCEPT_LOCAL_MACOS_BUILD002_PLUS_SEPARATE_STATIC_AUDIT004_ONLY",
                    "result": {"bytes": 8051, "sha256": ar["sha256"]},
                    "report": {"bytes": 9604, "path": str(roots["macAudit"] / "static-audit.json"),
                               "sha256": sha(report_raw)},
                    "wheel": {"bytes": len(wheel), "filename": filename,
                              "path": str(roots["macBuild"] / "dist" / filename), "sha256": wheel_sha},
                    "nativeSha256": sha(native), "sourceCommit": p_commit,
                    "sourceTree": p_tree, "originalBuildResultSha256": br["sha256"],
                    "originalBuildStatus": validator.MAC_ORIGINAL_BUILD_STATUS,
                    "originalAuditExitCode": 1, "runtimeQualification": "NOT_RUN", "hosted": False}
                data_ref = json_ref(self.root, "reviews/data-macos-arm64.json", data_review)
                gate_refs["macos-arm64-build-static"]["dataReview"] = data_ref
                self.mac_data_review_sha = data_ref["sha256"]
                self._copy_mac_audit_outputs(roots, (self.root / ar["path"]).parent, report)
                for source_name, target_name in (
                        ("static-audit.json", "static-audit.json"),
                        ("logs/static-audit.stdout.log", "logs/mac-static-audit.stdout.log"),
                        ("logs/static-audit.stderr.log", "logs/mac-static-audit.stderr.log")):
                    source_path = roots["macAudit"] / source_name
                    target_path = self.root / target_name
                    target_path.parent.mkdir(parents=True, exist_ok=True)
                    target_path.write_bytes(source_path.read_bytes())
                gate_audit = json.loads((self.root / ar["path"]).read_text())
                gate_audit["selectedEvidenceFiles"] = [
                    {"path": "static-audit.json", "bytes": 9604,
                     "sha256": sha((self.root / "static-audit.json").read_bytes())},
                    {"path": "logs/mac-static-audit.stdout.log",
                     "bytes": (self.root / "logs/mac-static-audit.stdout.log").stat().st_size,
                     "sha256": sha((self.root / "logs/mac-static-audit.stdout.log").read_bytes())},
                    {"path": "logs/mac-static-audit.stderr.log",
                     "bytes": (self.root / "logs/mac-static-audit.stderr.log").stat().st_size,
                     "sha256": sha((self.root / "logs/mac-static-audit.stderr.log").read_bytes())}]
                gate_audit["selectedEvidenceBytes"] = sum(x["bytes"] for x in gate_audit["selectedEvidenceFiles"])
                gate_refs["macos-arm64-build-static"]["receipt"] = json_ref(
                    self.root, "mac-gate.json", gate_audit)
                gate_receipt_digests["macos-arm64-build-static"] = gate_refs[
                    "macos-arm64-build-static"]["receipt"]["sha256"]
            else:
                ar = sr
            if role == "linux-x86_64":
                review_body = ("ACCEPTED ACTUAL ARTIFACT\n" + wheel_sha + "\n"
                               + br["sha256"] + "\n" + ar["sha256"] + "\n").encode()
            elif role == "linux-aarch64":
                command_rows = []
                for index in range(3):
                    log = roots["armBuild"] / "logs" / f"arm-command-{index}.log"
                    log.parent.mkdir(parents=True, exist_ok=True)
                    log.write_bytes(f"test ARM capture {index}\n".encode())
                    command_rows.append({"physicalClosure": {"verified": True, "errors": [],
                        "files": [{"path": str(log), "bytes": log.stat().st_size,
                                   "sha256": sha(log.read_bytes())}]}})
                    root_copy = self.root / "logs" / log.name
                    root_copy.parent.mkdir(parents=True, exist_ok=True)
                    root_copy.write_bytes(log.read_bytes())
                inspection_source = roots["armBuild"] / "wheel-inspection.json"
                inspection_copy = self.root / "wheel-inspection.json"
                inspection_copy.write_bytes(inspection_source.read_bytes())
                arm_gate = {"profile": "umf.g03-final-baseline-build-static/1",
                    "status": "LOCAL_FINAL_BASELINE_BUILD_STATIC_PASS",
                    "source": {"head": p_commit, "tree": p_tree},
                    "candidateWheel": {"memberCount": 9,
                        "inspectionPath": str(inspection_source),
                        "inspectionSha256": sha(inspection_source.read_bytes())},
                    "commands": command_rows, "cleanup": {"errors": []}}
                gate_refs["linux-aarch64-build-static"]["receipt"] = json_ref(
                    self.root, "arm-gate.json", arm_gate)
                gate_receipt_digests["linux-aarch64-build-static"] = gate_refs[
                    "linux-aarch64-build-static"]["receipt"]["sha256"]
                review_body = ("ACCEPTED ACTUAL BUILD/STATIC ARTIFACT ONLY\n"
                               + br["sha256"] + "\n" + wheel_sha + "\n").encode()
                gate_refs["linux-aarch64-build-static"]["review"] = ref(
                    self.root, "reviews/linux-aarch64-build-static.md", review_body)
            else:
                # Keep the separate gate wrapper with packet-relative selected
                # outputs; the wheel continues to reference the producer audit
                # receipt independently through its auditReceipt field.
                gate_receipt_digests["macos-arm64-build-static"] = gate_refs[
                    "macos-arm64-build-static"]["receipt"]["sha256"]
                review_body = (
                    "Disposition: ACCEPT the exact local macOS ARM64 build artifact plus its separate static-audit continuation.\n"
                    + br["sha256"] + "\n" + ar["sha256"] + "\n" + wheel_sha + "\n"
                    + sr["sha256"] + "\n").encode()
                gate_refs["macos-arm64-build-static"]["review"] = ref(
                    self.root, "reviews/macos-arm64-build-static.md", review_body)
            xr = ref(self.root, f"reviews/artifact-{role}.md", review_body)
            chain_refs += [br, ar]; artifact_refs.append(xr)
            wheel_rows.append({"role":role,"path":f"wheels/{filename}","filename":filename,
                               "sizeBytes":len(wheel),"sha256":wheel_sha,"staticReview":sr,
                               "buildReceipt":br,"auditReceipt":ar,"artifactReview":xr,
                               "producerEvidence":[]})
        prep_name="linux-x86_64-final-installed-admission"
        prep_ref=gate_refs[prep_name]["receipt"]
        prep_path=self.root/prep_ref["path"]
        prep_receipt=json.loads(prep_path.read_text())
        x64_members=wheel_members_by_role["linux-x86_64"]
        x64_native=x64_members["weft/weft.abi3.so"]
        x64_wheel_path=self.root/wheel_rows[0]["path"]
        prep_receipt["guestResult"]["wheelMemberProof"]={
            "wheelSha256":wheel_digests["linux-x86_64"],
            "wheelSizeBytes":x64_wheel_path.stat().st_size,"memberCount":2,
            "memberRosterExact":True,"members":[
                {"wheelPath":"weft/__init__.py","sizeBytes":99,"sha256":sha(wrapper)},
                {"wheelPath":"weft/weft.abi3.so","sizeBytes":len(x64_native),"sha256":sha(x64_native)}]}
        prep_receipt["guestResult"]["cohort"].update(
            weftWheelSha256=wheel_digests["linux-x86_64"],
            weftWheelSizeBytes=x64_wheel_path.stat().st_size)
        prep_raw=(json.dumps(prep_receipt,sort_keys=True)+"\n").encode()
        ref(self.root,prep_ref["path"],prep_raw)
        prep_ref.update(sizeBytes=len(prep_raw),sha256=sha(prep_raw))
        gate_receipt_digests[prep_name]=prep_ref["sha256"]
        review_ref=gate_refs[prep_name]["review"]
        review_path=self.root/review_ref["path"]
        review=json.loads(review_path.read_text()); review["receiptSha256"]=prep_ref["sha256"]
        review_raw=(json.dumps(review,sort_keys=True)+"\n").encode()
        ref(self.root,review_ref["path"],review_raw)
        review_ref.update(sizeBytes=len(review_raw),sha256=sha(review_raw))
        gate_receipt_digests[prep_name]=prep_ref["sha256"]
        gate_review_digests[prep_name]=review_ref["sha256"]
        # The actual values are pinned by the fixture test's temporary override only.
        sources = {"producer": p_ref, "metadata": m_ref, "ancestry": ancestry,
                   "gates":gate_refs, "reviews":[]}
        self.fixture_sources = (sources, pmap, mmap)
        local_source_root = self.root / "local-source"
        metadata_refs = []
        for index, source_path in enumerate(sorted(validator.METADATA_ONLY_PATHS)):
            content = metadata_m[source_path]
            local_path = local_source_root / source_path
            local_path.parent.mkdir(parents=True, exist_ok=True)
            local_path.write_bytes(content)
            data_ref = ref(self.root, f"metadata-files/{index}.bin", content)
            metadata_refs.append({"sourcePath": source_path, **data_ref})
        metadata_review = ref(self.root, "reviews/preassembly-producer-metadata-source.md",
            ("# Test-only source correspondence\n\n"
             "ACCEPTED ACTUAL P/M SOURCE CORRESPONDENCE\n"
             f"M map {m_ref['sha256']} ancestry {ancestry['sha256']}\n").encode())
        packet = {"profile":validator.PROFILE,"status":"HELD",
                  "release":{"distribution":"umf-weft","version":"0.4.56","tag":"v0.4.56","wheelCount":3,"sdist":False},
                  "producer":{"sourceMap":p_ref},"metadata":{"sourceMap":m_ref,"ancestry":ancestry,
                      "files":metadata_refs},
                  "wheels":wheel_rows,"gates":gate_refs,"preAssemblyReviews":[],
                  "hosted":{"workflowRunId":None,"artifactId":None,"workflowPath":None}}
        # Bind the synthetic x64 failure to the exact audit reference digest.
        audit_x64_path = self.root / "receipts/audit-linux-x86_64.json"
        audit_x64 = json.loads(audit_x64_path.read_text())
        audit_x64["originalInputs"]["buildResultSha256"] = sha((self.root/"receipts/build-linux-x86_64.json").read_bytes())
        ref(self.root,"receipts/audit-linux-x86_64.json",(json.dumps(audit_x64,sort_keys=True)+"\n").encode())
        for row in packet["wheels"]:
            if row["role"] == "linux-x86_64":
                row["auditReceipt"] = {"path":"receipts/audit-linux-x86_64.json",
                    "sizeBytes":(self.root/"receipts/audit-linux-x86_64.json").stat().st_size,
                    "sha256":sha((self.root/"receipts/audit-linux-x86_64.json").read_bytes())}
                review_body = ("ACCEPTED ACTUAL ARTIFACT\n"
                    + row["buildReceipt"]["sha256"] + "\n"
                    + row["auditReceipt"]["sha256"] + "\n"
                    + row["sha256"] + "\n").encode()
                review_path = self.root / row["artifactReview"]["path"]
                review_path.write_bytes(review_body)
                row["artifactReview"]["sizeBytes"] = len(review_body)
                row["artifactReview"]["sha256"] = sha(review_body)
        review_refs=[]
        review_pins={}
        source_review={"scope":"producer-metadata-source", **metadata_review}
        review_refs.append(source_review); review_pins["producer-metadata-source"]=source_review["sha256"]
        for row in packet["wheels"]:
            scope=row["artifactReviewScope"] if "artifactReviewScope" in row else {
                "linux-x86_64":"linux-x86_64-artifact",
                "linux-aarch64":"linux-aarch64-static-artifact",
                "macos-arm64":"macos-arm64-static-artifact"}[row["role"]]
            refrow=row["artifactReview"]
            review_refs.append({"scope": scope, **refrow})
            review_pins[scope]=refrow["sha256"]
        packet["preAssemblyReviews"]=review_refs
        for gate_name, refs in gate_refs.items():
            review_path = self.root / refs["review"]["path"]
            gate_review_digests[gate_name] = sha(review_path.read_bytes())
        gate_human = {name: None for name in gate_profiles}
        gate_human["linux-aarch64-build-static"] = gate_refs["linux-aarch64-build-static"]["review"]["sha256"]
        gate_human["macos-arm64-build-static"] = gate_refs["macos-arm64-build-static"]["review"]["sha256"]
        path=self.root/"packet.json"; path.write_text(json.dumps(packet,sort_keys=True)+"\n")
        patches={"PRODUCER":p_commit,"PRODUCER_TREE":p_tree,
                 "SOURCE_IDENTITY":{"trackedCount":5901,"fileRosterSha256":pmap["source"]["trackedPathSha256"],
                    "gitStageRosterSha256":pmap["source"]["gitStageRosterSha256"],"selectedInputCount":1459,
                    "selectedInputRosterSha256":pmap["source"]["selectedInputRosterSha256"]},
                 "GATE_PROFILES":gate_profiles,"GATE_RECEIPT_SHA256":gate_receipt_digests,"GATE_REVIEW_SHA256":gate_review_digests,
                 "GATE_HUMAN_REVIEW_SHA256":gate_human,
                 "SOURCE_GUARD_RECEIPT_SHA256":gate_receipt_digests.get("linux-source-package"),
                 "SOURCE_GUARD_REVIEW_SHA256":gate_refs.get("linux-source-package",{}).get("review",{}).get("sha256"),
                 "SOURCE_GUARD_RAW_BYTES":len(b"test-only complete captured source output\n"),
                 "SOURCE_GUARD_EVIDENCE_BYTES":200000,
                 "SOURCE_GUARD_ORIGINAL_LOG_ROOT":(self.root / "historical/source-run/logs").resolve(),
                 "X64_BUILD_RESULT_SHA256":packet["wheels"][0]["buildReceipt"]["sha256"],
                 "X64_AUDIT_RESULT_SHA256":packet["wheels"][0]["auditReceipt"]["sha256"],
                 "X64_ARTIFACT_REVIEW_SHA256":packet["wheels"][0]["artifactReview"]["sha256"],
                 "X64_STATIC_REPORT_SHA256":packet["wheels"][0]["staticReview"]["sha256"],
                 "X64_WHEEL_SHA256":wheel_digests["linux-x86_64"],
                 "ARM_BUILD_RESULT_SHA256":packet["wheels"][1]["buildReceipt"]["sha256"],
                 "ARM_STATIC_REPORT_SHA256":packet["wheels"][1]["auditReceipt"]["sha256"],
                 "ARM_ARTIFACT_REVIEW_SHA256":packet["wheels"][1]["artifactReview"]["sha256"],
                 "ARM_WHEEL_SHA256":wheel_digests["linux-aarch64"],
                 "MAC_STATIC_REPORT_SHA256":packet["wheels"][2]["staticReview"]["sha256"],
                 "MAC_BUILD_RESULT_SHA256":packet["wheels"][2]["buildReceipt"]["sha256"],
                 "MAC_WHEEL_SHA256":wheel_digests["macos-arm64"],
                 "MAC_ARTIFACT_REVIEW_SHA256":packet["wheels"][2]["artifactReview"]["sha256"],
                 "MAC_AUDIT_RESULT_SHA256":packet["wheels"][2]["auditReceipt"]["sha256"],
                 "MAC_DATA_REVIEW_SHA256":self.mac_data_review_sha,
                 "MAC_WHEEL_BYTES":packet["wheels"][2]["sizeBytes"],
                 "MAC_NATIVE_SHA256":sha(wheel_members_by_role["macos-arm64"]["weft/weft.abi3.so"]),
                 "MAC_BUILD_RUNNER_SHA256":sha((roots["macBuildOperator"] / "run_build_static.py").read_bytes()),
                 "MAC_BUILD_AUDIT_TOOL_SHA256":sha((roots["macBuildOperator"] / "static_audit.py").read_bytes()),
                 "MAC_AUDIT_RUNNER_SHA256":sha((roots["macAuditOperator"] / "run_static_audit.py").read_bytes()),
                 "MAC_AUDIT_HELPER_SHA256":sha((roots["macAuditOperator"] / "static_audit.py").read_bytes()),
                 "MAC_PYTHON_PATH":str(roots["macPython"]),
                 "MAC_PYTHON_BYTES":(roots["macPython"]).stat().st_size,
                 "MAC_PYTHON_SHA256":sha((roots["macPython"]).read_bytes()),
                 "MAC_BUILD_ORIGINAL_ROOT":roots["macBuild"],
                 "MAC_AUDIT_ORIGINAL_ROOT":roots["macAudit"],
                 "MAC_BUILD_OPERATOR_ROOT":roots["macBuildOperator"],
                 "MAC_AUDIT_OPERATOR_ROOT":roots["macAuditOperator"],
                 "MAC_CUSTODY_ROOT":roots["macCustody"],
                 "MAC_PRODUCER_SOURCE_ROOT":roots["macSource"],
                 "X64_BUILD_ORIGINAL_ROOT":roots["x64Build"],
                 "X64_AUDIT_ORIGINAL_ROOT":roots["x64Audit"],
                 "ARM_BUILD_ORIGINAL_ROOT":roots["armBuild"],
                 "PRE_ASSEMBLY_REVIEW_SHA256":review_pins,
                 "PRE_ASSEMBLY_HUMAN_REVIEW_SHA256":{
                     key:value for key,value in review_pins.items()
                     if key in {"linux-x86_64-artifact","linux-aarch64-static-artifact","macos-arm64-static-artifact"}},
                 "METADATA_SOURCE_ROOT":local_source_root,
                 "FINAL_PREP_SOURCE_SEAL":"2"*64,"FINAL_PREP_RUNNER":"7"*64,
                 "FINAL_ADMISSION_SOURCE":"8"*64,
                 "FINAL_INSTALLED_SNAPSHOT":json.loads((self.root/"receipts"/"linux-x86_64-final-installed-admission.json").read_text())["guestResult"]["installedSnapshot"]["snapshotSha256"],
                 "FINAL_WEFT_WHEEL":wheel_digests["linux-x86_64"],
                 "FINAL_WEFT_WHEEL_BYTES":wheel_rows[0]["sizeBytes"],
                 "FINAL_WEFT_NATIVE":sha(wheel_members_by_role["linux-x86_64"]["weft/weft.abi3.so"]),
                 "FINAL_WEFT_NATIVE_BYTES":len(wheel_members_by_role["linux-x86_64"]["weft/weft.abi3.so"]),
                 "WRAPPER_SHA256":sha(wrapper),"LICENSE_SHAS":{n:sha(v) for n,v in fake_constants.items()}}
        self.roots = roots
        self.patches = patches
        return path, list(patches.items()), metadata_review["sha256"]

    @staticmethod
    def _check_test_fixture_fields(checks: dict[str, bool]) -> None:
        failed = sorted(name for name, valid in checks.items() if not valid)
        if failed:
            raise AssertionError("synthetic Mac build fixture identity mismatch: " + ", ".join(failed))

    def attach_producer_evidence(self, packet_path: Path) -> None:
        packet = json.loads(packet_path.read_text())
        for row in packet["wheels"]:
            if row["role"] == "macos-arm64":
                build_ref = row["buildReceipt"]
                build = json.loads((self.root / build_ref["path"]).read_text())
                operator = build["operator"]
                expected_source = {
                    "commit": validator.PRODUCER, "tree": validator.PRODUCER_TREE,
                    "version": validator.VERSION, "status": "",
                    "trackedCount": validator.SOURCE_IDENTITY["trackedCount"],
                    "gitStageRosterSha256": validator.SOURCE_IDENTITY["gitStageRosterSha256"],
                    "selectedInputCount": validator.SOURCE_IDENTITY["selectedInputCount"],
                    "selectedInputRosterSha256": validator.SOURCE_IDENTITY["selectedInputRosterSha256"],
                }
                checks = {
                    "source": build.get("source") == expected_source,
                    "postRunSource": build.get("postRunSource") == expected_source,
                    "postRunOperator": build.get("postRunOperator") == {
                        "runnerBytes": operator.get("runnerBytes"),
                        "runnerSha256": operator.get("runnerSha256"),
                        "staticAuditBytes": operator.get("staticAuditBytes"),
                        "staticAuditSha256": operator.get("staticAuditSha256")},
                    "acceptedPlanSha256": operator.get("acceptedPlanSha256") ==
                        "cafc0eb9b3ed1346210e2e1b85e437df8fb85beb7bd70e5e454f992de5b5ac53",
                    "measurementRepairPlanSha256": operator.get("measurementRepairPlanSha256") ==
                        "fba4b31855acb8ab74a864ef4e5b84d080a52987d5e4a95ffe493c1d045b0ff4",
                    "runnerPath": operator.get("runnerPath") ==
                        str(validator.MAC_BUILD_OPERATOR_ROOT / "run_build_static.py"),
                    "runnerSha256": operator.get("runnerSha256") == validator.MAC_BUILD_RUNNER_SHA256,
                    "staticAuditPath": operator.get("staticAuditPath") ==
                        str(validator.MAC_BUILD_OPERATOR_ROOT / "static_audit.py"),
                    "staticAuditSha256": operator.get("staticAuditSha256") ==
                        validator.MAC_BUILD_AUDIT_TOOL_SHA256,
                }
                self._check_test_fixture_fields(checks)
            if row["role"] == "macos-arm64":
                audit_ref = row["auditReceipt"]
                audit = json.loads((self.root / audit_ref["path"]).read_text())
                operator = audit["operator"]
                helper = operator["helper"]
                checks = {
                    "runnerBeforeAfter": operator.get("runnerBefore") == operator.get("runnerAfter"),
                    "helperAfter": operator.get("helperAfter") == [helper.get("bytes"), helper.get("sha256")],
                    "runnerBefore": operator.get("runnerBefore") == [22917, validator.MAC_AUDIT_RUNNER_SHA256],
                    "runnerAfter": operator.get("runnerAfter") == [22917, validator.MAC_AUDIT_RUNNER_SHA256],
                    "pythonBeforeAfter": operator.get("pythonBefore") == operator.get("pythonAfter"),
                    "runnerPath": operator.get("runnerPath") == str(validator.MAC_AUDIT_OPERATOR_ROOT / "run_static_audit.py"),
                    "helperPath": helper.get("path") == str(validator.MAC_AUDIT_OPERATOR_ROOT / "static_audit.py"),
                    "helperSha": helper.get("sha256") == validator.MAC_AUDIT_HELPER_SHA256,
                    "pythonPath": operator.get("pythonPath") == validator.MAC_PYTHON_PATH,
                    "pythonIdentity": operator.get("pythonBefore") == [validator.MAC_PYTHON_BYTES, validator.MAC_PYTHON_SHA256],
                }
                self._check_test_fixture_fields(checks)
            selected = validator._verify_producer_chain(
                self.root, row, row["role"], row["filename"], row["sha256"])
            copied = []
            for index, evidence in enumerate(selected):
                target_name = f"producer-evidence/{row['role']}/{index:03d}.bin"
                target = self.root / target_name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(Path(evidence["sourcePath"]).read_bytes())
                copied.append({"sourcePath": evidence["sourcePath"], "path": target_name,
                               "sizeBytes": evidence["sizeBytes"], "sha256": evidence["sha256"]})
            row["producerEvidence"] = copied
        packet_path.write_text(json.dumps(packet, sort_keys=True) + "\n")


class SourceMapTests(unittest.TestCase):
    def test_exact_git_stage_row_encoding_is_accepted(self):
        with tempfile.TemporaryDirectory() as temp:
            ref_value, source = source_map(Path(temp), "synthetic", "a" * 40, "b" * 40)
            parsed, tracked = validator._validate_source_map(Path(temp), ref_value, "synthetic")
            self.assertEqual(len(parsed["stageRows"]), 5901)
            self.assertEqual(len(tracked), 5901)
            self.assertTrue(all(row.split("\t", 1)[0].split(" ")[2] == "0"
                                for row in parsed["stageRows"]))

    def test_tab_separated_non_git_stage_encoding_is_refused(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            ref_value, source = source_map(root, "synthetic", "a" * 40, "b" * 40)
            source["stageRows"][0] = "100644\t" + source["trackedFiles"][0]["gitBlobSha1"] + "\t" + source["trackedPaths"][0]
            source["source"]["gitStageRosterSha256"] = sha("\n".join(source["stageRows"]).encode())
            changed = json_ref(root, ref_value["path"], source)
            with self.assertRaisesRegex(validator.ProfileError, "stage row is malformed"):
                validator._validate_source_map(root, changed, "synthetic")

    def test_held_test_fixture_never_claims_ready_qualification(self):
        with tempfile.TemporaryDirectory() as temp:
            fixture = FullPacket(Path(temp))
            path, patches, _ = fixture.build()
            stack = patch.multiple(validator, **dict(patches))
            stack.start()
            self.addCleanup(stack.stop)
            packet = json.loads(path.read_text())
            self.assertEqual(packet["status"], "HELD")
            with self.assertRaisesRegex(validator.ProfileError, "not ready"):
                validator.validate_packet(path, expected_metadata_review_sha256="0" * 64)

    def test_synthetic_ready_packet_runs_real_validator_and_assembler(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            fixture = FullPacket(root)
            packet_path, patches, metadata_review_sha = fixture.build()
            with patch.multiple(validator, **dict(patches)):
                packet = json.loads(packet_path.read_text())
                packet["status"] = "READY_FOR_ASSEMBLY"
                packet_path.write_text(json.dumps(packet, sort_keys=True) + "\n")
                fixture.attach_producer_evidence(packet_path)
                direct_refs = [packet["producer"]["sourceMap"], packet["metadata"]["sourceMap"],
                    packet["metadata"]["ancestry"], *packet["metadata"]["files"],
                    *packet["preAssemblyReviews"]]
                for gate in packet["gates"].values():
                    direct_refs.extend((gate["receipt"], gate["review"]))
                    if "dataReview" in gate: direct_refs.append(gate["dataReview"])
                for wheel in packet["wheels"]:
                    direct_refs.extend(wheel[key] for key in
                        ("staticReview", "buildReceipt", "auditReceipt", "artifactReview"))
                    direct_refs.extend(wheel["producerEvidence"])
                missing = [row["path"] for row in direct_refs
                           if not (root / row["path"]).is_file()]
                self.assertEqual(missing, [], "synthetic direct evidence refs missing physical files")
                verified = validator.validate_packet(
                    packet_path, expected_metadata_review_sha256=metadata_review_sha)
                self.assertEqual(verified["status"], "VERIFIED_FOR_ASSEMBLY")
                self.assertEqual(len(verified["wheels"]), 3)
                notes = root / "local-source/docs/helix/05-deploy/releases/v0.4.56.md"
                output = root / "assembled"
                result = publisher.assemble(
                    packet_path, notes, output,
                    metadata_review_sha256=metadata_review_sha)
                self.assertEqual(result["status"], "ASSEMBLED_LOCAL_UNSIGNED")
                self.assertTrue((output / "qualification-evidence-0.4.56.zip").is_file())

    def test_ready_packet_refuses_coherent_semantic_mutation_table(self):
        mutations = (
            ("ancestry-source", "ancestry receipt identities differ"),
            ("selected-M-input", "metadata M changes selected producer build inputs"),
            ("consumed-M-bytes", "metadata source bytes differ from M's tracked Git blob"),
            ("external-M-review", "outside-Q trust anchor"),
            ("native-member-report", "member bytes differ from static report: weft/weft.abi3.so"),
            ("original-failure-chain", "separate audit does not bind the exact original failed build and wheel"),
            ("missing-producer-evidence", "producer evidence packet paths differ"),
            ("false-Mac-runtime", "Mac gate overclaims beyond its static-only audit"),
        )
        with tempfile.TemporaryDirectory() as temp:
            base = Path(temp)
            for name, expected_error in mutations:
                with self.subTest(mutation=name):
                    root = base / name
                    fixture = FullPacket(root)
                    packet_path, patch_values, metadata_review_sha = fixture.build()
                    scope = patch.multiple(validator, **dict(patch_values))
                    scope.start()
                    try:
                        packet = json.loads(packet_path.read_text())
                        packet["status"] = "READY_FOR_ASSEMBLY"
                        packet_path.write_text(json.dumps(packet, sort_keys=True) + "\n")
                        fixture.attach_producer_evidence(packet_path)
                        valid = validator.validate_packet(
                            packet_path, expected_metadata_review_sha256=metadata_review_sha)
                        self.assertEqual(valid["status"], "VERIFIED_FOR_ASSEMBLY",
                                         "each mutation must begin from a fresh valid READY fixture")
                        packet = json.loads(packet_path.read_text())

                        if name == "ancestry-source":
                            ancestry_ref = packet["metadata"]["ancestry"]
                            ancestry = json.loads((root / ancestry_ref["path"]).read_text())
                            ancestry["sourceCommit"] = "e" * 40
                            rewrite_json_ref(root, ancestry_ref, ancestry)
                        elif name == "selected-M-input":
                            source_ref = packet["metadata"]["sourceMap"]
                            source_map_value = json.loads((root / source_ref["path"]).read_text())
                            source_map_value["selectedBuildInputs"][0]["sha256"] = "e" * 64
                            canonical = source_map_value["selectedBuildInputs"]
                            source_map_value["source"]["selectedInputRosterSha256"] = sha(
                                (json.dumps(canonical, sort_keys=True) + "\n").encode())
                            rewrite_json_ref(root, source_ref, source_map_value)
                        elif name == "consumed-M-bytes":
                            row = packet["metadata"]["files"][0]
                            file_path = root / row["path"]
                            changed = file_path.read_bytes() + b" changed"
                            file_path.write_bytes(changed)
                            row.update(sizeBytes=len(changed), sha256=sha(changed))
                        elif name == "external-M-review":
                            row = next(item for item in packet["preAssemblyReviews"]
                                       if item["scope"] == "producer-metadata-source")
                            review_path = root / row["path"]
                            changed = review_path.read_bytes() + b"changed external trust text\n"
                            review_path.write_bytes(changed)
                            row.update(sizeBytes=len(changed), sha256=sha(changed))
                        elif name == "native-member-report":
                            wheel = packet["wheels"][0]
                            report_ref = wheel["staticReview"]
                            report = json.loads((root / report_ref["path"]).read_text())
                            report["members"]["weft/weft.abi3.so"]["sha256"] = "e" * 64
                            rewrite_json_ref(root, report_ref, report)
                            validator.X64_STATIC_REPORT_SHA256 = report_ref["sha256"]
                        elif name == "original-failure-chain":
                            audit_ref = packet["wheels"][0]["auditReceipt"]
                            audit = json.loads((root / audit_ref["path"]).read_text())
                            audit["originalInputs"]["buildCommandExitCode"] = 1
                            rewrite_json_ref(root, audit_ref, audit)
                            validator.X64_AUDIT_RESULT_SHA256 = audit_ref["sha256"]
                        elif name == "missing-producer-evidence":
                            packet["wheels"][0]["producerEvidence"].pop()
                        elif name == "false-Mac-runtime":
                            gate_ref = packet["gates"]["macos-arm64-build-static"]["receipt"]
                            gate_receipt = json.loads((root / gate_ref["path"]).read_text())
                            gate_receipt["nativeExecuted"] = True
                            rewrite_json_ref(root, gate_ref, gate_receipt)
                            validator.GATE_RECEIPT_SHA256[
                                "macos-arm64-build-static"] = gate_ref["sha256"]

                        packet_path.write_text(json.dumps(packet, sort_keys=True) + "\n")
                        with self.assertRaisesRegex(validator.ProfileError, expected_error):
                            validator.validate_packet(
                                packet_path, expected_metadata_review_sha256=metadata_review_sha)
                    finally:
                        scope.stop()

    def test_mac_audit_selected_relative_evidence_paths_close(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            for name, data in (("static-audit.json", b"report"),
                               ("logs/static-audit.stdout.log", b"stdout"),
                               ("logs/static-audit.stderr.log", b"")):
                path = root / name
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_bytes(data)
            receipt = {"selectedEvidenceFiles": [
                {"path": name, "bytes": (root / name).stat().st_size,
                 "sha256": sha((root / name).read_bytes())}
                for name in ("static-audit.json", "logs/static-audit.stdout.log",
                             "logs/static-audit.stderr.log")]}
            result = validator._mac_selected_output_rows(receipt, root / "result.json")
            self.assertEqual([row["path"] for row in result], [
                "static-audit.json", "logs/static-audit.stdout.log", "logs/static-audit.stderr.log"])
            receipt["selectedEvidenceFiles"][1]["path"] = "../outside.log"
            with self.assertRaises(validator.ProfileError):
                validator._mac_selected_output_rows(receipt, root / "result.json")

    def test_absolute_gate_outputs_map_historical_identity_to_packet_copy(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp); source = root / "historical"; packet = root / "packet"
            (source / "logs").mkdir(parents=True); (packet / "gate").mkdir(parents=True)
            payload = b"historical retained log"
            original = source / "logs" / "capture.log"; original.write_bytes(payload)
            copied = packet / "gate" / "logs" / "capture.log"
            copied.parent.mkdir(); copied.write_bytes(payload)
            row = {"path": str(original), "sizeBytes": len(payload), "sha256": sha(payload)}
            result = validator._absolute_evidence_rows(
                packet / "gate" / "result.json", [row], "synthetic gate",
                historical_root=source, verify=True)
            self.assertEqual(result, [{"path": "logs/capture.log",
                                       "sizeBytes": len(payload), "sha256": sha(payload)}])
            copied.write_bytes(b"changed")
            with self.assertRaisesRegex(validator.ProfileError, "bytes differ"):
                validator._absolute_evidence_rows(
                    packet / "gate" / "result.json", [row], "synthetic gate",
                    historical_root=source, verify=True)

    def test_mac_producer_evidence_keeps_historical_paths_without_python_binary(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            audit_root = root / "mac-audit"; audit_root.mkdir()
            operator_root = root / "mac-operator"; operator_root.mkdir()
            custody_root = root / "custody"; custody_root.mkdir()
            python_path = root / "python3.9"; python_path.write_bytes(b"test runtime identity")
            report = audit_root / "static-audit.json"; report.write_bytes(b"test report")
            stdout = audit_root / "static-audit.stdout.log"; stdout.write_bytes(b"test stdout")
            runner = operator_root / "run_static_audit.py"; runner.write_bytes(b"runner source")
            helper = operator_root / "static_audit.py"; helper.write_bytes(b"helper source")
            custody = {}
            for name, payload in (("custody.json", b"custody"),
                                  ("git-stage-roster.txt", b"stage"),
                                  ("source-inputs.rows.json", b"inputs")):
                path = custody_root / name; path.write_bytes(payload)
                custody[name] = {"bytes": len(payload), "path": str(path), "sha256": sha(payload)}
            record = {
                "selectedEvidenceFiles": [
                    {"path": p.name, "bytes": p.stat().st_size, "sha256": sha(p.read_bytes())}
                    for p in (report, stdout)],
                "operator": {"runnerPath": str(runner),
                    "runnerBefore": [runner.stat().st_size, sha(runner.read_bytes())],
                    "runnerAfter": [runner.stat().st_size, sha(runner.read_bytes())],
                    "helper": {"path": str(helper), "bytes": helper.stat().st_size,
                               "sha256": sha(helper.read_bytes())},
                    "helperAfter": [helper.stat().st_size, sha(helper.read_bytes())],
                    "pythonPath": str(python_path),
                    "pythonBefore": [python_path.stat().st_size, sha(python_path.read_bytes())],
                    "pythonAfter": [python_path.stat().st_size, sha(python_path.read_bytes())]},
                "originalInputs": {"source": {"custodyAssetsBefore": custody,
                                                "custodyAssetsAfter": custody}},
            }
            with patch.object(validator, "MAC_AUDIT_ORIGINAL_ROOT", audit_root), \
                 patch.object(validator, "MAC_AUDIT_OPERATOR_ROOT", operator_root), \
                 patch.object(validator, "MAC_CUSTODY_ROOT", custody_root):
                rows = validator._producer_evidence_rows(
                    record, audit_root / "result.json", where="macos-arm64 audit")
            sources = {row["sourcePath"] for row in rows}
            self.assertIn(str(report.resolve()), sources)
            self.assertIn(str(runner.resolve()), sources)
            self.assertIn(str(helper.resolve()), sources)
            self.assertNotIn(str(python_path.resolve()), sources)
            self.assertTrue(all(Path(path).is_absolute() for path in sources))

    def test_source_guard_historical_logs_relocate_to_packet_logs_and_reject_drift(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            receipt_dir = root / "gate"; (receipt_dir / "logs").mkdir(parents=True)
            payload = b"captured source output\n"
            retained = receipt_dir / "logs/source-command.stdout.bin"
            retained.write_bytes(payload)
            historical_root = root / "original-run/logs"
            processes = []
            for index in range(40):
                stdout = index == 0
                processes.append({
                    "exitCode": 0, "timedOut": False, "drainExpired": False,
                    "rawOverflow": False, "streamCaptureComplete": True,
                    "spawnOrCaptureError": None, "launchError": None,
                    "streamsEOF": {"stdout": True, "stderr": True},
                    "stdout": {"rawBytes": len(payload) if stdout else 0,
                        "sha256": sha(payload) if stdout else sha(b""),
                        "logPath": str(historical_root / "source-command.stdout.bin") if stdout else None},
                    "stderr": {"rawBytes": 0, "sha256": sha(b""), "logPath": None},
                })
            receipt = {"processes": processes,
                "terminalArtifacts": [{"type": "file", "path": "logs/source-command.stdout.bin",
                    "bytes": len(payload), "sha256": sha(payload)}],
                "rawOutputBytes": {"total": len(payload)}, "retainedLogBytes": len(payload),
                "inclusiveEvidenceBytes": 512, "failure": None}
            with patch.object(validator, "SOURCE_GUARD_ORIGINAL_LOG_ROOT", historical_root), \
                 patch.object(validator, "SOURCE_GUARD_RAW_BYTES", len(payload)), \
                 patch.object(validator, "SOURCE_GUARD_EVIDENCE_BYTES", 512):
                rows = validator._source_package_output_inventory(
                    root, receipt_dir / "result.json", receipt)
                self.assertEqual(rows, [{"path": "gate/logs/source-command.stdout.bin",
                                         "sizeBytes": len(payload), "sha256": sha(payload)}])
                retained.write_bytes(b"altered\n")
                with self.assertRaises(validator.ProfileError):
                    validator._source_package_output_inventory(
                        root, receipt_dir / "result.json", receipt)

if __name__ == "__main__": unittest.main()
