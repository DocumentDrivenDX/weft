/** @covers US-008-AC6 Allocation validity never claims executed acceptance. */
export type Artifact = { meta: { id: string; type: string }; body: string };
export type Allocation = { story: string; criteria: { id: string; plannedTest: string; assertion: string; state: string }[] }[];
function require(ok: unknown, message: string): asserts ok { if (!ok) throw new Error(message); }
function equal(actual: Set<string>, expected: Set<string>, label: string) {
  require(actual.size === expected.size && [...expected].every(id => actual.has(id)), `${label}: incomplete or extra allocation`);
}
export function checkAllocation(artifacts: Map<string, Artifact>, allocation: Allocation, requirements: Record<string, string[]>): Set<string> {
  const prd = artifacts.get('weft.prd');
  require(prd, 'Missing PRD');
  // Extract normative declaration rows, not references/ranges elsewhere in prose.
  const declared = new Set([...prd.body.matchAll(/^(?:- \*\*(FR-\d+):\*\*|\| (FR-\d+) \|)/gm)].map(m => m[1] ?? m[2]));
  require(declared.size > 0, 'No declared PRD requirements');
  equal(new Set(Object.keys(requirements)), declared, 'PRD requirements');
  const stories = new Set([...artifacts.values()].filter(a => a.meta.type === 'user-stories').map(a => a.meta.id));
  require(Array.isArray(allocation), 'Invalid story allocation');
  const allocatedStories = new Set(allocation.map(s => s.story));
  require(allocatedStories.size === allocation.length, 'Duplicate story allocation');
  equal(allocatedStories, stories, 'Stories');
  const mappedStories = new Set<string>();
  for (const [requirement, targets] of Object.entries(requirements)) {
    require(Array.isArray(targets) && targets.length > 0 && new Set(targets).size === targets.length, `${requirement}: invalid requirement mapping`);
    for (const story of targets) { require(stories.has(story), `${requirement}: unknown story ${story}`); mappedStories.add(story); }
  }
  equal(mappedStories, stories, 'Requirement/story coverage');
  for (const story of stories) {
    const body = artifacts.get(story)!.body;
    const field = body.match(/\*\*PRD Requirements:\*\*([^\n]*)/);
    require(field, `${story}: missing explicit PRD Requirements field`);
    const stated = new Set([...field[1].matchAll(/FR-\d+/g)].map(m => m[0]));
    require(stated.size > 0 && [...stated].every(r => declared.has(r)), `${story}: unknown or empty declared requirements`);
    const mapped = new Set(Object.entries(requirements).filter(([, targets]) => targets.includes(story)).map(([r]) => r));
    equal(mapped, stated, `${story} declared requirement links`);
  }
  const criteria = new Set<string>();
  for (const story of allocation) {
    const us = artifacts.get(story.story)!;
    const stp = artifacts.get(`STP-${story.story.slice(3)}`);
    require(stp?.meta.type === 'story-test-plan' && Array.isArray(story.criteria) && story.criteria.length > 0, `${story.story}: missing test allocation`);
    const declaredCriteria = new Set([...us.body.matchAll(/^- \*\*(US-\d{3}-AC\d+):\*\*/gm)].map(m => m[1]));
    require(declaredCriteria.size > 0, `${story.story}: no declared criteria`);
    equal(new Set(story.criteria.map(c => c.id)), declaredCriteria, `${story.story} criteria`);
    for (const c of story.criteria) {
      require(c.id.startsWith(`${story.story}-AC`) && !criteria.has(c.id), `Duplicate or foreign criterion ${c.id}`);
      require(typeof c.plannedTest === 'string' && /^[a-z][a-z0-9_]+$/.test(c.plannedTest) && typeof c.assertion === 'string' && c.assertion.trim().length > 0, `${c.id}: missing test/assertion`);
      // Both IDs must occur on the actual mapping row, not independent prose mentions.
      require(stp.body.split('\n').some(row => row.startsWith(`| ${c.id} |`) && row.includes('`'+c.plannedTest+'`')), `Missing trace ${c.id}`);
      require(['planned','open','deferred'].includes(c.state), `${c.id}: allocation cannot claim execution`);
      criteria.add(c.id);
    }
  }
  return criteria;
}
