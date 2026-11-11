# SPDX-License-Identifier: MIT OR Apache-2.0
"""Compile a host-supplied request to PostgreSQL SQL with the umf-weft wheel.

The request names a UMF document, a Truss runtime binding and a query. Weft
returns SQL, parameters and result decoding; it never connects to a database.
"""
import json
import pathlib

import weft

request = json.loads(pathlib.Path(__file__).with_name("plain-field.request.json").read_text())
result = json.loads(weft.compile_truss_runtime_json(json.dumps(request)))
assert result["status"] == "compiled", result
print(result["sql"])
