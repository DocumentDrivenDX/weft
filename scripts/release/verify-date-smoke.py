# SPDX-License-Identifier: MIT OR Apache-2.0
"""Require the seven date compiler checks from each exact release wheel."""
EXPECTED=[dict(id=name,status=status) for name,status in [('literal','compiled'),('typed-parameter','compiled'),('field-join','compiled'),('invalid-date','blocked'),('old-envelope','blocked'),('old-backend','blocked'),('old-abi','blocked')]]
ABI=dict(identity='truss-postgresql-logical-query',version='1.1',sha256='9b67333abf263078e7e7239fe5accc228b063dc2fcc5488ca068d7331986949f')
def validate(reports,baseline,commit,version):
    assert len(reports)==len(baseline)==5
    by_wheel={r['wheel']:r for r in baseline};assert len(by_wheel)==5
    assert {r['wheel'] for r in reports}==set(by_wheel)
    for report in reports:
        assert report['sourceCommit']==commit and report['version']==version
        assert report['pinsBeforeAfter']==by_wheel[report['wheel']]['pinsBeforeAfter']
        assert type(report['controls']) is int and report['controls']==len(EXPECTED)
        assert report['checks']==EXPECTED and report['queryAbi']==ABI
        assert report['nativeAdmission'] is False and report['securityReady'] is False
