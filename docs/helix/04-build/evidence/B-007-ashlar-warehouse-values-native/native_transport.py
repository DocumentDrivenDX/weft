"""Test-only authenticated statement transport; no provisioning or write retry."""
import json
from pathlib import Path
import time
from databricks.sdk import WorkspaceClient

class NativeFailure(RuntimeError):
    pass

class Client:
    def __init__(self, output):
        self.output = Path(output)
        self.output.mkdir(parents=True, exist_ok=True)
        self.workspace = WorkspaceClient(profile="aidev-cus")
        self.records = []

    def sql(self, label, statement, parameters=None):
        body = dict(warehouse_id="2439e1f2e37ac563", statement=statement,
                    wait_timeout="10s", on_wait_timeout="CONTINUE", disposition="INLINE",
                    format="JSON_ARRAY", row_limit=1000)
        if parameters is not None:
            body["parameters"] = parameters
        response = self.workspace.api_client.do("POST", "/api/2.0/sql/statements", body=body)
        sid = response["statement_id"]
        (self.output / "live-statement.json").write_text(json.dumps(dict(label=label, statementId=sid)))
        deadline = time.monotonic() + 180
        while response["status"]["state"] in ("PENDING", "RUNNING"):
            if time.monotonic() > deadline:
                raise RuntimeError("Inspect existing statement before proceeding: " + sid)
            time.sleep(0.5)
            response = self.workspace.api_client.do("GET", "/api/2.0/sql/statements/" + sid)
        record = dict(label=label, sql=statement, parameters=parameters, response=response)
        self.records.append(record)
        with (self.output / "statements.jsonl").open("a") as stream:
            stream.write(json.dumps(record, ensure_ascii=False) + "\n")
        if response["status"]["state"] != "SUCCEEDED":
            raise NativeFailure(json.dumps(response["status"]))
        manifest = response.get("manifest", {})
        assert not manifest.get("truncated") and manifest.get("total_chunk_count", 0) <= 1
        return response.get("result", {}).get("data_array", [])
