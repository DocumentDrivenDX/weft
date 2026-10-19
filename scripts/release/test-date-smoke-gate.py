# SPDX-License-Identifier: MIT OR Apache-2.0
"""Small publication-gate controls; synthetic reports are not wheel evidence."""
import copy,pathlib,runpy
module=runpy.run_path(str(pathlib.Path(__file__).with_name('verify-date-smoke.py')))
baseline=[dict(wheel=str(i)+'.whl',pinsBeforeAfter=dict(wheelSha256=str(i)*64)) for i in range(5)]
reports=[dict(row,sourceCommit='a'*40,version='0.4.39',controls=7,checks=copy.deepcopy(module['EXPECTED']),queryAbi=module['ABI'],nativeAdmission=False,securityReady=False) for row in baseline]
module['validate'](reports,baseline,'a'*40,'0.4.39')
for field,value in [('sourceCommit','b'*40),('version','0.4.38'),('checks',[]),('pinsBeforeAfter',{}),('queryAbi',{}),('nativeAdmission',True)]:
    changed=copy.deepcopy(reports);changed[0][field]=value
    try:module['validate'](changed,baseline,'a'*40,'0.4.39')
    except AssertionError:pass
    else:raise AssertionError(field)
for changed in [reports[:-1],[reports[0]]*5]:
    try:module['validate'](changed,baseline,'a'*40,'0.4.39')
    except AssertionError:pass
    else:raise AssertionError('Missing or duplicate wheel reports')
print({'validatorControls':9,'nativeAdmission':False})
