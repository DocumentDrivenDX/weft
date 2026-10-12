"""Verify the complete hosted wheel matrix and assemble auditable release assets."""
import argparse, hashlib, json, os, shutil, sys, zipfile
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('Validation requires assertions')
p=argparse.ArgumentParser()
p.add_argument('--wheels',type=Path,required=True)
p.add_argument('--reports',type=Path,required=True)
p.add_argument('--out',type=Path,required=True)
a=p.parse_args()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
wheels=sorted(a.wheels.glob('*.whl'));assert len(wheels)==5
reports=sorted(a.reports.glob('*.json'));assert len(reports)==30
loaded=[json.loads(p.read_text()) for p in reports]
source=os.environ['WEFT_RELEASE_SOURCE_COMMIT']
for report in loaded:
    assert report['status']=='passed' and report['sourceCommit']==source
    assert report['cases']==2181 and report['candidateCases']==174 and report['controls']==8
    assert report['subprocessDisabled'] is True
assert len({r['receiptsSha256'] for r in loaded})==1
assert len({r['candidateReceiptsSha256'] for r in loaded})==1
for wheel in wheels:
    matched=[r for r in loaded if r['wheel']==wheel.name]
    assert len(matched)==6 and all(r['wheelSha256']==sha(wheel) for r in matched)
    assert {'.'.join(r['python'].split()[0].split('.')[:2]) for r in matched}=={'3.9','3.10','3.11','3.12','3.13','3.14'}
    assert len({r['nativeExtensionSha256'] for r in matched})==1
assert {r['wheel'] for r in loaded}=={w.name for w in wheels}
a.out.mkdir(parents=True,exist_ok=True)
for wheel in wheels:shutil.copy2(wheel,a.out/wheel.name)
with zipfile.ZipFile(a.out/'validation-reports.zip','w',compression=zipfile.ZIP_DEFLATED) as z:
    for report in reports:z.writestr(report.name,report.read_bytes())
summary={'package':'weft-sql','version':'0.4.2','sourceCommit':source,'workflowRun':os.environ.get('GITHUB_RUN_ID'),'platformWheels':5,'installedInterpreterChecks':30,'ordinaryCasesPerCheck':2181,'candidateCasesPerCheck':174,'controlsPerCheck':8,'status':'passed','wheels':[{'name':w.name,'sha256':sha(w)} for w in wheels],'limitations':['53 changed ordinary outputs retain open native requalification','Candidate APIs remain experimental','No PyPy, free-threaded CPython or musl support','No sdist']}
(a.out/'validation-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
files=sorted(p for p in a.out.iterdir() if p.is_file() and p.name!='SHA256SUMS')
(a.out/'SHA256SUMS').write_text(''.join(sha(p)+'  '+p.name+'\n' for p in files))
print(json.dumps(summary))
