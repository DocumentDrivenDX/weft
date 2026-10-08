"""Native emitted-aggregate-pattern boundaries, not source-corpus acceptance.

@covers US-004-AC3 (exact overflow/empty boundary prerequisite)
"""
import hashlib
import json
import os
from pathlib import Path
from native_transport import Client, NativeFailure

output = Path(os.environ["WEFT_ASHLAR_EVIDENCE_OUTPUT"])
client = Client(output)
expression = "CASE WHEN MAX(1) IS NOT NULL AND TRY_SUM(v) IS NULL THEN raise_error('WFT-NUMERIC-DOMAIN') ELSE TRY_SUM(v) END"
sql = "SELECT " + expression + " FROM (SELECT cast(:v AS DECIMAL(38,0)) v FROM VALUES (1),(2) t(n))"
try:
    client.sql("nonempty-overflow-raises", sql, parameters=[dict(name="v", type="STRING", value="9" * 38)])
    raise AssertionError("Overflow accepted")
except NativeFailure as error:
    assert "WFT-NUMERIC-DOMAIN" in str(error), error
assert client.sql("empty-retains-null", "SELECT " + expression + " FROM (SELECT cast('0' AS DECIMAL(38,0)) v WHERE FALSE)") == [[None]]
assert client.sql("finite-sum-exact", sql, parameters=[dict(name="v", type="STRING", value="18446744073709551615")]) == [["36893488147419103230"]]
summary = dict(state="passed", statements=3, qualification="Read-only native aggregate pattern; DECIMAL38 overflow explicitly errors, empty input remains NULL, large finite sum exact. No claim of enumerating unbounded source group multiplicities.",
               harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
               transportSha256=hashlib.sha256((Path(__file__).parent / "native_transport.py").read_bytes()).hexdigest())
(output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
