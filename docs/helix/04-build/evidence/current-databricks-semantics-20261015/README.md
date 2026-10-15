# Managed Databricks current-value dialect observations

Six constant-only statements executed through the existing approved aidev-cus
profile and running data-gateway SQL warehouse. The response receipt retains
actual statement IDs, SQL, parameter values, result schema and rows. All inputs
are public test constants. No schema/table was created and no user data was read.

Run `python3 verify.py receipt.json` for the finite value/shape oracle and six
semantic receipt corruption controls. `native.py` performs fresh managed SQL and
requires the approved CLI profile; it never prints credentials. Its original
runner digest is retained in the receipt. A stored receipt verification is not
a fresh native execution. `request.json` from the local runner is intentionally
omitted because each executed SQL/parameter set is already retained in receipt.

This evidence informs the proposed CONTRACT-006 Databricks consumer. It does not
prove Weft emitted these statements, native source/caller/current authority,
Ashlar layout/binding admission, time travel, complete arithmetic capacity,
second-caller isolation or any original protected backend acceptance case.
