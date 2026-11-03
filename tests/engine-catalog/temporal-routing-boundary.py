"""Actual native Python routing regression; no native Truss/WASM qualification."""
import hashlib
import importlib.util
import json
from pathlib import Path
import sys

spec = importlib.util.spec_from_file_location("weft", sys.argv[1])
weft = importlib.util.module_from_spec(spec)
spec.loader.exec_module(weft)
root = Path(__file__).resolve().parents[2]
fixture = json.loads((root / "tests/engine-catalog/ordinary-abi12-public056.json").read_text())
limit = 4_194_304

def packet(value):
    return json.dumps(value, separators=(",", ":"), sort_keys=True)

def pad(raw, size):
    return raw + " " * (size - len(raw.encode()))

request_raw = fixture["request"]["utf8"]
assert hashlib.sha256(request_raw.encode()).hexdigest() == fixture["request"]["sha256"]
request = json.loads(request_raw)
assert json.loads(request["target"]["bindingJson"])["queryAbi"]["version"] == "1.2"
ordinary = packet({"version": "weft.engine-read/0.1", "compiler": "truss-runtime", "request": request})
before = weft.compile_engine_read_json(ordinary)
assert json.loads(before)["status"] == "compiled"
# Obtained by executing pristine2f62 native Rust on this authentic capture.
assert hashlib.sha256(before.encode()).hexdigest() == "65925a4646da51a6d8e53e859f7be88b67c35c98f37dafe0992a213fe2b4cec1"
assert weft.compile_engine_read_json(pad(ordinary, limit + 1)) == before
page = fixture["enginePageInput"]["utf8"]
before_page = weft.compile_engine_page_json(page)
assert json.loads(before_page) == fixture["expected"]["compile_engine_page_json"]["json"]
assert weft.compile_engine_page_json(pad(page, limit)) == before_page
assert json.loads(weft.compile_engine_page_json(pad(page, limit + 1))) == {
    "version": "weft.engine-page/0.1", "status": "blocked",
    "diagnostics": [{"code": "WFT-ENGINE-PAGE", "message": "bounded original page request required"}],
}
assert weft.compile_engine_read_json(pad("{}", limit + 1)) == weft.compile_engine_read_json("{}")
request = json.loads((root / "tests/engine-catalog/temporal-request.json").read_text())
for page in [False, True]:
    value = {"version": "weft.engine-page/0.4" if page else "weft.engine-read/0.4", "compiler": "truss-runtime-temporal", "request": request}
    if page:
        value.update(selectedKey={"module": "m", "record": "R", "key": "pair"}, pageSize=2)
    compile = weft.compile_engine_page_json if page else weft.compile_engine_read_json
    raw = packet(value)
    expected = compile(raw)
    assert json.loads(expected)["status"] == "compiled"
    assert compile(pad(raw, limit)) == expected
    refused = json.loads(compile(pad(raw, limit + 1)))
    assert refused["diagnostics"][0]["code"] == "WFT-LIMIT"
    assert refused["diagnostics"][0]["phase"] == "input"
print(json.dumps({"scope": "actual native Python routing only", "controls": 5, "passed": True}))
