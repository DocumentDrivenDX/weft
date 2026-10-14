---
ddx:
  id: TD-010
  type: technical-design
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: US-010
      kind: informed_by
    - id: SD-001
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
    - id: CONTRACT-009
      kind: informed_by
    - id: CONTRACT-001
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# TD-010: Compile bulk UPDATE to registered UMF actions

**User Story:** [US-010](../../01-frame/user-stories/US-010-compile-bulk-update.md). **Feature:** FEAT-001. **Solution Design:** SD-001.

## Scope

One vertical journey defined by US-010; inherit the [parent solution design](../solution-designs/SD-001-logical-dialect.md) and architecture's portable/compiler/native boundaries. Shared surfaces live in CONTRACT-009, CONTRACT-001, CONTRACT-003.

## Technical Approach

Reuse original UMF core/actions identity, input, declaration and assessment semantics. Select pinned artifacts/registries explicitly. Preserve original intent and use source-qualified refusal; never make a local dialect, key codec or authority substitute. Rust owns compilation; Python/WASM are thin bindings.

## Component Changes

Planned source files: `crates/weft-core/src/mutation.rs; crates/weft-core/src/mutation_selection.rs; crates/weft-postgresql/src/mutation_selection.rs; crates/weft-core/src/lib.rs`. These additions are not present implementations.

| AC | Component realization |
| --- | --- |
| US-010-AC1 | update plan through the governing contract and focused test below |
| US-010-AC2 | name and input resolution through the governing contract and focused test below |
| US-010-AC3 | bounded syntax through the governing contract and focused test below |
| US-010-AC4 | predicate exactness through the governing contract and focused test below |
| US-010-AC5 | compiler limits through the governing contract and focused test below |
| US-010-AC6 | selection_lowering through the governing contract and focused independent witness |

## API/Interface Design

CONTRACT-009, CONTRACT-001, CONTRACT-003 define the exact shared shapes and methods. New exports follow those contracts; no signatures are independently defined here. Upstream schemas/fixtures are original owner inputs. Construct adapters only in the host registration boundary. Core cannot import native I/O/credentials; compiler cannot import Truss storage.

### Boundary realization

Register planned integration targets mutation_compile and mutation_embedding in `crates/weft-core/Cargo.toml` with the allocated source paths. Gate pure dependencies with `cargo check -p weft-core --target wasm32-unknown-unknown` plus mutation forbidden-I/O/registration tests; run explicit installed-Python and actual Chromium fixture runners under tests/mutation. No storage/driver crate enters the pure compiler.

## Data Model Changes

Add separate mutation request/selection/plan types; existing read IR and schemas remain unchanged.

## Integration Points

UMF US-902 / CONTRACT-902 and shared original core/actions corpus; registry supplied explicitly. Unsupported/unavailable original dependencies refuse, with no fallback to fixtures or another executor.

## Security

Artifacts never load code or authenticate principals. Current protected read/execution/discovery authority remains executor-owned. Raw source/inputs stay out of diagnostics; retained manifests/outcomes require separately enforced access. Unknown relevant meaning blocks eligibility.

## Performance

Use the exact finite item/byte/node/deadline/result-capacity bounds in the contracts. Test both exact boundary and first overflow. No throughput/latency claim is made from structural tests.

## Testing

[STP-010](../../03-test/test-plans/STP-010-compile-bulk-update.md) allocates each criterion once to a planned test with canonical citation. Command: `cargo test -p weft-core --test mutation_compile`. The command/test target is planned, not currently passing evidence. Independent exact effects/refusals are the oracle, not generated output snapshots alone.

## Migration & Rollback

New profiles are opt-in; existing action/read/group behavior remains intact. Disable new admission while retaining all original manifests, terminal outcomes/tombstones and unresolved recovery custody. Do not reverse committed batches or drop native homes to simulate rollback.

## Implementation Sequence

1. Create the planned red tests in `tests/mutation/compile.rs` with canonical citations.
2. Implement contract-owned pure/component behavior in the named source files.
3. Integrate original dependency boundaries and run portable/native witnesses.
4. Retain exact versions, failed attempts, residual gaps and verified source correspondence.

## Risks

Incomplete upstream/native admission must block dependent execution without blocking independent pure work. Namespace/guard changes require new reviewed intent; they cannot migrate pending work silently. Architecture boundary check and concrete dependency closure must be current before public source readiness.

US-010-AC6 owns backend selection capability/lowering; add integration target mutation_lowering -> tests/mutation/lowering.rs in weft-postgresql Cargo.toml and run `cargo test -p weft-postgresql --features conformance-original --test mutation_lowering` against independently observed PostgreSQL projection/preflight/order/overflow behavior. Pure model match consumes host-installed original UMF preparation; new arbitrary declaration interpretation is not implemented in Rust.
