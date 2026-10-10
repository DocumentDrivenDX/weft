---
ddx:
  id: TD-009
  type: technical-design
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: US-009
      kind: informed_by
    - id: FEAT-006
      kind: informed_by
    - id: SD-004
      kind: informed_by
    - id: CONTRACT-005
      kind: informed_by
---

# TD-009: Security foundation assurance

**Story:** [[US-009-security-compilation]]. **Parent solution:** [[SD-004-conformance-evolution]].

## Technical Approach and Assurance Level

CONTRACT-005 and authored UMF policy/ontology meaning govern the foundation.
US-009-AC1–AC4 remain open for full product acceptance. B-009 R5 provides precise
reviewed specifications plus bounded executable SMT checks of selected foundation
properties; it does not claim arbitrary compiler refinement or native enforcement.
Use `z3-solver==4.15.4.0`, pinned in host-only reliability requirements. Z3 checks
negated safety formulas for UNSAT and explicit success/failure witnesses for SAT;
unknown, timeout, missing solver or mismatched version fails the assurance gate.
[Official Z3 guide](https://microsoft.github.io/z3guide/programming/Z3%20Python/Introduction/)
governs solver result interpretation. Tool choice is a design pin, not evidence
that the tool or formulas have already run.

## State, Authority and Transitions

State comprises pinned source identity, admitted rule truths, selected field
protection/dispositions and public activation state. Truths are True/False/Unknown;
caller facts are simulations. Admission → evaluation → composition is a pure
foundation flow. Public 0.3 compilation transitions directly to unsupported
refusal before backend construction and cannot emit a security artifact.
Native authentication, complete cuts, current authority, publication and final
release are host assumptions for future integration, never simulator outputs.

## Properties and Bounded Executable Scope

| Stable property | Authority and safety assertion | Bounds / correspondence |
| --- | --- | --- |
| WFT-FM-001 | CONTRACT-005: public security requests cannot produce executable artifacts | Actual compile.rs refusal tests; guard/factory negative controls; no model proof of native behavior |
| WFT-FM-002 | CONTRACT-005: unknown scope-matching truth makes composition indeterminate with empty disclosure | One permit, one require, one forbid; three truth values each; all 27 combinations compared to authored oracle and actual Rust compose |
| WFT-FM-003 | CONTRACT-005: permit needs true permit, true require and false forbid; denial has no disclosure | Same finite rule set; denied/allowed SAT witnesses and deliberately broken permit gate |
| WFT-FM-004 | CONTRACT-005: withheld dominates; incompatible selected transform identities conflict | Selected one field, up to three active permit dispositions; abstract exact normalized identities; Rust exact-literal/custody tests supply correspondence |
| WFT-FM-005 | CONTRACT-005: a stale source/catalog cannot be admitted as the same prepared plan | Actual source/pin/admission tests; precise custody invariant rather than SMT hash-collision claim |

FM004 starts after the rule truth gate has admitted at least one active permit.
Slot0 means that an active permit contributes no disposition for the selected
field, not an absent grant. The Rust correspondence fixture uses three true
permit rules and one true requirement for all432 disposition/protection tuples;
it compares the independent authored table and reversed rule order. Three
transform equivalence classes cover up to three distinct exact identities.

Formal abstraction separately encodes authored decision/disclosure rules and a
bounded counterpart of implementation behavior. R5 records formulas, source
fingerprints, tool/version, SAT witnesses, UNSAT results and broken-model SAT
counterexamples. Real Rust tests check oracle correspondence and exact literals;
model success alone cannot establish implementation refinement. Transform identity
abstraction excludes scalar arithmetic, hash collision resistance and arbitrary
AST induction. Population/expression/resource bounds remain actual Rust test and
review obligations, not hidden solver assumptions.

## Liveness and Exclusions

Finite SMT queries use a fixed timeout and treat unknown as failure. Pure Rust
simulation has existing finite row/expression/payload budgets, reviewed and tested
at boundaries; no temporal native-service liveness is proved. Authentication,
Record-backed native evidence, arbitrary source/IR/evaluator equivalence, physical
lowering, private predicate effects and protected release remain deferred.

## Component Changes and Interfaces

Retain the existing blocked B-008 source, independent truth/composition oracles
and source fixtures. R5 adds host-only formulas/checker/evidence, not a new public
security activation path. CONTRACT-005 owns exact APIs. Core remains deterministic
and does not import solver, configuration, logger, storage or credentials.

## Testing

STP-009 distinguishes component admission/simulation tests from full acceptance.
US-008-AC6 (owned by STP-008) covers exhaustive allocation and scoped formal evidence. Record missing
correspondence or negative-control failures as failures. Every property must have
an authority, code/test mapping and explicit limits in retained R5 evidence.

## Migration, Trade-offs and Rollback

No native schema migration. Bounded assurance gives inspectable counterexamples
at low execution cost but cannot prove larger populations, native custody or
arbitrary policy semantics. Withdraw a property claim when its mapping changes;
keep public activation blocked until separately governed native acceptance passes.

## R5 executable gate

`python scripts/reliability/gate.py` consumes CONTRACT-006 configuration. It runs
the pinned solver and exact selected Rust correspondence tests; missing tools,
unknown/timeout, mismatched solver versions, zero/ignored/failed selected tests
or an altered oracle fail qualification. `formal.py` contains separate authored
tables and implementation abstractions, negated UNSAT checks, SAT witnesses and
deliberately broken permit/withheld counterexamples. Host-only dependencies
remain outside the core. Retained evidence maps each stable property to its
actual code, test and fixture fingerprints; no full FR19–22 acceptance is claimed.
