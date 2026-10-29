#!/usr/bin/env python3
"""Validate the closed, local-only input packet for Weft 0.4.56 assembly.

This module deliberately does not import Weft, contact a registry, or accept
GitHub Actions producer identities.  It verifies bounded evidence and wheel
bytes that have already been produced and independently reviewed.
"""

from __future__ import annotations

import hashlib
import base64
import csv
import io
import json
import os
import re
import stat
import struct
import zipfile
from email.parser import BytesParser
from pathlib import Path
from typing import Any

PROFILE = "weft-local-build-static-release/1"
DISTRIBUTION = "umf-weft"
VERSION = "0.4.56"
TAG = "v0.4.56"
PRODUCER = "973d864c8d26b2bc173a411eac8368da5efc1ab3"
PRODUCER_TREE = "8b20d1f01bea72ea08ed01fcaac97266ed3cb4ff"
UMF_GATE_SOURCE = "e4299aff8e87e9d211f9d86df2f2013ab9c95945"
UMF_GATE_TREE = "8b98c0d895983ba2b7637dd3aea567dab0943883"
FINAL_COHORT_LOCK = "cfe589472a5340446e5b1ddf5e39dbf00e6b6d83370fbe066122f52139fe2274"
FINAL_COHORT_INDEX = "78026c05d3824613f0e2a3bf30cf24bd32af24de1d560f0b7fa555a1e75f8d60"
FINAL_COHORT_CUSTODY = "95e9493f06adefb323193f9d8fbe5577ebfa335140a9841f549d2caf098df8ed"
FINAL_MIRROR = "809a418b9f6f97e403e3fbcc7066d23657347d1323efb6baf2d4d3bb3f17bd36"
FINAL_REQUIREMENTS = "4847944f3cafa5fcc9813890f85a4db24a064ed49b479c890e3f4363cb97de51"
FINAL_PREP_SOURCE_SEAL = "23a16e87f53df619ad06ffcfe9e234efe0394287b5437070a13209e06c945ccf"
FINAL_PREP_RUNNER = "73a732711ae150ff55a1f4fa1b656ae0762779e18cd94b82d4a2227930aaefb9"
FINAL_ADMISSION_SOURCE = "829159a4c06106db76e0daf8c27e52b53ef2b8136174c0a93cd2d4a44b1ff916"
FINAL_INSTALLED_SNAPSHOT = "5e002268af9fa6105e88fd7fd5e10f252e914ba284a6985afef42124892b4d6a"
FINAL_WEFT_WHEEL = "32f4c38d9a614dd4c3a367f8b24d5cd4ba41ec80c2a4a2e3423e2082a294dbdd"
FINAL_WEFT_WHEEL_BYTES = 14358901
FINAL_WEFT_NATIVE = "e8a0c19e8a3a974a97b141b12adf532a52a2c92d03d558c087f002c6d317a1d9"
FINAL_WEFT_NATIVE_BYTES = 44271496
FINAL_NATIVE_OPERATOR = "97ced9208aff574f2b9f86f156748ff9ff554865fa8283c4214547e532fae8c7"
FINAL_NATIVE_CAMPAIGN = "6a2549595c43528c52cd4a2d3287841a91504d1349ecc517a7c29b7639ea96a8"
FINAL_NATIVE_REANALYSIS = "3303432a175a4e213bde87f0fbb2feeeb821346b94aecf649fdc3236b01423c2"
FINAL_NATIVE_REVIEW = "aa45f6375445fbd4a78fe6b726b38675d7dfcd7c13b3a992073b52ccb526117a"
FINAL_NATIVE_CAMPAIGN_ID = "661e79df-3dcb-40e3-87ed-29afcc472124"
FINAL_NATIVE_EXECUTION_ID = "45389ee1fa7a40238d403ff0048626a1"
FINAL_NATIVE_OUTPUT_ROOT = Path(
    "/private/tmp/g03-full14-bounded-operator-007-output/45389ee1fa7a40238d403ff0048626a1")
INSTALLED_ADMISSION_OUTPUT_ROOT = Path(
    "/private/tmp/g03-final-installed-prep-operator-001/runs/13fd35e735e542d6b5f1ee92b50c8796")
FINAL_NATIVE_REANALYSIS_PROFILE = "umf.g03-full14-predicate-reanalysis/6"
TABLE012_PROFILE = "umf.g03-table-operations-local-operator/2"
TABLE012_STATUS = "LOCAL_TABLE_OPERATIONS_BOUNDARY_PASS"
TABLE012_EXECUTION_ID = "a67a2c4a2acb4fdb9d99c509aa19c0a1"
TABLE012_RECEIPT_SHA256 = "c425a68cd8eda41ae42c2b134e1859925e4f8f89aa1edb45ba49c44514477a73"
TABLE012_RECEIPT_BYTES = 328783
TABLE012_REVIEW_SHA256 = "ef302c5196b6c9ebbbf55317d0265fca45a5c4770d12f052edfd13c69ed4e40c"
TABLE012_ROOT = Path("/private/tmp/g03-final-table-operations-operator-012/runs/a67a2c4a2acb4fdb9d99c509aa19c0a1")
TABLE012_SOFTWARE_ROOT = TABLE012_ROOT
TABLE012_PUBLIC_COUNT = 251
TABLE012_PUBLIC_BYTES = 2035016
TABLE012_SELECTED_BYTES = 54449411
TABLE012_SELECTED = {
    "prepare/artifacts/operation-browser-cases.json": (2537368, "6dfc3ed5da783d8e17094cb30dc9746d5fdd41c1216bd4ad574fa9fa123af71c"),
    "native-wasm/artifacts/table-web/weft_wasm.d.ts": (7007, "761a00e0f17ea857bb6dc2332183546d17c970d33adb9588ea3197f1cce10335"),
    "native-wasm/artifacts/table-web/weft_wasm.js": (27452, "c3c3b25d6a7f996bdc82e2dfb6defc3f9250e1977f24c9454bdd209164a63be2"),
    "native-wasm/artifacts/table-web/weft_wasm_bg.wasm": (51874180, "6266e03d4d0075384943ad4c5e19f58772cb202520f0c5041d917b01765f4294"),
    "native-wasm/artifacts/table-web/weft_wasm_bg.wasm.d.ts": (3404, "454be90b5e3e862dcd1779f6c3c2104cc706ca21b56b6cc5cd170d47d1ba958a"),
}
TRUSS011_PROFILE = "umf.g03-truss-local-lane/1"
TRUSS011_STATUS = "PASS"
TRUSS011_EXECUTION_ID = "ed8f1786-a3be-407c-b303-280ef3dffb46"
TRUSS011_RECEIPT_SHA256 = "69832734f3bc7b9668616708a8286a662bf51460e5bb46293749d14bd6f2c3f0"
TRUSS011_RECEIPT_BYTES = 205139
TRUSS011_REVIEW_SHA256 = "6ec4a1261534ab7faf1904c31bd05a196e24dff7d90c6c57436cee77d5a122b7"
TRUSS011_ROOT = Path("/private/tmp/g03-final-truss-operator-011-runs/ed8f1786-a3be-407c-b303-280ef3dffb46")
TRUSS011_SOFTWARE_ROOT = TRUSS011_ROOT
TRUSS011_OUTPUT_BYTES = 30635031
TRUSS011_SELECTED_BYTES = 28366524
TRUSS011_METADATA_BYTES = 2268507
TRUSS011_SELECTED = {
    "owned/out/artifacts/browser-wrapper/index.js": (2935, "ad9198fb79ec8dc80eeac9d37d126993374d7135628bf11ee303b0e60669884b"),
    "owned/out/artifacts/truss-web/weft_wasm.d.ts": (3350, "ec10303c3ce50ae1a39ef36352f9445618f27ee326bfa21a1182f4c2f092e491"),
    "owned/out/artifacts/truss-web/weft_wasm.js": (13916, "fce153e7f95ee82821e492a9e8732063e690b7640e2fbf22ec8324f8b3bca579"),
    "owned/out/artifacts/truss-web/weft_wasm_bg.wasm": (28345027, "86e5facad64ec12a2f1176dad9155b0a33ed63446dceebbe8e30369d64fb48b4"),
    "owned/out/artifacts/truss-web/weft_wasm_bg.wasm.d.ts": (1296, "79682ff29166ed93f5fa95a37d95dff8dfd23d89770f88700728ef97e33410f2"),
}
SOURCE_GUARD_RECEIPT_SHA256 = "58b9f92e6ab60fa2cd092ab0f17987cdb864c8659847a7f28a0473570253c2af"
SOURCE_GUARD_REVIEW_SHA256 = "21bf2fa2477d41e93e89e4714f6e45fae872f5ec201378861dfe80d2611b0728"
SOURCE_GUARD_SELECTED_COUNT = 1459
SOURCE_GUARD_SELECTED_BYTES = 134880002
SOURCE_GUARD_SELECTED_ROSTER_SHA256 = "6a671799d5be7a511cc6b5f7c9282d8745fbc80f4a5d796538b6643e8492f2a2"
SOURCE_GUARD_RAW_BYTES = 32866
SOURCE_GUARD_EVIDENCE_BYTES = 3821574
SOURCE_GUARD_ORIGINAL_LOG_ROOT = Path(
    "/private/tmp/g03-local-source-package-guards-operator-003/runs/5cfb93d1-07c/logs")
TRUSS011_BROWSER_EXPECTED = {
    "facts": {"browserReceiptsSha256": "cea472779b90551c37ae043fa1fa2592854c6f73b07bf9f7841b1c5970638506",
              "browserSummarySha256": "d3e4c1cb2893658191d0729c7c7a611bcd3e9bdf198abc728e9368c31a7ff8e8",
              "corpusBytes": 24724,
              "corpusSha256": "34f4694030aa822172200002275ee3e1b36ee7a248df960e60bb731648fb0d83",
              "nativeReportBytes": 186460,
              "nativeReportSha256": "c45b385d21c64d4abec74849a7cad50229a6eaa054b2a14a192b13c2566b8982"},
    "entity": {"browserReceiptsSha256": "2a0e1479a78eaacbb2f10da4559be734209af74ad52b805b659aa8a97cc81f03",
               "browserSummarySha256": "26d95a63b26483b47c5c8916429439039967cf0c2bb4229c99b949c1ffd3de06",
               "corpusBytes": 184480,
               "corpusSha256": "cdddb2454b033c5753177cdedb447efbf45f5131c42beb17aeb6d319eac71537",
               "nativeReportBytes": 994279,
               "nativeReportSha256": "ff8803dda03a4ef376e44984c1a010ead8065b817a002d6178dba8b0343256bf"},
}

SHA = re.compile(r"[0-9a-f]{64}\Z")
COMMIT = re.compile(r"[0-9a-f]{40}\Z")
MAX_PACKET = 2 * 1024 * 1024
MAX_EVIDENCE_FILE = 16 * 1024 * 1024
MAX_EVIDENCE_TOTAL = 16 * 1024 * 1024
MAX_RAW_LOGS = 1024 * 1024
MAX_WHEEL = 32 * 1024 * 1024
MAX_WHEEL_FAMILY = 70 * 1024 * 1024
MAX_EXPANDED_WHEEL = 128 * 1024 * 1024
MAX_WHEEL_ENTRIES = 160
MAX_WHEEL_MEMBER = 128 * 1024 * 1024
MAX_METADATA_MEMBER = 16 * 1024 * 1024
MAX_SELECTED_SOFTWARE = 256 * 1024 * 1024

# Filename suffixes are the selected release targets, not general wheel tags.
ROLES = {
    "linux-x86_64": r"(?:manylinux_2_17_x86_64\.manylinux2014_x86_64|manylinux2014_x86_64)",
    "linux-aarch64": r"(?:manylinux_2_17_aarch64\.manylinux2014_aarch64|manylinux2014_aarch64)",
    "macos-arm64": r"macosx_11_0_arm64",
}
EXPECTED_MEMBERS = {
    "umf_weft-0.4.56.dist-info/METADATA",
    "umf_weft-0.4.56.dist-info/WHEEL",
    "umf_weft-0.4.56.dist-info/RECORD",
    "umf_weft-0.4.56.dist-info/licenses/LICENSE.md",
    "umf_weft-0.4.56.dist-info/licenses/LICENSE-MIT",
    "umf_weft-0.4.56.dist-info/licenses/LICENSE-APACHE",
    "umf_weft-0.4.56.dist-info/licenses/THIRD-PARTY-LICENSES.txt",
    "weft/__init__.py",
    "weft/weft.abi3.so",
}
WRAPPER_SHA256 = "cd0fa27c9580742333d14e24d5ebfd74babb47d31720bb4c577278d05486f11d"
LICENSE_SHAS = {
    "LICENSE.md": "76dfb87aed22180562cee6180096117e659e66d130285c56a67d5f09a788e0e5",
    "LICENSE-MIT": "f1409ffd0dff471f9f5078c8f2cc2c070147daae3197812234b5919c1f820702",
    "LICENSE-APACHE": "a7f097f28b47176d49dddc086575d24d94149493f5b8e9bb829a0eee3eba74ee",
    "THIRD-PARTY-LICENSES.txt": "f86af39ab061e9b9474f8f3b151f69a42211d72fa9ac2930daecbda458c17f97",
}
SOURCE_IDENTITY = {
    "commit": PRODUCER,
    "tree": PRODUCER_TREE,
    "version": VERSION,
    "trackedCount": 5901,
    "fileRosterSha256": "187a8c5b00e46d38a7d25d7a497370414cafd0fd6adebb9377030e7128a7d3a1",
    "gitStageRosterSha256": "bc3018c0d674cd2fc80794f9f36c523565c91558f70649cd3fe415f8e88b701b",
    "selectedInputCount": 1459,
    "selectedInputRosterSha256": "6a671799d5be7a511cc6b5f7c9282d8745fbc80f4a5d796538b6643e8492f2a2",
}
METADATA_ONLY_PREFIXES = ("docs/helix/05-deploy/", "scripts/release/")
METADATA_ONLY_PATHS = {
    "docs/helix/05-deploy/README.md",
    "docs/helix/05-deploy/releases/v0.4.56.md",
    "docs/helix/05-deploy/local-release-runbook-0.4.56.md",
    "scripts/release/verify_local_release_profile.py",
    "scripts/release/publish_local_release.py",
    "scripts/release/test_verify_local_release_profile.py",
    "scripts/release/test_publish_local_release.py",
}
METADATA_SOURCE_ROOT = Path(__file__).resolve().parents[2]
METADATA_REVIEW_SCOPE = "producer-metadata-source"
# M is deliberately an input, not a constant in this reader: the verifier is
# itself part of M, so pinning M's own tree here would be self-referential.
# Its actual commit/tree/map/ancestry are closed and independently observed in
# the packet, then checked against P's complete build-input map below.
X64_BUILD_RESULT_SHA256 = "33bfe7e733b42fac5667bf8529afccb7c48dda255105ef9eea5152218a0def2b"
X64_AUDIT_RESULT_SHA256 = "2868c5c7bc298ccfeb96a9a8099ed22c1738f9bedf0a263acec7c6156352de5f"
X64_ARTIFACT_REVIEW_SHA256 = "c0b11e668096c3e95bbe581ddc2d1cb6749766eb636a30bed9ea71223921362e"
X64_STATIC_REPORT_SHA256 = "544d8617776cea6bb98179a82564e78f85f0793d279ae1d2910cd43af296bdf2"
X64_WHEEL_SHA256 = "32f4c38d9a614dd4c3a367f8b24d5cd4ba41ec80c2a4a2e3423e2082a294dbdd"
ARM_BUILD_RESULT_SHA256 = "dfb752976db4898ed56781ac345a6f757ce5a51064d0338c9049ff97dfea2861"
ARM_STATIC_REPORT_SHA256 = "bb35a92edfc16ee8a2debb744bd14a9936e921de07a8c11e0f0323352544927a"
ARM_ARTIFACT_REVIEW_SHA256 = "82cde49249876cae1f078c6842b839867655ce2c4c4d449cb3a7b89a0f172ad7"
ARM_WHEEL_SHA256 = "2de2156e283c34e45154f5d9fbc83a3eb1dd43fad497d787509d66cc96cfbe5a"
MAC_ARTIFACT_REVIEW_SHA256: str | None = None  # must be pinned after actual artifact acceptance
MAC_WHEEL_SHA256: str | None = None
MAC_BUILD_RESULT_SHA256: str | None = None
MAC_STATIC_REPORT_SHA256: str | None = None
EMPTY_SHA256 = hashlib.sha256(b"").hexdigest()
REQUIRED_GATES = (
    "linux-x86_64-final-installed-admission", "linux-x86_64-final-native-campaign",
    "linux-inherited-boundaries", "linux-browser", "linux-wasm",
    "linux-source-package", "linux-aarch64-build-static", "macos-arm64-build-static",
)
GATE_PROFILES = {
    "linux-x86_64-final-installed-admission": ("umf.g03-final-installed-prep-operator/1",
                                                "PASS_INSTALLED_SNAPSHOT_AND_COMPILER_ADMISSION_ONLY"),
    "linux-x86_64-final-native-campaign": (
        "umf.g03-full14-local-linux-operator/1", "FAIL_OR_UNKNOWN"),
    "linux-inherited-boundaries": (TABLE012_PROFILE, TABLE012_STATUS),
    "linux-browser": (TRUSS011_PROFILE, TRUSS011_STATUS),
    "linux-wasm": (TRUSS011_PROFILE, TRUSS011_STATUS),
    "linux-source-package": ("umf.g03-local-source-package-guards/2",
                              "LOCAL_BOUNDED_SOURCE_PACKAGE_GUARDS_PASS"),
    "linux-aarch64-build-static": ("umf.g03-final-baseline-build-static/1", "LOCAL_FINAL_BASELINE_BUILD_STATIC_PASS"),
    # This is a static-only local artifact chain. It does not qualify Mac
    # installation, imports, runtime behavior, browser parity, or hosted CI.
    "macos-arm64-build-static": ("weft-local-macos-static-audit-only/1",
                                   "LOCAL_MACOS_STATIC_AUDIT_ONLY_PASS"),
}
GATE_RECEIPT_SHA256: dict[str, str | None] = {name: None for name in REQUIRED_GATES}
GATE_RECEIPT_SHA256.update({
    "linux-inherited-boundaries": TABLE012_RECEIPT_SHA256,
    "linux-browser": TRUSS011_RECEIPT_SHA256,
    "linux-wasm": TRUSS011_RECEIPT_SHA256,
    "linux-x86_64-final-installed-admission":
        "5202e1b1567e7cfd7950b7e8205eace9d63f88bd67a3633743faef52be85be27",
    "linux-source-package": SOURCE_GUARD_RECEIPT_SHA256,
    "linux-aarch64-build-static": ARM_BUILD_RESULT_SHA256,
    "macos-arm64-build-static": "526cc38f854370f6bc34e044e9dc262b137bc1b4e95252c897578b5369c08fbf",
    "linux-x86_64-final-native-campaign": FINAL_NATIVE_OPERATOR,
})
GATE_HUMAN_REVIEW_SHA256: dict[str, str | None] = {name: None for name in REQUIRED_GATES}
GATE_HUMAN_REVIEW_SHA256.update({
    "linux-inherited-boundaries": TABLE012_REVIEW_SHA256,
    "linux-browser": TRUSS011_REVIEW_SHA256,
    "linux-wasm": TRUSS011_REVIEW_SHA256,
    "linux-x86_64-final-installed-admission":
        "0d02ff3cd7e697e07e354bf42b151992b7acf05d7f944f9d63332949b3dfe711",
    "linux-source-package": SOURCE_GUARD_REVIEW_SHA256,
    "linux-aarch64-build-static": ARM_ARTIFACT_REVIEW_SHA256,
    "macos-arm64-build-static":
        "7b38622a5d48e8c3f626ae0ca9fba8e2cc10f6666a51650e30b25d689cdbe7ee",
    "linux-x86_64-final-native-campaign": FINAL_NATIVE_REVIEW,
})
GATE_REVIEW_SHA256: dict[str, str | None] = {name: None for name in REQUIRED_GATES}
PRE_ASSEMBLY_REVIEW_SHA256: dict[str, str | None] = {
    "producer-metadata-source": None,
    "linux-x86_64-artifact": X64_ARTIFACT_REVIEW_SHA256,
    "linux-aarch64-static-artifact": ARM_ARTIFACT_REVIEW_SHA256,
    "macos-arm64-static-artifact":
        "7b38622a5d48e8c3f626ae0ca9fba8e2cc10f6666a51650e30b25d689cdbe7ee",
}
PRE_ASSEMBLY_HUMAN_REVIEW_SHA256 = {
    "linux-x86_64-artifact": X64_ARTIFACT_REVIEW_SHA256,
    "linux-aarch64-static-artifact": ARM_ARTIFACT_REVIEW_SHA256,
    "macos-arm64-static-artifact":
        "7b38622a5d48e8c3f626ae0ca9fba8e2cc10f6666a51650e30b25d689cdbe7ee",
}
MAC_BUILD_RESULT_SHA256 = "2aaa9dbe418cae86db3cac1f3c688a60d8b75746e31ec02badb1456941b663f0"
MAC_AUDIT_RESULT_SHA256 = "526cc38f854370f6bc34e044e9dc262b137bc1b4e95252c897578b5369c08fbf"
MAC_STATIC_REPORT_SHA256 = "2a79490c17a4856823a9eb9ea8db5cb8cf342f4c961705e8e3fe5f22f9b96424"
MAC_ARTIFACT_REVIEW_SHA256 = "7b38622a5d48e8c3f626ae0ca9fba8e2cc10f6666a51650e30b25d689cdbe7ee"
MAC_DATA_REVIEW_SHA256 = "fe720768de4141ab4514513921348926b6cd924a91e0733bcc9a2c791e284100"
MAC_WHEEL_SHA256 = "68f9297b4b55df483b86d3041040991c4dadfb9b4d63afcff1420c43f93ae41d"
MAC_WHEEL_BYTES = 11190603
MAC_NATIVE_SHA256 = "7f746042c88dde86b0a096b831a4243d40e2448255c1146cf515787b22c30225"
MAC_BUILD_RUNNER_SHA256 = "12518861a8ebf24e9cace3256f3c24e5d2d3d6e5ac6888dbc4143d583443c202"
MAC_BUILD_AUDIT_TOOL_SHA256 = "fbfbf1c0214e98d26987cc625084a1429ab578d9f48f546a5d4a4002803dbe0b"
MAC_AUDIT_RUNNER_SHA256 = "7657c0a79f34a67ddcdbe359ab1910a56642b1b95ee06af18881b77afbf3e4ce"
MAC_AUDIT_HELPER_SHA256 = "ff38aded4a48b285a7c0c6f1bc02def7b24fcd7890dd896a440ea38ebf833fb6"
MAC_PYTHON_PATH = "/Library/Developer/CommandLineTools/Library/Frameworks/Python3.framework/Versions/3.9/bin/python3.9"
MAC_PRODUCER_SOURCE_ROOT = Path("/private/tmp/weft-g03-ordinary-module")
MAC_PYTHON_BYTES = 53200
MAC_PYTHON_SHA256 = "6e7ae61f68a3838094fc56590f84c52069a97d7816f6bb79e0e85995b340e464"
X64_BUILD_ORIGINAL_ROOT = Path("/private/tmp/g03-final-baseline-amd64-002")
X64_AUDIT_ORIGINAL_ROOT = Path("/private/tmp/g03-final-baseline-amd64-audit-003")
ARM_BUILD_ORIGINAL_ROOT = Path("/private/tmp/g03-final-baseline-arm64-003")
MAC_BUILD_ORIGINAL_ROOT = Path("/private/tmp/g03-weft-macos-arm64-build-002")
MAC_AUDIT_ORIGINAL_ROOT = Path("/private/tmp/g03-weft-macos-arm64-audit-004")
MAC_BUILD_OPERATOR_ROOT = Path("/private/tmp/g03-macos-build-only-operator-002")
MAC_AUDIT_OPERATOR_ROOT = Path("/private/tmp/g03-macos-static-audit-only-operator-004")
MAC_CUSTODY_ROOT = Path("/private/tmp/g03-macos-source-custody-for-audit003-001")
MAC_ORIGINAL_BUILD_PROFILE = "weft-local-macos-arm64-build-static/1"
MAC_ORIGINAL_BUILD_STATUS = "FAIL_OR_UNKNOWN"
MAC_AUDIT_PROFILE = "weft-local-macos-static-audit-only/1"


class ProfileError(ValueError):
    pass


def _need(ok: bool, message: str) -> None:
    if not ok:
        raise ProfileError(message)


def _sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def _git_blob_sha1(data: bytes) -> str:
    return hashlib.sha1(b"blob " + str(len(data)).encode("ascii") + b"\0" + data).hexdigest()


def _closed(value: Any, keys: set[str], where: str) -> dict[str, Any]:
    _need(isinstance(value, dict), f"{where} must be an object")
    _need(set(value) == keys, f"{where} has an open or incomplete field set")
    return value


