# Exact Truss registration numeric-domain qualification

All 1,124 cases pass through `truss.postgresql` version `0.1.0-native-review`,
profile `pg17.9-native-review`. Each storage home (JSONB props and typed numeric
rows) covers all 434 decimal precision 1..28 / scale 0..precision pairs and
128 signed/unsigned integer widths 1..64. This evidence directly exercises the
registration intended for qualification, rather than transferring the older
original-definition backend's claims. Original evidence remains unchanged.

One PostgreSQL 17.9 Debian repeatable-read rollback transaction executes 4,496
assertions: a zero valid-input guard, exact repeated-boundary SUM, a guard counting
five invalid inputs, and nullable empty SUM per case. Decimal invalid inputs are
both precision overflow directions, excessive scale, explicit null and absence.
Integer inputs are both range overflow directions, a fraction, null and absence.
The exact UTF8/C, standard-conforming-string and transaction settings are captured
inside the executing transaction. Temporary data is discarded by rollback.

Independent reconciliation calculates every expected result from integer
coefficients, checks complete domain/home coverage and exact executed SQL, fixture
values, parameters, guards, original module/binding snapshots and result scale/
decoding metadata. Eight semantic controls refuse altered sums/guards, empty SUM
coercion, engine drift, weakened expectations, changed decoding scale and OR
substitution, after recomputing valid custody hashes.

The compiler hash is verified before and after execution. Compressed artifacts and
executed SQL, native CSV, frozen producer, source/custody hashes and reconciliation
receipts live in `B-007-truss-review-numerics/`. These are exact scalar-domain
semantics over owned fixtures, not stored application-table/publication custody or
production authorization. Qualification still needs separately versioned supported
manifest/assessment declarations and final host/acceptance checks; this review
registration remains Candidate.
