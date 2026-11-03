# SPDX-License-Identifier: MIT OR Apache-2.0
"""Replay whole Rust results through an actual native Python extension."""
import importlib.util
import json
import pathlib
import sys

extension, corpus = map(pathlib.Path, sys.argv[1:])
spec = importlib.util.spec_from_file_location("weft", extension)
owner = importlib.util.module_from_spec(spec)
spec.loader.exec_module(owner)
cases = json.loads(corpus.read_text())["cases"]
for i, case in enumerate(cases):
    actual = json.loads(getattr(owner, case["name"])(*case["arguments"]))
    assert actual == case["expected"], (i, case["name"], actual)
for bad in ["\ud800", 42, None]:
    try:
        owner.check_engine_temporal_read_shape_json(bad, "{}")
    except (UnicodeError, TypeError, ValueError):
        pass
    else:
        raise AssertionError("Native boundary admitted a non-Unicode/non-string carrier")

print(json.dumps({"scope": "Native exported Python boundary only", "cases": len(cases), "passed": True}))
