# SPDX-License-Identifier: MIT OR Apache-2.0
"""Synthetic release-validator counterexamples, not five-platform execution."""
import copy,importlib.util,pathlib
path=pathlib.Path(__file__).with_name('verify-refinement-smoke.py');spec=importlib.util.spec_from_file_location('gate',path);gate=importlib.util.module_from_spec(spec);spec.loader.exec_module(gate)
# Independent closed expected outcome list; no call to the validator's constants.
checks=[]
for prefix in ['postgresql/rows','postgresql/contributors','postgresql/sum','databricks/contributors','databricks/sum']:
 checks.append({'id':prefix,'matched':True})
 for suffix in ['foreign-source','wrong-public','duplicate-key','missing-input','unknown-frame']:checks.append({'id':prefix+'/'+suffix,'refused':True})
baseline=[{'wheel':str(i)+'.whl','pinsBeforeAfter':{'wheelSha256':str(i)*64,'nativeModuleSha256':'a'*64,'shimSha256':'b'*64}}for i in range(5)]
reports=[dict(r,sourceCommit='c'*40,version='0.4.35',controls=30,nativeAdmission=False,securityReady=False,checks=copy.deepcopy(checks))for r in baseline]
gate.validate(reports,baseline,'c'*40,'0.4.35');count=1
def reject(name,change):
 global count
 altered=copy.deepcopy(reports);change(altered)
 try:gate.validate(altered,baseline,'c'*40,'0.4.35')
 except (AssertionError,KeyError,TypeError):count+=1
 else:raise AssertionError(name)
reject('missing-report',lambda r:r.pop())
reject('duplicate-report',lambda r:r.__setitem__(1,copy.deepcopy(r[0])))
reject('foreign-source',lambda r:r[0].update(sourceCommit='d'*40))
reject('foreign-version',lambda r:r[0].update(version='0.4.34'))
reject('foreign-wheel',lambda r:r[0].update(wheel='foreign.whl'))
reject('swapped-wheel-pins',lambda r:r[0].update(pinsBeforeAfter=r[1]['pinsBeforeAfter']))
reject('empty-checks-with-claimed30',lambda r:[x.update(checks=[])for x in r])
reject('omitted-check',lambda r:r[0]['checks'].pop())
reject('duplicate-check',lambda r:r[0]['checks'].__setitem__(1,copy.deepcopy(r[0]['checks'][0])))
reject('renamed-check',lambda r:r[0]['checks'][1].update(id='invented'))
reject('wrong-status-key',lambda r:r[0]['checks'].__setitem__(0,{'id':'postgresql/rows','refused':True}))
reject('false-status',lambda r:r[0]['checks'][1].update(refused=False))
reject('numeric-status',lambda r:r[0]['checks'][1].update(refused=1))
reject('extra-check-key',lambda r:r[0]['checks'][1].update(extra=True))
reject('wrong-total',lambda r:r[0].update(controls=29))
reject('string-total',lambda r:r[0].update(controls='30'))
reject('native-authority',lambda r:r[0].update(nativeAdmission=True))
reject('numeric-native-flag',lambda r:r[0].update(nativeAdmission=0))
reject('security-ready',lambda r:r[0].update(securityReady=True))
print({'validatorControls':count,'nativeAdmission':False})
