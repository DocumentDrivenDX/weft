#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Offline, nonpublishing 0.4.52 selected-wheel custody verifier.

This is a deliberately narrow final-asset gate. It does not build, publish,
qualify native execution, or reinterpret the existing compiler evidence.
"""
import argparse
import base64
import csv
import hashlib
import io
import json
import re
import zipfile
from pathlib import Path

VERSION = "0.4.52"
DEFAULT_SOURCE = "32ec02f0c3fd24a97b7726500dc5992c6475908d"
LINUX_WORKFLOW = "Build selected Weft 0.4.52 Linux wheel"
PAGE_COMMAND = "python -m unittest test_managed_current_page_target -v"
MAC_FUNCTIONAL_PACKET_SHA256 = "16b987ebb2e0ad5e419676e5a63cd827ca3eb94716fd75d3afff31374c10e441"


def require(ok, message):
    if not ok:
        raise SystemExit(message)


def sha_bytes(raw):
    return hashlib.sha256(raw).hexdigest()


def read_json(path, label):
    require(path.is_file(), f"{label}: missing file: {path}")
    try:
        return json.loads(path.read_text())
    except (OSError, json.JSONDecodeError) as exc:
        raise SystemExit(f"{label}: invalid JSON at {path}: {exc}")


def wheel_identity(path, platform):
    require(path.is_file(), f"{platform}: wheel missing: {path}")
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        require(len(names) == len(set(names)), f"{platform}: duplicate wheel members")
        require(all(not name.startswith("/") and ".." not in Path(name).parts for name in names),
                f"{platform}: unsafe wheel member path")
        records = [name for name in names if name.endswith(".dist-info/RECORD")]
        metas = [name for name in names if name.endswith(".dist-info/METADATA")]
        require(len(records) == len(metas) == 1, f"{platform}: expected one RECORD and METADATA")
        record_name = records[0]
        rows = list(csv.reader(io.StringIO(archive.read(record_name).decode("utf-8"))))
        require({row[0] for row in rows} == set(names), f"{platform}: RECORD membership mismatch")
        for name, digest, size in rows:
            if name == record_name:
                require(digest == size == "", f"{platform}: invalid RECORD self row")
                continue
            raw = archive.read(name)
            expected = "sha256=" + base64.urlsafe_b64encode(hashlib.sha256(raw).digest()).decode().rstrip("=")
            require(digest == expected and size == str(len(raw)), f"{platform}: RECORD mismatch: {name}")
        metadata = archive.read(metas[0]).decode("utf-8")
        require(f"Name: weft-sql\nVersion: {VERSION}\n" in metadata,
                f"{platform}: wheel metadata is not weft-sql {VERSION}")
        native = [name for name in names if name.endswith((".so", ".pyd"))]
        require(len(native) == 1, f"{platform}: expected one native extension")
        if platform == "linux":
            require("manylinux_2_17_x86_64.manylinux2014_x86_64" in path.name,
                    "linux: expected x86_64 manylinux wheel filename")
            require("Tag: cp39-abi3-manylinux" in archive.read(next(n for n in names if n.endswith(".dist-info/WHEEL"))).decode(),
                    "linux: expected cp39 abi3 manylinux tag")
        else:
            require("macosx_11_0_arm64" in path.name, "macos: expected macOS ARM64 wheel filename")
            require("Tag: cp39-abi3-macosx_11_0_arm64" in archive.read(next(n for n in names if n.endswith(".dist-info/WHEEL"))).decode(),
                    "macos: expected cp39 abi3 macOS ARM64 tag")
        return {
            "wheel": path.name,
            "wheelSha256": sha_bytes(path.read_bytes()),
            "nativeModule": native[0],
            "nativeModuleSha256": sha_bytes(archive.read(native[0])),
            "recordSha256": sha_bytes(archive.read(record_name)),
        }


def check_build_reports(label, wheel, build_path, packaging_path, source, target):
    build = read_json(build_path, f"{label} build report")
    package = read_json(packaging_path, f"{label} packaging report")
    require(build.get("status") == package.get("status") == "passed", f"{label}: build/packaging status not passed")
    require(build.get("version") == package.get("version") == VERSION, f"{label}: wrong report version")
    require(build.get("sourceCommit") == source, f"{label}: build report source SHA mismatch")
    require(build.get("target") == target, f"{label}: build report target mismatch")
    ident = wheel_identity(wheel, label)
    for report, name in ((build, "build"), (package, "packaging")):
        require(report.get("wheel") == ident["wheel"], f"{label}: {name} report names a different wheel")
        require(report.get("wheelSha256") == ident["wheelSha256"], f"{label}: {name} report wheel SHA mismatch")
        require(report.get("nativeModuleSha256") == ident["nativeModuleSha256"], f"{label}: {name} report native SHA mismatch")
        require(report.get("recordSha256") == ident["recordSha256"], f"{label}: {name} report RECORD SHA mismatch")
    return ident


def receipt_asset_path(receipt_path, relative_path, label):
    base = receipt_path.resolve().parent
    candidate = (base / relative_path).resolve()
    require(candidate.is_relative_to(base), f"functional receipt: {label} path escapes receipt directory")
    require(candidate.is_file(), f"functional receipt: missing {label}: {candidate}")
    return candidate


def check_mac_functional_receipt(receipt, receipt_path, expected_source, mac_ident):
    candidate = receipt.get("candidate", {})
    require(candidate.get("version") == VERSION, "functional receipt: not 0.4.52")
    require(candidate.get("source_commit") == expected_source,
            "functional receipt: compiler evidence source SHA does not equal selected final source SHA")
    artifact = receipt.get("artifact", {})
    umf = receipt.get("umf_assets", {})
    require(umf.get("core_0.10.0_sha256") == "49633924b45a3c8499d4fcc1a24c30d5a8fcb34d974ab019bbda987cb47da0f9",
            "functional receipt: wrong UMF core asset")
    require(umf.get("access_0.1.0.dev6_sha256") == "8f6e86ed94983d0156c2d6e9e1bf83fae186fe1d4a288a3846722e64aa365cf0",
            "functional receipt: wrong UMF access asset")
    require(artifact.get("sha256") == mac_ident["wheelSha256"], "functional receipt: Mac wheel SHA mismatch")
    require(artifact.get("native_extension_sha256") == mac_ident["nativeModuleSha256"],
            "functional receipt: Mac native extension SHA mismatch")
    checks = receipt.get("checks", [])
    page = next((item for item in checks if item.get("command") == PAGE_COMMAND), None)
    current_read = next((item for item in checks if "test_managed_current_query.ManagedCurrentQueryTests" in item.get("command", "")), None)
    page_result = page.get("result", "") if page else ""
    read_result = current_read.get("result", "") if current_read else ""
    require(page and ("4 passed" in page_result or ("Ran 4 tests" in page_result and "OK" in page_result)),
            "functional receipt: four managed page-target compiler tests missing")
    require(current_read and ("2 passed" in read_result or ("Ran 2 tests" in read_result and "OK" in read_result)),
            "functional receipt: two managed current-read compiler controls missing")
    for label, item in (("page-test log", page), ("current-read log", current_read)):
        asset = receipt_asset_path(receipt_path, item.get("log", ""), label)
        require(sha_bytes(asset.read_bytes()) == item.get("log_sha256"),
                f"functional receipt: {label} SHA mismatch")
    source_f = receipt.get("owner_public_0_4_51_source_f_controls", {})
    expected_cases = {
        "arras-view-0.1.0-opaque-selected-payload": "e60798fce3bd8ef8126e91e556528fae6cd5074864e68f1d1af4c2247730631e",
        "arras-view-unsupported-version-refuses": "acb21c14f990802a7022a2a2aa0ca320c50b6efd96868eb1dab53ea2f0773974",
    }
    results = {item.get("caseId"): item for item in source_f.get("results", [])}
    require(source_f.get("source_freeze_commit") == "557153731e9f5d0583193a1189a07c6ed485d3d3",
            "functional receipt: expected public .51 Source-F baseline missing")
    require(source_f.get("input_fixture_sha256") == "cdddb2454b033c5753177cdedb447efbf45f5131c42beb17aeb6d319eac71537",
            "functional receipt: public .51 Source-F fixture digest mismatch")
    require(source_f.get("public_0_4_51_expected_output_sha256") == "ff8803dda03a4ef376e44984c1a010ead8065b817a002d6178dba8b0343256bf",
            "functional receipt: public .51 expected output receipt digest mismatch")
    source_result = next((item for item in checks if item.get("command", "").endswith("run_source_f_pair.py")), None)
    require(source_result is not None, "functional receipt: Source-F result asset reference missing")
    source_result_path = receipt_asset_path(receipt_path, source_result.get("log", ""), "Source-F result")
    require(sha_bytes(source_result_path.read_bytes()) == source_result.get("log_sha256"),
            "functional receipt: Source-F result asset SHA mismatch")
    source_result_json = read_json(source_result_path, "Source-F result asset")
    require(source_result_json.get("status") == "passed" and
            source_result_json.get("fixtureSha256") == source_f.get("input_fixture_sha256") and
            source_result_json.get("publicExpectedOutputSha256") == source_f.get("public_0_4_51_expected_output_sha256"),
            "functional receipt: Source-F result asset disagrees with receipt")
    for asset_info in source_f.get("raw_case_assets", []):
        asset = receipt_asset_path(receipt_path, asset_info.get("path", ""), "Source-F raw case asset")
        require(sha_bytes(asset.read_bytes()) == asset_info.get("sha256"),
                "functional receipt: Source-F raw case asset SHA mismatch")
    source_result_cases = {item.get("caseId"): item for item in source_result_json.get("cases", [])}
    for case_id, digest in expected_cases.items():
        item = results.get(case_id)
        require(item and item.get("fullRawOutputParity") is True and
                item.get("expectedRawSha256") == item.get("candidateRawSha256") == digest,
                f"functional receipt: Source-F parity missing or mismatched for {case_id}")
        generated = source_result_cases.get(case_id)
        require(generated and generated.get("fullRawOutputParity") is True and
                generated.get("expectedRawSha256") == generated.get("candidateRawSha256") == digest,
                f"functional receipt: retained Source-F output asset mismatch for {case_id}")
    limits = receipt.get("limits", [])
    require(any("No native SQL backend or Unity Catalog execution" in item for item in limits),
            "functional receipt: native execution limitation must remain explicit")
    return {
        "pageCompilerTests": 4,
        "currentReadCompilerControls": 2,
        "sourceFCompilerControls": sorted(expected_cases),
        "nativeExecution": "not-qualified",
    }


def check_mac_build(wheel, build_path, packaging_path, source):
    receipt = read_json(build_path, "Mac build receipt")
    package = read_json(packaging_path, "Mac packaging report")
    source_info = receipt.get("source", {})
    artifact = receipt.get("artifact", {})
    build = receipt.get("build", {})
    check = receipt.get("check", {})
    require(receipt.get("profile") == "weft-0.4.52-local-macos-wheel-build/0.1" and
            source_info.get("commit") == source and source_info.get("worktree_clean") is True,
            "Mac build receipt profile, source SHA or worktree custody mismatch")
    require(build.get("result") == "success" and build.get("profile") == "release",
            "Mac release build is not recorded successful")
    ident = wheel_identity(wheel, "macos")
    require(artifact.get("filename") == ident["wheel"] and artifact.get("sha256") == ident["wheelSha256"],
            "Mac build receipt wheel identity mismatch")
    require(artifact.get("native_module_sha256") == ident["nativeModuleSha256"] and
            artifact.get("wheel_tag") == "cp39-abi3-macosx_11_0_arm64",
            "Mac build receipt native module or ABI tag mismatch")
    require(package.get("status") == "passed" and package.get("version") == VERSION and
            package.get("wheel") == ident["wheel"] and package.get("wheelSha256") == ident["wheelSha256"] and
            package.get("nativeModuleSha256") == ident["nativeModuleSha256"],
            "Mac packaging report does not bind the selected wheel")
    require(check.get("result") == "passed" and
            check.get("receipt_sha256") == sha_bytes(packaging_path.read_bytes()),
            "Mac build receipt does not bind the exact packaging report")
    return ident


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("--source-commit", default=DEFAULT_SOURCE)
    p.add_argument("--linux-run-id", required=True)
    p.add_argument("--linux-run-report", type=Path, required=True)
    p.add_argument("--linux-wheel", type=Path, required=True)
    p.add_argument("--linux-build-report", type=Path, required=True)
    p.add_argument("--linux-packaging-report", type=Path, required=True)
    p.add_argument("--mac-wheel", type=Path, required=True)
    p.add_argument("--mac-build-receipt", type=Path, required=True)
    p.add_argument("--mac-packaging-report", type=Path, required=True)
    p.add_argument("--mac-functional-receipt", type=Path, required=True)
    p.add_argument("--output", type=Path, required=True)
    a = p.parse_args()
    require(re.fullmatch(r"[0-9a-f]{40}", a.source_commit) is not None, "source commit must be full lowercase Git SHA")
    require(re.fullmatch(r"[0-9]+", a.linux_run_id) is not None, "Linux run ID must be numeric")

    # Validate the available final-SHA Mac artifact and its compiler-only
    # receipt before checking whether the Linux half of the pair exists.
    mac = check_mac_build(a.mac_wheel, a.mac_build_receipt, a.mac_packaging_report, a.source_commit)
    mac_receipt = read_json(a.mac_functional_receipt, "Mac functional receipt")
    require(sha_bytes(a.mac_functional_receipt.read_bytes()) == MAC_FUNCTIONAL_PACKET_SHA256,
            "Mac functional receipt differs from the frozen reviewed packet")
    functional = check_mac_functional_receipt(mac_receipt, a.mac_functional_receipt,
                                               a.source_commit, mac)

    run = read_json(a.linux_run_report, "Linux Actions run")
    require(run.get("conclusion") == "success" and run.get("event") == "workflow_dispatch",
            "Linux Actions run is not a successful manual workflow")
    require(run.get("workflowName") == LINUX_WORKFLOW, "Linux Actions workflow identity mismatch")
    require(run.get("headSha") == a.source_commit, "Linux Actions run did not build the selected exact commit")
    require(str(run.get("id")) == a.linux_run_id,
            "Linux run JSON ID differs from the independently selected Actions run ID")
    linux = check_build_reports("linux", a.linux_wheel, a.linux_build_report,
                                a.linux_packaging_report, a.source_commit,
                                "x86_64-unknown-linux-gnu")
    require(str(read_json(a.linux_build_report, "Linux build report").get("buildRunId")) == a.linux_run_id,
            "Linux build report run ID differs from selected Actions run ID")
    linux_build = read_json(a.linux_build_report, "Linux build report")
    require(linux_build.get("manylinux") == "2014" and linux_build.get("buildWorkflowName") == LINUX_WORKFLOW,
            "Linux build report lacks exact manylinux2014 workflow custody")

    report = {
        "schema": "weft-selected-assets-0.4.52/0.1",
        "status": "passed",
        "version": VERSION,
        "sourceCommit": a.source_commit,
        "linuxRun": {
            "id": a.linux_run_id,
            "workflowName": run.get("workflowName"),
            "event": run.get("event"),
            "headSha": run.get("headSha"),
        },
        "linux": linux,
        "macos": mac,
        "macBuildReceiptSha256": sha_bytes(a.mac_build_receipt.read_bytes()),
        "macPackagingReportSha256": sha_bytes(a.mac_packaging_report.read_bytes()),
        "compilerOnlyFunctionalEvidence": functional,
        "umf": {
            "core": {"version": "0.10.0", "sha256": mac_receipt["umf_assets"]["core_0.10.0_sha256"]},
            "access": {"version": "0.1.0.dev6", "sha256": mac_receipt["umf_assets"]["access_0.1.0.dev6_sha256"]},
        },
        "limits": ["No publication or release action is performed by this verifier.",
                   "Compiler-only evidence does not qualify native SQL backend or Unity Catalog execution."],
    }
    a.output.parent.mkdir(parents=True, exist_ok=True)
    a.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))


if __name__ == "__main__":
    main()
