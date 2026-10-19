# SPDX-License-Identifier: MIT OR Apache-2.0
"""Compare installed PyO3 and native CLI using synthetic original table source.

Run with the installed candidate Python and the compile_raw_read example path.
This test opens no database and grants no native execution qualification.
"""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys

import weft

ROOT = Path(__file__).resolve().parent


def encode(value):
    return json.dumps(value, ensure_ascii=False, separators=(",", ":"))


def pin(request, source, pretty=False):
    text = json.dumps(source, ensure_ascii=False, indent=2) if pretty else encode(source)
    request["binding"]["modelJson"] = text
    request["binding"]["modelSha256"] = hashlib.sha256(text.encode()).hexdigest()


def main():
    cli = Path(sys.argv[1]).resolve(strict=True)
    source = json.loads((ROOT / "table-lifecycle-model.json").read_text())
    base = json.loads((ROOT / "table-lifecycle-read-template.json").read_text())
    pin(base, source)
    cases = [("keyset", base, "compiled")]
    lakebase = copy.deepcopy(base)
    lakebase["target"] = "lakebase18"
    cases.append(("lakebase-compilation-only", lakebase, "compiled"))
    count = copy.deepcopy(base)
    count.update(sql='SELECT COUNT(*) AS total FROM demo."Note" n', parameters=[], keyset=False)
    cases.append(("count", count, "compiled"))
    pretty = copy.deepcopy(base)
    pin(pretty, source, pretty=True)
    cases.append(("original-source-format", pretty, "compiled"))
    for key in ("nativeLifecycle", "verificationProfile"):
        request = copy.deepcopy(base)
        del (request["binding"] if key == "nativeLifecycle" else request)[key]
        cases.append(("missing-" + key, request, "blocked"))
    for name, mutate in (
        ("mapping", lambda r: r["binding"]["tables"][0].update(native="wrong")),
        ("digest", lambda r: r["binding"].update(modelSha256="0" * 64)),
        ("context", lambda r: r["binding"]["nativeLifecycle"].update(module="wrong")),
    ):
        request = copy.deepcopy(base)
        mutate(request)
        cases.append((name, request, "blocked"))
    opaque = copy.deepcopy(base)
    changed = copy.deepcopy(source)
    changed["modules"][0]["extensions"]["umf.tables"]["commands"] = [{"future": True}]
    pin(opaque, changed)
    cases.append(("uninterpreted-command", opaque, "blocked"))
    for path in (
        (), ("tables", 0), ("tables", 0, "record"),
        ("tables", 0, "columns", 0), ("tables", 0, "columns", 0, "nativeType"),
        ("tables", 0, "policy"), ("tables", 0, "policy", "person"),
        ("tables", 0, "policy", "person", "select"),
    ):
        request = copy.deepcopy(base)
        changed = copy.deepcopy(source)
        selected = changed["modules"][0]["extensions"]["umf.tables"]
        for part in path:
            selected = selected[part]
        selected["future"] = True
        pin(request, changed)
        cases.append(("unknown-selected-member-" + "/".join(map(str, path)), request, "blocked"))
    results = []
    for name, request, expected in cases:
        original = encode(request)
        python_result = json.loads(weft.compile_raw_read_json(original))
        native_result = json.loads(subprocess.run(
            [str(cli)], input=original, text=True, capture_output=True,
            check=True, timeout=60,
        ).stdout)
        assert python_result == native_result, name + ": binding result mismatch"
        assert python_result["status"] == expected, name + ": unexpected admission"
        if expected == "compiled":
            assert python_result["version"] == "weft.table-read/0.1", name
            assert python_result["binding"]["modelJson"] == request["binding"]["modelJson"], name
            assert python_result["obligations"]["tableLifecycle"]["nativeExecutionQualified"] is False, name
        results.append({"name": name, "status": expected, "nativePythonEqual": True})
    print(json.dumps({"cases": results, "nativeExecutionQualified": False,
                      "browserExecutionQualified": False}, indent=2))


if __name__ == "__main__":
    main()
