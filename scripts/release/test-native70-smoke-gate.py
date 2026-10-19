# SPDX-License-Identifier: MIT OR Apache-2.0
"""Synthetic publication-gate refusal controls, not platform/native executions."""
import copy,importlib.util,pathlib
p=pathlib.Path(__file__).with_name('verify-native70-smoke.py');s=importlib.util.spec_from_file_location('gate',p);g=importlib.util.module_from_spec(s);s.loader.exec_module(g)
checks=copy.deepcopy(g.EXPECTED)
base=[{'wheel':str(i)+'.whl','pinsBeforeAfter':{'wheelSha256':str(i)*64}}for i in range(5)]
reports=[dict(b,sourceCommit='a'*40,version='0.4.37',controls=5,nativeAdmission=False,securityReady=False,checks=copy.deepcopy(checks))for b in base]
g.validate(reports,base,'a'*40,'0.4.37');count=1
def refuse(label,change):
 global count
 r=copy.deepcopy(reports);change(r)
 try:g.validate(r,base,'a'*40,'0.4.37')
 except (AssertionError,KeyError,TypeError):count+=1
 else:raise AssertionError(label)
refuse('missing-report',lambda r:r.pop())
refuse('duplicate-report',lambda r:r.__setitem__(1,copy.deepcopy(r[0])))
refuse('wrong-source',lambda r:r[0].update(sourceCommit='b'*40))
refuse('wrong-version',lambda r:r[0].update(version='0.4.35'))
refuse('swapped-pins',lambda r:r[0].update(pinsBeforeAfter=r[1]['pinsBeforeAfter']))
refuse('claimed5-empty-checks',lambda r:[x.update(checks=[])for x in r])
refuse('missing-check',lambda r:r[0]['checks'].pop())
refuse('duplicate-check',lambda r:r[0]['checks'].__setitem__(1,copy.deepcopy(r[0]['checks'][0])))
refuse('changed-identity',lambda r:r[0]['checks'][0].update(id='changed'))
refuse('changed-status',lambda r:r[0]['checks'][0].update(status='blocked'))
refuse('extra-check-field',lambda r:r[0]['checks'][0].update(extra=True))
refuse('wrong-total',lambda r:r[0].update(controls='22'))
refuse('authority',lambda r:r[0].update(nativeAdmission=True))
refuse('numeric-false',lambda r:r[0].update(securityReady=0))
print({'validatorControls':count,'nativeAdmission':False})
