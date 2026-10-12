"""Finite admission-design model, not native SQL or executable-compiler proof."""
import json
import z3
N = 5
required = [z3.Bool(f'required_{s}') for s in range(N)]
enforced = [z3.Bool(f'enforced_{o}') for o in range(N)]
proof = [z3.Bool(f'accepted_evidence_{o}') for o in range(N)]
current = [z3.Bool(f'current_context_{o}') for o in range(N)]
refused = [z3.Bool(f'refused_{o}') for o in range(N)]
rank = [z3.Int(f'rank_{o}') for o in range(N)]
claims = [[z3.Bool(f'claims_{o}_{s}') for s in range(N)] for o in range(N)]
dep = [[z3.Bool(f'dep_{o}_{p}') for p in range(N)] for o in range(N)]
structure = [z3.And(rank[o] >= 0, rank[o] < N) for o in range(N)]
structure += [z3.Implies(dep[o][p], rank[p] < rank[o]) for o in range(N) for p in range(N)]
structure += [z3.Implies(claims[o][s], required[s]) for o in range(N) for s in range(N)]
settlement = [z3.Implies(enforced[o], z3.And(proof[o], current[o], z3.Not(refused[o]),
    *[z3.Implies(dep[o][p], enforced[p]) for p in range(N)])) for o in range(N)]
coverage = [z3.Implies(required[s], z3.Or(*[z3.And(claims[o][s], enforced[o]) for o in range(N)])) for s in range(N)]
ready = structure + settlement + coverage
observations = []
def check(name, constraints, expected):
    solver = z3.Solver(); solver.add(*constraints); outcome = str(solver.check())
    assert outcome == expected, (name, outcome)
    item = {'name': name, 'expected': expected, 'observed': outcome}
    if outcome == 'sat':
        model = solver.model()
        item['witness'] = {str(d): str(model[d]) for d in model.decls()}
    observations.append(item)
check('required_source_cannot_be_uncovered', ready + [required[0], z3.Not(z3.Or(*[claims[o][0] for o in range(N)]))], 'unsat')
check('stale_only_cover_cannot_publish', ready + [required[0]] + [z3.Implies(claims[o][0], z3.Not(current[o])) for o in range(N)], 'unsat')
check('refused_prerequisite_cannot_settle_consumer', ready + [enforced[0], dep[0][1], refused[1]], 'unsat')
check('unverified_prerequisite_cannot_settle_consumer', ready + [enforced[0], dep[0][1], z3.Not(proof[1])], 'unsat')
check('two_node_dependency_cycle_refuses', ready + [dep[0][1], dep[1][0]], 'unsat')
check('unowned_semantic_source_refuses', ready + [claims[0][0], z3.Not(required[0])], 'unsat')
check('valid_chain_with_joint_coverage_exists', ready + [required[0], required[1], claims[0][0], claims[1][1], enforced[0], enforced[1], dep[1][0]], 'sat')
# Weak declaration-only checking has no owner-source coverage constraint.
check('weak_all_declared_enforced_misses_authority_source', structure + settlement + [required[4]] + enforced + [z3.Not(claims[o][4]) for o in range(N)], 'sat')
# Caller-asserted proof flags cannot establish the truth of a native mechanism.
native_enforces = z3.Bool('native_actually_enforces')
check('accepted_flags_alone_do_not_prove_native_enforcement', ready + [required[0], claims[0][0], enforced[0], z3.Not(native_enforces)], 'sat')
print(json.dumps({'version': 'weft.security.obligation-design-model/0.1.0', 'z3': z3.get_version_string(), 'bound': {'sources': N, 'obligations': N}, 'observations': observations, 'limits': ['Conditional finite relational design only; proof/current flags are assumed accepted facts, not authenticated evidence.', 'No refinement proof for Rust, PostgreSQL, Truss, Delta or Ashlar; no native SQL semantics or lifetime claim.']}, indent=2))
