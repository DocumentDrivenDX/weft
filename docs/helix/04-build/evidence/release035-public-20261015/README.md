# Public Weft 0.4.35 artifacts

[Public release](https://github.com/DocumentDrivenDX/weft/releases/tag/v0.4.35)
selects source/tag ff3695af5ece9d386b3dcbf266cf23d632997733. Build-only run
38837705038 passes all five platforms. Normal tagged run 38839037643 repeats those
checks and passes publication. The original run/job metadata is retained.

The verifier checks all 22 official asset digests, five wheel checksums and native
module/shim bytes, all three embedded smoke families and exact source/version
anchors. Each platform has four current-PG checks, 32 Databricks controls and 30
named refinement outcomes; release CI also runs 17 query ABI controls and 20
publication-validator controls. `release-pins.json` distinguishes the closed
runtime backend 0.4.37/native68 from the new explicit query-ABI backend 0.4.38.

Actual installed public macOS ARM wheel:

- Wheel SHA-256: 7b8e37c34cf157bf399bda7b430b4891d7a0b37d87db16979ade0355e0b0d79d
- Native module: b3467b40a3b80a33b71f7400cdf513cb064c82bf64cd91c239afa41cbb204e4f
- Shared shim: cd0fa27c9580742333d14e24d5ebfd74babb47d31720bb4c577278d05486f11d

The actual installed public wheel passes 67 fresh PostgreSQL 18.6 controls against
original independent fixture oracles, with stopped server/released lease/removed
data directory; 229 offline controls over 22 retained managed Databricks queries;
30 candidate API and 17 query ABI controls. The unchanged retained public Truss 0.0.33
consumer gate refuses the actual new module through all three entrypoints.
These are separate public-byte executions, not relabeled prototype receipts.
Local sourceCommit=null in the standalone smoke reflects its non-Git export;
public source attribution comes from the verified tag/workflow/embedded reports.
No fresh managed Databricks run was performed.

All platform wheel/module/shim pins are retained in verified-public.json and the
original official release-pins.json. Truss and Ashlar received those actual
public cohorts, not local prototype hashes, for explicit integration. Ashlar's
opt-in checker adapter is on main d7167a20; genuine managed capture and authority
remain separate host gates. This compiler release issues no native permission.

All 132 original native/shared requirements remain required. Previously accepted
cases remain accepted; this increment promotes zero native cases. B-008 remains
incomplete. No accepted ledger is reset by the earlier shorthand that its full
native acceptance remained open.
