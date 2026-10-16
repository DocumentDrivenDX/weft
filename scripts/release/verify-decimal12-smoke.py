"""Closed Decimal12 compiler smoke gate; no native storage acceptance."""
EXPECTED = ([{'id':'projection','status':'compiled'}]
 +[{'id':'exact-parameter-'+str(i),'status':'compiled'}for i in range(6)]
 +[{'id':'nonrepresentable-'+str(i),'status':'blocked'}for i in range(7)]
 +[{'id':'old-version-'+v,'status':'blocked'}for v in ['0.4.38','0.4.37']]
 +[{'id':'related-decimal-key','status':'compiled'},{'id':'first-typed-page','status':'compiled'},
   {'id':'next-typed-page','status':'compiled'},{'id':'bad-typed-page','status':'blocked'}])
def validate(reports, baseline, commit, version):
    assert len(EXPECTED)==20 and len(reports)==len(baseline)==5
    by_wheel={r['wheel']:r for r in baseline};assert len(by_wheel)==5
    assert {r['wheel']for r in reports}==set(by_wheel)
    for r in reports:
        assert r['sourceCommit']==commit and r['version']==version
        assert type(r['controls'])is int and r['controls']==20
        assert r['nativeAdmission']is False and r['securityReady']is False
        assert r['pinsBeforeAfter']==by_wheel[r['wheel']]['pinsBeforeAfter']
        assert type(r['checks'])is list and r['checks']==EXPECTED
