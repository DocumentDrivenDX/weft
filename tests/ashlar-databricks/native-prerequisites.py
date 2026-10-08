"""Read-only native prerequisite probes; these do not exercise the compiler.

Use the existing aidev-cus SDK profile/warehouse. Never provision compute,
print credentials, retry submissions, or treat local Spark as native evidence.
@covers US-004-AC3 (prerequisite only, not acceptance)
"""
import hashlib
import json
import os
from pathlib import Path
import time

from databricks.sdk import WorkspaceClient

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(os.environ["WEFT_ASHLAR_EVIDENCE_OUTPUT"])
OUT.mkdir(parents=True, exist_ok=True)
WAREHOUSE = "2439e1f2e37ac563"
client = WorkspaceClient(profile="aidev-cus")
records = []


def query(label, sql, parameters=None, error=None):
    body = dict(warehouse_id=WAREHOUSE, statement=sql, wait_timeout="10s",
                on_wait_timeout="CONTINUE", disposition="INLINE",
                format="JSON_ARRAY", row_limit=100)
    if parameters is not None:
        body["parameters"] = parameters
    response = client.api_client.do("POST", "/api/2.0/sql/statements", body=body)
    sid = response["statement_id"]
    (OUT / "live-statement.json").write_text(json.dumps({"statementId": sid, "label": label}))
    deadline = time.monotonic() + 180
    while response["status"]["state"] in ("PENDING", "RUNNING"):
        if time.monotonic() > deadline:
            raise RuntimeError("Inspect existing statement before continuing: " + sid)
        time.sleep(0.5)
        response = client.api_client.do("GET", "/api/2.0/sql/statements/" + sid)
    record = dict(label=label, sql=sql, parameters=parameters, response=response)
    records.append(record)
    with (OUT / "statements.jsonl").open("a") as stream:
        stream.write(json.dumps(record, ensure_ascii=False) + "\n")
    if error:
        assert response["status"]["state"] == "FAILED", record
        assert error in json.dumps(response["status"].get("error", {})), record
        return None
    assert response["status"]["state"] == "SUCCEEDED", record
    manifest = response.get("manifest", {})
    assert not manifest.get("truncated"), record
    assert manifest.get("total_chunk_count", 0) <= 1, record
    return response.get("result", {}).get("data_array", [])


engine = query("engine-version", "SELECT version() AS engine_version")
assert len(engine) == 1 and len(engine[0]) == 1
# Expectations use authored scalar sequences and exact base-ten strings.
assert query("scalar-sequence-equality", """SELECT
  :a = :a AS identical, :a = :b AS normalized,
  :c = :d AS padded, :e = :f AS empty_padded""", [
    {"name": name, "type": "STRING", "value": value}
    for name, value in zip("abcdef", ["é", "e\u0301", "x", "x ", "", " "])
]) == [["true", "false", "false", "false"]]
assert query("exact-decimal-carrier", """SELECT
  cast(cast(:v AS DECIMAL(28,2)) AS STRING) AS exact_value,
  cast(sum(cast(:v AS DECIMAL(28,2))) AS STRING) AS exact_sum
FROM VALUES (1), (2) t(n) GROUP BY cast(:v AS DECIMAL(28,2))""", [
    {"name": "v", "type": "STRING", "value": "99999999999999999999999999.99"}
]) == [["99999999999999999999999999.99", "199999999999999999999999999.98"]]
assert query("json-text-observation", """SELECT
  get_json_object(:payload, '$.23') AS exact_number,
  get_json_object(:payload, '$.24') AS explicit_null,
  get_json_object(:payload, '$.25') AS absent""", [
    {"name": "payload", "type": "STRING",
     "value": '{"23":18446744073709551615,"24":null}'}
]) == [["18446744073709551615", None, None]]
query("sum-overflow-refuses", """SELECT sum(cast(:v AS DECIMAL(38,0)))
FROM VALUES (1), (2) t(n)""", [
    {"name": "v", "type": "STRING", "value": "9" * 38}
], error="ARITHMETIC_OVERFLOW")
summary = dict(state="passed", warehouseId=WAREHOUSE, engine=engine[0][0],
               statements=len(records), harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
               qualification="Read-only semantic prerequisites on one existing warehouse. No compiler, layout admission, publication, delegation, performance or release qualification. JSON extraction conflates absent/null and is insufficient alone.")
(OUT / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
print(json.dumps(summary, indent=2))
