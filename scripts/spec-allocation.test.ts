import { test, expect } from 'bun:test';
import { readFileSync, readdirSync } from 'node:fs';
import YAML from 'yaml';
import { checkAllocation, type Artifact, type Allocation } from './spec-allocation';
// @covers US-008-AC6 Use real governed input and the production checker.
function inputs() {
  const artifacts = new Map<string, Artifact>();
  for (const file of readdirSync('docs/helix', { recursive: true }).filter(f => String(f).endsWith('.md'))) {
    const body = readFileSync(`docs/helix/${file}`, 'utf8');
    if (!body.startsWith('---\n')) continue;
    const meta = YAML.parse(body.slice(4, body.indexOf('\n---\n',4))).ddx;
    artifacts.set(meta.id,{meta,body});
  }
  const allocation: Allocation = JSON.parse(readFileSync('docs/helix/03-test/story-test-allocation.json','utf8'));
  const requirements: Record<string,string[]> = JSON.parse(readFileSync('docs/helix/03-test/requirement-allocation.json','utf8'));
  return { artifacts, allocation, requirements };
}
test('all declared requirements and criteria allocate without claiming execution', () => {
  const {artifacts,allocation,requirements} = inputs();
  expect(checkAllocation(artifacts,allocation,requirements).has('US-009-AC4')).toBe(true);
  expect(allocation.find(s=>s.story==='US-009')!.criteria.every(c=>c.state==='open')).toBe(true);
});
for (const [name,mutate] of [
  ['deleted requirement mapping', (x:any)=>delete x.requirements['FR-22']],
  ['deleted story allocation', (x:any)=>x.allocation.splice(x.allocation.findIndex((s:any)=>s.story==='US-009'),1)],
  ['deleted criterion', (x:any)=>x.allocation.at(-1).criteria.pop()],
  ['new unallocated requirement', (x:any)=>x.artifacts.get('weft.prd').body+='\n| FR-23 | P0 | New requirement |\n'],
  ['new unallocated criterion', (x:any)=>x.artifacts.get('US-009').body+='\n- **US-009-AC5:** New criterion\n'],
  ['criterion mentioned outside mapping', (x:any)=>x.artifacts.get('STP-009').body=x.artifacts.get('STP-009').body.replace('| US-009-AC4 |','US-009-AC4')],
  ['executed state disguised as allocation', (x:any)=>x.allocation[0].criteria[0].state='passed'],
  ['duplicate criterion', (x:any)=>x.allocation[0].criteria.push(x.allocation[0].criteria[0])],
  ['wrong known requirement story', (x:any)=>x.requirements['FR-22']=['US-001']],
  ['unknown requirement story', (x:any)=>x.requirements['FR-22']=['US-999']],
] as const) test(name, () => { const x=inputs();mutate(x);expect(()=>checkAllocation(x.artifacts,x.allocation,x.requirements)).toThrow(); });
