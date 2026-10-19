# SPDX-License-Identifier: MIT OR Apache-2.0
"""Verify same-repository tag wheel provenance before reusing existing artifacts."""
import json,os,re,subprocess,sys
TARGETS={'ubuntu-24.04, x86_64-unknown-linux-gnu, 2014','ubuntu-24.04-arm, aarch64-unknown-linux-gnu, 2014','macos-14, aarch64-apple-darwin, auto','macos-15-intel, x86_64-apple-darwin, auto','windows-2022, x86_64-pc-windows-msvc, auto'}
EXPECTED={'wheels ('+target+')' for target in TARGETS}
def verify(run,jobs,*,run_id,repository,tag,commit):
    assert run['id']==int(run_id)
    assert run['repository']['full_name']==repository
    assert run['path']=='.github/workflows/release.yml'
    assert run['head_sha']==commit and run['head_branch']==tag
    assert run['event']=='push' and run['status']=='completed'
    assert run['conclusion'] in ('success','failure')
    assert len(jobs)==6 and len({j['name'] for j in jobs})==6
    assert {j['name'] for j in jobs}==EXPECTED|{'publish'}
    for job in jobs:
        assert job['status']=='completed'
        if job['name'] in EXPECTED:assert job['conclusion']=='success'
        else:assert job['conclusion'] in ('success','failure','skipped')
    return True
if __name__=='__main__':
    run_id,tag=sys.argv[1:]
    assert re.fullmatch('[1-9][0-9]{0,19}',run_id)
    assert re.fullmatch(r'v[0-9]+\.[0-9]+\.[0-9]+',tag)
    repository=os.environ['GITHUB_REPOSITORY']
    assert re.fullmatch('[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+',repository)
    def api(path):return json.loads(subprocess.check_output(['gh','api','repos/'+repository+path]))
    commit=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
    assert commit==subprocess.check_output(['git','rev-parse','refs/tags/'+tag+'^{commit}'],text=True).strip()
    run=api('/actions/runs/'+run_id)
    jobs=api('/actions/runs/'+run_id+'/jobs?per_page=100')
    assert jobs['total_count']==len(jobs['jobs'])
    verify(run,jobs['jobs'],run_id=run_id,repository=repository,tag=tag,commit=commit)
    print(json.dumps({'verifiedRun':int(run_id),'tag':tag,'sourceCommit':commit,'successfulWheelJobs':5}))
