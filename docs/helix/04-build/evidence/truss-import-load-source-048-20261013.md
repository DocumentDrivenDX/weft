# Truss import load projection source — 2026-10-13

Backend0.4.8 targets Truss0.35 and compile/IR/backend interfaces0.2.4. Entity-read0.3 adds nullable-string LoadId with exact JSON text transport, original native helper lowering and output-name collision refusal. Earlier schemas and entity profile output contracts remain unchanged. Pages admit the new transport.

Rust core/PostgreSQL/Python checking with truss-postgresql-pg18 passed; module boundaries checked11 packages; diff check passed. Native installation/upgrade/Python execution remain pending in Truss. This is source readiness, not public runtime activation or a passing consumer provenance case.