def _canonical_relative(raw: Any, where: str) -> str:
    _need(isinstance(raw, str) and raw and not raw.startswith("/") and "\\" not in raw,
          f"{where} relative path is malformed")
    _need(all(part not in {"", ".", ".."} for part in raw.split("/")),
          f"{where} relative path is not canonical")
    return raw


def _historical_relative(raw: Any, root: Path, where: str) -> str:
    """Translate a pinned historical absolute path lexically, without reading it."""
    _need(isinstance(raw, str) and raw.startswith("/") and "\\" not in raw,
          f"{where} historical path is not an absolute POSIX path")
    parts = raw.split("/")
    _need(parts[0] == "" and all(part not in {"", ".", ".."} for part in parts[1:]),
          f"{where} historical path is not canonical")
    root_text = root.as_posix()
    _need(root.is_absolute() and root_text.startswith("/")
          and "\\" not in root_text
          and all(part not in {"", ".", ".."} for part in root_text.split("/")[1:]),
          f"{where} accepted historical root is not canonical")
    prefix = root_text + "/"
    _need(raw.startswith(prefix), f"{where} historical path escapes its accepted root")
    relative = raw[len(prefix):]
    rel_parts = relative.split("/")
    _need(relative and all(part not in {"", ".", ".."} for part in rel_parts)
          and "/".join(root_text.split("/")[1:] + rel_parts) == raw[1:],
          f"{where} historical path is not a canonical descendant")
    return relative


def _read_bounded(path: Path, cap: int, where: str) -> bytes:
    _need(not path.is_symlink(), f"{where} is a symlink")
    try:
        st = path.stat(follow_symlinks=False)
    except OSError as exc:
        raise ProfileError(f"{where} cannot be statted") from exc
    _need(stat.S_ISREG(st.st_mode), f"{where} is not a regular file")
    _need(0 <= st.st_size <= cap, f"{where} exceeds its byte cap")
    chunks: list[bytes] = []
    size = 0
    try:
        with path.open("rb") as stream:
            while True:
                chunk = stream.read(1024 * 1024)
                if not chunk:
                    break
                size += len(chunk)
                _need(size <= cap, f"{where} exceeds its byte cap while reading")
                chunks.append(chunk)
        after = path.stat(follow_symlinks=False)
    except OSError as exc:
        raise ProfileError(f"{where} could not be read consistently") from exc
    _need((after.st_dev, after.st_ino, after.st_size) == (st.st_dev, st.st_ino, size),
          f"{where} changed while being read")
    return b"".join(chunks)


def _regular(path: Path, cap: int, where: str) -> tuple[int, str]:
    data = _read_bounded(path, cap, where)
    return len(data), _sha(data)


def _under(root: Path, raw: Any, where: str) -> Path:
    _canonical_relative(raw, f"{where} path")
    rel = Path(raw)
    path = root / rel
    current = root
    for part in rel.parts:
        current = current / part
        _need(not current.is_symlink(), f"{where} path contains a symlink")
    _need(path.resolve(strict=True).is_relative_to(root.resolve(strict=True)),
          f"{where} path escapes its packet root")
    return path


def _receipt(root: Path, row: Any, label: str) -> tuple[Path, bytes, str]:
    item = _closed(row, {"path", "sizeBytes", "sha256"}, label)
    path = _under(root, item["path"], label)
    data = _read_bounded(path, MAX_EVIDENCE_FILE, label)
    size, digest = len(data), _sha(data)
    _need(type(item["sizeBytes"]) is int and item["sizeBytes"] == size,
          f"{label} size differs from its receipt")
    _need(isinstance(item["sha256"], str) and SHA.fullmatch(item["sha256"])
          and item["sha256"] == digest, f"{label} digest differs from its receipt")
    return path, data, digest


def _json_receipt(root: Path, row: Any, label: str) -> tuple[dict[str, Any], Path, str, int]:
    path, raw, digest = _receipt(root, row, label)
    try:
        value = json.loads(raw)
    except (UnicodeError, json.JSONDecodeError) as exc:
        raise ProfileError(f"{label} is not valid JSON") from exc
    _need(isinstance(value, dict), f"{label} must contain an object")
    return value, path, digest, len(raw)


def _selected_rows_for_gate(name: str, receipt: dict[str, Any]) -> tuple[Path, dict[str, tuple[int, str]], list[dict[str, Any]]]:
    """Return the two finite, pinned external-software classes for Table012/Truss011."""
    if name == "linux-inherited-boundaries":
        root, expected, actual = TABLE012_SOFTWARE_ROOT, TABLE012_SELECTED, receipt.get("selectedArtifacts")
        expected_bytes = TABLE012_SELECTED_BYTES
        receipt_sha = TABLE012_RECEIPT_SHA256
        kind = "table012-selected-artifact"
    elif name in {"linux-browser", "linux-wasm"}:
        root, expected, actual = TRUSS011_SOFTWARE_ROOT, TRUSS011_SELECTED, receipt.get("outputFiles")
        expected_bytes = TRUSS011_SELECTED_BYTES
        receipt_sha = TRUSS011_RECEIPT_SHA256
        kind = "truss011-selected-runtime-software"
    else:
        raise ProfileError(f"{name} has no finite selected-software class")
    _need(isinstance(actual, list), f"{name} selected-software roster is absent")
    actual_selected: dict[str, tuple[int, str]] = {}
    for raw in actual:
        row = _closed(raw, {"path", "sha256", "sizeBytes"}, f"{name} selected/output row")
        _need(isinstance(row["path"], str) and row["path"] not in actual_selected
              and type(row["sizeBytes"]) is int and row["sizeBytes"] >= 0
              and isinstance(row["sha256"], str) and SHA.fullmatch(row["sha256"]),
              f"{name} selected/output row is malformed or duplicated")
        actual_selected[row["path"]] = (row["sizeBytes"], row["sha256"])
    selected = {path: actual_selected[path] for path in expected if path in actual_selected}
    _need(selected == expected and sum(size for size, _ in selected.values()) == expected_bytes,
          f"{name} selected-software identities differ from the independently pinned roster")
    witnesses = []
    for relative, (size, digest) in sorted(expected.items()):
        _need(not Path(relative).is_absolute() and all(part not in {"", ".", ".."}
              for part in Path(relative).parts), f"{name} selected-software path is unsafe")
        external = root / relative
        witnesses.append({"scope": "external selected runtime software",
                          "kind": kind, "sourceRoot": str(root),
                          "sourcePath": relative, "receiptSha256": receipt_sha,
                          "sizeBytes": size, "sha256": digest})
        try:
            st = external.stat(follow_symlinks=False)
        except OSError as exc:
            raise ProfileError(f"{name} selected software is unavailable: {relative}") from exc
        _need(stat.S_ISREG(st.st_mode) and st.st_size == size and not external.is_symlink(),
              f"{name} selected software is not the pinned regular file: {relative}")
    return root, expected, witnesses


def _verify_selected_software(name: str, receipt: dict[str, Any], *, verify: bool) -> list[dict[str, Any]]:
    """Hash pinned external selected software twice around a stable-stat check."""
    root, expected, witnesses = _selected_rows_for_gate(name, receipt)
    if not verify:
        return witnesses
    _need(not root.is_symlink(), f"{name} selected-software root is a symlink")
    canonical_root = root.resolve(strict=True)
    for relative, (size, digest) in expected.items():
        path = root / relative
        current = root
        for part in Path(relative).parts:
            current = current / part
            _need(not current.is_symlink(),
                  f"{name} selected software path contains a symlink: {relative}")
        _need(path.resolve(strict=True).is_relative_to(canonical_root),
              f"{name} selected software escapes its pinned root: {relative}")
        before = path.stat(follow_symlinks=False)
        before_id = (before.st_dev, before.st_ino, before.st_size, before.st_mtime_ns)
        first = _read_bounded(path, MAX_SELECTED_SOFTWARE, f"{name} external selected software")
        middle = path.stat(follow_symlinks=False)
        second = _read_bounded(path, MAX_SELECTED_SOFTWARE, f"{name} external selected software recheck")
        after = path.stat(follow_symlinks=False)
        after_id = (after.st_dev, after.st_ino, after.st_size, after.st_mtime_ns)
        _need(before_id == after_id == (middle.st_dev, middle.st_ino, middle.st_size,
                                        middle.st_mtime_ns)
              and stat.S_ISREG(after.st_mode) and len(first) == len(second) == size
              and _sha(first) == _sha(second) == digest,
              f"{name} external selected software changed or differs: {relative}")
    return witnesses


def _closed_operator_output_rows(root: Path, receipt_path: Path, rows: Any, *,
                                 where: str, expected_count: int,
                                 expected_total: int, verify: bool) -> list[dict[str, Any]]:
    _need(isinstance(rows, list) and len(rows) == expected_count,
          f"{where} complete public metadata roster differs")
    base = receipt_path.parent
    packet_root = root.resolve(strict=True)
    result: list[dict[str, Any]] = []
    seen: set[str] = set()
    total = 0
    for raw in rows:
        item = _closed(raw, {"path", "sha256", "sizeBytes"}, f"{where} public metadata row")
        rel = item["path"]
        _need(isinstance(rel, str) and rel and "\\" not in rel,
              f"{where} public metadata path is malformed")
        _canonical_relative(rel, f"{where} public metadata path")
        relative = Path(rel)
        output = base / relative
        current = base
        for part in relative.parts:
            current = current / part
            _need(not current.is_symlink(), f"{where} public metadata contains a symlink")
        resolved = output.resolve(strict=True)
        _need(resolved.is_relative_to(packet_root), f"{where} public metadata escapes packet evidence")
        packet_relative = resolved.relative_to(packet_root).as_posix()
        _need(packet_relative not in seen and type(item["sizeBytes"]) is int
              and 0 <= item["sizeBytes"] <= MAX_EVIDENCE_FILE
              and isinstance(item["sha256"], str) and SHA.fullmatch(item["sha256"]),
              f"{where} public metadata row is duplicated or malformed")
        seen.add(packet_relative)
        try:
            st = output.stat(follow_symlinks=False)
        except OSError as exc:
            raise ProfileError(f"{where} public metadata file is unavailable") from exc
        _need(stat.S_ISREG(st.st_mode) and st.st_size == item["sizeBytes"],
              f"{where} public metadata size/type differs")
        if verify:
            actual_size, actual_sha = _regular(output, MAX_EVIDENCE_FILE,
                                               f"{where} public metadata")
            _need(actual_size == item["sizeBytes"] and actual_sha == item["sha256"],
                  f"{where} public metadata bytes differ")
        total += st.st_size
        result.append({"path": packet_relative, "sizeBytes": st.st_size,
                       "sha256": item["sha256"]})
    _need(total == expected_total, f"{where} public metadata byte total differs")
    prefix = receipt_path.parent.resolve().relative_to(packet_root).as_posix()
    expected_rel = {Path(row["path"]).relative_to(prefix).as_posix()
                    if prefix != "." else Path(row["path"]).as_posix()
                    for row in result}
    expected_rel.add(receipt_path.name)
    actual_rel: set[str] = set()
    for directory, subdirs, filenames in os.walk(base, followlinks=False):
        current_dir = Path(directory)
        _need(not current_dir.is_symlink()
              and all(not (current_dir / subdir).is_symlink() for subdir in subdirs),
              f"{where} copied public metadata tree contains a symlink")
        for filename in filenames:
            child = current_dir / filename
            _need(not child.is_symlink(), f"{where} copied public metadata tree contains a symlink")
            actual_rel.add(child.relative_to(base).as_posix())
    _need(actual_rel == expected_rel,
          f"{where} packet copy has an omitted or extra public metadata file")
    return sorted(result, key=lambda row: row["path"])


def _crossbind_capture(path_root: Path, row_by_relative: dict[str, dict[str, Any]],
                       stream: Any, *, where: str, guest: bool = False) -> None:
    _need(isinstance(stream, dict) and stream.get("physicalClosed") is True
          and stream.get("truncated") is False
          and type(stream.get("rawBytes")) is int
          and stream.get("bytes") == stream.get("rawBytes")
          and stream.get("sha256") == stream.get("rawSha256"),
          f"{where} capture stream is not fully retained")
    raw_path = stream.get("path")
    _need(isinstance(raw_path, str) and raw_path, f"{where} capture path is absent")
    if guest:
        prefix = Path("/out")
        absolute = Path(raw_path)
        _need(absolute.is_absolute() and absolute.is_relative_to(prefix),
              f"{where} guest capture path is outside /out")
        relative = (Path("owned/out") / absolute.relative_to(prefix)).as_posix()
    else:
        relative = _historical_relative(raw_path, path_root, where)
    row = row_by_relative.get(relative)
    _need(row is not None and row["sizeBytes"] == stream["rawBytes"]
          and row["sha256"] == stream["rawSha256"],
          f"{where} capture does not match the physically closed metadata roster")


def _table012_output_rows(root: Path, receipt_path: Path, receipt: dict[str, Any], *,
                          verify: bool) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    _need(receipt.get("profile") == TABLE012_PROFILE
          and receipt.get("status") == TABLE012_STATUS
          and receipt.get("executionId") == TABLE012_EXECUTION_ID
          and receipt.get("operatorResultBytes") == TABLE012_RECEIPT_BYTES
          and receipt.get("publicBytes") == TABLE012_PUBLIC_BYTES
          and receipt.get("selectedArtifactBytes") == TABLE012_SELECTED_BYTES,
          "Table012 original receipt identity/totals differ")
    public = _closed_operator_output_rows(root, receipt_path, receipt.get("publicFiles"),
                                          where="Table012", expected_count=TABLE012_PUBLIC_COUNT,
                                          expected_total=TABLE012_PUBLIC_BYTES, verify=verify)
    selected_rows = receipt.get("selectedArtifacts")
    _need(isinstance(selected_rows, list) and len(selected_rows) == len(TABLE012_SELECTED),
          "Table012 selected-artifact roster differs")
    selected_map = {row.get("path"): (row.get("sizeBytes"), row.get("sha256"))
                    for row in selected_rows if isinstance(row, dict)}
    _need(selected_map == TABLE012_SELECTED,
          "Table012 selected-artifact roster was omitted, expanded, or reclassified")
    software = _verify_selected_software("linux-inherited-boundaries", receipt, verify=verify)
    return public, software


def _truss011_output_rows(root: Path, receipt_path: Path, receipt: dict[str, Any], *,
                          verify: bool) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    _need(receipt.get("profile") == TRUSS011_PROFILE
          and receipt.get("status") == TRUSS011_STATUS
          and receipt.get("executionId") == TRUSS011_EXECUTION_ID
          and receipt.get("outputBytes") == TRUSS011_OUTPUT_BYTES,
          "Truss011 original receipt identity/totals differ")
    rows = receipt.get("outputFiles")
    _need(isinstance(rows, list) and len(rows) == 133,
          "Truss011 complete output roster differs")
    selected_names = set(TRUSS011_SELECTED)
    actual_paths = [row.get("path") for row in rows if isinstance(row, dict)]
    _need(len(actual_paths) == 133 and len(set(actual_paths)) == 133
          and {path for path in actual_paths if path in selected_names} == selected_names,
          "Truss011 selected software and metadata paths are not a disjoint complete roster")
    metadata = [row for row in rows if row.get("path") not in selected_names]
    _need(len(metadata) == 128
          and sum(row.get("sizeBytes", -1) for row in metadata) == TRUSS011_METADATA_BYTES
          and sum(row.get("sizeBytes", -1) for row in rows) == TRUSS011_OUTPUT_BYTES,
          "Truss011 metadata/software byte classes differ")
    public = _closed_operator_output_rows(root, receipt_path, metadata,
                                          where="Truss011", expected_count=128,
                                          expected_total=TRUSS011_METADATA_BYTES,
                                          verify=verify)
    software = _verify_selected_software("linux-browser", receipt, verify=verify)
    return public, software


def _preflight_packet_payloads(root: Path, packet: dict[str, Any], packet_size: int) -> None:
    """Stat the complete declared family before hashing/decompressing payloads."""
    direct: list[tuple[str, Path, int, bool]] = []
    output_candidates: list[tuple[str, Path, int, bool]] = []
    forced_log_paths: set[str] = set()

    def add(ref: Any, label: str, *, software: bool = False) -> str:
        item = _closed(ref, {"path", "sizeBytes", "sha256"}, label)
        path = _under(root, item["path"], label)
        try:
            st = path.stat(follow_symlinks=False)
        except OSError as exc:
            raise ProfileError(f"{label} cannot be statted") from exc
        cap = MAX_WHEEL if software else MAX_EVIDENCE_FILE
        _need(stat.S_ISREG(st.st_mode) and type(item["sizeBytes"]) is int
              and st.st_size == item["sizeBytes"] and 0 <= st.st_size <= cap
              and isinstance(item["sha256"], str) and SHA.fullmatch(item["sha256"]),
              f"{label} is not a bounded regular input")
        logical = Path(item["path"]).as_posix()
        direct.append((logical, path, st.st_size, software))
        return logical

    add(packet["producer"]["sourceMap"], "producer source map")
    metadata = _closed(packet["metadata"], {"sourceMap", "ancestry", "files"}, "metadata source")
    add(metadata["sourceMap"], "metadata source map")
    _need(isinstance(metadata["files"], list), "metadata source file roster is malformed")
    for index, row in enumerate(metadata["files"]):
        item = _closed(row, {"sourcePath", "path", "sizeBytes", "sha256"},
                       f"metadata source file {index}")
        add({key: item[key] for key in ("path", "sizeBytes", "sha256")},
            f"metadata source file {index}")
    add(packet["metadata"]["ancestry"], "metadata ancestry")
    for name in REQUIRED_GATES:
        if name == "macos-arm64-build-static":
            gate_keys = {"receipt", "review", "dataReview"}
        elif name == "linux-x86_64-final-native-campaign":
            gate_keys = {"receipt", "review", "reanalysis"}
        else:
            gate_keys = {"receipt", "review"}
        gate = _closed(packet["gates"][name], gate_keys, f"gate {name}")
        add(gate["receipt"], f"gate {name} receipt")
        add(gate["review"], f"gate {name} review")
        if name == "macos-arm64-build-static":
            add(gate["dataReview"], f"gate {name} data review")
        elif name == "linux-x86_64-final-native-campaign":
            add(gate["reanalysis"], f"gate {name} predicate reanalysis")
    _need(isinstance(packet["preAssemblyReviews"], list),
          "pre-assembly review roster is malformed")
    for index, ref in enumerate(packet["preAssemblyReviews"]):
        item = _closed(ref, {"scope", "path", "sizeBytes", "sha256"},
                       f"pre-assembly review {index}")
        add({key: item[key] for key in ("path", "sizeBytes", "sha256")},
            f"pre-assembly review {index}")
    _need(isinstance(packet["wheels"], list) and len(packet["wheels"]) == 3,
          "wheel roster is malformed")
    for index, row in enumerate(packet["wheels"]):
        item = _closed(row, {"role", "path", "filename", "sizeBytes", "sha256", "staticReview",
                             "buildReceipt", "auditReceipt", "artifactReview", "producerEvidence"}, f"wheel row {index}")
        for key in ("staticReview", "buildReceipt", "auditReceipt", "artifactReview"):
            add(item[key], f"wheel {index} {key}")
        _need(isinstance(item["producerEvidence"], list) and item["producerEvidence"],
              f"wheel {index} producer evidence family is absent")
        for evidence_index, evidence_row in enumerate(item["producerEvidence"]):
            evidence = _closed(evidence_row, {"sourcePath", "path", "sizeBytes", "sha256"},
                               f"wheel {index} producer evidence {evidence_index}")
            _need(isinstance(evidence["sourcePath"], str) and evidence["sourcePath"],
                  "producer evidence source path is absent")
            logical = add({key: evidence[key] for key in ("path", "sizeBytes", "sha256")},
                          f"wheel {index} producer evidence {evidence_index}")
            source_name = Path(evidence["sourcePath"])
            if ("logs" in source_name.parts
                    or source_name.name.endswith((".log", ".stdout", ".stderr", ".bin"))):
                forced_log_paths.add(logical)
        add({"path": item["path"], "sizeBytes": item["sizeBytes"], "sha256": item["sha256"]},
            f"wheel {index} bytes", software=True)

    unique: dict[str, tuple[int, bool]] = {}
    for logical, _, size, software in direct:
        previous = unique.get(logical)
        if previous is not None:
            _need(previous == (size, software), "input path has conflicting packet identities")
        unique[logical] = (size, software)
    evidence_total = packet_size + sum(size for size, software in unique.values() if not software)
    wheel_total = sum(size for size, software in unique.values() if software)
    _need(evidence_total <= MAX_EVIDENCE_TOTAL and wheel_total <= MAX_WHEEL_FAMILY,
          "declared evidence or wheel family exceeds its cap before payload reads")

    # Operator summaries contain their retained-file roster. Their bounded
    # JSON has now been preflighted with the other metadata; stat every output
    # before any of those outputs are hashed or copied.
    output_seen: dict[str, int] = {}
    for name in REQUIRED_GATES:
        ref = packet["gates"][name]["receipt"]
        summary_path = _under(root, ref["path"], f"gate {name} receipt")
        try:
            summary_raw = _read_bounded(summary_path, MAX_EVIDENCE_FILE, f"gate {name} receipt")
            summary = json.loads(summary_raw)
        except (OSError, UnicodeError, json.JSONDecodeError) as exc:
            raise ProfileError(f"gate {name} receipt cannot supply its output roster") from exc
        if name == "linux-source-package" and isinstance(summary, dict):
            terminal = summary.get("terminalArtifacts")
            _need(isinstance(terminal, list),
                  "source/package gate omits its terminal artifact roster")
            outputs = []
            for row in terminal:
                item = _closed(row, {"bytes", "path", "sha256", "type"},
                               "source/package terminal artifact")
                _need(item["type"] == "file", "source/package artifact is not a file")
                outputs.append({"path": item["path"], "sizeBytes": item["bytes"],
                                "sha256": item["sha256"]})
        elif name == "linux-inherited-boundaries":
            outputs, _ = _table012_output_rows(root, summary_path, summary, verify=False)
        elif name in {"linux-browser", "linux-wasm"}:
            outputs, _ = _truss011_output_rows(root, summary_path, summary, verify=False)
        elif name == "linux-aarch64-build-static":
            outputs = _arm_selected_output_rows(summary, summary_path, verify=False)
        elif name == "macos-arm64-build-static":
            outputs = _mac_selected_output_rows(summary, summary_path, verify=False)
        elif (name == "linux-x86_64-final-native-campaign"
              and isinstance(summary, dict)
              and summary.get("profile") == "umf.g03-full14-local-linux-operator/1"):
            outputs = _native_campaign_output_rows(summary, summary_path)
        else:
            outputs = summary.get("outputFiles") if isinstance(summary, dict) else None
        _need(isinstance(outputs, list) and outputs,
              f"gate {name} omits its physical output/log roster")
        for row in outputs:
            item = _closed(row, {"path", "sha256", "sizeBytes"}, f"gate {name} output file")
            rel = _canonical_relative(item["path"], f"gate {name} output")
            relative = Path(rel)
            # The Table012/Truss011 native-roster adapters already return
            # packet-root-relative paths. Other operator output adapters retain
            # the native receipt-relative path convention.
            packet_root_relative = name in {
                "linux-inherited-boundaries", "linux-browser", "linux-wasm"}
            source_base = root if packet_root_relative else summary_path.parent
            path = source_base / relative
            current = source_base
            for part in relative.parts:
                current = current / part
                _need(not current.is_symlink(), f"gate {name} output path contains a symlink")
            try:
                st = path.stat(follow_symlinks=False)
            except OSError as exc:
                raise ProfileError(f"gate {name} output cannot be statted") from exc
            _need(stat.S_ISREG(st.st_mode) and type(item["sizeBytes"]) is int
                  and 0 <= st.st_size == item["sizeBytes"] <= MAX_EVIDENCE_FILE
                  and isinstance(item["sha256"], str) and SHA.fullmatch(item["sha256"]),
                  f"gate {name} output is not a bounded regular file")
            packet_relative = path.resolve(strict=True).relative_to(root.resolve(strict=True)).as_posix()
            identity = (st.st_size, item["sha256"])
            prior = output_seen.get(packet_relative)
            _need(prior is None or prior == identity,
                  "operator output path has conflicting selected identities")
            output_seen[packet_relative] = identity
            output_candidates.append((packet_relative, path, st.st_size, False))
    # Enforce caps over the deduplicated union of packet inputs and each gate's
    # retained outputs before the later validator hashes or copies payloads.
    for logical, path, size, software in output_candidates:
        prior = unique.get(logical)
        identity = (size, software)
        _need(prior is None or prior == identity,
              "gate output conflicts with an already selected packet input")
        unique[logical] = identity
    evidence_total = packet_size + sum(size for size, software in unique.values() if not software)
    wheel_total = sum(size for size, software in unique.values() if software)
    _need(evidence_total <= MAX_EVIDENCE_TOTAL and wheel_total <= MAX_WHEEL_FAMILY,
          "complete evidence or software family exceeds its cap before output reads")
    log_total = 0
    for logical, (size, software) in unique.items():
        logical_path = Path(logical)
        if not software and (logical in forced_log_paths or "logs" in logical_path.parts
                or logical_path.name.endswith((".log", ".stdout", ".stderr", ".bin"))):
            log_total += size
    _need(log_total <= MAX_RAW_LOGS,
          "complete retained producer/gate raw logs exceed the 1 MiB cap before reads")


