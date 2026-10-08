"""Execute emitted scalar SQL against independently authored native Delta rows.

@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 (candidate component)
Owned synthetic tables only; no production/publication/delegation admission claim.
"""
from decimal import Decimal, localcontext
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys
import time
from databricks.sdk import WorkspaceClient

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(os.environ["WEFT_ASHLAR_EVIDENCE_OUTPUT"])
OUT.mkdir(parents=True, exist_ok=True)
SCHEMA = "client_dev.weft_b006_20261008_scalar"
TABLE = SCHEMA + ".object_current"
MANIFEST = SCHEMA + ".publication_manifest"
WAREHOUSE = "2439e1f2e37ac563"
SQL = "SELECT c.name, SUM(o.total) AS total FROM Customer c JOIN Orders o ON o.customer_id = c.id GROUP BY c.name"
client = WorkspaceClient(profile="aidev-cus")
statements = []


def sql(label, statement, parameters=None):
    body = dict(warehouse_id=WAREHOUSE, statement=statement, wait_timeout="10s",
                on_wait_timeout="CONTINUE", disposition="INLINE", format="JSON_ARRAY", row_limit=1000)
    if parameters is not None:
        body["parameters"] = parameters
    # Submissions are never retried. A timeout retains the exact live handle.
    response = client.api_client.do("POST", "/api/2.0/sql/statements", body=body)
    sid = response["statement_id"]
    (OUT / "live-statement.json").write_text(json.dumps(dict(label=label, statementId=sid)))
    deadline = time.monotonic() + 180
    while response["status"]["state"] in ("PENDING", "RUNNING"):
        if time.monotonic() > deadline:
            raise RuntimeError("Inspect existing statement before proceeding: " + sid)
        time.sleep(0.5)
        response = client.api_client.do("GET", "/api/2.0/sql/statements/" + sid)
    record = dict(label=label, sql=statement, parameters=parameters, response=response)
    statements.append(record)
    with (OUT / "statements.jsonl").open("a") as stream:
        stream.write(json.dumps(record, ensure_ascii=False) + "\n")
    if response["status"]["state"] != "SUCCEEDED":
        raise RuntimeError(json.dumps(response["status"]))
    manifest = response.get("manifest", {})
    assert not manifest.get("truncated") and manifest.get("total_chunk_count", 0) <= 1
    return response.get("result", {}).get("data_array", [])


def describe(table):
    rows = sql("identity-" + table.rsplit(".", 1)[-1], "DESCRIBE DETAIL " + table)
    columns = [c["name"] for c in statements[-1]["response"]["manifest"]["schema"]["columns"]]
    assert len(rows) == 1
    return dict(zip(columns, rows[0]))


def string_parameter(name, value):
    return dict(name=name, type="STRING", value=value)


MAX = 18446744073709551615
customers = [(MAX, "é"), (MAX, "é"), (7, "e\u0301"), (8, "x "), (9, "isolated")]
orders = [(MAX, "99999999999999999999999999.99"), (MAX, "0.02"), (7, "-1.25"), (8, "2.00"), (123, "3.00")]
cases = [dict(id="exact", customers=customers, orders=orders),
         dict(id="empty", customers=[], orders=[])]
for label, bad in [
    ("hidden-foreign-domain", '{"26":18446744073709551616,"27":1.00}'),
    ("hidden-decimal-scale", '{"26":123,"27":1.001}'),
    ("hidden-decimal-domain", '{"26":123,"27":100000000000000000000000000.00}'),
    ("hidden-number-string", '{"26":"123","27":1.00}'),
    ("hidden-exponent-carrier", '{"26":123,"27":1e-2}'),
    ("hidden-null", '{"26":123,"27":null}'),
    ("hidden-absent", '{"26":123}'),
]:
    cases.append(dict(id=label, customers=customers, orders=orders, badOrder=bad))
cases.append(dict(id="hidden-wrong-name", customers=customers, orders=orders,
                  badCustomer='{"23":42,"24":17,"25":true}'))


def oracle(case):
    with localcontext() as context:
        context.prec = 100
        groups = {}
        for key, name in case["customers"]:
            for foreign, total in case["orders"]:
                if key == foreign:
                    groups[name] = groups.get(name, Decimal(0)) + Decimal(total)
        return sorted([[name, format(total.quantize(Decimal("0.01")), "f")]
                       for name, total in groups.items()])


