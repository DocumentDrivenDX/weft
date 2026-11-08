import hashlib
import json
import os
from pathlib import Path
import sys

repo = Path(__file__).resolve().parents[5]
evidence = Path(__file__).resolve().parent
frozen = repo / "docs/helix/04-build/evidence/generated-ordinary-page-v6-wheel-20261105"
sys.path.insert(0, os.environ["WEFT_V7_SITE_PACKAGES"])
import weft

assert weft.__version__ == "0.4.66.dev1", weft.__version__
assert hasattr(weft, "prepare_generated_read_algebra_v7_json")
for profile in ("v5", "v6"):
    request = json.loads((frozen / f"request-{profile}.json").read_text())
    actual = getattr(weft, f"prepare_generated_read_algebra_{profile}_json")(
        json.dumps(request, separators=(",", ":"))
    )
    expected = (frozen / f"wheel-issued-{profile}.json").read_text().strip()
    assert actual == expected, profile
    print(f"{profile}: byte-equal; {len(actual)} bytes")
request = json.loads((evidence / "request-v7.json").read_text())
actual = weft.prepare_generated_read_algebra_v7_json(
    json.dumps(request, separators=(",", ":"))
)
expected = (evidence / "issued-v7.json").read_text()
assert actual == expected, "v7 output differs from retained candidate vector"
parsed = json.loads(actual)
assert parsed["version"] == "weft.generated-read-algebra/7"
assert parsed["executorAbi"] == {"major": 1, "minor": 7}
print(f"v7: byte-equal; {len(actual)} bytes; SHA-256 {hashlib.sha256(actual.encode()).hexdigest()}")
request["version"] = "weft.generated-read-algebra-request/6"
try:
    weft.prepare_generated_read_algebra_v7_json(json.dumps(request, separators=(",", ":")))
except Exception:
    print("wrong request profile: refused")
else:
    raise AssertionError("v7 accepted a v6 request")
