# weft-sql

Native Python binding for the [Weft SQL compiler](https://github.com/DocumentDrivenDX/weft).
Distribution name: `weft-sql`; import name: `weft`.

```python
import weft
response_json = weft.compile_json(request_json)
```

`compile_json(str) -> str` compiles supplied pinned UMF modules and a source query.
It returns a complete JSON response or explicit refusal. It does not connect to
or execute SQL against a database, fetch models, authorize access, or run JavaScript.
Requests and responses follow Weft's versioned compile contracts.

The wheel selects the Truss/PostgreSQL and Ashlar/Databricks qualified-registration
features. Profile names and native semantics are narrower than operating-system
wheel compatibility. Consult the repository support inventory and current release
record for exact engine versions, domains, exclusions, and host obligations.
Candidate 0.4.1 PathsKeys (including COUNT(*)/HAVING) is exposed through
`compile_paths_keys_json(str) -> str`. Candidate 0.4.2 adds implicit arithmetic
labels and positioned outputs through `compile_implicit_arithmetic_json(str) -> str`.
These entrypoints require exact versioned requests, backend versions/profiles and
candidate opt-in; they do not silently change `compile_json` composition.
Protected-read security activation remains blocked. Package version `0.4.2`
is distinct from the compiler and backend version fields.
Changed compiler outputs require their own native requalification; historical
native receipts do not qualify a new source revision automatically.

CPython ABI3 wheels target Python 3.9 and later conventional GIL builds.
PyPy, free-threaded CPython, musl Linux and unvalidated architectures are excluded.
A platform/interpreter is supported only after its installed-wheel checks pass.

Licensed at your option under MIT or Apache-2.0. Third-party dependencies retain
their own licenses.
