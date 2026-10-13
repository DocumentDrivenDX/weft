# Truss runtime0.36 compatibility

The existing import-load backend0.4.8 accepts exactly runtime0.35 or additive runtime0.36, preserving the previous profile and every other release comparison. Truss0.36 combines original core0.8 catalog metadata with the unchanged native import load helper. The backend still checks exact release digest, installation/schema/catalog identities, limits and complete binding inventories; this amendment authenticates no issuer and activates no protected security profile.

Original Python module build passes. Actual PostgreSQL18.6 public import projection passes pending and committed with exact load/null/author/caller values in7.986seconds. Complete cargo test-postgresql remains blocked by its pre-existing nonexhaustive ExactScalar test fixture match; no full-suite pass is claimed. Truss retains native logs and source-pinned Astra review separately.
