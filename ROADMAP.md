# Roadmap

Direction, not commitments. Nothing here is claimed as supported until CI tests it.

- Keep the UMF tag current (0.4.58 consumes `umf-v0.10.11`).
- Publish `umf-weft` to PyPI from the tag-driven release workflow; retire the `weft-sql` name.
- Run the native PostgreSQL 18.6 table-operation test in nightly CI against released
  `umf-access` and `umf-codegen` (the packages are not yet on PyPI at the required version).
- Broaden the generated read algebra beyond the current v6 to v8 shapes (more query
  forms, grouped and joined reads) with conformance cases for each.
- Security policy lowering for Truss reads (row and column authorization expressed in the plan).
- Ashlar/Databricks: execution-backed tests against a Databricks warehouse; today the
  backend is compile-only.
- Smaller and faster `cargo test --workspace`; move the slowest tests to nightly.
