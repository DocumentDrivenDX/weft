# Truss numeric facets — 2026-10-13

The next Python distribution 0.4.4 exposes Truss runtime backend 0.4.9.
It accepts the exact signed64 and decimal precision18/scale4 facet maps and
requires Truss runtime 0.39-candidate. Existing backends retain their admission
rules. Values and parameters use existing exact integer/decimal lowering;
no float conversion or rounding typmod is introduced.

The production Rust ABI3 wheel was built with Rust1.90.0 and maturin1.9.6.
Truss installed-wheel workflows exercise original UMF0.7/0.8 acceptance,
create/update/import/read and parameterized queries returning Python int and
Decimal without precision loss. Basic tests pass on PostgreSQL18.6/macOSarm64.
A broad Rust lib-test build remains blocked by an unrelated existing
nonexhaustive Shape::ExactScalar match in property_definition.rs; the release
library build and actual query execution pass.

This is a source iteration after published v0.4.3. No v0.4.4 release is claimed.