def _source_map_identity(value: Any, role: str) -> dict[str, Any]:
    source = _closed(value, {"commit", "tree", "status", "trackedCount",
                              "trackedPathSha256", "gitStageRosterSha256",
                              "selectedInputCount", "selectedInputRosterSha256", "version"},
                     f"{role} source identity")
    _need(isinstance(source["commit"], str) and COMMIT.fullmatch(source["commit"])
          and isinstance(source["tree"], str) and COMMIT.fullmatch(source["tree"])
          and source["status"] == "" and source["version"] == VERSION,
          f"{role} source identity is malformed or dirty")
    return source


_SOURCEGUARD_SELECTED_PREFIXES = (
    "crates/", "spec/", "vendor/", ".cargo/", "tests/",
    "docs/helix/02-design/contracts/", "docs/helix/03-test/fixtures/",
    "spikes/b001/fixtures/",
)


def _selected_rows_from_source_guard(root: Path, ref: Any, role: str
                                    ) -> tuple[list[dict[str, Any]], list[str]]:
    guard, _, _, _ = _json_receipt(root, ref, f"{role} pinned source-guard receipt")
    actual_rows = guard.get("source", {}).get("ownedCopyAfter", {}).get("files")
    _need(isinstance(actual_rows, list) and len(actual_rows) == 5901,
          f"{role} source guard lacks the exact complete P physical roster")
    paths: list[str] = []
    selected: list[dict[str, Any]] = []
    previous = ""
    for row in actual_rows:
        item = _closed(row, {"bytes", "linkCount", "mode", "path", "sha256"},
                       f"{role} physical P source file")
        _need(isinstance(item["path"], str) and item["path"] > previous
              and "\\" not in item["path"]
              and all(part not in {"", ".", ".."} for part in Path(item["path"]).parts)
              and type(item["bytes"]) is int and item["bytes"] >= 0
              and type(item["linkCount"]) is int and item["linkCount"] == 1
              and type(item["mode"]) is int and item["mode"] in {0o664, 0o775}
              and isinstance(item["sha256"], str) and SHA.fullmatch(item["sha256"]),
              f"{role} physical P source roster is malformed or unordered")
        paths.append(item["path"])
        if "/" not in item["path"] or item["path"].startswith(_SOURCEGUARD_SELECTED_PREFIXES):
            selected.append({"path": item["path"], "bytes": item["bytes"],
                             "sha256": item["sha256"]})
        previous = item["path"]
    _need(len(selected) == SOURCE_GUARD_SELECTED_COUNT
          and sum(row["bytes"] for row in selected) == SOURCE_GUARD_SELECTED_BYTES
          and _sha((json.dumps(selected, sort_keys=True) + "\n").encode()) ==
          SOURCE_GUARD_SELECTED_ROSTER_SHA256,
          f"{role} fixed source-guard selection differs from the accepted roster")
    return selected, paths


def _validate_source_map(root: Path, ref: Any, role: str, *,
                         source_guard_ref: Any = None,
                         producer_map: dict[str, Any] | None = None) -> tuple[dict[str, Any], dict[str, str]]:
    value, _, _, _ = _json_receipt(root, ref, f"{role} source map")
    if isinstance(value, dict) and value.get("profile") in {
            "weft-local-release-source-map/2", "weft-local-release-source-map-delta/2"}:
        is_delta = value["profile"] == "weft-local-release-source-map-delta/2"
        if is_delta:
            _closed(value, {"profile", "source", "base", "changes", "selectedInputsFrom"},
                    "metadata M delta map")
            _need(role == "metadata M", "producer P cannot use a source-map delta")
        else:
            _closed(value, {"profile", "source", "stageRows", "selectedInputsFrom"},
                    f"{role} source map v2")
            _need(role == "producer P", "metadata M requires a one-hop source-map delta")
        _need(source_guard_ref is not None and value["selectedInputsFrom"] == source_guard_ref,
              f"{role} selected-input sourceguard reference differs from the packet gate receipt")
        selected, source_paths = _selected_rows_from_source_guard(root, source_guard_ref, role)
        source = _source_map_identity(value["source"], role)
        if is_delta:
            _need(producer_map is not None
                  and producer_map.get("profile") == "weft-local-release-source-map/2"
                  and "_packetRef" in producer_map,
                  "metadata M delta has no validated producer base")
            stage_rows = []
        else:
            _closed(value, {"profile", "source", "stageRows", "selectedInputsFrom"},
                    f"{role} source map v2")
            stage_rows = value["stageRows"]
            _need(isinstance(stage_rows, list) and type(source["trackedCount"]) is int
                  and len(stage_rows) == source["trackedCount"] <= 10000,
                  f"{role} v2 stage-row roster is incomplete or exceeds its bound")
        tracked_map: dict[str, str] = {}
        stage_map: dict[str, tuple[str, str]] = {}
        tracked_paths: list[str] = []
        previous_path = ""
        for line in stage_rows:
            _need(isinstance(line, str) and "\\" not in line,
                  f"{role} git index stage row is malformed")
            left, separator, path = line.partition("\t")
            fields = left.split(" ")
            _need(separator == "\t" and len(fields) == 3
                  and fields[0] in {"100644", "100755", "120000"}
                  and re.fullmatch(r"[0-9a-f]{40}", fields[1]) is not None
                  and fields[2] == "0" and path and path == Path(path).as_posix()
                  and not Path(path).is_absolute()
                  and all(part not in {"", ".", ".."} for part in Path(path).parts)
                  and "\n" not in path and "\r" not in path and "\t" not in path
                  and path > previous_path,
                  f"{role} git index stage row is malformed or unordered")
            tracked_paths.append(path)
            tracked_map[path] = fields[1]
            stage_map[path] = (fields[0], fields[1])
            previous_path = path
        if not is_delta:
            _need(tracked_paths == source_paths,
                  f"{role} full stage roster differs from sourceguard physical P roster")
            _need(_sha(("\n".join(tracked_paths) + "\n").encode()) == source["trackedPathSha256"]
                  and _sha("\n".join(stage_rows).encode()) == source["gitStageRosterSha256"],
                  f"{role} v2 stage/path digest differs")
        _need(source["selectedInputCount"] == len(selected)
              and source["selectedInputRosterSha256"] ==
              _sha((json.dumps(selected, sort_keys=True) + "\n").encode()),
              f"{role} selected build-input digest differs from the fixed sourceguard projection")
        if is_delta:
            _need(type(source["trackedCount"]) is int and 0 < source["trackedCount"] <= 10000,
                  "metadata M tracked count is outside the bounded roster range")
            base_ref = value.get("base")
            changes = value.get("changes")
            _need(isinstance(base_ref, dict)
                  and set(base_ref) == {"path", "sizeBytes", "sha256"}
                  and base_ref.get("path") == producer_map["_packetRef"]["path"]
                  and base_ref.get("sizeBytes") == producer_map["_packetRef"]["sizeBytes"]
                  and base_ref.get("sha256") == producer_map["_packetRef"]["sha256"],
                  "metadata M delta base is not the exact validated P full map")
            p_rows = {line.split("\t", 1)[1]: line for line in producer_map["stageRows"]}
            _need(isinstance(changes, list) and len(changes) == 7,
                  "metadata M delta does not contain the exact seven-path roster")
            prior = ""
            for row in changes:
                item = _closed(row, {"path", "oldStageRow", "newStageRow"},
                               "metadata M delta row")
                path = item["path"]
                _need(isinstance(path, str) and path > prior and path in METADATA_ONLY_PATHS
                      and item["oldStageRow"] == p_rows.get(path)
                      and item["oldStageRow"] != item["newStageRow"],
                      "metadata M delta old row, order, or path is invalid")
                new_line = item["newStageRow"]
                if new_line is not None:
                    _need(isinstance(new_line, str) and new_line.endswith("\t" + path),
                          "metadata M delta new row path differs")
                    p_rows[path] = new_line
                else:
                    _need(path in p_rows, "metadata M delta null row claims a missing path")
                    del p_rows[path]
                prior = path
            stage_rows = [p_rows[path] for path in sorted(p_rows)]
            tracked_paths = [line.split("\t", 1)[1] for line in stage_rows]
            _need(len(stage_rows) == source["trackedCount"]
                  and _sha(("\n".join(tracked_paths) + "\n").encode()) == source["trackedPathSha256"]
                  and _sha(("\n".join(stage_rows)).encode()) == source["gitStageRosterSha256"],
                  "metadata M reconstructed stage/path roster differs from its source identity")
            _need(all(isinstance(line, str) and "\\" not in line
                      and len(line.partition("\t")[0].split(" ")) == 3
                      and line.partition("\t")[0].split(" ")[0] in {"100644", "100755", "120000"}
                      and re.fullmatch(r"[0-9a-f]{40}", line.partition("\t")[0].split(" ")[1])
                      and line.partition("\t")[0].split(" ")[2] == "0"
                      and line.partition("\t")[1] == "\t"
                      and line.partition("\t")[2] == path
                      for path, line in zip(tracked_paths, stage_rows, strict=True)),
                  "metadata M reconstructed stage row is malformed")
            tracked_map = {line.split("\t", 1)[1]: line.split(" ")[1]
                           for line in stage_rows}
            stage_map = {line.split("\t", 1)[1]: (line.split(" ")[0], line.split(" ")[1])
                         for line in stage_rows}
        _need(all(row["path"] in tracked_map for row in selected),
              f"{role} selected sourceguard input is absent from the stage roster")
        logical = {**value, "trackedPaths": tracked_paths,
                   "trackedFiles": [{"path": path, "mode": pair[0], "gitBlobSha1": pair[1]}
                                    for path, pair in sorted(stage_map.items())],
                   "selectedBuildInputs": selected, "stageRows": stage_rows,
                   "_packetRef": ref}
        return logical, tracked_map
    _closed(value, {"profile", "source", "trackedPaths", "stageRows", "trackedFiles",
                    "selectedBuildInputs"}, f"{role} source map")
    _need(value["profile"] == "weft-local-release-source-map/1",
          f"{role} source map profile differs")
    source = _source_map_identity(value["source"], role)
    _need(isinstance(source["commit"], str) and COMMIT.fullmatch(source["commit"])
          and isinstance(source["tree"], str) and COMMIT.fullmatch(source["tree"])
          and source["status"] == "" and source["version"] == VERSION,
          f"{role} source identity is malformed or dirty")
    tracked = value["trackedPaths"]
    _need(isinstance(tracked, list) and all(isinstance(path, str) for path in tracked)
          and tracked == sorted(set(tracked))
          and type(source["trackedCount"]) is int and len(tracked) == source["trackedCount"],
          f"{role} complete tracked-path roster is malformed")
    tracked_digest = _sha(("\n".join(tracked) + "\n").encode("utf-8"))
    _need(source["trackedPathSha256"] == tracked_digest,
          f"{role} tracked-path roster digest differs")
    stage_rows = value["stageRows"]
    _need(isinstance(stage_rows, list) and len(stage_rows) == len(tracked)
          and all(isinstance(line, str) and "\t" in line for line in stage_rows),
          f"{role} git index stage roster is incomplete")
    # The pinned git ls-files --stage asset has no final newline. Preserve its
    # exact raw row encoding: mode SP blob SP stage TAB path.
    stage_digest = _sha("\n".join(stage_rows).encode("utf-8"))
    _need(stage_digest == source["gitStageRosterSha256"],
          f"{role} git stage roster digest differs")
    files = value["trackedFiles"]
    _need(isinstance(files, list) and len(files) == len(tracked),
          f"{role} tracked source map is incomplete")
    tracked_map: dict[str, str] = {}
    for row in files:
        item = _closed(row, {"path", "mode", "gitBlobSha1"}, f"{role} tracked source row")
        _need(item["path"] in tracked and item["path"] not in tracked_map
              and item["mode"] in {"100644", "100755", "120000"}
              and isinstance(item["gitBlobSha1"], str)
              and re.fullmatch(r"[0-9a-f]{40}", item["gitBlobSha1"]),
              f"{role} tracked source row is invalid")
        tracked_map[item["path"]] = item["gitBlobSha1"]
    _need(set(tracked_map) == set(tracked), f"{role} tracked source rows do not close")
    stage_map: dict[str, tuple[str, str]] = {}
    for line in stage_rows:
        left, separator, path = line.partition("\t")
        fields = left.split(" ")
        _need(separator == "\t" and len(fields) == 3
              and fields[0] in {"100644", "100755", "120000"}
              and re.fullmatch(r"[0-9a-f]{40}", fields[1]) is not None
              and fields[2] == "0" and path in tracked and path not in stage_map,
              f"{role} git index stage row is malformed")
        stage_map[path] = (fields[0], fields[1])
    _need(set(stage_map) == set(tracked)
          and all(stage_map[path] == (row["mode"], row["gitBlobSha1"])
                  for row in files for path in (row["path"],)),
          f"{role} index rows differ from tracked source map")
    selected = value["selectedBuildInputs"]
    _need(isinstance(selected, list) and len(selected) == source["selectedInputCount"],
          f"{role} selected build-input roster is incomplete")
    canonical: list[dict[str, Any]] = []
    previous = ""
    for row in selected:
        item = _closed(row, {"path", "bytes", "sha256"}, f"{role} selected build input")
        _need(isinstance(item["path"], str) and item["path"] > previous
              and not Path(item["path"]).is_absolute() and ".." not in Path(item["path"]).parts
              and item["path"] in tracked_map and type(item["bytes"]) is int
              and item["bytes"] >= 0 and isinstance(item["sha256"], str)
              and SHA.fullmatch(item["sha256"]),
              f"{role} selected build-input row is invalid")
        canonical.append(item)
        previous = item["path"]
    canonical_digest = _sha((json.dumps(canonical, sort_keys=True) + "\n").encode())
    _need(canonical_digest == source["selectedInputRosterSha256"],
          f"{role} selected build-input digest differs")
    return value, tracked_map


def _bind_selected_inputs_to_source_guard(pmap: dict[str, Any], guard: dict[str, Any]) -> None:
    """Tie P's selected source hashes to the separately accepted physical P copy."""
    actual_rows = guard.get("source", {}).get("ownedCopyAfter", {}).get("files")
    _need(isinstance(actual_rows, list) and len(actual_rows) == 5901,
          "accepted source/package receipt lacks the complete physical P file roster")
    actual: dict[str, tuple[int, str]] = {}
    paths: list[str] = []
    for row in actual_rows:
        item = _closed(row, {"bytes", "linkCount", "mode", "path", "sha256"},
                       "physical P source file")
        _need(isinstance(item["path"], str) and item["path"] not in actual
              and type(item["bytes"]) is int and item["bytes"] >= 0
              and type(item["linkCount"]) is int and item["linkCount"] == 1
              and type(item["mode"]) is int and isinstance(item["sha256"], str)
              and SHA.fullmatch(item["sha256"]),
              "physical P source file row is malformed")
        actual[item["path"]] = (item["bytes"], item["sha256"])
        paths.append(item["path"])
    _need(paths == pmap["trackedPaths"],
          "producer source-map path roster differs from the accepted physical P copy")
    selected = pmap["selectedBuildInputs"]
    _need(all(actual.get(row["path"]) == (row["bytes"], row["sha256"])
              for row in selected),
          "producer selected build-input hashes differ from accepted physical P bytes")


def _verify_ancestry(root: Path, ref: Any, p_commit: str, m_commit: str) -> dict[str, Any]:
    value, _, _, _ = _json_receipt(root, ref, "P-to-M Git ancestry observation")
    _closed(value, {"profile", "sourceCommit", "metadataCommit", "metadataTree", "commands"}, "ancestry receipt")
    _need(value["profile"] == "weft-local-git-ancestry-observation/1"
          and value["sourceCommit"] == p_commit and value["metadataCommit"] == m_commit,
          "ancestry receipt identities differ")
    commands = value["commands"]
    _need(isinstance(commands, list), "ancestry command roster is malformed")
    by_name = {}
    for row in commands:
        item = _closed(row, {"name", "argv", "exitCode", "durationSeconds", "timedOut",
                             "outputCapped", "stdout", "stderr"}, "ancestry command")
        _need(isinstance(item["name"], str) and item["name"] not in by_name
              and isinstance(item["argv"], list) and item["exitCode"] == 0
              and item["timedOut"] is False and item["outputCapped"] is False
              and type(item["durationSeconds"]) in {int, float}
              and 0 <= item["durationSeconds"] <= 60
              and isinstance(item["stdout"], str) and isinstance(item["stderr"], str)
              and not item["stderr"], "ancestry command did not complete cleanly")
        by_name[item["name"]] = item
    _need(set(by_name) == {"merge-base", "metadata-head", "metadata-tree", "clean-status"},
          "ancestry command roster differs")
    expected = {
        "merge-base": ["git", "merge-base", "--is-ancestor", p_commit, m_commit],
        "metadata-head": ["git", "rev-parse", "HEAD"],
        "metadata-tree": ["git", "rev-parse", "HEAD^{tree}"],
        "clean-status": ["git", "status", "--porcelain"],
    }
    for name, argv in expected.items():
        _need(by_name[name]["argv"] == argv, f"ancestry command argv differs: {name}")
    _need(by_name["metadata-head"]["stdout"].strip() == m_commit
          and by_name["metadata-tree"]["stdout"].strip() == value["metadataTree"]
          and by_name["clean-status"]["stdout"] == "",
          "metadata HEAD/tree/status observation differs")
    _need(by_name["merge-base"]["stdout"] == "", "merge-base unexpectedly returned data")
    return value


def _source_correspondence(root: Path, producer_ref: Any, metadata_ref: Any,
                           ancestry_ref: Any, source_guard_ref: Any = None
                           ) -> tuple[dict[str, Any], dict[str, Any], dict[str, Any]]:
    pmap, p_files = _validate_source_map(root, producer_ref, "producer P",
                                         source_guard_ref=source_guard_ref)
    mmap, m_files = _validate_source_map(root, metadata_ref, "metadata M",
                                         source_guard_ref=source_guard_ref,
                                         producer_map=pmap)
    p_v2 = pmap.get("profile") == "weft-local-release-source-map/2"
    m_v2 = mmap.get("profile") == "weft-local-release-source-map-delta/2"
    _need(p_v2 == m_v2,
          "P full source map and M delta must use the same reviewed storage profile")
    _, _, metadata_map_sha = _receipt(root, metadata_ref, "metadata M source map")
    _, _, ancestry_sha = _receipt(root, ancestry_ref, "metadata M ancestry")
    p = pmap["source"]
    m = mmap["source"]
    _need(p["commit"] == PRODUCER and p["tree"] == PRODUCER_TREE
          and p["trackedCount"] == SOURCE_IDENTITY["trackedCount"]
          and p["trackedPathSha256"] == SOURCE_IDENTITY["fileRosterSha256"]
          and p["gitStageRosterSha256"] == SOURCE_IDENTITY["gitStageRosterSha256"]
          and p["selectedInputCount"] == SOURCE_IDENTITY["selectedInputCount"]
          and p["selectedInputRosterSha256"] == SOURCE_IDENTITY["selectedInputRosterSha256"],
          "producer source map is not the frozen P56 identity")
    ancestry = _verify_ancestry(root, ancestry_ref, p["commit"], m["commit"])
    p_rows = {row["path"]: row for row in pmap["selectedBuildInputs"]}
    m_rows = {row["path"]: row for row in mmap["selectedBuildInputs"]}
    _need(set(p_rows) == set(m_rows), "P/M selected build-input path roster differs")
    changed = []
    for path in sorted(p_rows):
        if p_rows[path] != m_rows[path]:
            changed.append(path)
    _need(not changed, "metadata M changes selected producer build inputs")
    differing = sorted(set(p_files) | set(m_files))
    metadata_only = []
    for path in differing:
        if p_files.get(path) != m_files.get(path):
            _need(path.startswith(METADATA_ONLY_PREFIXES),
                  f"P/M source difference is outside reviewed metadata-only paths: {path}")
            metadata_only.append(path)
    _need(m["commit"] != p["commit"] and m["tree"] != p["tree"]
          and SHA.fullmatch(metadata_map_sha) and SHA.fullmatch(ancestry_sha)
          and ancestry["metadataTree"] == m["tree"],
          "actual merged M commit/tree/source-map/ancestry observation is inconsistent")
    _need(set(metadata_only) <= METADATA_ONLY_PATHS
          and METADATA_ONLY_PATHS <= set(m_files),
          "M metadata-only difference set is unreviewed or required release files are absent")
    return pmap, mmap, {"receipt": ancestry, "metadataOnlyPaths": metadata_only,
                        "selectedBuildInputsEqual": True,
                        "metadataSourceMapSha256": metadata_map_sha,
                        "ancestrySha256": ancestry_sha}


def _validate_metadata_files(root: Path, files_value: Any,
                             metadata_map: dict[str, Any]) -> dict[str, dict[str, Any]]:
    _need(isinstance(files_value, list) and len(files_value) == len(METADATA_ONLY_PATHS),
          "metadata file evidence roster is incomplete")
    tracked = {row["path"]: row["gitBlobSha1"] for row in metadata_map["trackedFiles"]}
    resolved: dict[str, dict[str, Any]] = {}
    previous = ""
    for row in files_value:
        item = _closed(row, {"sourcePath", "path", "sizeBytes", "sha256"},
                       "metadata source file reference")
        source_path = item["sourcePath"]
        _need(isinstance(source_path, str) and source_path > previous
              and source_path in METADATA_ONLY_PATHS,
              "metadata source file path is not in the exact reviewed seven-file set")
        ref = {key: item[key] for key in ("path", "sizeBytes", "sha256")}
        physical, raw, digest = _receipt(root, ref, f"metadata source file {source_path}")
        _need(_git_blob_sha1(raw) == tracked.get(source_path),
              f"metadata source bytes differ from M's tracked Git blob: {source_path}")
        current = _under(METADATA_SOURCE_ROOT, source_path, "current M source file")
        current_raw = _read_bounded(current, MAX_EVIDENCE_FILE,
                                    f"current M source file {source_path}")
        _need(current_raw == raw and digest == _sha(current_raw),
              f"consumed metadata bytes differ from the actual M checkout: {source_path}")
        resolved[source_path] = {**ref, "physicalPath": physical}
        previous = source_path
    _need(set(resolved) == METADATA_ONLY_PATHS,
          "metadata file evidence does not close the seven M-only files")
    return resolved


