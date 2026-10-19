# SPDX-License-Identifier: MIT OR Apache-2.0
"""Closed source/artifact correspondence gate; no native security admission."""
PAIRS = [('postgresql','rows'),('postgresql','contributors'),('postgresql','sum'),('databricks','contributors'),('databricks','sum')]
REFUSALS = ['foreign-source','wrong-public','duplicate-key','missing-input','unknown-frame']
def validate(reports, baseline, commit, version):
    expected=[]
    for backend,kind in PAIRS:
        expected.append((backend+'/'+kind,'matched'))
        expected.extend((backend+'/'+kind+'/'+reason,'refused')for reason in REFUSALS)
    assert len(expected)==30
    assert len(baseline)==5 and len(reports)==5
    by_wheel={r['wheel']:r for r in baseline}
    assert len(by_wheel)==5 and {r['wheel']for r in reports}==set(by_wheel)
    for report in reports:
        assert report['sourceCommit']==commit and report['version']==version
        assert type(report['controls'])is int and report['controls']==30
        assert report['nativeAdmission']is False and report['securityReady']is False
        assert report['pinsBeforeAfter']==by_wheel[report['wheel']]['pinsBeforeAfter']
        assert type(report['checks'])is list and len(report['checks'])==30
        for check,(identity,flag) in zip(report['checks'],expected):
            assert type(check)is dict and set(check)=={'id',flag}
            assert check['id']==identity and check[flag]is True