rows = []
for case in cases:
    source = "weft-b006-" + case["id"]
    for i, (key, name) in enumerate(case["customers"], 1):
        rows.append(dict(source_system=source, type_id=1, id=i,
                         props_json='{"23":' + str(key) + ',"24":' + json.dumps(name) + ',"25":true}'))
    for i, (foreign, total) in enumerate(case["orders"], 1):
        rows.append(dict(source_system=source, type_id=2, id=i,
                         props_json='{"26":' + str(foreign) + ',"27":' + total + '}'))
    for kind, type_id in [("badCustomer", 1), ("badOrder", 2)]:
        if kind in case:
            rows.append(dict(source_system=source, type_id=type_id, id=99, props_json=case[kind]))
payload = json.dumps(rows, ensure_ascii=False, separators=(",", ":"))
payload_sha = hashlib.sha256(payload.encode()).hexdigest()
layout = (ROOT / "spec/upstream/ashlar-delta-v03.sql").read_text()
layout_sha = hashlib.sha256(layout.encode()).hexdigest()
assert layout_sha == "ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e"
state_path = OUT / "fixture-state.json"
if "--resume-pinned" in sys.argv:
    state = json.loads(state_path.read_text())
    assert state["phase"] == "ready" and state["payloadSha256"] == payload_sha
else:
    assert not state_path.exists(), "Inspect existing fixture state; do not replay mutations"
    assert sql("schema-custody", "SHOW SCHEMAS IN client_dev LIKE 'weft_b006_20261008_scalar'") == []
    sql("create-owned-schema", "CREATE SCHEMA " + SCHEMA)
    for table in ["object_current", "publication_manifest"]:
        ddl = re.search(r"CREATE TABLE " + table + r" \(.*?;", layout, re.S).group(0)
        sql("create-" + table, ddl.replace("CREATE TABLE " + table, "CREATE TABLE " + SCHEMA + "." + table, 1))
    sql("insert-independent-fixtures", """INSERT INTO """ + TABLE + """
    SELECT r.source_system, r.type_id, r.id, cast(r.id AS STRING), 'fixture-schema-1',
      1, r.props_json, '{}', NULL, 'weft-fixture-feed', 'fixture-epoch', NULL,
      TIMESTAMP '2026-10-08 00:00:00',
      sha2(to_json(named_struct('source_system',r.source_system,'type_id',r.type_id,'id',r.id)),256),
      'weft-fixture-batch', '{"position":"fixture"}', cast(r.id AS STRING)
    FROM (SELECT explode(from_json(:rows,
      'ARRAY<STRUCT<source_system:STRING,type_id:BIGINT,id:BIGINT,props_json:STRING>>')) r)""",
        [string_parameter("rows", payload)])
    detail = describe(TABLE)
    history = sql("fixture-history", "DESCRIBE HISTORY " + TABLE + " LIMIT 1")
    version = int(history[0][0])
    publication_id = "weft-b006-scalar-fixtures"
    sql("publish-fixture-vector", "INSERT INTO " + MANIFEST + " VALUES (:id, 'ashlar-delta/0.3', :versions, '{}', :revisions, :report, TIMESTAMP '2026-10-08 00:00:00')", [
        string_parameter("id", publication_id),
        string_parameter("versions", json.dumps({TABLE: version})),
        string_parameter("revisions", json.dumps({"weft-fixture-feed": "fixture-schema-1"})),
        string_parameter("report", json.dumps({"qualification": "synthetic compiler fixtures; corruption intentional", "payloadSha256": payload_sha})),
    ])
    state = dict(phase="ready", payloadSha256=payload_sha, tableUuid=detail["id"], version=version,
                 manifestUuid=describe(MANIFEST)["id"], publicationId=publication_id)
    state_path.write_text(json.dumps(state, indent=2) + "\n")

