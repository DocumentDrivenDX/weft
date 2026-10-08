"""Independent native global SUM oracle over the retained scalar fixtures.

@covers US-004-AC1 @covers US-004-AC3 (candidate component)
"""
from decimal import Decimal, localcontext
import hashlib
import json
import os
from pathlib import Path
import subprocess
from native_transport import Client

output = Path(os.environ["WEFT_ASHLAR_EVIDENCE_OUTPUT"])
fixture = Path(os.environ["WEFT_ASHLAR_SCALAR_FIXTURE"])
client = Client(output)
outcomes = []
for name in ["exact", "empty"]:
    captured = json.loads((fixture / (name + "-compile.json")).read_text())
    request = captured["request"]
    request["sql"] = "SELECT SUM(o.total) AS total FROM Orders o"
    response = subprocess.run([os.environ["WEFT_ASHLAR_COMPILER"]], input=json.dumps(request), text=True, capture_output=True, check=True)
    artifact = json.loads(response.stdout)
    assert artifact["status"] == "compiled", artifact
    assert artifact["columns"][0]["nullable"] is True
    assert artifact["columns"][0]["logicalType"]["facets"] == {"scale": 2}
    assert artifact["columns"][0]["decoder"] == "exact-decimal"
    parameters = [dict(name="p" + str(p["position"]), type="STRING", value=p["value"]) for p in artifact["parameters"]]
    checks = next(o for o in artifact["obligations"] if o["id"] == "ashlar.candidate.scalarIntegrity")["parameters"]["checks"]
    assert len(checks) == 1
    assert client.sql(name + "-integrity", checks[0]["sql"], parameters) == [["0"]]
    actual = client.sql(name + "-global-sum", artifact["sql"], parameters)
    with localcontext() as context:
        context.prec = 100
        expected = [[format(sum(map(Decimal, ["99999999999999999999999999.99", "0.02", "-1.25", "2.00", "3.00"])), ".2f")]] if name == "exact" else [[None]]
    assert actual == expected, (actual, expected)
    metadata = client.records[-1]["response"]["manifest"]["schema"]["columns"]
    assert [(c["name"], c["type_name"]) for c in metadata] == [("total", "STRING")]
    (output / (name + "-compile.json")).write_text(json.dumps(dict(request=request, response=artifact), indent=2) + "\n")
    outcomes.append(dict(id=name, rows=actual, nullable=True))
summary = dict(state="passed", outcomes=outcomes, harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
               qualification="Actual registered Rust/compiler SQL and native STRING/null result metadata on two immutable synthetic Delta fixture cuts. Independent decimal oracle includes unmatched orders for global SUM. No application, policy, embedding or production qualification.")
(output / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
