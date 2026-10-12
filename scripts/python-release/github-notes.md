Native Python wheels for the UMF-powered Weft SQL compiler.

Install a matching wheel with `pip install PATH_TO_WHEEL.whl`. Import `weft`.
The package version is 0.4.2; compiler, dialect and backend versions remain
separate fields and must match the selected request/profile.

- `compile_json` retains the two qualified-registration profiles.
- `compile_paths_keys_json` exposes candidate 0.4.1 PathsKeys and COUNT(*)/HAVING.
- `compile_implicit_arithmetic_json` exposes candidate 0.4.2 implicit arithmetic labels.
- All three run natively without a JavaScript sidecar. The compiler does not
  connect to databases or enforce host authorization/publication obligations.
- Licensed MIT OR Apache-2.0, with both licenses and dependency notices in each wheel.

This is a regular package release. Candidate APIs remain experimental and require
exact backend versions/profiles and explicit candidate opt-in. Protected-read
security activation stays blocked. Historical native evidence remains scoped:
53 changed ordinary outputs require native requalification. Wheel compatibility
and compiler parity do not establish additional database or production support.

The attached CPython ABI3 wheels target conventional GIL CPython 3.9–3.14 on their
named platforms. PyPy, free-threaded CPython, musl Linux and other architectures
are excluded. No source distribution is included. See the attached validation
reports and SHA256SUMS for tested interpreter versions and exact artifact hashes.
