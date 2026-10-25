# PR30 owned-surface diagnostic evidence — 2026-10-25

This retained diagnostic is run `40242991838`, attempt 1, against triggering
source `1b035f0d97783e13dc028921588316687199af86` (`codex/weft-owned-surface-diagnostic`).
The report is complete for all 11 packages: 10 exact matches and one ordered
public-declaration mismatch in `crates/weft-databricks/src/lib.rs`. It reports no
missing or stale paths, duplicate paths, omitted discrepancies, or truncation.

The only mismatch was `mod:unity_current_read` at declaration index 29. The
module was already declared in the source; the expected inventory omitted it
before `mod:local_spark4_sdk`. The inventory now includes that declaration.
This correction is based on the complete recorded run; the diagnostic has not
been rerun against the corrected inventory. The corrected map is **UNRUN** until
the normal PR gate checks it.

`report.json`, `receipt.tsv`, `commands.tsv`, and `diagnostic.log` are retained
verbatim from the hosted artifact. Their SHA-256 values are in `SHA256SUMS`.