def _gate(root: Path, name: str, row: Any) -> dict[str, Any]:
    if name == "macos-arm64-build-static":
        gate_keys = {"receipt", "review", "dataReview"}
    elif name == "linux-x86_64-final-native-campaign":
        gate_keys = {"receipt", "review", "reanalysis"}
    else:
        gate_keys = {"receipt", "review"}
    item = _closed(row, gate_keys, f"gate {name}")
    receipt, path, digest, size = _json_receipt(root, item["receipt"], f"gate {name} result")
    expected_gate = GATE_PROFILES[name]
    _need(expected_gate is not None,
          f"gate {name} actual receipt profile is not implemented/frozen")
    profile, status = expected_gate
    _need((GATE_RECEIPT_SHA256[name] is None or digest == GATE_RECEIPT_SHA256[name])
          and receipt.get("profile") == profile and receipt.get("status") == status,
          f"gate {name} has wrong actual profile/status")
    human_review_sha = GATE_HUMAN_REVIEW_SHA256[name]
    if name == "linux-inherited-boundaries":
        output_refs, software_refs = _table012_output_rows(root, path, receipt, verify=True)
        review_path, review_raw, review_digest = _receipt(root, item["review"],
                                                          "Table012 independent review")
        review_size = len(review_raw)
        _need(review_digest == TABLE012_REVIEW_SHA256
              and digest.encode("ascii") in review_raw
              and b"ACCEPT the completed local Linux table-operations boundary evidence" in review_raw,
              "Table012 review does not accept the exact immutable receipt and scope")
        review = {"kind": "pinned-human-review", "sha256": review_digest,
                  "scope": name, "boundReceiptSha256": digest}
        _validate_gate_details(name, receipt, root=root, receipt_path=path)
    elif name in {"linux-browser", "linux-wasm"}:
        output_refs, software_refs = _truss011_output_rows(root, path, receipt, verify=True)
        review_path, review_raw, review_digest = _receipt(root, item["review"],
                                                          "Truss011 independent review")
        review_size = len(review_raw)
        _need(review_digest == TRUSS011_REVIEW_SHA256
              and digest.encode("ascii") in review_raw
              and b"ACCEPTED scoped full local Truss facts/entity lane PASS" in review_raw,
              "Truss011 review does not accept the exact immutable receipt and scope")
        review = {"kind": "pinned-human-review", "sha256": review_digest,
                  "scope": name, "boundReceiptSha256": digest}
        _validate_gate_details(name, receipt, root=root, receipt_path=path)
    elif name == "linux-source-package":
        _need(digest == SOURCE_GUARD_RECEIPT_SHA256,
              "source/package guard receipt is not the independently reviewed exact run")
        output_refs = _source_package_output_inventory(root, path, receipt)
        review_path, review_raw, review_digest = _receipt(root, item["review"],
                                                          f"gate {name} review")
        review_size = len(review_raw)
        _need(review_digest == SOURCE_GUARD_REVIEW_SHA256
              and digest.encode("ascii") in review_raw
              and b"ACCEPT these actual bounded Linux source/package guard results" in review_raw,
              "source/package guard review is not the exact independent acceptance")
        review = {"profile": "weft-local-gate-review/1", "status": "ACCEPTED",
                  "gate": name, "receiptSha256": digest}
    elif name == "macos-arm64-build-static":
        review_path, review_raw, review_digest = _receipt(
            root, item["review"], f"gate {name} reviewer acceptance")
        review_size = len(review_raw)
        _need(review_digest == MAC_ARTIFACT_REVIEW_SHA256
              and MAC_BUILD_RESULT_SHA256.encode("ascii") in review_raw
              and MAC_AUDIT_RESULT_SHA256.encode("ascii") in review_raw
              and MAC_WHEEL_SHA256.encode("ascii") in review_raw
              and MAC_STATIC_REPORT_SHA256.encode("ascii") in review_raw
              and b"Disposition: ACCEPT the exact local macOS ARM64 build artifact plus its separate static-audit continuation." in review_raw,
              "Mac reviewer acceptance does not bind the actual failed-build/static-audit chain")
        data_review, data_path, data_digest, data_size = _json_receipt(
            root, item["dataReview"], "Mac static-audit data review")
        _need(data_digest == MAC_DATA_REVIEW_SHA256
              and data_review.get("disposition") ==
                    "ACCEPT_LOCAL_MACOS_BUILD002_PLUS_SEPARATE_STATIC_AUDIT004_ONLY"
              and data_review.get("result") == {
                    "bytes": 8051, "sha256": MAC_AUDIT_RESULT_SHA256}
              and data_review.get("report") == {
                    "bytes": 9604,
                    "path": str(MAC_AUDIT_ORIGINAL_ROOT / "static-audit.json"),
                    "sha256": MAC_STATIC_REPORT_SHA256}
              and data_review.get("wheel") == {
                    "bytes": MAC_WHEEL_BYTES,
                    "filename": "umf_weft-0.4.56-cp39-abi3-macosx_11_0_arm64.whl",
                    "path": str(MAC_BUILD_ORIGINAL_ROOT / "dist" /
                                "umf_weft-0.4.56-cp39-abi3-macosx_11_0_arm64.whl"),
                    "sha256": MAC_WHEEL_SHA256}
              and data_review.get("nativeSha256") == MAC_NATIVE_SHA256
              and data_review.get("sourceCommit") == PRODUCER
              and data_review.get("sourceTree") == PRODUCER_TREE
              and data_review.get("originalBuildResultSha256") == MAC_BUILD_RESULT_SHA256
              and data_review.get("originalBuildStatus") == MAC_ORIGINAL_BUILD_STATUS
              and data_review.get("originalAuditExitCode") == 1
              and data_review.get("runtimeQualification") == "NOT_RUN"
              and data_review.get("hosted") is False,
              "Mac machine data review is not the exact accepted audit")
        review = {"kind": "pinned-human-review", "sha256": review_digest,
                  "scope": name, "boundReceiptSha256": digest}
        output_refs = _mac_selected_output_rows(receipt, path)
    elif name == "linux-aarch64-build-static":
        output_refs = _arm_selected_output_rows(receipt, path)
        review_path, review_raw, review_digest = _receipt(
            root, item["review"], f"gate {name} human review")
        review_size = len(review_raw)
        _need(review_digest == ARM_ARTIFACT_REVIEW_SHA256
              and ARM_BUILD_RESULT_SHA256.encode("ascii") in review_raw
              and ARM_WHEEL_SHA256.encode("ascii") in review_raw
              and b"ACCEPTED ACTUAL BUILD/STATIC ARTIFACT ONLY" in review_raw,
              "ARM review does not bind exact build, wheel, and static-only scope")
        review = {"kind": "pinned-human-review", "sha256": review_digest,
                  "scope": name, "boundReceiptSha256": digest}
    elif (name == "linux-x86_64-final-native-campaign"
          and profile == "umf.g03-full14-local-linux-operator/1"):
        review_path, review_raw, review_digest = _receipt(
            root, item["review"], "native007 predicate-reanalysis acceptance")
        review_size = len(review_raw)
        derived, derived_path, derived_digest, derived_size = _json_receipt(
            root, item["reanalysis"], "native007 derived predicate receipt")
        _need(review_digest == FINAL_NATIVE_REVIEW
              and digest == FINAL_NATIVE_OPERATOR
              and derived_digest == FINAL_NATIVE_REANALYSIS
              and digest.encode("ascii") in review_raw
              and derived_digest.encode("ascii") in review_raw
              and FINAL_NATIVE_CAMPAIGN.encode("ascii") in review_raw
              and b"ACCEPTED: the selected Linux311 G03 fourteen-case/nineteen-scenario evidence conjunction closes through reviewed pure-predicate reanalysis" in review_raw,
              "native007 acceptance does not bind the immutable failed run and its derived receipt")
        _validate_native_reanalysis(receipt, derived, root=root,
                                    receipt_path=path, derived_path=derived_path)
        output_refs = _native_campaign_output_rows(receipt, path)
        review = {"kind": "pinned-human-review", "sha256": review_digest,
                  "scope": name, "boundReceiptSha256": digest,
                  "derivedPredicateReceiptSha256": derived_digest}
        _validate_gate_details(name, receipt, root=root, receipt_path=path,
                               reanalysis=derived)
        return {"profile": profile,
                "status": "PREDICATE_REANALYSIS_PASS_ORIGINAL_RUN_FAIL_OR_UNKNOWN",
                "receipt": item["receipt"], "receiptSha256": digest,
                "receiptBytes": size, "review": item["review"],
                "reviewSha256": review_digest, "reviewBytes": review_size,
                "reanalysis": item["reanalysis"], "reanalysisSha256": derived_digest,
                "reanalysisBytes": derived_size, "outputFiles": output_refs}
    else:
        output_refs = _operator_output_inventory(root, path, receipt, name)
        if human_review_sha is not None:
            review_path, review_raw, review_digest = _receipt(
                root, item["review"], f"gate {name} human review")
            review_size = len(review_raw)
            _need(review_digest == human_review_sha,
                  f"gate {name} human review is not the exact accepted review")
            if name == "linux-x86_64-final-installed-admission":
                _need(digest.encode("ascii") in review_raw
                      and b"ACCEPTED ACTUAL INSTALLED SNAPSHOT" in review_raw
                      and b"FIVE COMPILER-ADMISSION CONTROLS ONLY" in review_raw,
                      "installed-prep review does not accept this exact receipt and scope")
            elif name == "linux-aarch64-build-static":
                _need(ARM_BUILD_RESULT_SHA256.encode("ascii") in review_raw
                      and ARM_WHEEL_SHA256.encode("ascii") in review_raw
                      and b"ACCEPTED ACTUAL BUILD/STATIC ARTIFACT ONLY" in review_raw,
                      "ARM review does not bind the exact build, wheel, and static-only scope")
            review = {"kind": "pinned-human-review", "sha256": review_digest,
                      "scope": name, "boundReceiptSha256": digest}
        else:
            review, review_path, review_digest, review_size = _json_receipt(
                root, item["review"], f"gate {name} review")
            _need(review.get("profile") == "weft-local-gate-review/1"
                  and review.get("status") == "ACCEPTED" and review.get("gate") == name
                  and review.get("receiptSha256") == digest,
                  f"gate {name} independent review does not bind its exact receipt")
    _need((GATE_REVIEW_SHA256[name] is None or review_digest == GATE_REVIEW_SHA256[name]),
          f"gate {name} independent review digest differs")
    _validate_gate_details(name, receipt, root=root, receipt_path=path)
    return {"profile": profile, "status": status, "receipt": item["receipt"],
            "receiptSha256": digest, "receiptBytes": size, "review": item["review"],
            "reviewSha256": review_digest, "reviewBytes": review_size,
            "outputFiles": output_refs,
            **({"externalSelectedSoftware": software_refs}
               if name in {"linux-inherited-boundaries", "linux-browser", "linux-wasm"}
               else {})}


def _absolute_evidence_rows(receipt_path: Path, rows: Any, where: str, *,
                             historical_root: Path | None = None,
                             verify: bool = True) -> list[dict[str, Any]]:
    _need(isinstance(rows, list) and rows, f"{where} evidence roster is absent")
    base = receipt_path.parent.resolve(strict=True)
    result: list[dict[str, Any]] = []
    seen: set[str] = set()
    for row in rows:
        _need(isinstance(row, dict), f"{where} evidence row is malformed")
        path_value = row.get("path")
        size = row.get("sizeBytes", row.get("bytes"))
        digest = row.get("sha256")
        _need(isinstance(path_value, str) and path_value
              and type(size) is int and 0 <= size <= MAX_EVIDENCE_FILE
              and isinstance(digest, str) and SHA.fullmatch(digest),
              f"{where} evidence identity is malformed")
        absolute = Path(path_value)
        if absolute.is_absolute():
            _need(historical_root is not None,
                  f"{where} absolute evidence path has no accepted historical root")
            relative_path = Path(_historical_relative(path_value, historical_root, where))
            absolute = base / relative_path
        else:
            _canonical_relative(path_value, f"{where} relative evidence")
            relative_path = absolute
            absolute = base / relative_path
        current = base
        for part in relative_path.parts:
            current = current / part
            _need(not current.is_symlink(), f"{where} evidence path contains a symlink")
        relative = relative_path.as_posix()
        _need(relative not in seen, f"{where} evidence path is duplicated")
        seen.add(relative)
        if verify:
            actual_size, actual_digest = _regular(absolute, MAX_EVIDENCE_FILE, f"{where} evidence")
            _need(actual_size == size and actual_digest == digest,
                  f"{where} evidence bytes differ from its receipt")
        else:
            try:
                st = absolute.stat(follow_symlinks=False)
            except OSError as exc:
                raise ProfileError(f"{where} evidence cannot be statted") from exc
            _need(stat.S_ISREG(st.st_mode) and st.st_size == size,
                  f"{where} evidence is not a bounded regular file")
        result.append({"path": relative, "sizeBytes": size, "sha256": digest})
    return result


def _arm_selected_output_rows(receipt: dict[str, Any], receipt_path: Path, *,
                              verify: bool = True) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    commands = receipt.get("commands")
    _need(isinstance(commands, list) and len(commands) == 3,
          "ARM build command roster is incomplete")
    for command in commands:
        closure = command.get("physicalClosure") if isinstance(command, dict) else None
        _need(isinstance(closure, dict) and closure.get("verified") is True
              and closure.get("errors") == [], "ARM command capture closure is absent")
        rows.extend(closure.get("files", []))
    candidate = receipt.get("candidateWheel")
    _need(isinstance(candidate, dict) and isinstance(candidate.get("inspectionPath"), str),
          "ARM candidate inspection identity is absent")
    inspection = candidate["inspectionPath"]
    relative = _historical_relative(inspection, ARM_BUILD_ORIGINAL_ROOT,
                                    "ARM candidate inspection")
    copy_size, copy_sha = _regular(receipt_path.parent / relative, MAX_EVIDENCE_FILE,
                                   "ARM candidate inspection packet copy")
    _need(copy_sha == candidate.get("inspectionSha256"),
          "ARM candidate inspection packet copy digest differs")
    size = copy_size
    rows.append({"path": str(inspection), "sizeBytes": size,
                 "sha256": candidate.get("inspectionSha256")})
    return _absolute_evidence_rows(receipt_path, rows, "ARM build",
                                   historical_root=ARM_BUILD_ORIGINAL_ROOT, verify=verify)


def _mac_selected_output_rows(receipt: dict[str, Any], receipt_path: Path, *,
                              verify: bool = True) -> list[dict[str, Any]]:
    return _absolute_evidence_rows(receipt_path, receipt.get("selectedEvidenceFiles"),
                                   "Mac audit", historical_root=MAC_AUDIT_ORIGINAL_ROOT,
                                   verify=verify)


def _producer_evidence_rows(record: dict[str, Any], receipt_path: Path,
                            *, where: str,
                            copied_rows: list[dict[str, Any]] | None = None) -> list[dict[str, Any]]:
    """Return receipt-declared identities, resolving bytes only from packet copies."""
    base = receipt_path.parent
    is_mac_build = where == "macos-arm64 build"
    is_mac_audit = where == "macos-arm64 audit"
    original_root = (MAC_BUILD_ORIGINAL_ROOT if is_mac_build else
                     MAC_AUDIT_ORIGINAL_ROOT if is_mac_audit else
                     X64_BUILD_ORIGINAL_ROOT if where == "linux-x86_64 build" else
                     X64_AUDIT_ORIGINAL_ROOT if where == "linux-x86_64 audit" else
                     ARM_BUILD_ORIGINAL_ROOT if where == "linux-aarch64 build" else None)
    _need(copied_rows is None or isinstance(copied_rows, list),
          f"{where} copied evidence mapping is malformed")
    copied_by_source: dict[str, dict[str, Any]] = {}
    for item in copied_rows or []:
        _need(isinstance(item, dict)
              and set(item) == {"sourcePath", "path", "sizeBytes", "sha256"}
              and isinstance(item.get("sourcePath"), str)
              and item["sourcePath"] not in copied_by_source
              and isinstance(item.get("path"), str)
              and type(item.get("sizeBytes")) is int
              and 0 <= item["sizeBytes"] <= MAX_EVIDENCE_FILE
              and isinstance(item.get("sha256"), str)
              and SHA.fullmatch(item["sha256"]),
              f"{where} copied evidence mapping has a malformed or duplicate source")
        copied_by_source[item["sourcePath"]] = item
    rows: list[dict[str, Any]] = []

    def source_identity(raw_path: Any, size: Any, digest: Any, label: str,
                        allowed_root: Path | None = original_root) -> None:
        _need(allowed_root is not None,
              f"{where} {label} has no fixed historical evidence root")
        _need(isinstance(raw_path, str) and raw_path,
              f"{where} {label} historical path is absent")
        path = Path(raw_path)
        if path.is_absolute():
            _historical_relative(raw_path, allowed_root, f"{where} {label}")
            source_path = raw_path
        else:
            _canonical_relative(raw_path, f"{where} {label}")
            source_path = (allowed_root / path).as_posix()
        _need(isinstance(digest, str) and SHA.fullmatch(digest),
              f"{where} {label} digest is malformed")
        supplied = copied_by_source.get(source_path)
        if size is None and copied_rows is not None:
            _need(supplied is not None and supplied["sha256"] == digest,
                  f"{where} {label} omitted size lacks a uniquely matching packet copy")
            size = supplied["sizeBytes"]
        _need(size is None or type(size) is int and 0 <= size <= MAX_EVIDENCE_FILE,
              f"{where} {label} size is malformed")
        rows.append({"sourcePath": source_path, "sizeBytes": size, "sha256": digest})

    if isinstance(record.get("selectedEvidenceFiles"), list):
        for item in record["selectedEvidenceFiles"]:
            _need(isinstance(item, dict), f"{where} selected-evidence row is malformed")
            raw_path = item.get("path")
            size = item.get("sizeBytes", item.get("bytes"))
            digest = item.get("sha256")
            source_identity(raw_path, size, digest, "selected evidence")
    if isinstance(record.get("commands"), list):
        for command in record["commands"]:
            closure = command.get("physicalClosure") if isinstance(command, dict) else None
            physical = closure.get("files") if isinstance(closure, dict) else None
            if isinstance(physical, list):
                for item in physical:
                    _need(isinstance(item, dict), f"{where} command physical file is malformed")
                    raw_path = item.get("path")
                    size = item.get("sizeBytes", item.get("bytes"))
                    digest = item.get("sha256")
                    _need(isinstance(raw_path, str) and Path(raw_path).is_absolute(),
                          f"{where} command physical identity is malformed")
                    source_identity(raw_path, size, digest, "command evidence")
    candidate = record.get("candidateWheel")
    if isinstance(candidate, dict) and isinstance(candidate.get("inspectionPath"), str):
        source_identity(candidate["inspectionPath"], candidate.get("inspectionSizeBytes"),
                        candidate.get("inspectionSha256"), "wheel inspection")
    if isinstance(record.get("files"), list):
        for item in record["files"]:
            _need(isinstance(item, dict), f"{where} file evidence is malformed")
            raw_path = item.get("path")
            size = item.get("sizeBytes", item.get("bytes"))
            digest = item.get("sha256")
            _need(isinstance(raw_path, str) and type(size) is int
                  and isinstance(digest, str) and SHA.fullmatch(digest),
                  f"{where} file evidence identity is malformed")
            source_identity(raw_path, size, digest, "file evidence")
    operator = record.get("operator")
    if isinstance(operator, dict):
        operator_files: list[tuple[str, Any, Any]] = []
        for prefix, facts in (("runner", operator), ("staticAudit", operator)):
            path_key = prefix + "Path"
            bytes_key = prefix + "Bytes"
            sha_key = prefix + "Sha256"
            if all(key in facts for key in (path_key, bytes_key, sha_key)):
                operator_files.append((facts[path_key], facts[bytes_key], facts[sha_key]))
        if is_mac_audit and isinstance(operator.get("runnerPath"), str):
            runner_before = operator.get("runnerBefore")
            _need(isinstance(runner_before, list) and len(runner_before) == 2,
                  "Mac audit runner pre/post identity is malformed")
            operator_files.append((operator["runnerPath"], runner_before[0], runner_before[1]))
        for nested in ("runner", "helper"):
            facts = operator.get(nested)
            if isinstance(facts, dict) and all(key in facts for key in ("path", "bytes", "sha256")):
                operator_files.append((facts["path"], facts["bytes"], facts["sha256"]))
        # Runtime binaries are JSON identity observations, never metadata evidence.
        for raw_path, size, digest in operator_files:
            _need(isinstance(raw_path, str) and Path(raw_path).is_absolute()
                  and type(size) is int and 0 <= size <= MAX_EVIDENCE_FILE
                  and isinstance(digest, str) and SHA.fullmatch(digest),
                  f"{where} operator source identity is malformed")
            operator_root = (MAC_BUILD_OPERATOR_ROOT if is_mac_build else
                             MAC_AUDIT_OPERATOR_ROOT if is_mac_audit else original_root)
            source_identity(raw_path, size, digest, "operator source", operator_root)
    if is_mac_audit:
        source = record.get("originalInputs", {}).get("source", {})
        before = source.get("custodyAssetsBefore")
        after = source.get("custodyAssetsAfter")
        _need(isinstance(before, dict) and before == after
              and set(before) == {"custody.json", "git-stage-roster.txt", "source-inputs.rows.json"},
              "Mac source custody assets changed or have an open roster")
        custody_root = MAC_CUSTODY_ROOT
        for name, fact in before.items():
            _need(isinstance(fact, dict) and set(fact) == {"bytes", "path", "sha256"}
                  and fact["path"] == str(custody_root / name)
                  and type(fact["bytes"]) is int
                  and isinstance(fact["sha256"], str) and SHA.fullmatch(fact["sha256"]),
                  "Mac source custody asset identity differs")
            source_identity(fact["path"], fact["bytes"], fact["sha256"],
                            "Mac source custody", custody_root)
    by_path: dict[str, dict[str, Any]] = {}
    for row in rows:
        previous = by_path.get(row["sourcePath"])
        _need(previous is None or (previous["sha256"] == row["sha256"]
              and (previous["sizeBytes"] is None or row["sizeBytes"] is None
                   or previous["sizeBytes"] == row["sizeBytes"])),
              f"{where} duplicates a physical file with conflicting identity")
        if previous is None or previous["sizeBytes"] is None:
            by_path[row["sourcePath"]] = row
    if copied_rows is not None:
        for source_path, row in by_path.items():
            copied = copied_by_source.get(source_path)
            _need(copied is not None and row["sizeBytes"] is not None
                  and (copied["sizeBytes"], copied["sha256"])
                      == (row["sizeBytes"], row["sha256"]),
                  f"{where} packet copy identity differs from original receipt")
    return [by_path[key] for key in sorted(by_path)]


def _source_package_output_inventory(root: Path, receipt_path: Path,
                                     receipt: dict[str, Any]) -> list[dict[str, Any]]:
    """Close source-guard stream records against their physical retained files."""
    artifacts = receipt.get("terminalArtifacts")
    processes = receipt.get("processes")
    _need(isinstance(artifacts, list) and artifacts
          and isinstance(processes, list) and len(processes) == 40,
          "source/package guard process or terminal-artifact roster is incomplete")
    base = receipt_path.parent
    packet_root = root.resolve(strict=True)
    artifact_by_path: dict[str, tuple[Path, int, str]] = {}
    artifact_by_logical: dict[str, tuple[str, Path, int, str]] = {}
    total = 0
    for row in artifacts:
        item = _closed(row, {"bytes", "path", "sha256", "type"},
                       "source/package terminal artifact")
        _need(item["type"] == "file",
              "source/package terminal artifact path/type is invalid")
        _canonical_relative(item["path"], "source/package terminal artifact")
        rel = Path(item["path"])
        output = base / rel
        current = base
        for part in rel.parts:
            current = current / part
            _need(not current.is_symlink(), "source/package terminal artifact contains a symlink")
        resolved = output.resolve(strict=True)
        _need(resolved.is_relative_to(packet_root.resolve(strict=True)),
              "source/package terminal artifact escapes packet evidence")
        key = resolved.relative_to(packet_root).as_posix()
        _need(key not in artifact_by_path, "source/package terminal artifact is duplicated")
        st = output.stat(follow_symlinks=False)
        _need(stat.S_ISREG(st.st_mode) and type(item["bytes"]) is int
              and 0 <= st.st_size == item["bytes"] <= MAX_EVIDENCE_FILE
              and isinstance(item["sha256"], str) and SHA.fullmatch(item["sha256"]),
              "source/package terminal artifact is not bounded regular evidence")
        total += st.st_size
        artifact_by_path[key] = (output, st.st_size, item["sha256"])
        artifact_by_logical[rel.as_posix()] = (key, output, st.st_size, item["sha256"])

    matched: set[str] = set()
    raw_total = 0
    for process in processes:
        _need(isinstance(process, dict) and process.get("exitCode") == 0
              and process.get("timedOut") is False and process.get("drainExpired") is False
              and process.get("rawOverflow") is False
              and process.get("streamCaptureComplete") is True
              and process.get("spawnOrCaptureError") is None
              and process.get("launchError") is None
              and process.get("streamsEOF") == {"stdout": True, "stderr": True},
              "source/package process did not complete with closed captured streams")
        for stream_name in ("stdout", "stderr"):
            stream = process.get(stream_name)
            _need(isinstance(stream, dict), "source/package process stream is absent")
            raw_bytes = stream.get("rawBytes")
            stream_sha = stream.get("sha256")
            _need(type(raw_bytes) is int and raw_bytes >= 0
                  and isinstance(stream_sha, str) and SHA.fullmatch(stream_sha),
                  "source/package process stream metadata is malformed")
            raw_total += raw_bytes
            log_path = stream.get("logPath")
            if raw_bytes == 0:
                _need(log_path is None and stream_sha == EMPTY_SHA256,
                      "empty source/package stream has unexpected retained bytes")
                continue
            _need(isinstance(log_path, str) and Path(log_path).is_absolute(),
                  "nonempty source/package stream lacks an absolute physical log path")
            logical_log = "logs/" + _historical_relative(
                log_path, SOURCE_GUARD_ORIGINAL_LOG_ROOT, "source/package stream")
            artifact = artifact_by_logical.get(logical_log)
            _need(artifact is not None and artifact[0] not in matched
                  and artifact[2] == raw_bytes and artifact[3] == stream_sha,
                  "source/package stream bytes differ from terminal artifact roster")
            actual_size, actual_sha = _regular(artifact[1], MAX_EVIDENCE_FILE,
                                               "source/package stream log")
            _need(actual_size == raw_bytes and actual_sha == stream_sha,
                  "source/package stream physical bytes differ from process capture")
            matched.add(artifact[0])
    raw = receipt.get("rawOutputBytes")
    _need(raw_total == SOURCE_GUARD_RAW_BYTES and raw_total <= MAX_RAW_LOGS
          and isinstance(raw, dict) and raw.get("total") == raw_total,
          "source/package complete raw output exceeds the original 1 MiB bound")
    _need(matched == set(artifact_by_path) and total == receipt.get("retainedLogBytes")
          and total == raw_total and total <= MAX_RAW_LOGS,
          "source/package retained logs differ from complete process streams")
    _need(receipt.get("inclusiveEvidenceBytes") == SOURCE_GUARD_EVIDENCE_BYTES
          and receipt["inclusiveEvidenceBytes"] <= MAX_EVIDENCE_TOTAL
          and receipt.get("failure") is None,
          "source/package inclusive evidence or failure state differs from reviewed run")
    return [{"path": path, "sizeBytes": size, "sha256": digest}
            for path, (_, size, digest) in sorted(artifact_by_path.items())]


