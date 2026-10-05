# Working in Weft

Weft is a universal UMF-powered SQL dialect with pluggable backends. Initially:
Ashlar on Databricks and Truss on PostgreSQL. Start with docs/helix/README.md,
read .helix.yml, and engage the installed HELIX skill for governed work.

- Keep governed artifacts under docs/helix/ in the matching activity.
- Resolve the graph/templates/prompts from the installed HELIX plugin; never
  copy the methodology catalog into this repo.
- Read upstream artifacts before editing downstream specs. Preserve artifact
  IDs/frontmatter and deliberate ddx.links. Draft status never means approval.
- Weft owns source-dialect semantics and the logical plan. UMF owns metamodel
  meaning. Backends own physical mappings, target SQL and execution obligations.
- Never silently lose meaning. Preserve unknown model content; block selected
  semantics that cannot be established. Qualify claims by versions/subset/evidence.
- Rust is the proposed compiler core; native Python and browser WASM bindings
  must share that core. No JavaScript sidecar is permitted for the Python API.
- Keep connections, credentials, data execution, authorization and publication
  enforcement in hosts/backends. Model content never loads executable plugins.
- The v0.1 grammar is a bounded first delivery slice, not Weft's product ceiling.
  New capabilities extend governed language/plan contracts and conformance data.
- The bootstrap is specifications only. Do not report compiler, plugin, Python,
  browser or database support without fresh executable evidence.
