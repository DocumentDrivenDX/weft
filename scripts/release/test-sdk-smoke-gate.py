"""Synthetic negative controls exercise admission only, not wheel behavior."""
import copy,pathlib,runpy
m=runpy.run_path(str(pathlib.Path(__file__).with_name('verify-sdk-smoke.py')))
b=[dict(wheel=str(i)+'.whl',pinsBeforeAfter=dict(wheelSha256=str(i)*64)) for i in range(5)]
r=[dict(row,sourceCommit='a'*40,version='0.4.41',controls=len(m['EXPECTED']),checks=copy.deepcopy(m['EXPECTED']),nativeAdmission=False,securityReady=False,temporalQualification='held') for row in b]
m['validate'](r,b,'a'*40,'0.4.41');controls=0
for field,value in [('sourceCommit','b'*40),('version','0.4.40'),('controls',True),('controls',len(m['EXPECTED'])-1),('checks',[]),('pinsBeforeAfter',{}),('nativeAdmission',True),('securityReady',True),('temporalQualification','qualified'),('unexpected',True)]:
    c=copy.deepcopy(r);c[0][field]=value
    try:m['validate'](c,b,'a'*40,'0.4.41')
    except AssertionError:controls+=1
    else:raise AssertionError(field)
for c in [r[:-1],[r[0]]*5]:
    try:m['validate'](c,b,'a'*40,'0.4.41')
    except AssertionError:controls+=1
    else:raise AssertionError('missing/duplicate reports')
print(dict(validatorControls=controls,nativeAdmission=False,temporalQualification='held'))
