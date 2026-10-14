---
ddx:
  id: TD-011
  type: technical-design
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: US-011
      kind: informed_by
    - id: SD-001
      kind: informed_by
    - id: weft.architecture
      kind: informed_by
    - id: CONTRACT-009
      kind: informed_by
    - id: CONTRACT-002
      kind: informed_by
    - id: CONTRACT-003
      kind: informed_by
---

# TD-011: Share mutation plans through Python and browser bindings

**User Story:** [US-011](../../01-frame/user-stories/US-011-portable-mutation-embedding.md). **Feature:** FEAT-001. **Solution Design:** SD-001.

## Scope

One vertical journey defined by US-011; inherit the [parent solution design](../solution-designs/SD-001-logical-dialect.md) and architecture's portable/compiler/native boundaries. Shared surfaces live in CONTRACT-009, CONTRACT-002, CONTRACT-003.

## Technical Approach

Reuse original UMF core/actions identity, input, declaration and assessment semantics. Select pinned artifacts/registries explicitly. Preserve original intent and use source-qualified refusal; never make a local dialect, key codec or authority substitute. Rust owns compilation; Python/WASM are thin bindings.

## Component Changes

Planned source files: `crates/weft-python/src/lib.rs; crates/weft-wasm/src/lib.rs; crates/weft-core/src/mutation.rs`. These additions are not present implementations.

| AC | Component realization |
| --- | --- |
| US-011-AC1 | python no sidecar through the governing contract and focused test below |
| US-011-AC2 | browser artifact parity through the governing contract and focused test below |
| US-011-AC3 | independent executor through the governing contract and focused test below |
| US-011-AC4 | no execution claim through the governing contract and focused test below |
| US-011-AC5 | read compatibility through the governing contract and focused test below |

## API/Interface Design

CONTRACT-009, CONTRACT-002, CONTRACT-003 define the exact shared shapes and methods. New exports follow those contracts; no signatures are independently defined here. Upstream schemas/fixtures are original owner inputs. Construct adapters only in the host registration boundary. Core cannot import native I/O/credentials; compiler cannot import Truss storage.

### Boundary realization

Register planned integration targets mutation_compile and mutation_embedding in `crates/weft-core/Cargo.toml` with the allocated source paths. Gate pure dependencies with `cargo check -p weft-core --target wasm32-unknown-unknown` plus mutation forbidden-I/O/registration tests; run explicit installed-Python and actual Chromium fixture runners under tests/mutation. No storage/driver crate enters the pure compiler.

## Data Model Changes

Add separate mutation request/selection/plan types; existing read IR and schemas remain unchanged.

## Integration Points

US-010; UMF US-902/903; real Truss execution is owned by Truss US-048. Unsupported/unavailable original dependencies refuse, with no fallback to fixtures or another executor.

## Security

Artifacts never load code or authenticate principals. Current protected read/execution/discovery authority remains executor-owned. Raw source/inputs stay out of diagnostics; retained manifests/outcomes require separately enforced access. Unknown relevant meaning blocks eligibility.

## Performance

Use the exact finite item/byte/node/deadline/result-capacity bounds in the contracts. Test both exact boundary and first overflow. No throughput/latency claim is made from structural tests.

## Testing

[STP-011](../../03-test/test-plans/STP-011-portable-mutation-embedding.md) allocates each criterion once to a planned test with canonical citation. Command: `cargo test -p weft-core --test mutation_embedding`. The command/test target is planned, not currently passing evidence. Independent exact effects/refusals are the oracle, not generated output snapshots alone.

## Migration & Rollback

New profiles are opt-in; existing action/read/group behavior remains intact. Disable new admission while retaining all original manifests, terminal outcomes/tombstones and unresolved recovery custody. Do not reverse committed batches or drop native homes to simulate rollback.

## Implementation Sequence

1. Create the planned red tests in `tests/mutation/embedding.rs` with canonical citations.
2. Implement contract-owned pure/component behavior in the named source files.
3. Integrate original dependency boundaries and run portable/native witnesses.
4. Retain exact versions, failed attempts, residual gaps and verified source correspondence.

## Risks

Incomplete upstream/native admission must block dependent execution without blocking independent pure work. Namespace/guard changes require new reviewed intent; they cannot migrate pending work silently. Architecture boundary check and concrete dependency closure must be current before public source readiness.

New bridge runners: tests/mutation/python-check.py and browser-check.mjs; allocate installed-wheel/actual-Chromium commands from STP-011, not a Rust-only replacement.