def _operator_output_inventory(root: Path, receipt_path: Path, receipt: dict[str, Any],
                               name: str) -> list[dict[str, Any]]:
    """Close the operator's retained files/logs, not only its summary JSON."""
    files = receipt.get("outputFiles")
    _need(isinstance(files, list) and files,
          f"gate {name} omits its physical output/log roster")
    base = receipt_path.parent
    packet_root = root.resolve(strict=True)
    preflight: list[tuple[str, Path, int, str]] = []
    seen: set[str] = set()
    total = log_total = guest_logs = outer_logs = 0
    for row in files:
        item = _closed(row, {"path", "sha256", "sizeBytes"}, f"gate {name} output file")
        rel = _canonical_relative(item["path"], f"gate {name} output")
        relative = Path(rel)
        output_path = base / relative
        for ancestor in [base, *(base / Path(*relative.parts[:i])
                                 for i in range(1, len(relative.parts) + 1))]:
            _need(not ancestor.is_symlink(), f"gate {name} output contains a symlink")
        resolved = output_path.resolve(strict=True)
        _need(resolved.is_relative_to(packet_root),
              f"gate {name} output escapes the packet evidence root")
        packet_relative = resolved.relative_to(packet_root).as_posix()
        _need(packet_relative not in seen, f"gate {name} output roster has duplicates")
        seen.add(packet_relative)
        st = output_path.stat(follow_symlinks=False)
        _need(stat.S_ISREG(st.st_mode) and type(item["sizeBytes"]) is int
              and 0 <= st.st_size == item["sizeBytes"] <= MAX_EVIDENCE_FILE
              and isinstance(item["sha256"], str) and SHA.fullmatch(item["sha256"]),
              f"gate {name} output is not a bounded regular file")
        total += st.st_size
        if relative.name.endswith((".log", ".stdout", ".stderr")):
            log_total += st.st_size
            if relative.parts[0] == "guest":
                guest_logs += st.st_size
            elif relative.name.startswith("outer."):
                outer_logs += st.st_size
        preflight.append((packet_relative, output_path, st.st_size, item["sha256"]))
    _need(total <= MAX_EVIDENCE_TOTAL and log_total <= MAX_RAW_LOGS,
          f"gate {name} output/evidence or raw-log cap is exceeded")
    _need(receipt.get("outputBytes") == total,
          f"gate {name} aggregate output-byte accounting differs")
    declared_logs = receipt.get("logBytes")
    _need(isinstance(declared_logs, dict)
          and declared_logs.get("limit") == MAX_RAW_LOGS
          and declared_logs.get("total") == log_total
          and declared_logs.get("guest") == guest_logs
          and declared_logs.get("outer") == outer_logs
          and declared_logs.get("total", MAX_RAW_LOGS + 1) <= MAX_RAW_LOGS,
          f"gate {name} raw-log accounting differs from physical outputs")
    closure = receipt.get("outerCapturePhysicalClosure")
    _need(isinstance(closure, dict) and closure.get("verified") is True
          and closure.get("errors") == [] and isinstance(closure.get("files"), list),
          f"gate {name} outer captures lack durable physical closure")
    _need(len(closure["files"]) == 2
          and all(isinstance(row, dict) and set(row) == {"path", "sha256", "sizeBytes"}
                  and isinstance(row.get("path"), str) for row in closure["files"]),
          f"gate {name} outer capture roster is not the exact two-stream set")
    outer_by_name = {Path(row["path"]).name: row for row in closure["files"]}
    _need(set(outer_by_name) == {"outer.stdout.log", "outer.stderr.log"},
          f"gate {name} outer capture names differ")
    for outer_name in ("outer.stdout.log", "outer.stderr.log"):
        matching = next((entry for entry in preflight
                         if Path(entry[0]).name == outer_name), None)
        closure_path = outer_by_name[outer_name]["path"] if outer_name in outer_by_name else None
        if name == "linux-x86_64-final-installed-admission":
            closure_relative = _historical_relative(
                closure_path, INSTALLED_ADMISSION_OUTPUT_ROOT,
                f"gate {name} outer capture") if closure_path is not None else None
            copied_relative = (matching[1].relative_to(base).as_posix()
                               if matching is not None else None)
            closure_matches = closure_relative == copied_relative
        else:
            closure_matches = (
                closure_path is not None
                and Path(closure_path).is_absolute()
                and Path(closure_path).resolve(strict=True)
                    == matching[1].resolve(strict=True) if matching is not None else False)
        _need(matching is not None and outer_name in outer_by_name
              and closure_matches
              and outer_by_name[outer_name].get("sha256") == matching[3]
              and outer_by_name[outer_name].get("sizeBytes") == matching[2],
              f"gate {name} outer capture bytes do not match the selected physical file")
    output_refs = []
    for packet_relative, output_path, expected_size, expected_sha in preflight:
        actual_size, actual_sha = _regular(output_path, MAX_EVIDENCE_FILE,
                                           f"gate {name} output closure")
        _need(actual_size == expected_size and actual_sha == expected_sha,
              f"gate {name} output changed after preflight")
        output_refs.append({"path": packet_relative, "sizeBytes": actual_size,
                            "sha256": actual_sha})
    return output_refs


def _native_campaign_output_rows(receipt: dict[str, Any], receipt_path: Path) -> list[dict[str, Any]]:
    """Map the immutable native007 evidence roster into the packet copy."""
    _need(receipt.get("outputRoot") == str(FINAL_NATIVE_OUTPUT_ROOT)
          and receipt.get("executionId") == FINAL_NATIVE_EXECUTION_ID
          and receipt.get("campaignId") == FINAL_NATIVE_CAMPAIGN_ID,
          "native007 evidence root/execution identity differs")
    rows = receipt.get("evidenceFiles")
    _need(isinstance(rows, list) and len(rows) == 181,
          "native007 selected evidence roster is not the exact 181-file set")
    by_path: dict[str, dict[str, Any]] = {}
    for row in rows:
        item = _closed(row, {"path", "sizeBytes", "sha256"}, "native007 evidence row")
        rel = _canonical_relative(item["path"], "native007 evidence")
        path = Path(rel)
        _need(rel not in by_path and type(item["sizeBytes"]) is int
              and 0 <= item["sizeBytes"] <= MAX_EVIDENCE_FILE
              and isinstance(item["sha256"], str) and SHA.fullmatch(item["sha256"]),
              "native007 evidence row is unsafe or duplicated")
        by_path[rel] = item
    commands = receipt.get("commands")
    _need(isinstance(commands, list) and len(commands) == 51,
          "native007 outer command roster differs")
    seen_commands: set[str] = set()
    for command in commands:
        _need(isinstance(command, dict) and isinstance(command.get("name"), str)
              and command["name"] not in seen_commands,
              "native007 command names are malformed or duplicated")
        seen_commands.add(command["name"])
        _need(command.get("timedOut") is False and command.get("drainExpired") is False
              and command.get("logCapExceeded") is False,
              "native007 outer command timed out or exceeded a capture cap")
        for stream_name in ("stdout", "stderr"):
            stream = command.get(stream_name)
            _need(isinstance(stream, dict) and stream.get("truncated") is False
                  and type(stream.get("rawBytes")) is int
                  and stream.get("rawBytes") == stream.get("retainedBytes")
                  and stream.get("rawSha256") == stream.get("sha256"),
                  "native007 outer stream capture is incomplete")
            relative = _historical_relative(stream.get("path"), FINAL_NATIVE_OUTPUT_ROOT,
                                            "native007 outer log")
            evidence = by_path.get(relative)
            _need(evidence is not None
                  and evidence["sizeBytes"] == stream["retainedBytes"]
                  and evidence["sha256"] == stream["sha256"],
                  "native007 outer stream does not match its selected physical evidence row")
    return [{"path": row["path"], "sizeBytes": row["sizeBytes"], "sha256": row["sha256"]}
            for row in rows]


def _validate_native_reanalysis(r: dict[str, Any], derived: dict[str, Any], *,
                                root: Path, receipt_path: Path,
                                derived_path: Path) -> None:
    """Require reviewed predicates over the exact preserved native007 failure."""
    _need(r.get("profile") == "umf.g03-full14-local-linux-operator/1"
          and r.get("status") == "FAIL_OR_UNKNOWN"
          and r.get("executionKind") == "local_linux_diagnostic"
          and r.get("executionId") == FINAL_NATIVE_EXECUTION_ID
          and r.get("campaignId") == FINAL_NATIVE_CAMPAIGN_ID
          and r.get("campaignCompleted") is False
          and r.get("campaignExitCode") == 2
          and r.get("hostedQualificationClaim") is False
          and r.get("nativeQualificationClaim") is False
          and r.get("hosted") is not True,
          "native007 must remain the exact failed local campaign")
    campaign = r.get("campaignResult")
    _need(isinstance(campaign, dict)
          and campaign == {"elapsedSeconds": 844.566,
              "path": str(FINAL_NATIVE_OUTPUT_ROOT / "campaign/full14-result.json"),
              "rawLogBytes": 6389, "scenarioDispatchCount": 19,
              "sha256": FINAL_NATIVE_CAMPAIGN, "sizeBytes": 2325196,
              "status": "FAIL_OR_UNKNOWN"}
          and r.get("campaignResultPath") == campaign["path"]
          and r.get("scenarioDispatchCount") == 19
          and r.get("inputsStableAfterRun") is True
          and r.get("captureFailed") is False
          and r.get("failure", {}).get("type") is not None,
          "native007 original campaign failure/result identity changed")
    _need(r.get("sourceSeal") == {
              "manifestSha256": FINAL_PREP_SOURCE_SEAL,
              "profile": "umf.g03-reviewed-harness-source-subset/2",
              "root": "/private/tmp/g03-full14-source-seal-005"}
          and r.get("cohort") == {
              "aggregateIndexSha256": FINAL_COHORT_INDEX,
              "custodySha256": FINAL_COHORT_CUSTODY,
              "mirrorManifestSha256": FINAL_MIRROR,
              "requirementsLockSha256": FINAL_REQUIREMENTS,
              "root": "/private/tmp/g03-final-cohort-056-linux311-001",
              "selectedLockSha256": FINAL_COHORT_LOCK,
              "weftWheel": {"path": "aggregate/mirror/files/"
                  "umf_weft-0.4.56-cp39-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl",
                  "sha256": FINAL_WEFT_WHEEL, "sizeBytes": FINAL_WEFT_WHEEL_BYTES}}
          and r.get("images", {}).get("client") == {
              "id": "sha256:7e44ba6b5c483f84c0d630180ba4cd17a82f1bba950577bf1dc9200e7f26e556",
              "platform": "linux/amd64"}
          and r.get("images", {}).get("postgres") == {
              "id": "sha256:74935e72241653ca55e0414067e6d8763aceb8a810eb51b452253ec3dcfc4336",
              "platform": "linux/arm64"}
          and r.get("postgresRuntime", {}).get("shortVersion") ==
              "18.6 (Debian 18.6-1.pgdg13+2)"
          and r.get("postgresRuntime", {}).get("serverVersionNum") == "180006"
          and r.get("postgresRuntime", {}).get("platform") == "linux/arm64"
          and r.get("postgresRuntime", {}).get("publishedPorts") is False,
          "native007 source/cohort/image/PostgreSQL target differs from its accepted profile")
    _need(r.get("limits") == {"aggregateSeconds": 1800,
              "evidenceBytes": MAX_EVIDENCE_TOTAL,
              "guestRawLogBytes": 786432, "hostRawLogBytes": 262144,
              "perScenarioChildSeconds": 180, "perStreamRetainedBytes": 65536,
              "rawLogBytes": MAX_RAW_LOGS, "terminalReserveSeconds": 60,
              "terminationGraceSeconds": 10}
          and type(r.get("elapsedSeconds")) in {int, float}
          and 0 <= r["elapsedSeconds"] < 1800
          and r.get("physicalEvidenceFileCount") == 181
          and r.get("physicalEvidenceBytes") == 4341728
          and r.get("physicalLogBytes") == 9008
          and r.get("outerRawLogBytes") == 2619
          and r.get("combinedRawLogBytes") == 2619
          and r.get("evidenceBytesBeforeResult") == 4341728,
          "native007 actual elapsed/evidence/log bounds differ")
    cleanup = r.get("cleanup")
    _need(isinstance(cleanup, dict) and cleanup.get("status") == "PASS"
          and cleanup.get("errors") == []
          and cleanup.get("elapsedSeconds") == r.get("elapsedSeconds")
          and isinstance(cleanup.get("resources"), dict)
          and set(cleanup["resources"]) == {
              f"g03-f14-campaign-{FINAL_NATIVE_EXECUTION_ID}",
              f"g03-f14-net-{FINAL_NATIVE_EXECUTION_ID}",
              f"g03-f14-pg-{FINAL_NATIVE_EXECUTION_ID}",
              f"g03-f14-pgdata-{FINAL_NATIVE_EXECUTION_ID}",
              f"g03-f14-setup-{FINAL_NATIVE_EXECUTION_ID}",
              f"g03-f14-venv-{FINAL_NATIVE_EXECUTION_ID}"},
          "native007 owned-resource cleanup is not closed")
    commands = r.get("commands")
    _need(isinstance(commands, list) and len(commands) == 51
          and [c.get("name") for c in commands if c.get("exitCode") != 0]
              == ["postgres-readiness", "guest-campaign"]
          and all(c.get("timedOut") is False and c.get("drainExpired") is False
                  and c.get("logCapExceeded") is False
                  and type(c.get("elapsedSeconds")) in {int, float}
                  and 0 <= c["elapsedSeconds"] <= c.get("timeoutSeconds", -1)
                  for c in commands),
          "native007 command failures/captures exceed or differ from the reviewed run")
    _need(derived.get("profile") == FINAL_NATIVE_REANALYSIS_PROFILE
          and derived.get("status") == "PREDICATE_REANALYSIS_PASS"
          and derived.get("executionKind") == "metadata_only_pure_predicate_reanalysis"
          and derived.get("scope") ==
              "Reinterprets only the immutable actual007 raw corpus; original run stays FAIL_OR_UNKNOWN; no DB, product import, runner, or live observation."
          and derived.get("originalFailuresPreserved") is True
          and derived.get("elapsedSeconds") == 0.116
          and derived.get("originalPredicateReport", {}).get("status") == "FAIL_OR_UNKNOWN",
          "native predicate reanalysis changed scope or erased the original failure")
    expected_run = {"campaignExitCode": 2, "campaignId": FINAL_NATIVE_CAMPAIGN_ID,
        "campaignResult": {"path": str(FINAL_NATIVE_OUTPUT_ROOT / "campaign/full14-result.json"),
            "sha256": FINAL_NATIVE_CAMPAIGN, "sizeBytes": 2325196},
        "campaignStatus": "FAIL_OR_UNKNOWN", "executionId": FINAL_NATIVE_EXECUTION_ID,
        "expectedData": {"path": str(FINAL_NATIVE_OUTPUT_ROOT / "campaign/campaign-expected-data.json"),
            "sha256": "d8d66928cc3090312c93a13761084f21a1dd8dc66b807650c9e5e39f23dc8460",
            "sizeBytes": 87149},
        "inputPacket": {"path": "/private/tmp/g03-full14-input-packet-001/inputs.json",
            "sha256": "c91b7b4d688a51c95c09cdd51ed5084899aba8adaa83c9b7f6b48a3c48d7478b",
            "sizeBytes": 76566},
        "operatorResult": {"path": str(FINAL_NATIVE_OUTPUT_ROOT / "operator-result.json"),
            "sha256": FINAL_NATIVE_OPERATOR, "sizeBytes": 115374},
        "scenarioDispatchCount": 19,
        "sourceSealManifest": {"path": "/private/tmp/g03-full14-source-seal-005/manifest.json",
            "sha256": FINAL_PREP_SOURCE_SEAL}, "status": "FAIL_OR_UNKNOWN"}
    _need(derived.get("originalRun") == expected_run,
          "derived predicates do not bind the exact immutable native007 failure chain")
    predicates = derived.get("derivedPredicateResult")
    expected_cases = [f"G03-N{i:02d}" for i in range(1, 15)]
    _need(isinstance(predicates, dict) and predicates.get("status") == "PASS"
          and predicates.get("predicatePassed") is True
          and predicates.get("exactRoster") is True
          and predicates.get("casePredicates") == [
              {"caseId": case, "missingRequirements": [],
               "predicatePassed": True, "status": "PASS"} for case in expected_cases],
          "reviewed derived native predicate roster is not exact N01-N14 PASS")
    custody = derived.get("custody")
    _need(isinstance(custody, dict)
          and custody.get("outerCleanupStatus") == "PASS"
          and custody.get("setupInventoryUnchanged") is True
          and custody.get("physicalEvidenceFileCount") == 181
          and custody.get("physicalEvidenceBytes") == 4341728
          and custody.get("physicalLogBytes") == 9008
          and custody.get("allRawLogBytes") == 9008
          and custody.get("outerRawLogBytes") == 2619
          and custody.get("campaignRawLogBytes") == 954
          and custody.get("setupRawLogBytes") == 5435
          and custody.get("rawScenarioCount") == 19
          and custody.get("sourceSealFileCount") == 8,
          "derived native custody counts differ from the independently reviewed raw corpus")
    _need(derived_path.is_file(),
          "derived reanalysis must be the bounded packet receipt")


def _validate_gate_details(name: str, r: dict[str, Any], *, root: Path,
                           receipt_path: Path,
                           reanalysis: dict[str, Any] | None = None) -> None:
    """Check the finite facts that distinguish each required actual gate."""
    if name == "linux-inherited-boundaries":
        _validate_table012_details(r, receipt_path)
        return
    if name in {"linux-browser", "linux-wasm"}:
        _validate_truss011_details(r, receipt_path)
        return
    _need(r.get("hosted") is not True and r.get("workflowRunId") is None
          and r.get("hostedWorkflowIdentity") is None,
          f"gate {name} fabricates a hosted identity")
    if name == "linux-x86_64-final-installed-admission":
        guest = r.get("guestResult")
        _need(r.get("profile") == "umf.g03-final-installed-prep-operator/1"
              and r.get("executionKind") == "local_linux_diagnostic"
              and r.get("platform") == "linux/amd64"
              and r.get("image") == "sha256:7e44ba6b5c483f84c0d630180ba4cd17a82f1bba950577bf1dc9200e7f26e556"
              and r.get("producerVolume") == "umf-migration-source-267675510"
              and r.get("limits") == {"aggregateSeconds": 600, "commandSeconds": 180,
                  "evidenceBytes": MAX_EVIDENCE_TOTAL, "logsBytes": MAX_RAW_LOGS,
                  "reserveSeconds": 60, "stopGraceSeconds": 10}
              and type(r.get("elapsedSeconds")) in {int, float}
              and 0 <= r["elapsedSeconds"] < 600,
              "installed admission operator identity/resource/deadline differs")
        _need(isinstance(guest, dict)
              and guest.get("profile") == "umf.g03-final-installed-prep-guest/1"
              and guest.get("status") == "PASS_INSTALLED_SNAPSHOT_AND_COMPILER_ADMISSION_ONLY"
              and guest.get("producer") == {
                  "commit": UMF_GATE_SOURCE, "tree": UMF_GATE_TREE, "mount": "read-only"}
              and guest.get("cohort", {}).get("selectedLockSha256") == FINAL_COHORT_LOCK
              and guest.get("cohort", {}).get("aggregateIndexSha256") == FINAL_COHORT_INDEX
              and guest.get("cohort", {}).get("custodySha256") == FINAL_COHORT_CUSTODY
              and guest.get("cohort", {}).get("mirrorManifestSha256") == FINAL_MIRROR
              and guest.get("cohort", {}).get("requirementsLockSha256") == FINAL_REQUIREMENTS
              and guest.get("cohort", {}).get("weftSourceCommit") == PRODUCER
              and guest.get("cohort", {}).get("weftSourceTree") == PRODUCER_TREE
              and guest.get("cohort", {}).get("weftWheelSha256") == FINAL_WEFT_WHEEL
              and guest.get("cohort", {}).get("weftWheelSizeBytes") == FINAL_WEFT_WHEEL_BYTES,
              "installed admission is not bound to the final 18-wheel cohort")
        _need(guest.get("admissionStatus") == "PASS"
              and guest.get("failure") is None
              and guest.get("admissionSource") == {"path": "/probe/admission_controls.py",
                                                     "sha256": FINAL_ADMISSION_SOURCE}
              and guest.get("runtimeVersions") == {"python": "3.11.17", "bun": "1.4.3"}
              and guest.get("sourceSeal", {}).get("manifestSha256") == FINAL_PREP_SOURCE_SEAL
              and guest.get("sourceSeal", {}).get("runnerSha256") == FINAL_PREP_RUNNER,
              "installed admission runtime/source/probe identity differs")
        snapshot = guest.get("installedSnapshot")
        _need(isinstance(snapshot, dict)
              and snapshot.get("profile") == "umf.g03-installed-cohort-snapshot/1"
              and snapshot.get("canonical") is True
              and snapshot.get("runnerSha256") == FINAL_PREP_RUNNER
              and snapshot.get("snapshotSha256") == FINAL_INSTALLED_SNAPSHOT,
              "installed environment snapshot is not the accepted canonical snapshot")
        snapshot_path = receipt_path.parent / "guest/installed-cohort-snapshot.json"
        snapshot_raw = _read_bounded(snapshot_path, MAX_EVIDENCE_FILE,
                                     "installed environment snapshot")
        _need(len(snapshot_raw) == snapshot.get("fileSizeBytes")
              and _sha(snapshot_raw) == snapshot.get("fileSha256")
              and len(snapshot_raw) == snapshot.get("snapshotBytes") + 1
              and snapshot_raw.endswith(b"\n"),
              "installed snapshot file size/hash closure differs")
        try:
            snapshot_doc = json.loads(snapshot_raw)
        except (UnicodeError, json.JSONDecodeError) as exc:
            raise ProfileError("installed cohort snapshot is invalid JSON") from exc
        canonical_snapshot = json.dumps(snapshot_doc, sort_keys=True,
                                        separators=(",", ":")).encode("utf-8")
        _need(canonical_snapshot == snapshot_raw[:-1]
              and _sha(canonical_snapshot) == snapshot["snapshotSha256"],
              "installed cohort snapshot is not the exact canonical JSON payload")
        proof = guest.get("wheelMemberProof")
        _need(isinstance(proof, dict) and proof.get("wheelSha256") == FINAL_WEFT_WHEEL
              and proof.get("wheelSizeBytes") == FINAL_WEFT_WHEEL_BYTES
              and proof.get("memberCount") == 2 and proof.get("memberRosterExact") is True
              and proof.get("members") == [
                  {"wheelPath": "weft/__init__.py", "sizeBytes": 99,
                   "sha256": WRAPPER_SHA256},
                  {"wheelPath": "weft/weft.abi3.so", "sizeBytes": FINAL_WEFT_NATIVE_BYTES,
                   "sha256": FINAL_WEFT_NATIVE}],
              "installed native wheel-member bytes differ from accepted x64 wheel")
        cases = guest.get("admissionCases")
        _need(isinstance(cases, list) and len(cases) == 5
              and [x.get("caseId") for x in cases] == [
                  "baseline-absent-envelope-01", "matching-logical5-envelope-02",
                  "matching-logical5-legacy-envelope-01", "native-only-set5-envelope-02",
                  "logical9-native5-mismatch-envelope-02"]
              and [x.get("status") for x in cases] == ["ACCEPTED", "ACCEPTED", "REFUSED", "ACCEPTED", "REFUSED"]
              and all(x.get("leaseAttempts") == 0 for x in cases)
              and cases[0].get("programSha256") == "c8be43e133c5869b2ac01c520e92177d620b558c326cc8d6aa6b087454ecaf5e",
              "installed admission controls/lease boundary differ")
        commands = guest.get("commandResults")
        expected_commands = ["runtime-python", "runtime-bun", "source-commit", "source-tree",
                             "source-clean-status", "offline-install-and-pip-check",
                             "installed-cohort-snapshot", "fixture-generation",
                             "five-compiler-admission-controls"]
        _need(isinstance(commands, list) and [c.get("name") for c in commands] == expected_commands,
              "installed admission command roster differs")
        retained = {row["path"]: row for row in r["outputFiles"]}
        for command in commands:
            _need(command.get("exitCode") == 0 and command.get("timedOut") is False
                  and command.get("failure") is None and command.get("timeoutSeconds") == 180
                  and type(command.get("elapsedSeconds")) in {int, float}
                  and 0 <= command["elapsedSeconds"] <= 180,
                  "installed admission command failed or exceeded its limit")
            streams = command.get("streams")
            _need(isinstance(streams, list) and len(streams) == 2,
                  "installed admission command lacks both captured streams")
            for stream in streams:
                physical = retained.get("guest/" + stream.get("path", ""))
                _need(physical is not None and stream.get("physicalClosure") is True
                      and stream.get("truncated") is False
                      and stream.get("rawBytes") == stream.get("retainedBytes")
                      and stream.get("rawSha256") == stream.get("physicalSha256")
                      and physical["sizeBytes"] == stream.get("retainedBytes")
                      and physical["sha256"] == stream.get("physicalSha256"),
                      "installed admission command log does not close to retained bytes")
        guest_files = guest.get("outputFiles")
        _need(isinstance(guest_files, list) and len(guest_files) >= 20,
              "installed admission guest evidence roster is incomplete")
        for file_row in guest_files:
            physical = retained.get("guest/" + file_row.get("path", ""))
            _need(physical is not None and physical["sizeBytes"] == file_row.get("sizeBytes")
                  and physical["sha256"] == file_row.get("sha256"),
                  "installed admission guest export differs from outer physical evidence")
        _need(r.get("cleanup", {}).get("verified") is True,
              "installed prep resource cleanup is not verified")
    elif name == "linux-source-package":
        source = r.get("source", {})
        identity = source.get("identity", {})
        runtime = r.get("runtime", {})
        toolchain = runtime.get("toolchain", {})
        deps = r.get("javascriptDependencies", {})
        before = deps.get("before", {})
        after = deps.get("after", {})
        release = r.get("releaseMetadata", {})
        _need(r.get("profile") == "umf.g03-local-source-package-guards/2"
              and r.get("status") == "LOCAL_BOUNDED_SOURCE_PACKAGE_GUARDS_PASS"
              and r.get("failure") is None and source.get("equalityStatus") == "VERIFIED"
              and identity.get("head") == PRODUCER and identity.get("tree") == PRODUCER_TREE
              and identity.get("trackedCount") == 5901
              and source.get("producerCheckoutTouched") is False
              and source.get("containerAbsenceVerified") is True,
              "source/package guard does not bind the clean P56 producer input")
        _need(runtime.get("image") == "sha256:1f983eafda3fd12f069837f7e972179ec3b8d74587190751db85b555b3430c10"
              and runtime.get("imageId") == runtime.get("image")
              and runtime.get("platform") == "linux/amd64"
              and runtime.get("pythonQualification") ==
                  "3.11.17 explicit replacement for requested 3.12; narrower evidence"
              and toolchain.get("PYTHON_VERSION") == "Python 3.11.17"
              and toolchain.get("BUN_VERSION") == "1.4.3"
              and toolchain.get("RUSTC_VERSION", "").startswith("rustc 1.90.0 ")
              and toolchain.get("CARGO_VERSION", "").startswith("cargo 1.90.0 "),
              "source/package guard environment is outside its recorded narrower profile")
        _need(before == after
              and before.get("sourceBunLockSha256") ==
                  "28b740f61429640ea5c005837d37b5bf7abf67b414c8139659cccc0daee6071a"
              and before.get("sourcePackageJsonSha256") ==
                  "dd22198e0de80eaab805fae19db057668fcf3c180a84c142d74318c541ad6455"
              and before.get("inventory", {}).get("fileBytes") == 44271120
              and before.get("inventory", {}).get("fileCount") == 1081
              and before.get("inventory", {}).get("rosterSha256") ==
                  "b4f2331b8c4d85dadaa2d361d2b1dd89a27a0ebfdee7885f94c94ba6af5fd5f7",
              "source/package JavaScript dependency inventory changed or is unbound")
        _need(release.get("version") == VERSION and release.get("cargoVersion") == VERSION
              and release.get("projectName") == DISTRIBUTION
              and release.get("publicationTagRequirement") ==
                  "v0.4.56 (not checked by source-only guard)",
              "source/package release-metadata guard is not scoped to the actual candidate")
        processes = r["processes"]
        _need(sum(row.get("phase") == "source-command" for row in processes) == 20
              and sum(row.get("phase") == "release-metadata" for row in processes) == 1
              and all(type(row.get("effectiveDeadlineSeconds")) in {int, float}
                      and type(row.get("durationSeconds")) in {int, float}
                      and 0 <= row["durationSeconds"] <= row["effectiveDeadlineSeconds"]
                      for row in processes),
              "source/package process roster or per-command deadlines differ")
        cleanup_rows = [row for row in processes if row.get("phase") in {
            "container-list-before-remove", "container-owner-before-remove",
            "remove-owned-container", "verify-container-absence",
            "volume-list-before-remove", "volume-owner-before-remove",
            "remove-owned-volume", "verify-volume-absence"}]
        cleanup = r.get("cleanup", {})
        _need(cleanup_rows and max(row["startedMonotonic"] + row["durationSeconds"]
                                  for row in cleanup_rows)
              - min(row["startedMonotonic"] for row in cleanup_rows) < 30
              and cleanup.get("errors") == []
              and cleanup.get("volumeAbsenceVerified") == {
                  "g03-source-cargo-5cfb93d1-07c": True,
                  "g03-source-target-5cfb93d1-07c": True},
              "source/package owned-resource cleanup evidence is not closed")
    elif name == "linux-x86_64-final-native-campaign":
        if r.get("profile") == "umf.g03-full14-local-linux-operator/1":
            _need(reanalysis is not None,
                  "native campaign qualification requires separate reviewed predicate reanalysis")
        else:
            _need(r.get("status") == "PASS",
                  "synthetic test gate status is not accepted")
    elif name.endswith("build-static"):
        if name == "linux-aarch64-build-static":
            _need(r.get("source", {}).get("head") == PRODUCER
                  and r.get("source", {}).get("tree") == PRODUCER_TREE
                  and r.get("candidateWheel", {}).get("memberCount") == 9,
                  "ARM static build is not bound to P56 and the exact wheel")
        elif name == "macos-arm64-build-static":
            _need(r.get("profile") == MAC_AUDIT_PROFILE
                  and r.get("status") == "LOCAL_MACOS_STATIC_AUDIT_ONLY_PASS"
                  and r.get("hosted") is False and r.get("installed") is False
                  and r.get("imported") is False and r.get("nativeExecuted") is False
                  and r.get("browserParity") is False and r.get("compilerParity") is False
                  and r.get("functionalQualification") == "NOT_RUN"
                  and r.get("failure") is None,
                  "Mac gate overclaims beyond its static-only audit")
        else:
            _need(r.get("candidateInstall") is False and r.get("candidateImport") is False
                  and r.get("candidateExecution") is False,
                  f"{name} overclaims static-only evidence")
    else:
        _need(r.get("status") == "PASS", f"{name} did not pass")
        commands = r.get("commands")
        _need(isinstance(commands, list) and commands
              and all(isinstance(c, dict) and c.get("exitCode") == 0
                      and c.get("timedOut") is False and c.get("outputCapped") is False
                      for c in commands), f"{name} command roster is not clean")
    cleanup = r.get("cleanup")
    source_guard_cleanup = (name == "linux-source-package" and isinstance(cleanup, dict)
                            and cleanup.get("errors") == []
                            and cleanup.get("volumeAbsenceVerified")
                            == {"g03-source-cargo-5cfb93d1-07c": True,
                                "g03-source-target-5cfb93d1-07c": True})
    static_cleanup = (
        name == "linux-aarch64-build-static" and isinstance(cleanup, dict)
        and cleanup.get("errors") == []
        or name == "macos-arm64-build-static" and r.get("cleanupErrors") == []
        and r.get("closureErrors") == [])
    _need(r.get("cleanupVerified") is True or source_guard_cleanup
          or static_cleanup
          or isinstance(cleanup, dict) and (cleanup.get("verified") is True
                                               or cleanup.get("status") == "PASS"),
          f"{name} cleanup is not verified")


