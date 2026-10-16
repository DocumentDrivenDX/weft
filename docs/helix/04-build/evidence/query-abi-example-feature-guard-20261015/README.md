# Query ABI example feature guard — 2026-10-15

Base: `fa55e7b73117544d6351b0f866abe0d6a28c9f65`. The four-line Cargo example declaration requires `truss-postgresql-pg18`, matching the feature-gated callee. Default example builds previously failed with E0425. No library, SQL, checker, or Python implementation changes.

Retained owner logs show the original failure and successful default examples and explicit PG18 example checks. Independent Astra review is static only; root review confirms the exact manifest delta. Neither review replays the owner executions. Fresh five-platform CI remains pending when this packet is committed. No native acceptance case is promoted; all original 132 shared/native cases remain required.
