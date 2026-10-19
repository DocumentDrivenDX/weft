# SPDX-License-Identifier: MIT OR Apache-2.0
"""Require genuine native70 compiler evidence from each exact release wheel."""
EXPECTED=[{'id':k,'status':v} for k,v in [('projection','compiled'),('count','compiled'),('page','compiled'),('old-backend','blocked'),('wrong-runtime-digest','blocked')]]
def validate(reports, baseline, commit, version):
    assert len(EXPECTED)==5 and len(reports)==len(baseline)==5
    by_wheel={r['wheel']:r for r in baseline};assert len(by_wheel)==5
    assert {r['wheel']for r in reports}==set(by_wheel)
    for r in reports:
        assert r['sourceCommit']==commit and r['version']==version
        assert type(r['controls'])is int and r['controls']==5
        assert r['nativeAdmission']is False and r['securityReady']is False
        assert r['pinsBeforeAfter']==by_wheel[r['wheel']]['pinsBeforeAfter']
        assert type(r['checks'])is list and r['checks']==EXPECTED