def _validate_table012_details(r: dict[str, Any], receipt_path: Path) -> None:
    _need(r.get("profile") == TABLE012_PROFILE and r.get("status") == TABLE012_STATUS
          and r.get("executionId") == TABLE012_EXECUTION_ID
          and r.get("operatorResultBytes") == TABLE012_RECEIPT_BYTES
          and r.get("candidateStatus") == TABLE012_STATUS
          and r.get("failure") is None and r.get("elapsedSeconds") == 246.421
          and r.get("evidenceClosureVerified") is True
          and r.get("source") == {"commit": PRODUCER, "tree": PRODUCER_TREE}
          and r.get("scope", {}).get("lane") == "table-operations"
          and r.get("scope", {}).get("trussFactsAndEntitySelection") is False
          and r.get("scope", {}).get("trussWasmProfile") is False
          and r.get("limits", {}).get("inheritedTotalSeconds") == 2700
          and r.get("limits", {}).get("terminalReserveSeconds") == 60
          and r.get("limits", {}).get("terminationGraceSeconds") == 10
          and r.get("limits", {}).get("publicEvidenceBytes") == 8 * 1024 * 1024
          and r.get("limits", {}).get("publicLogBytes") == MAX_RAW_LOGS
          and r.get("limits", {}).get("selectedArtifactBytes") == MAX_SELECTED_SOFTWARE
          and r.get("publicBytes") == TABLE012_PUBLIC_BYTES
          and r.get("selectedArtifactBytes") == TABLE012_SELECTED_BYTES,
          "Table012 identity, status, scope, or original resource limits differ")
    _need(r.get("hostSourceIdentity") == {
        "before": {"commit": PRODUCER, "tree": PRODUCER_TREE, "status": "",
                   "identityBasis": "bounded_host_git_metadata"},
        "after": {"commit": PRODUCER, "tree": PRODUCER_TREE, "status": "",
                  "identityBasis": "bounded_host_git_metadata"}},
        "Table012 host P56 identity is dirty or changed")
    custody = r.get("sourceCustody")
    _need(isinstance(custody, dict) and custody.get("fileCount") == 5901
          and custody.get("totalBytes") == 860543639
          and custody.get("stageBasis", {}).get("producerCommit") == PRODUCER
          and custody.get("stageBasis", {}).get("producerTree") == PRODUCER_TREE
          and custody.get("stageBasis", {}).get("fileCount") == 5901
          and custody.get("stageBasis", {}).get("bytes") == 780359
          and custody.get("stageBasis", {}).get("sha256") == SOURCE_IDENTITY["gitStageRosterSha256"],
          "Table012 source/stage custody differs from P56")
    prep, native = r.get("prepareResult"), r.get("nativeWasmResult")
    _need(isinstance(prep, dict) and prep.get("phase") == "prepare"
          and prep.get("profile") == "umf.g03-table-operations-phase/1"
          and prep.get("status") == "PREPARE_PASS" and prep.get("failure") is None
          and prep.get("evidenceClosureVerified") is True
          and prep.get("logBytes") == 15815
          and prep.get("publicBytes") == 982819
          and isinstance(native, dict) and native.get("phase") == "native-wasm"
          and native.get("profile") == "umf.g03-table-operations-phase/1"
          and native.get("status") == "NATIVE_WASM_PASS" and native.get("failure") is None
          and native.get("evidenceClosureVerified") is True
          and native.get("logBytes") == 19950
          and native.get("publicBytes") == 983551
          and native.get("nativeAssertions") == 50
          and native.get("tableWasm", {}).get("featureProfile") == ["raw-postgresql-reads"],
          "Table012 prepare/native-WASM phase receipts differ")
    phase_closure = r.get("phaseReceiptPhysicalClosure")
    _need(isinstance(phase_closure, dict)
          and phase_closure.get("prepare", {}).get("status") == "PREPARE_PASS"
          and phase_closure.get("prepare", {}).get("fileCount") == 38
          and phase_closure.get("nativeWasm", {}).get("status") == "NATIVE_WASM_PASS"
          and phase_closure.get("nativeWasm", {}).get("fileCount") == 13,
          "Table012 phase receipt physical closure differs")
    for phase, expected_commands in ((prep, 16), (native, 3)):
        phase_commands = phase.get("commands")
        _need(isinstance(phase_commands, list) and len(phase_commands) == expected_commands,
              "Table012 prepare/native command roster differs")
        for command in phase_commands:
            _need(isinstance(command, dict) and command.get("exitCode") == 0
                  and command.get("failure") is None and command.get("timedOut") is False
                  and command.get("terminationGraceIncludedInOriginalCap") == 10
                  and isinstance(command.get("streams"), list)
                  and len(command["streams"]) == 2,
                  "Table012 phase command failed or lost a capture stream")
            for stream in command["streams"]:
                _need(isinstance(stream, dict) and stream.get("physicalClosed") is True
                      and stream.get("truncated") is False
                      and stream.get("bytes") == stream.get("rawBytes")
                      and stream.get("sha256") == stream.get("rawSha256"),
                      "Table012 phase command capture is truncated or unclosed")
                rel = stream.get("path")
                _canonical_relative(rel, "Table012 phase command")
                expected_path = Path(rel)
                public_rel = f"{phase.get('phase')}/{rel}"
                row = next((row for row in r["publicFiles"] if row["path"] == public_rel), None)
                _need(row is not None and row["sizeBytes"] == stream["bytes"]
                      and row["sha256"] == stream["sha256"],
                      "Table012 phase command capture does not bind a public file")
    captures = r.get("metadataCommands")
    _need(isinstance(captures, list) and len(captures) == 97,
          "Table012 host metadata command roster differs")
    for command in captures:
        _need(isinstance(command, dict) and command.get("captureComplete") is True
              and command.get("timedOut") is False and command.get("truncated") is False
              and command.get("streamsEOF") == [True, True]
              and isinstance(command.get("physicalCaptureFiles"), list)
              and len(command["physicalCaptureFiles"]) == 2,
              "Table012 host metadata capture is incomplete")
        for stream_row in command["physicalCaptureFiles"]:
            rel = stream_row.get("path")
            _need(isinstance(rel, str) and rel in {row["path"] for row in r["publicFiles"]}
                  and next((row for row in r["publicFiles"] if row["path"] == rel), {}).get("sizeBytes")
                      == stream_row.get("sizeBytes")
                  and next((row for row in r["publicFiles"] if row["path"] == rel), {}).get("sha256")
                      == stream_row.get("sha256"),
                  "Table012 host metadata capture does not bind a public file")
    command_captures = r.get("commandCaptures")
    expected_capture_names = {
        f"g03-table-{suffix}-{TABLE012_EXECUTION_ID}"
        for suffix in ("browser-identity", "browser-identity-after", "browser-oracle",
                       "native-wasm", "prepare", "pg")
    }
    _need(isinstance(command_captures, dict)
          and set(command_captures) == expected_capture_names,
          "Table012 five bounded command captures and PostgreSQL observation differ")
    for name, value in command_captures.items():
        if "-pg-" in name:
            version = value.get("postgresVersion") if isinstance(value, dict) else None
            _need(isinstance(version, dict)
                  and version.get("serverVersion", "").startswith("18.6 ")
                  and version.get("serverVersionNum") == "180006"
                  and isinstance(value.get("identity"), dict)
                  and isinstance(value.get("mounts"), list),
                  "Table012 PostgreSQL 18.6 readiness observation differs")
            continue
        cap = value.get("capture") if isinstance(value, dict) else None
        _need(isinstance(cap, dict) and cap.get("captureErrors") == []
              and cap.get("captureFailed") is False and cap.get("timedOut") is False
              and cap.get("drainExpired") is False and cap.get("streamsEOF") == [True, True]
              and cap.get("truncated") == [False, False]
              and value.get("physicalClosure", {}).get("verified") is True
              and value.get("physicalClosure", {}).get("errors") == []
              and len(value.get("physicalClosure", {}).get("files", [])) == 2,
              "Table012 outer process capture is incomplete")
        closure = value.get("physicalClosure")
        _need(isinstance(closure, dict) and closure.get("verified") is True
              and closure.get("errors") == [] and isinstance(closure.get("files"), list)
              and len(closure["files"]) == 2,
              "Table012 outer capture physical closure is incomplete")
        for rel, raw_size, raw_sha in zip(cap["paths"], cap["rawBytes"], cap["rawSha256"], strict=True):
            relative = _historical_relative(rel, TABLE012_ROOT, "Table012 outer capture")
            row = next((row for row in r["publicFiles"] if row["path"] == relative), None)
            _need(row is not None and row.get("sizeBytes") == raw_size
                  and row.get("sha256") == raw_sha,
                  "Table012 outer capture bytes differ from the public metadata roster")
        closure_paths = {_historical_relative(item["path"], TABLE012_ROOT,
                                              "Table012 outer physical closure")
                         for item in closure["files"]}
        capture_paths = {_historical_relative(path, TABLE012_ROOT,
                                              "Table012 outer command capture")
                         for path in cap["paths"]}
        _need(closure_paths == capture_paths,
              "Table012 outer physical closure paths differ from command capture paths")
    raw_logs = sum(row["sizeBytes"] for row in r["publicFiles"]
                   if Path(row["path"]).name.endswith((".log", ".stdout", ".stderr")))
    _need(raw_logs == 50486 and raw_logs <= MAX_RAW_LOGS,
          "Table012 public raw capture total differs or exceeds its original cap")
    cleanup = r.get("cleanup")
    _need(isinstance(cleanup, dict) and cleanup.get("verified") is True
          and cleanup.get("errors", []) == [] and cleanup.get("daemonReachable") is True
          and cleanup.get("networkAbsent") is True and cleanup.get("networkRemoved") is True
          and all(isinstance(row, dict) and row.get("absent") is True
                  and row.get("removeExit") == 0 for row in cleanup.get("containers", {}).values())
          and len(cleanup.get("containers", {})) == 6
          and all(isinstance(row, dict) and row.get("absent") is True
                  and row.get("removeExit") == 0 for row in cleanup.get("volumes", {}).values())
          and len(cleanup.get("volumes", {})) == 4,
          "Table012 owned container/volume/network cleanup is not fully closed")


def _validate_truss011_details(r: dict[str, Any], receipt_path: Path) -> None:
    _need(r.get("profile") == TRUSS011_PROFILE and r.get("status") == "PASS"
          and r.get("executionId") == TRUSS011_EXECUTION_ID and r.get("failures") == []
          and r.get("elapsedSeconds") == 163.236
          and r.get("ownedContainersAbsent") is True and r.get("ownedScratchAbsent") is True,
          "Truss011 original receipt identity, terminal status, or cleanup differs")
    guest = r.get("guestPhase")
    _need(isinstance(guest, dict) and guest.get("status") == "PASS"
          and guest.get("commandCount") == 20 and isinstance(guest.get("commands"), list)
          and len(guest["commands"]) == 20 and guest.get("postSnapshotEqual") is True
          and guest.get("sourceFileCount") == 5901 and guest.get("sourceBytes") == 860543639
          and guest.get("gitStageRoster", {}).get("matchingPhysicalFiles") == 5901
          and guest.get("gitStageRoster", {}).get("sha256") == SOURCE_IDENTITY["gitStageRosterSha256"]
          and guest.get("commandLogBytes") == 29404
          and guest.get("commandLogBytes") <= 512 * 1024
          and guest.get("wasmFeatureProfile") == ["truss-postgresql-pg18"],
          "Truss011 guest source/install/WASM/capture identity differs")
    for command in guest["commands"]:
        record = command if isinstance(command, dict) else None
        _need(isinstance(record, dict) and record.get("exitCode") == 0
              and record.get("timedOut") is False and record.get("drainExpired") is False
              and record.get("truncated") is False and record.get("captureErrors") == []
              and record.get("streamsEOF") == [True, True],
              "Truss011 guest command was not captured cleanly")
    _need(r.get("sourcePost", {}).get("fileCount") == 5901
          and r.get("sourcePost", {}).get("totalBytes") == 860543639
          and r.get("sourcePost", {}).get("rosterSha256") == "b410647a6bdded7fbfce9c72aee66575dc67da8305df7fea3dcb8206cffac19d"
          and r.get("sourcePost", {}).get("gitStageRoster", {}).get("sha256") == SOURCE_IDENTITY["gitStageRosterSha256"]
          and r.get("sourceArchiveBefore") == r.get("sourceArchiveAfter")
          and r.get("sourceArchiveBefore") == {"sha256": "8291da1e97ceb5ebd9cd29f0dab1ce233b14cd20651b1e840f7e3e59802ce77f",
                                                "sizeBytes": 865597440}
          and r.get("frozenSourceManifest") == r.get("frozenSourceManifestAfter")
          and r.get("portableJsBefore") == r.get("portableJsAfter")
          and r.get("browserIdentityBefore") == r.get("browserIdentityAfter")
          and r.get("selectedSoftwareArtifactsBeforeBrowser") == r.get("selectedSoftwareArtifactsAfterBrowser"),
          "Truss011 source, cache, portable dependency, browser, or software pre/post equality differs")
    cache = r.get("lockedCargoCacheInputBefore")
    _need(isinstance(cache, dict) and cache == r.get("lockedCargoCacheInputAfter")
          and cache.get("archiveSha256") == "b678cc11533ea60fcc2d0792092d835ba279de5ddc065f3bcc700581d0865fea"
          and cache.get("archiveBytes") == 44830720
          and cache.get("cacheContentBytes") == 44245301
          and cache.get("inventoryRowCount") == 833
          and cache.get("inventoryRowsSha256") == "c072486446d6081c82eade71500ee7cf6299699f6ab150f555c422da7b8d60a9"
          and r.get("cohortCustody") == {"custodySha256": "95e9493f06adefb323193f9d8fbe5577ebfa335140a9841f549d2caf098df8ed",
                                         "fileCount": 42, "totalBytes": 49129362},
          "Truss011 locked cache or final cohort custody differs")
    software_rows = [
        {"path": path.removeprefix("owned/out/artifacts/"),
         "sizeBytes": size, "sha256": digest}
        for path, (size, digest) in sorted(TRUSS011_SELECTED.items())]
    expected_selected = {"files": software_rows, "totalBytes": TRUSS011_SELECTED_BYTES}
    _need(r.get("selectedSoftwareArtifactsBeforeBrowser") == expected_selected
          and r.get("selectedSoftwareArtifactsAfterBrowser") == expected_selected,
          "Truss011 before/after selected-software manifests differ from exact five-file class")
    profiles = r.get("effectiveContainerProfiles")
    _need(isinstance(profiles, dict) and set(profiles) == {"entity", "facts"},
          "Truss011 browser effective profile roster differs")
    for lane, profile in profiles.items():
        _need(isinstance(profile, dict), f"Truss011 {lane} effective profile is malformed")
        expected_report = "entity.json" if lane == "entity" else "facts.json"
        expected_browser_out = f"{TRUSS011_ROOT}/browser-{lane}"
        mounts = {mount.get("Destination"): mount for mount in profile.get("mounts", [])
                  if isinstance(mount, dict)}
        expected_mounts = {
            "/browser-out": (expected_browser_out, True),
            "/truss-web": (str(TRUSS011_ROOT / "owned/out/artifacts/truss-web"), False),
            "/workspace": (str(TRUSS011_ROOT / "owned/source/p56"), False),
            "/wrapper": (str(TRUSS011_ROOT / "owned/out/artifacts/browser-wrapper"), False),
            "/reports/native.json": (str(TRUSS011_ROOT / "owned/out/reports" / expected_report), False),
            "/deps/node_modules/playwright": ("/Users/erik/Projects/weft/node_modules/playwright", False),
            "/deps/node_modules/playwright-core": ("/Users/erik/Projects/weft/node_modules/playwright-core", False),
        }
        _need(profile.get("status") == "exited"
              and profile.get("exitCode") == 0 and profile.get("network") == "none"
              and profile.get("readonlyRootfs") is True and profile.get("capDrop") == ["ALL"]
              and profile.get("securityOpt") == ["no-new-privileges"]
              and profile.get("memoryBytes") == 8589934592
              and profile.get("nanoCpus") == 4000000000 and profile.get("pids") == 8192
              and profile.get("shmBytes") == 1073741824
              and profile.get("tmpfs") == {"/tmp": "rw,nosuid,nodev,exec,size=256m"}
              and profile.get("ulimits") == [{"Hard": 268435456, "Name": "fsize", "Soft": 268435456}]
              and profile.get("labels", {}).get("codex.g03.execution-id") == TRUSS011_EXECUTION_ID
              and profile.get("imageId") == "sha256:eff16c30e6f3f4af0a03fa4b706120d5e9b0891c344a27d64559aff5900a4a27"
              and profile.get("labels", {}).get("codex.g03.kind") == "truss-browser"
              and profile.get("labels", {}).get("org.opencontainers.image.version") == "24.04"
              and set(mounts) == set(expected_mounts)
              and all(mounts[destination].get("Source") == source
                      and mounts[destination].get("RW") == writable
                      and mounts[destination].get("Type") == "bind"
                      and mounts[destination].get("Propagation") == "rprivate"
                      for destination, (source, writable) in expected_mounts.items()),
              f"Truss011 {lane} effective resource/security/mount profile differs")
    runtime = r.get("runtimeMetadata")
    expected_runtime = {"chromiumSha256": "839efe5fd8b6a773dd81b2e10afdc15f3c0a17316fb82b5908c0533306c4ed9e",
                        "chromiumVersion": "Google Chrome for Testing 153.0.8010.12",
                        "dfTmpLine": "tmpfs 262144 0 262144 0% /tmp",
                        "maxFileSizeHardBytes": 268435456,
                        "maxFileSizeSoftBytes": 268435456,
                        "nodeVersion": "v24.20.0",
                        "source": "captured Node/Chromium identity, /proc/self/limits and df -Pk /tmp"}
    _need(isinstance(runtime, dict) and runtime.get("facts") == expected_runtime
          and runtime.get("entity") == expected_runtime,
          "Truss011 runtime/process-limit/df metadata differs")
    custody = r.get("browserInputCustodyBeforeAfter")
    _need(isinstance(custody, dict) and set(custody) == {"facts", "entity"}
          and len(custody["facts"].get("caseIds", [])) == 5
          and len(custody["entity"].get("caseIds", [])) == 33,
          "Truss011 facts/entity browser case roster differs")
    expected_lanes = TRUSS011_BROWSER_EXPECTED
    for lane, expected in expected_lanes.items():
        facts = custody[lane]
        _need(all(facts.get(key) == value for key, value in expected.items())
              and facts.get("browserReceiptsSha256") == facts.get("postBrowserReceiptsSha256")
              and facts.get("browserSummarySha256") == facts.get("postBrowserSummarySha256")
              and facts.get("corpusSha256") == facts.get("postBrowserCorpusSha256")
              and facts.get("nativeReportSha256") == facts.get("postBrowserNativeReportSha256"),
              f"Truss011 {lane} browser/corpus/native pre-post custody differs")
        expected_count = 5 if lane == "facts" else 33
        browser_dir = receipt_path.parent / f"browser-{lane}"
        receipts_path, summary_path = browser_dir / "receipts.json", browser_dir / "summary.json"
        receipts_raw = _read_bounded(receipts_path, MAX_EVIDENCE_FILE,
                                     f"Truss011 {lane} browser receipts")
        summary_raw = _read_bounded(summary_path, MAX_EVIDENCE_FILE,
                                    f"Truss011 {lane} browser summary")
        _need(_sha(receipts_raw) == facts["browserReceiptsSha256"]
              and _sha(summary_raw) == facts["browserSummarySha256"],
              f"Truss011 {lane} browser files differ from captured custody")
        try:
            browser_receipt = json.loads(receipts_raw)
            browser_summary = json.loads(summary_raw)
        except (UnicodeError, json.JSONDecodeError) as exc:
            raise ProfileError(f"Truss011 {lane} browser receipt JSON is invalid") from exc
        case_rows = browser_receipt.get("cases") if isinstance(browser_receipt, dict) else None
        _need(isinstance(browser_summary, dict)
              and isinstance(case_rows, list) and len(case_rows) == expected_count
              and [item.get("id") for item in case_rows] == facts["caseIds"]
              and browser_summary.get("cases") == expected_count
              and browser_summary.get("nodeGlobals") is False
              and browser_summary.get("byteParity") is True
              and browser_summary.get("wasmSha256") == TRUSS011_SELECTED["owned/out/artifacts/truss-web/weft_wasm_bg.wasm"][1]
              and browser_summary.get("networkRequests") == ["http://weft.spike.test/",
                  "http://weft.spike.test/spike.wasm", "http://weft.spike.test/spike.js",
                  "http://weft.spike.test/wrapper.js"]
              and all(isinstance(case, dict)
                      and SHA.fullmatch(case.get("requestSha256", ""))
                      and SHA.fullmatch(case.get("expectedSha256", ""))
                      and case.get("expectedSha256") == case.get("actualSha256")
                      for case in case_rows),
              f"Truss011 {lane} browser result/raw request parity differs")
        report_path = receipt_path.parent / "owned/out/reports" / f"{lane}.json"
        report_raw = _read_bounded(report_path, MAX_EVIDENCE_FILE,
                                   f"Truss011 {lane} native report")
        _need(len(report_raw) == facts["nativeReportBytes"]
              and _sha(report_raw) == facts["nativeReportSha256"],
              f"Truss011 {lane} native report differs from browser custody")
        try:
            native_rows = json.loads(report_raw)
        except (UnicodeError, json.JSONDecodeError) as exc:
            raise ProfileError(f"Truss011 {lane} native report JSON is invalid") from exc
        _need(isinstance(native_rows, list) and len(native_rows) == expected_count
              and [item.get("id") for item in native_rows] == facts["caseIds"],
              f"Truss011 {lane} native report case count differs")
    commands = r.get("commands")
    _need(isinstance(commands, list) and len(commands) == 42,
          "Truss011 outer command capture roster differs")
    raw_outer = 0
    for command in commands:
        record = command.get("record") if isinstance(command, dict) else None
        _need(isinstance(record, dict), "Truss011 outer command record is malformed")
        # The native roster includes semantic container-identity observations as
        # well as captured processes. Only process records have capture fields.
        if "streams" not in record:
            base_identity = {"containerId", "containerName", "exitCode", "imageId",
                "memoryBytes", "mounts", "nanoCpus", "network", "pids",
                "readonlyRootfs", "status", "tmpfs"}
            extended_identity = base_identity | {"capDrop", "labels", "securityOpt", "shmBytes", "ulimits"}
            command_name = command.get("name")
            identity_fields = (extended_identity if command_name in {
                "browser-container-identity-facts", "browser-container-identity-entity"}
                else base_identity)
            _need(set(record) == identity_fields
                  and command_name in {"builder-container-identity", "browser-container-identity-before",
                      "browser-container-identity-facts", "browser-container-identity-entity",
                      "browser-container-identity-after"}
                  and record.get("exitCode") == 0
                  and isinstance(record.get("containerId"), str)
                  and re.fullmatch(r"[0-9a-f]{64}", record.get("containerId", ""))
                  and isinstance(record.get("containerName"), str)
                  and record.get("containerName", "").startswith("g03-truss-")
                  and isinstance(record.get("imageId"), str)
                  and re.fullmatch(r"sha256:[0-9a-f]{64}", record.get("imageId", ""))
                  and record.get("status") == "exited" and record.get("network") == "none"
                  and record.get("readonlyRootfs") is True
                  and isinstance(record.get("mounts"), list)
                  and type(record.get("memoryBytes")) is int and record["memoryBytes"] > 0
                  and type(record.get("nanoCpus")) is int and record["nanoCpus"] > 0
                  and type(record.get("pids")) is int and record["pids"] > 0
                  and isinstance(record.get("tmpfs"), dict),
                  "Truss011 non-capture container-identity observation is malformed")
            continue
        _need(record.get("exitCode") == 0 and record.get("timedOut") is False
              and record.get("drainExpired") is False and record.get("truncated") is False
              and record.get("captureErrors") == []
              and record.get("streamsEOF") == [True, True]
              and isinstance(record.get("streams"), list) and len(record["streams"]) == 2,
              "Truss011 outer command capture was incomplete")
        raw_outer += sum(stream.get("rawBytes", 0) for stream in record["streams"])
    output_by_path = {row["path"]: row for row in r.get("outputFiles", [])}
    _need(len(output_by_path) == 133,
          "Truss011 original output rows are not unique")
    for command in commands:
        record = command["record"]
        if "streams" not in record:
            continue
        for stream in record["streams"]:
            _crossbind_capture(TRUSS011_ROOT, output_by_path, stream,
                               where=f"Truss011 outer {command['name']}")
    for command in guest["commands"]:
        for stream in command["streams"]:
            _crossbind_capture(TRUSS011_ROOT, output_by_path, stream,
                               where=f"Truss011 guest {command['name']}", guest=True)
    for binding in r.get("physicalEvidenceBindings", []):
        relative = _historical_relative(binding.get("path"), TRUSS011_ROOT,
                                        "Truss011 physical evidence binding")
        row = output_by_path.get(relative)
        _need(row is not None and row.get("sizeBytes") == binding.get("sizeBytes")
              and row.get("sha256") == binding.get("sha256"),
              "Truss011 physical evidence binding differs from output roster")
    _need(raw_outer == 30536 and raw_outer + guest["commandLogBytes"] <= 512 * 1024,
          "Truss011 raw capture total differs or exceeds its original 512 KiB cap")