assert describe(TABLE)["id"] == state["tableUuid"]
assert describe(MANIFEST)["id"] == state["manifestUuid"]
publication = sql("resolve-fixture-vector", "SELECT table_versions_json FROM " + MANIFEST + " WHERE publication_id=:id", [string_parameter("id", state["publicationId"])])
assert publication == [[json.dumps({TABLE: state["version"]})]]
document = (ROOT / "docs/helix/03-test/fixtures/sales.umf.json").read_text()
pin = dict(documentId="sales-fixture", revision="1", umfVersion="0.7.0", sha256=hashlib.sha256(document.encode()).hexdigest())


def identity(element):
    return dict(documentId="sales-fixture", revision="1", module="sales", element=element)


outcomes = []
for case in cases:
    records = []
    for record, type_id, fields in [("customer", "1", [("customer-id", "23"), ("customer-name", "24"), ("customer-active", "25")]),
                                   ("orders", "2", [("order-customer", "26"), ("order-total", "27")])]:
        records.append(dict(logical=identity(record), table=0, kind="object", sourceSystem="weft-b006-" + case["id"], typeId=type_id,
                            schemaRevision="fixture-schema-1", properties=[dict(logical=identity(e), home=dict(kind="props", propertyId=p)) for e, p in fields]))
    binding = json.dumps(dict(profile="ashlar-databricks-candidate/0.1.0", layoutRevision="ashlar-delta/0.3", layoutSha256=layout_sha,
                             modelPins=[pin], publication=dict(id=state["publicationId"], manifestUuid=state["manifestUuid"],
                             tables=[dict(name=TABLE.split("."), uuid=state["tableUuid"], version=state["version"])]), records=records))
    request = dict(interfaceVersion="weft-compile/0.1.0", dialect="weft-sql/0.1.0", sql=SQL,
                   modules=[dict(documentJson=document, pin=pin, selectedModuleIds=["sales"])],
                   target=dict(backendId="ashlar.databricks", backendVersion="0.1.0-candidate", targetProfile="dbsql-candidate",
                               bindingJson=binding, bindingSha256=hashlib.sha256(binding.encode()).hexdigest()), options=dict(allowCandidate=True))
    process = subprocess.run([os.environ["WEFT_ASHLAR_COMPILER"]], input=json.dumps(request), text=True, capture_output=True, check=True)
    artifact = json.loads(process.stdout)
    (OUT / (case["id"] + "-compile.json")).write_text(json.dumps(dict(request=request, response=artifact), indent=2, ensure_ascii=False) + "\n")
    assert artifact["status"] == "compiled", artifact
    parameters = [string_parameter("p" + str(p["position"]), p["value"]) for p in artifact["parameters"]]
    obligation = next(o for o in artifact["obligations"] if o["id"] == "ashlar.candidate.scalarIntegrity")
    counts = [sql(case["id"] + "-integrity-" + str(i), check["sql"], parameters)
              for i, check in enumerate(obligation["parameters"]["checks"])]
    corrupt = "badOrder" in case or "badCustomer" in case
    assert all(len(c) == 1 and len(c[0]) == 1 and type(c[0][0]) is str for c in counts)
    if corrupt:
        assert any(int(c[0][0]) > 0 for c in counts), (case["id"], counts)
        outcome = dict(id=case["id"], outcome="refused-before-user-query", counts=counts)
    else:
        assert all(c == [["0"]] for c in counts), (case["id"], counts)
        actual = sql(case["id"] + "-user-query", artifact["sql"], parameters)
        assert sorted(actual) == oracle(case), (actual, oracle(case))
        columns = statements[-1]["response"]["manifest"]["schema"]["columns"]
        assert [(c["name"], c["type_name"]) for c in columns] == [("name", "STRING"), ("total", "STRING")]
        outcome = dict(id=case["id"], outcome="published-fixture-result", rows=actual)
    outcomes.append(outcome)

summary = dict(state="passed", cases=len(cases), outcomes=outcomes, fixture=state, warehouseId=WAREHOUSE,
               harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), ownerLayoutSha256=layout_sha,
               qualification="Actual registered Rust compiler and native Delta scalar SQL. Synthetic admin fixtures, fixed immutable vector. No accepted source binding, live delegation/pin custody, application-read, Python/browser or production qualification.")
(OUT / "summary.json").write_text(json.dumps(summary, indent=2, ensure_ascii=False) + "\n")
print(json.dumps(summary, indent=2, ensure_ascii=False))
