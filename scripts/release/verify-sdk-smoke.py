"""Closed five-wheel SDK compiler publication gate; reports confer no authority."""
EXPECTED=[dict(id=name,status=status) for name,status in [('read','compiled'),('read-check','compiled'),('page','compiled'),('page-check','compiled'),('export','compiled'),('read-context-drift','blocked'),('page-context-drift','blocked'),('export-context-drift','blocked'),('export-program-drift','blocked'),('distinct','compiled'),('distinct-null','compiled'),('not-distinct','compiled'),('not-distinct-null','compiled'),('distinct-carrier-mismatch','blocked')]]
EXPECTED += [dict(id=compiler+'-'+suffix,status='compiled') for compiler in ('ashlar-spark4','ashlar-dbsql-20261016') for suffix in ('read','read-check','page','page-check')]
FIELDS={'version','sourceCommit','wheel','pinsBeforeAfter','controls','checks','nativeAdmission','securityReady','temporalQualification'}
def validate(reports,baseline,commit,version):
    assert len(reports)==len(baseline)==5
    by_wheel={r['wheel']:r for r in baseline};assert len(by_wheel)==5
    assert len({r['wheel'] for r in reports})==5 and {r['wheel'] for r in reports}==set(by_wheel)
    for report in reports:
        assert set(report)==FIELDS
        assert report['sourceCommit']==commit and report['version']==version
        assert report['pinsBeforeAfter']==by_wheel[report['wheel']]['pinsBeforeAfter']
        assert type(report['controls']) is int and report['controls']==len(EXPECTED)
        assert report['checks']==EXPECTED
        assert report['nativeAdmission'] is False and report['securityReady'] is False
        assert report['temporalQualification']=='held'
