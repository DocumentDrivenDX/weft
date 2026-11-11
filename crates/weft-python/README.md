# umf-weft

Python bindings for [Weft](https://github.com/DocumentDrivenDX/weft), a compiler
that turns a request (a UMF document, a storage binding and a query) into SQL for
a chosen backend: Truss on PostgreSQL or Ashlar on Databricks. The distribution
is `umf-weft`; the import is `weft`. Functions take a JSON request and return a
JSON response with SQL, parameters, result types and diagnostics, or an explicit
refusal. Weft never connects to a database or executes SQL.

Status: experimental.

## Install

```sh
pip install umf-weft
```

Wheels are abi3 and support CPython 3.9 or later on manylinux x86_64, manylinux
aarch64 and macOS arm64.

## Quickstart

```python
import json, weft

# Any request object from the repository's examples; this one selects one field.
request = json.load(open("plain-field.request.json"))
result = json.loads(weft.compile_truss_runtime_json(json.dumps(request)))
assert result["status"] == "compiled", result
print(result["sql"])
```

Get [plain-field.request.json](https://github.com/DocumentDrivenDX/weft/blob/main/examples/plain-field.request.json)
and the full [quickstart](https://github.com/DocumentDrivenDX/weft/blob/main/examples/quickstart.py)
from the repository. CI runs the quickstart against each built wheel.

## Links

[Source and issues](https://github.com/DocumentDrivenDX/weft) ·
[Changelog](https://github.com/DocumentDrivenDX/weft/blob/main/CHANGELOG.md) ·
[Contracts](https://github.com/DocumentDrivenDX/weft/tree/main/docs/helix/02-design/contracts)

Licensed at your option under MIT or Apache-2.0. Dependencies retain their own
licenses and bundled notices.