def _wheel(root: Path, row: Any, expected_role: str) -> dict[str, Any]:
    item = _closed(row, {"role", "path", "filename", "sizeBytes", "sha256", "staticReview",
                         "buildReceipt", "auditReceipt", "artifactReview", "producerEvidence"},
                   f"wheel {expected_role}")
    _need(item["role"] == expected_role, "wheel role order differs")
    filename = item["filename"]
    _need(isinstance(filename, str) and Path(filename).name == filename
          and filename.startswith("umf_weft-0.4.56-"), "wheel filename is not selected")
    _need(re.fullmatch(
        rf"umf_weft-0\.4\.56-cp39-abi3-{ROLES[expected_role]}\.whl",
        filename,
    ) is not None,
          f"wheel platform differs for {expected_role}")
    path = _under(root, item["path"], f"wheel {expected_role}")
    size, digest = _regular(path, MAX_WHEEL, f"wheel {expected_role}")
    _need(type(item["sizeBytes"]) is int and item["sizeBytes"] == size,
          f"wheel size differs for {expected_role}")
    _need(item["sha256"] == digest and isinstance(item["sha256"], str)
          and SHA.fullmatch(item["sha256"]), f"wheel digest differs for {expected_role}")
    report, _, report_sha, _ = _json_receipt(root, item["staticReview"], f"{expected_role} static report")
    _need(isinstance(report, dict), f"{expected_role} static review is invalid")
    expected_report_wheel = {"filename": filename, "bytes": size, "sha256": digest}
    if expected_role == "macos-arm64":
        expected_report_wheel["path"] = (
            str(MAC_BUILD_ORIGINAL_ROOT / "dist" / filename))
    _need(report.get("wheel") == expected_report_wheel,
          f"{expected_role} static report is for different wheel bytes")
    expected_report_sha = {"linux-x86_64": X64_STATIC_REPORT_SHA256,
                           "linux-aarch64": ARM_STATIC_REPORT_SHA256,
                           "macos-arm64": MAC_STATIC_REPORT_SHA256}[expected_role]
    if expected_report_sha is not None:
        _need(report_sha == expected_report_sha,
              f"{expected_role} static report is not the actual accepted report")
    members = report.get("members")
    _need(isinstance(members, dict) and isinstance(report.get("nativeMember"), str),
          f"{expected_role} static report omits member inventory")
    _need(report["nativeMember"] in members, f"{expected_role} native member is absent")
    try:
        with zipfile.ZipFile(path) as archive:
            infos = archive.infolist()
            _need(len(infos) == len(EXPECTED_MEMBERS) <= MAX_WHEEL_ENTRIES,
                  "wheel entry count differs from the exact nine-member contract")
            names = [entry.filename for entry in infos]
            _need(len(names) == len(set(names)), "wheel contains duplicate member names")
            _need(set(names) == EXPECTED_MEMBERS,
                  "wheel members differ from the exact wrapper/native/metadata/license family")
            compressed = expanded = 0
            for entry in infos:
                member_path = Path(entry.filename)
                mode = (entry.external_attr >> 16) & 0o170000
                _need(entry.filename and not member_path.is_absolute()
                      and "\\" not in entry.filename
                      and all(part not in {"", ".", ".."} for part in member_path.parts)
                      and not entry.is_dir() and mode in {0, stat.S_IFREG},
                      "wheel contains an unsafe or non-regular member")
                _need(not (entry.flag_bits & 1) and entry.compress_type in
                      {zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED},
                      "wheel contains encrypted or unsupported compression")
                _need(entry.file_size <= MAX_WHEEL_MEMBER,
                      "wheel member exceeds the 128 MiB per-member cap")
                if entry.filename.endswith(".dist-info/METADATA"):
                    _need(entry.file_size <= MAX_METADATA_MEMBER,
                          "wheel METADATA exceeds the 16 MiB cap")
                compressed += entry.compress_size
                expanded += entry.file_size
                _need(compressed <= MAX_WHEEL and expanded <= MAX_EXPANDED_WHEEL,
                      "wheel compressed/expanded size cap exceeded")
            _need(expanded == report.get("expandedBytes"),
                  "static report expanded size differs from the wheel")
            for name in names:
                member_path = Path(name)
                _need("\\" not in name and not member_path.is_absolute()
                      and all(part not in {"", ".", ".."} for part in member_path.parts),
                      "wheel contains an unsafe member path")
            metadata_info = archive.getinfo("umf_weft-0.4.56.dist-info/METADATA")
            _need(metadata_info.file_size <= MAX_METADATA_MEMBER,
                  "wheel METADATA exceeds the 16 MiB cap before read")
            metadata_raw = archive.read(metadata_info)
            metadata = BytesParser().parsebytes(metadata_raw)
            _need(metadata.get("Name") == DISTRIBUTION and metadata.get("Version") == VERSION,
                  "wheel package metadata differs")
            _need(metadata.get("Requires-Python") == ">=3.9"
                  and metadata.get("License-Expression") == "MIT OR Apache-2.0"
                  and not metadata.get_all("Requires-Dist", [])
                  and sorted(metadata.get_all("License-File", [])) == sorted(LICENSE_SHAS),
                  "wheel Python/dependency/license declarations differ")
            _need(metadata.get_all("Name") == [DISTRIBUTION]
                  and metadata.get_all("Version") == [VERSION]
                  and metadata.get_all("Requires-Python") == [">=3.9"]
                  and metadata.get_all("License-Expression") == ["MIT OR Apache-2.0"],
                  "wheel metadata identity headers are duplicated or conflicting")
            wheel_info = archive.getinfo("umf_weft-0.4.56.dist-info/WHEEL")
            _need(wheel_info.file_size <= MAX_METADATA_MEMBER,
                  "wheel WHEEL exceeds metadata cap before read")
            wheel_text = archive.read(wheel_info)
            wheel_message = BytesParser().parsebytes(wheel_text)
            tags = wheel_message.get_all("Tag", [])
            expected_tag = {
                "linux-x86_64": "cp39-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64",
                "linux-aarch64": "cp39-abi3-manylinux_2_17_aarch64.manylinux2014_aarch64",
                "macos-arm64": "cp39-abi3-macosx_11_0_arm64",
            }[expected_role]
            _need(wheel_message.get("Wheel-Version") == "1.0"
                  and tags == [expected_tag]
                  and wheel_message.get_all("Root-Is-Purelib") == ["false"]
                  and report.get("tags") == tags and report.get("recordExact") is True,
                  "WHEEL tag, purelib bit, or static report differs")
            wrapper = archive.read("weft/__init__.py")
            _need(len(wrapper) == 99 and _sha(wrapper) == WRAPPER_SHA256,
                  "wrapper differs from the accepted P56 source bytes")
            for license_name, license_sha in LICENSE_SHAS.items():
                member_name = "umf_weft-0.4.56.dist-info/licenses/" + license_name
                _need(_sha(archive.read(member_name)) == license_sha,
                      f"license/notice bytes differ from P56 source: {license_name}")
            _need(set(names) == set(members), "static report member roster differs from wheel")
            for name, fact in members.items():
                _need(isinstance(fact, dict) and set(fact) == {"bytes", "sha256"},
                      "static member record is malformed")
                info = archive.getinfo(name)
                _need(type(fact["bytes"]) is int and fact["bytes"] == info.file_size,
                      f"member size differs: {name}")
                _need(isinstance(fact["sha256"], str) and SHA.fullmatch(fact["sha256"]),
                      f"member digest is malformed: {name}")
                _need(_sha(archive.read(info)) == fact["sha256"],
                      f"member bytes differ from static report: {name}")
            record_name = "umf_weft-0.4.56.dist-info/RECORD"
            try:
                record_rows = list(csv.reader(io.StringIO(archive.read(record_name).decode("utf-8"))))
            except (UnicodeError, csv.Error) as exc:
                raise ProfileError("wheel RECORD is malformed") from exc
            _need(len(record_rows) == len(names) and all(len(row_values) == 3 for row_values in record_rows)
                  and {row_values[0] for row_values in record_rows} == set(names),
                  "wheel RECORD rows do not cover the exact member set")
            for member_name, recorded_hash, recorded_size in record_rows:
                if member_name == record_name:
                    _need(recorded_hash == recorded_size == "", "RECORD self cells must be empty")
                    continue
                member_bytes = archive.read(member_name)
                expected_hash = "sha256=" + base64.urlsafe_b64encode(
                    hashlib.sha256(member_bytes).digest()).rstrip(b"=").decode("ascii")
                _need(recorded_hash == expected_hash and recorded_size == str(len(member_bytes)),
                      f"RECORD bytes differ for {member_name}")
            native = archive.read("weft/weft.abi3.so")
            _need(report.get("nativeMember") == "weft/weft.abi3.so"
                  and report.get("members", {}).get("weft/weft.abi3.so") ==
                  {"bytes": len(native), "sha256": _sha(native)},
                  "static report native member differs")
            if expected_role == "macos-arm64":
                _need(_sha(native) == MAC_NATIVE_SHA256,
                      "Mac native bytes differ from the accepted static artifact")
            if expected_role.startswith("linux-"):
                machine = {"linux-x86_64": 62, "linux-aarch64": 183}[expected_role]
                _need(len(native) >= 20 and native[:4] == b"\x7fELF"
                      and native[4:6] == b"\x02\x01"
                      and struct.unpack_from("<H", native, 18)[0] == machine,
                      "ELF class, byte order, or architecture differs")
                expected_machine = {62: "Advanced Micro Devices X86-64", 183: "AArch64"}[machine]
                _need(report.get("nativeMachine") == expected_machine
                      and report.get("architecture") == {62: "amd64", 183: "arm64"}[machine]
                      and report.get("status") == "STATIC_BASELINE_WHEEL_PASS"
                      and report.get("version") == VERSION
                      and report.get("noCandidateImportOrInstallation") is True,
                      "Linux static report identity/scope differs")
                versions = report.get("glibcVersions")
                _need(isinstance(versions, list) and bool(versions)
                      and all(isinstance(version, str)
                              and re.fullmatch(r"[0-9]+\.[0-9]+(?:\.[0-9]+)?", version)
                              and tuple(map(int, version.split("."))) <= (2, 17, 0)
                              for version in versions),
                      "Linux GLIBC versions exceed the selected baseline")
                allowed_needed = {"libgcc_s.so.1", "libm.so.6", "libpthread.so.0", "libc.so.6",
                                  "libdl.so.2", "librt.so.1", "ld-linux-x86-64.so.2",
                                  "ld-linux-aarch64.so.1"}
                needed = report.get("neededLibraries")
                _need(isinstance(needed, list) and len(needed) == len(set(needed))
                      and set(needed) <= allowed_needed,
                      "Linux native dependencies exceed allowed baseline system libraries")
                _need(report.get("noCandidateImportOrInstallation") is True,
                      "Linux static report profile/scope differs")
            else:
                _need(report.get("profile") == MAC_AUDIT_PROFILE
                      and report.get("status") == "STATIC_WHEEL_PASS"
                      and report.get("licenseExpression") == "MIT OR Apache-2.0",
                      "Mac static report profile/status/license differs")
                macho = report.get("macho")
                _need(isinstance(macho, dict) and macho.get("cpuType") == 16777228
                      and macho.get("fileType") == 6 and macho.get("minimumOS") == [11, 0, 0]
                      and isinstance(macho.get("sdk"), list) and macho["sdk"][:2] == [27, 0]
                      and macho.get("dylibIds") == [{"command": 13, "name": "@rpath/weft.abi3.so"}]
                      and macho.get("loadDependencies") == [
                          {"command": 12, "name": "/usr/lib/libiconv.2.dylib"},
                          {"command": 12, "name": "/usr/lib/libSystem.B.dylib"}]
                      and macho.get("rpaths") == []
                      and macho.get("exports", {}).get("names") == ["_PyInit_weft"]
                      and macho.get("exports", {}).get("initializerAddress") == 388144
                      and macho.get("symbolTable", {}).get("initializer", {}).get("external") is True
                      and macho.get("symbolTable", {}).get("initializer", {}).get("section") == 1
                      and any(isinstance(section, dict) and section.get("ordinal") == 1
                              and section.get("segment") == "__TEXT"
                              and section.get("section") == "__text"
                              and section.get("initProtection") == 5
                              and section.get("fileOffset", -1) <= 388144
                                  < section.get("fileOffset", 0) + section.get("size", 0)
                              for section in macho.get("sections", [])),
                      "Mac Mach-O architecture/platform/dependency predicates differ")
                _need(report.get("wrapper") == {"member": "weft/__init__.py",
                    "sha256": WRAPPER_SHA256, "acceptedLinuxSameSourceSha256": WRAPPER_SHA256},
                    "Mac wrapper does not match accepted Linux source bytes")
                source_licenses = report.get("producerLicenseSources")
                _need(isinstance(source_licenses, dict) and set(source_licenses) == set(LICENSE_SHAS)
                      and all(isinstance(source_licenses[name], dict)
                              and source_licenses[name].get("sha256") == LICENSE_SHAS[name]
                              for name in LICENSE_SHAS),
                      "Mac producer license source proofs differ")
                _need(report.get("scope") ==
                      "static archive/Mach-O data inspection only; not installed, imported, or executed",
                      "Mac static report overclaims runtime qualification")
            producer_evidence = _verify_producer_chain(root, item, expected_role, filename, digest)
    except (OSError, zipfile.BadZipFile, KeyError) as exc:
        raise ProfileError(f"{expected_role} wheel is not a valid bounded archive") from exc
    final_size, final_digest = _regular(path, MAX_WHEEL, f"wheel {expected_role} final identity")
    _need(final_size == size and final_digest == digest,
          f"wheel changed while validating: {expected_role}")
    given = item["producerEvidence"]
    _need(isinstance(given, list)
          and all(isinstance(row, dict)
                  and set(row) == {"sourcePath", "path", "sizeBytes", "sha256"}
                  and isinstance(row.get("sourcePath"), str) and row["sourcePath"]
                  and isinstance(row.get("path"), str)
                  and type(row.get("sizeBytes")) is int
                  and 0 <= row["sizeBytes"] <= MAX_EVIDENCE_FILE
                  and isinstance(row.get("sha256"), str) and SHA.fullmatch(row["sha256"])
                  for row in given), "producer evidence packet roster is malformed")
    expected_by_path = {row["sourcePath"]: (row["sizeBytes"], row["sha256"])
                        for row in producer_evidence}
    given_by_path = {row["sourcePath"]: row for row in given}
    _need(len(given_by_path) == len(given) and set(given_by_path) == set(expected_by_path),
          "producer evidence packet paths differ from the original producer receipts")
    copied_paths = [row["path"] for row in given]
    _need(len(copied_paths) == len(set(copied_paths)),
          "distinct producer source identities alias one packet evidence path")
    for source_path, expected_identity in expected_by_path.items():
        row = given_by_path[source_path]
        _need((row["sizeBytes"], row["sha256"]) == expected_identity,
              "producer evidence packet identity differs from original receipts")
        evidence_path = _under(root, row["path"], "producer evidence packet path")
        actual_size, actual_sha = _regular(evidence_path, MAX_EVIDENCE_FILE,
                                           "copied producer evidence")
        _need((actual_size, actual_sha) == expected_identity,
              "copied producer evidence bytes differ from original receipts")
    return {"role": expected_role, "filename": filename, "bytes": size,
            "sha256": digest, "path": path, "staticReview": item["staticReview"],
            "buildReceipt": item["buildReceipt"], "auditReceipt": item["auditReceipt"],
            "artifactReview": item["artifactReview"], "producerEvidence": given,
            "staticReportSha256": report_sha}


def _verify_producer_chain(root: Path, item: dict[str, Any], role: str,
                           filename: str, wheel_sha: str) -> list[dict[str, Any]]:
    build, build_path, build_sha, _ = _json_receipt(root, item["buildReceipt"], f"{role} build receipt")
    audit, audit_path, audit_sha, _ = _json_receipt(root, item["auditReceipt"], f"{role} static audit receipt")
    _, _, review_sha = _receipt(root, item["artifactReview"], f"{role} artifact review")
    if role == "linux-x86_64":
        _need(build_sha == X64_BUILD_RESULT_SHA256
              and build.get("profile") == "umf.g03-final-local-baseline-build-static/1"
              and build.get("status") == "FAIL_OR_UNKNOWN"
              and build.get("failure") == "RuntimeError: static-audit failed or did not close"
              and len(build.get("commands", [])) == 3
              and build["commands"][1].get("exitCode") == 0
              and build["commands"][2].get("exitCode") == 1,
              "x64 original build failure chain was rewritten or differs")
        _need(audit_sha == X64_AUDIT_RESULT_SHA256
              and audit.get("profile") == "umf.g03-local-baseline-static-audit-only/1"
              and audit.get("status") == "LOCAL_BASELINE_AUDIT_ONLY_PASS"
              and audit.get("originalInputs", {}).get("buildResultSha256") == build_sha
              and audit.get("originalInputs", {}).get("buildCommandExitCode") == 0
              and audit.get("originalInputs", {}).get("wheelSha256") == wheel_sha
              and audit.get("originalInputs", {}).get("producerCommit") == PRODUCER,
              "x64 separate audit does not bind the exact original failed build and wheel")
        _need(review_sha == X64_ARTIFACT_REVIEW_SHA256,
              "x64 artifact review is not the actual accepted review")
        _need(filename == "umf_weft-0.4.56-cp39-abi3-manylinux_2_17_x86_64.manylinux2014_x86_64.whl"
              and wheel_sha == X64_WHEEL_SHA256,
              "x64 wheel differs from the original audited artifact")
    elif role == "linux-aarch64":
        _need(build_sha == ARM_BUILD_RESULT_SHA256
              and build.get("profile") == "umf.g03-final-local-baseline-build-static/1"
              and build.get("status") == "LOCAL_FINAL_BASELINE_BUILD_STATIC_PASS"
              and build.get("candidateWheel", {}).get("sha256") == wheel_sha
              and build.get("candidateWheel", {}).get("memberCount") == 9
              and build.get("candidateWheel", {}).get("expandedBytes", MAX_EXPANDED_WHEEL + 1)
                    <= MAX_EXPANDED_WHEEL,
              "ARM64 build receipt does not bind the selected static wheel")
        _need(audit_sha == ARM_STATIC_REPORT_SHA256
              and review_sha == ARM_ARTIFACT_REVIEW_SHA256,
              "ARM64 static report or actual artifact review differs")
        _need(wheel_sha == ARM_WHEEL_SHA256,
              "ARM64 wheel differs from the accepted static artifact")
    else:
        original = audit.get("originalInputs", {})
        source_identity = {
            "commit": PRODUCER, "tree": PRODUCER_TREE, "version": VERSION,
            "status": "", "trackedCount": SOURCE_IDENTITY["trackedCount"],
            "gitStageRosterSha256": SOURCE_IDENTITY["gitStageRosterSha256"],
            "selectedInputCount": SOURCE_IDENTITY["selectedInputCount"],
            "selectedInputRosterSha256": SOURCE_IDENTITY["selectedInputRosterSha256"],
        }
        build_operator = build.get("operator")
        _need(isinstance(build_operator, dict)
              and build.get("source") == source_identity
              and build.get("postRunSource") == source_identity
              and build.get("postRunOperator") == {
                  "runnerBytes": build_operator.get("runnerBytes"),
                  "runnerSha256": build_operator.get("runnerSha256"),
                  "staticAuditBytes": build_operator.get("staticAuditBytes"),
                  "staticAuditSha256": build_operator.get("staticAuditSha256")}
              and build_operator.get("acceptedPlanSha256") ==
                  "cafc0eb9b3ed1346210e2e1b85e437df8fb85beb7bd70e5e454f992de5b5ac53"
              and build_operator.get("measurementRepairPlanSha256") ==
                  "fba4b31855acb8ab74a864ef4e5b84d080a52987d5e4a95ffe493c1d045b0ff4"
              and build_operator.get("runnerPath") ==
                  str(MAC_BUILD_OPERATOR_ROOT / "run_build_static.py")
              and build_operator.get("runnerSha256") == MAC_BUILD_RUNNER_SHA256
              and build_operator.get("staticAuditPath") ==
                  str(MAC_BUILD_OPERATOR_ROOT / "static_audit.py")
              and build_operator.get("staticAuditSha256") == MAC_BUILD_AUDIT_TOOL_SHA256,
              "Mac build source/operator pre-post identity is not closed")
        build_selected = build.get("selectedEvidenceFiles")
        _need(isinstance(build_selected, list) and len(build_selected) == 5
              and build.get("selectedEvidenceBytes") == sum(row.get("bytes", -1)
                    for row in build_selected),
              "Mac build selected output/evidence roster is malformed")
        selected_build = {row.get("path"): (row.get("bytes"), row.get("sha256"))
                          for row in build_selected}
        _need(len(selected_build) == len(build_selected),
              "Mac build selected evidence has duplicate paths")
        for stage in build.get("stages", []):
            physical = stage.get("physical") if isinstance(stage, dict) else None
            _need(isinstance(physical, list) and len(physical) == 2,
                  "Mac build stage lacks both physical capture streams")
            for stream in physical:
                relative = _historical_relative(stream.get("path"), MAC_BUILD_ORIGINAL_ROOT,
                                                "Mac build stage log")
                _need(selected_build.get(relative) ==
                      (stream.get("bytes"), stream.get("sha256")),
                      "Mac build stage log differs from selected evidence")
        audit_operator = audit.get("operator")
        python_size, python_sha = _regular(Path(MAC_PYTHON_PATH), MAX_EVIDENCE_FILE,
                                           "Mac observed Python executable")
        _need((python_size, python_sha) == (MAC_PYTHON_BYTES, MAC_PYTHON_SHA256),
              "Mac observed Python executable differs from its recorded identity")
        _need(isinstance(audit_operator, dict)
              and audit_operator.get("runnerBefore") == audit_operator.get("runnerAfter")
              and audit_operator.get("helperAfter") == [
                  audit_operator.get("helper", {}).get("bytes"),
                  audit_operator.get("helper", {}).get("sha256")]
              and audit_operator.get("runnerBefore") == [22917, MAC_AUDIT_RUNNER_SHA256]
              and audit_operator.get("runnerAfter") == [22917, MAC_AUDIT_RUNNER_SHA256]
              and audit_operator.get("pythonBefore") == audit_operator.get("pythonAfter")
              and audit_operator.get("runnerPath") ==
                  str(MAC_AUDIT_OPERATOR_ROOT / "run_static_audit.py")
              and audit_operator.get("helper", {}).get("path") ==
                  str(MAC_AUDIT_OPERATOR_ROOT / "static_audit.py")
              and audit_operator.get("helper", {}).get("sha256") == MAC_AUDIT_HELPER_SHA256
              and audit_operator.get("pythonPath") == MAC_PYTHON_PATH
              and audit_operator.get("pythonBefore") == [MAC_PYTHON_BYTES, MAC_PYTHON_SHA256],
              "Mac audit operator/Python pre-post byte identities are not closed")
        audit_selected = audit.get("selectedEvidenceFiles")
        _need(isinstance(audit_selected, list) and len(audit_selected) == 3
              and audit.get("selectedEvidenceBytes") == sum(row.get("bytes", -1)
                    for row in audit_selected)
              and {row.get("path") for row in audit_selected} == {
                    "static-audit.json", "logs/static-audit.stdout.log",
                    "logs/static-audit.stderr.log"},
              "Mac audit selected output/log roster is not the exact three-file closure")
        capture = audit.get("capture", {})
        physical_capture = capture.get("physical")
        _need(isinstance(physical_capture, list) and len(physical_capture) == 2
              and capture.get("paths") == [row.get("path") for row in physical_capture]
              and capture.get("rawSha256") == [row.get("sha256") for row in physical_capture]
              and capture.get("retainedSha256") == capture.get("rawSha256")
              and capture.get("retainedBytes") == [row.get("bytes") for row in physical_capture]
              and capture.get("truncated") == [False, False]
              and capture.get("captureErrors") == [],
              "Mac audit raw capture does not close to both physical stream files")
        expected_audit_argv = [MAC_PYTHON_PATH,
            str(MAC_AUDIT_OPERATOR_ROOT / "static_audit.py"),
            str(MAC_BUILD_ORIGINAL_ROOT / "dist" / filename),
            str(MAC_AUDIT_ORIGINAL_ROOT / "static-audit.json"),
            str(MAC_PRODUCER_SOURCE_ROOT)]
        _need(capture.get("argv") == expected_audit_argv
              and capture.get("cwd") == str(MAC_PRODUCER_SOURCE_ROOT)
              and capture.get("limitSeconds") == 120
              and capture.get("effectiveDeadlineSeconds", 0) <= 120,
              "Mac audit command identity/deadline differs from the accepted invocation")
        audit_by_path = {row.get("path"): row for row in audit_selected}
        _need(all(audit_by_path.get("logs/" + Path(row.get("path", "")).name,
                  {}).get("bytes") == row.get("bytes")
                  and audit_by_path.get("logs/" + Path(row.get("path", "")).name,
                  {}).get("sha256") == row.get("sha256")
                  for row in physical_capture),
              "Mac audit capture bytes differ from selected physical logs")
        limits = audit.get("limits", {})
        _need(limits.get("aggregateSeconds") == 180 and limits.get("auditSeconds") == 120
              and limits.get("terminalReserveSeconds") == 60
              and limits.get("aggregateLogBytes") == MAX_RAW_LOGS
              and limits.get("metadataBytes") == MAX_EVIDENCE_TOTAL,
              "Mac static-audit resource/deadline caps differ")
        _need(build_sha == MAC_BUILD_RESULT_SHA256
              and build.get("profile") == MAC_ORIGINAL_BUILD_PROFILE
              and build.get("status") == MAC_ORIGINAL_BUILD_STATUS
              and build.get("executionKind") == "local-build-only"
              and build.get("hosted") is False and build.get("installed") is False
              and build.get("source", {}).get("commit") == PRODUCER
              and build.get("source", {}).get("tree") == PRODUCER_TREE
              and build.get("source", {}).get("status") == ""
              and build.get("postRunSource") == build.get("source")
              and build.get("candidate") is None
              and build.get("stages", [])[0].get("exitCode") == 0
              and build.get("stages", [])[1].get("exitCode") == 1
              and build.get("cleanupErrors") == [],
              "Mac original failed build evidence was changed or overclaimed")
        _need(audit_sha == MAC_AUDIT_RESULT_SHA256
              and audit.get("profile") == MAC_AUDIT_PROFILE
              and audit.get("status") == "LOCAL_MACOS_STATIC_AUDIT_ONLY_PASS"
              and audit.get("executionKind") == "local-static-audit-only"
              and audit.get("hosted") is False and audit.get("installed") is False
              and audit.get("imported") is False and audit.get("nativeExecuted") is False
              and audit.get("functionalQualification") == "NOT_RUN"
              and audit.get("cleanupErrors") == [] and audit.get("closureErrors") == []
              and original.get("result", {}).get("sha256") == build_sha
              and original.get("result", {}).get("path") ==
                  str(MAC_BUILD_ORIGINAL_ROOT / "result.json")
              and original.get("result", {}).get("bytes") == build_path.stat(follow_symlinks=False).st_size
              and original.get("result", {}).get("before") == [376450, build_sha]
              and original.get("result", {}).get("after") == [376450, build_sha]
              and original.get("source", {}).get("commit") == PRODUCER
              and original.get("source", {}).get("tree") == PRODUCER_TREE
              and original.get("source", {}).get("selectedInputCount") ==
                  SOURCE_IDENTITY["selectedInputCount"]
              and original.get("source", {}).get("selectedInputRosterSha256") ==
                  SOURCE_IDENTITY["selectedInputRosterSha256"]
              and original.get("source", {}).get("sourceInputsBefore", {}).get("rosterSha256")
                    == SOURCE_IDENTITY["selectedInputRosterSha256"]
              and original.get("source", {}).get("sourceInputsAfter")
                    == original.get("source", {}).get("sourceInputsBefore")
              and original.get("wheel") == {
                    "bytes": MAC_WHEEL_BYTES,
                    "path": str(MAC_BUILD_ORIGINAL_ROOT / "dist" / filename),
                    "sha256": wheel_sha}
              and audit.get("wheelBefore") == [MAC_WHEEL_BYTES, wheel_sha]
              and audit.get("wheelAfter") == [MAC_WHEEL_BYTES, wheel_sha]
              and audit.get("staticReport") == {
                    "bytes": 9604,
                    "path": str(MAC_AUDIT_ORIGINAL_ROOT / "static-audit.json"),
                    "sha256": MAC_STATIC_REPORT_SHA256}
              and audit.get("capture", {}).get("exitCode") == 0
              and audit.get("capture", {}).get("captureVerified") is True
              and audit.get("capture", {}).get("timedOut") is False
              and audit.get("capture", {}).get("drainExpired") is False,
              "Mac separate static-audit continuation does not close actual inputs/capture")
        _need(audit.get("staticReport", {}).get("sha256") == MAC_STATIC_REPORT_SHA256
              and audit.get("staticReport", {}).get("bytes") == 9604
              and audit.get("staticReport", {}).get("path") ==
                    str(MAC_AUDIT_ORIGINAL_ROOT / "static-audit.json"),
              "Mac static report identity differs")
    if role == "linux-x86_64":
        _need(review_sha == X64_ARTIFACT_REVIEW_SHA256,
              "x64 artifact review is not the pinned actual acceptance")
    elif role == "linux-aarch64":
        _need(review_sha == ARM_ARTIFACT_REVIEW_SHA256,
              "ARM64 artifact review is not the pinned actual acceptance")
    else:
        _need(review_sha == MAC_ARTIFACT_REVIEW_SHA256,
              "Mac artifact review is not the exact accepted build/static review")
    evidence = (_producer_evidence_rows(build, build_path, where=f"{role} build",
                                        copied_rows=item["producerEvidence"])
                + _producer_evidence_rows(audit, audit_path, where=f"{role} audit",
                                          copied_rows=item["producerEvidence"]))
    by_path: dict[str, dict[str, Any]] = {}
    for row in evidence:
        existing = by_path.get(row["sourcePath"])
        _need(existing is None or existing == row,
              "build/audit evidence paths overlap with different bytes")
        by_path[row["sourcePath"]] = row
    _need(by_path, f"{role} producer chain has no retained raw evidence")
    return [by_path[path] for path in sorted(by_path)]


def validate_packet(packet_path: Path, *, expected_metadata_review_sha256: str | None = None) -> dict[str, Any]:
    """Validate one READY packet against the configured release trust anchors."""
    _need(not packet_path.is_symlink(), "packet must not be a symlink")
    packet_path = packet_path.resolve(strict=True)
    packet_size, packet_sha = _regular(packet_path, MAX_PACKET, "packet")
    try:
        packet = json.loads(_read_bounded(packet_path, MAX_PACKET, "packet"))
    except (OSError, UnicodeError, json.JSONDecodeError) as exc:
        raise ProfileError("packet is invalid JSON") from exc
    keys = {"profile", "status", "release", "producer", "metadata", "wheels", "gates",
            "preAssemblyReviews", "hosted"}
    _closed(packet, keys, "packet")
    _need(packet["profile"] == PROFILE and packet["status"] == "READY_FOR_ASSEMBLY",
          "local release packet is not ready")
    release = _closed(packet["release"], {"distribution", "version", "tag", "wheelCount", "sdist"},
                      "release")
    _need(release == {"distribution": DISTRIBUTION, "version": VERSION, "tag": TAG,
                      "wheelCount": 3, "sdist": False}, "release tuple is not the selected profile")
    producer = _closed(packet["producer"], {"sourceMap"}, "producer")
    metadata = _closed(packet["metadata"], {"sourceMap", "ancestry", "files"}, "metadata source")
    gates = packet["gates"]
    _need(isinstance(gates, dict) and set(gates) == set(REQUIRED_GATES),
          "required qualification gate roster differs")
    _need(isinstance(packet["wheels"], list) and len(packet["wheels"]) == 3
          and isinstance(packet["preAssemblyReviews"], list),
          "wheel or pre-assembly review roster is malformed")
    _preflight_packet_payloads(packet_path.parent, packet, packet_size)
    pmap, mmap, source_comparison = _source_correspondence(
        packet_path.parent, producer["sourceMap"], metadata["sourceMap"], metadata["ancestry"],
        gates["linux-source-package"]["receipt"])
    metadata_files = _validate_metadata_files(packet_path.parent, metadata["files"], mmap)
    producer_identity = pmap["source"]
    metadata_identity = mmap["source"]
    _need(packet["hosted"] == {"workflowRunId": None, "artifactId": None, "workflowPath": None},
          "local profile must not claim hosted producer identity")
    wheels = packet["wheels"]
    _need(isinstance(wheels, list) and len(wheels) == 3, "exactly three wheels are required")
    checked_wheels = [_wheel(packet_path.parent, row, role)
                      for row, role in zip(wheels, ROLES, strict=True)]
    _need(sum(row["bytes"] for row in checked_wheels) <= MAX_WHEEL_FAMILY,
          "three-wheel software family exceeds its 70 MiB cap")
    gate_rows: dict[str, dict[str, Any]] = {}
    evidence_refs: list[dict[str, Any]] = [
        producer["sourceMap"], metadata["sourceMap"], metadata["ancestry"]]
    evidence_refs.extend({key: row[key] for key in ("path", "sizeBytes", "sha256")}
                         for row in packet["metadata"]["files"])
    for name in REQUIRED_GATES:
        gate_rows[name] = _gate(packet_path.parent, name, gates[name])
        evidence_refs.extend((gates[name]["receipt"], gates[name]["review"]))
        if name == "macos-arm64-build-static":
            evidence_refs.append(gates[name]["dataReview"])
        elif name == "linux-x86_64-final-native-campaign":
            evidence_refs.append(gates[name]["reanalysis"])
        evidence_refs.extend(gate_rows[name]["outputFiles"])
    _need(gates["linux-browser"]["receipt"] == gates["linux-wasm"]["receipt"]
          and gates["linux-browser"]["review"] == gates["linux-wasm"]["review"]
          and gate_rows["linux-browser"]["receiptSha256"] == gate_rows["linux-wasm"]["receiptSha256"]
          and gate_rows["linux-browser"]["reviewSha256"] == gate_rows["linux-wasm"]["reviewSha256"]
          and gate_rows["linux-browser"]["outputFiles"] == gate_rows["linux-wasm"]["outputFiles"],
          "browser and WASM gate obligations must bind the same complete Truss011 receipt/review")
    external_by_identity: dict[tuple[str, str], dict[str, Any]] = {}
    for gate_name in ("linux-inherited-boundaries", "linux-browser", "linux-wasm"):
        for witness in gate_rows[gate_name].get("externalSelectedSoftware", []):
            identity = (witness["sourceRoot"], witness["sourcePath"])
            prior = external_by_identity.get(identity)
            _need(prior is None or prior == witness,
                  "external selected-software witness has conflicting gate identities")
            external_by_identity[identity] = witness
    external_selected_software = [external_by_identity[key]
                                  for key in sorted(external_by_identity)]
    source_guard, _, _, _ = _json_receipt(
        packet_path.parent, gates["linux-source-package"]["receipt"],
        "accepted source/package guard receipt")
    _bind_selected_inputs_to_source_guard(pmap, source_guard)
    reviews = packet["preAssemblyReviews"]
    _need(isinstance(reviews, list), "pre-assembly artifact/source review is absent")
    review_facts = []
    review_scopes: set[str] = set()
    for row in reviews:
        review_row = _closed(row, {"scope", "path", "sizeBytes", "sha256"},
                             "pre-assembly review reference")
        row_scope = review_row["scope"]
        ref = {key: review_row[key] for key in ("path", "sizeBytes", "sha256")}
        if row_scope in PRE_ASSEMBLY_REVIEW_SHA256:
            expected_review_sha = PRE_ASSEMBLY_REVIEW_SHA256[row_scope]
            human_pin = PRE_ASSEMBLY_HUMAN_REVIEW_SHA256.get(row_scope)
            if row_scope == METADATA_REVIEW_SCOPE:
                _need(isinstance(expected_metadata_review_sha256, str)
                      and SHA.fullmatch(expected_metadata_review_sha256),
                      "actual M source review digest must be supplied outside packet/M")
                expected_review_sha = expected_metadata_review_sha256
                human_pin = expected_review_sha
            _need(expected_review_sha is not None,
                  f"{row_scope} has no independently accepted review anchor")
            if human_pin is not None:
                review_path, review_raw, digest = _receipt(
                    packet_path.parent, ref, "pre-assembly human review")
                size = len(review_raw)
                _need(digest == expected_review_sha,
                      f"{row_scope} review digest differs from its outside-Q trust anchor")
            else:
                review, review_path, digest, size = _json_receipt(
                    packet_path.parent, ref, "pre-assembly review")
                review_raw = b""
        else:
            raise ProfileError(f"unsupported pre-assembly review scope: {row_scope}")
        if human_pin is not None:
            size = len(review_raw)
            if row_scope == "linux-x86_64-artifact":
                binding_values = (X64_BUILD_RESULT_SHA256, X64_AUDIT_RESULT_SHA256,
                                  X64_WHEEL_SHA256)
                accepted_phrase = b"ACCEPTED ACTUAL ARTIFACT"
            elif row_scope == "linux-aarch64-static-artifact":
                binding_values = (ARM_BUILD_RESULT_SHA256, ARM_WHEEL_SHA256)
                accepted_phrase = b"ACCEPTED ACTUAL BUILD/STATIC ARTIFACT ONLY"
            elif row_scope == "macos-arm64-static-artifact":
                binding_values = (MAC_BUILD_RESULT_SHA256, MAC_AUDIT_RESULT_SHA256,
                                  MAC_WHEEL_SHA256, MAC_STATIC_REPORT_SHA256)
                accepted_phrase = b"Disposition: ACCEPT the exact local macOS ARM64 build artifact plus its separate static-audit continuation."
            else:
                binding_values = (source_comparison["metadataSourceMapSha256"],
                                  source_comparison["ancestrySha256"])
                accepted_phrase = b"ACCEPTED ACTUAL P/M SOURCE CORRESPONDENCE"
            _need(all(value.encode("ascii") in review_raw for value in binding_values)
                  and accepted_phrase in review_raw,
                  f"{row_scope} review does not bind the exact source/artifact chain")
            review = {"scope": row_scope, "status": "ACCEPTED",
                      "subjectSha256": (source_comparison["metadataSourceMapSha256"]
                          if row_scope == METADATA_REVIEW_SCOPE else
                          {"linux-x86_64-artifact": X64_WHEEL_SHA256,
                           "linux-aarch64-static-artifact": ARM_WHEEL_SHA256,
                           "macos-arm64-static-artifact": MAC_WHEEL_SHA256}[row_scope]
                          if row_scope != METADATA_REVIEW_SCOPE else
                          source_comparison["metadataSourceMapSha256"])}
        scope = review.get("scope")
        if scope == "producer-metadata-source":
            expected_subject = source_comparison["metadataSourceMapSha256"]
        elif scope in {"linux-x86_64-artifact", "linux-aarch64-static-artifact",
                       "macos-arm64-static-artifact"}:
            role = {"linux-x86_64-artifact": "linux-x86_64",
                    "linux-aarch64-static-artifact": "linux-aarch64",
                    "macos-arm64-static-artifact": "macos-arm64"}[scope]
            selected = next((entry for entry in wheels if entry["role"] == role), None)
            _need(selected is not None, f"{role} review has no selected wheel")
            _need(ref["sha256"] == selected["artifactReview"]["sha256"],
                  f"{role} pre-assembly review differs from its artifact review")
            expected_subject = selected["sha256"]
        else:
            expected_subject = None
        _need((review.get("profile") == "weft-local-release-evidence-review/1"
               or human_pin is not None)
              and review.get("status") == "ACCEPTED"
              and isinstance(scope, str) and scope in PRE_ASSEMBLY_REVIEW_SHA256
              and scope not in review_scopes
              and digest == expected_review_sha
              and isinstance(review.get("subjectSha256"), str)
              and SHA.fullmatch(review["subjectSha256"])
              and (expected_subject is None or review["subjectSha256"] == expected_subject),
              "pre-assembly review is unbound or unaccepted")
        review_scopes.add(scope)
        review_facts.append({"scope": scope, "path": ref["path"],
                             "sizeBytes": size, "sha256": digest})
        evidence_refs.append(ref)
    _need(review_scopes == set(PRE_ASSEMBLY_REVIEW_SHA256),
          "pre-assembly source/artifact review roster is incomplete")
    for wheel in checked_wheels:
        evidence_refs.extend((wheel["staticReview"], wheel["buildReceipt"],
                              wheel["auditReceipt"], wheel["artifactReview"]))
        evidence_refs.extend({key: row[key] for key in ("path", "sizeBytes", "sha256")}
                             for row in wheel["producerEvidence"])
    unique: dict[str, tuple[int, str]] = {}
    for ref in evidence_refs:
        path, data, digest = _receipt(packet_path.parent, ref, "selected evidence file")
        logical = Path(ref["path"]).as_posix()
        if logical in unique:
            _need(unique[logical] == (len(data), digest),
                  "same evidence path was referenced with conflicting identity")
            continue
        unique[logical] = (len(data), digest)
    evidence_bytes = sum(size for size, _ in unique.values())
    evidence_bytes += packet_size
    _need(evidence_bytes <= MAX_EVIDENCE_TOTAL,
          "complete deduplicated packet/source/gate/log evidence exceeds 16 MiB")
    return {"profile": PROFILE, "status": "VERIFIED_FOR_ASSEMBLY", "packetSha256": packet_sha,
            "packetPath": packet_path,
            "packetBytes": packet_size, "release": release,
            "producer": producer_identity, "producerSourceMap": producer["sourceMap"],
            "metadata": metadata_identity, "metadataSourceMap": metadata["sourceMap"],
            "metadataFiles": metadata_files,
            "ancestry": metadata["ancestry"], "sourceComparison": source_comparison,
            "wheels": checked_wheels, "gates": gate_rows,
            "externalSelectedSoftware": external_selected_software,
            "reviews": review_facts,
            "evidenceRefs": [{"path": name, "sizeBytes": size, "sha256": digest}
                             for name, (size, digest) in sorted(unique.items())],
            "evidenceInventory": [{"path": name, "bytes": size, "sha256": digest}
                                  for name, (size, digest) in sorted(unique.items())],
            "evidenceBytes": evidence_bytes,
            "qualification": "local unsigned provenance; hosted and PyPI proof are separate"}

