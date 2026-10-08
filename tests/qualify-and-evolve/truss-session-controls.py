"""Negative controls for saved session provenance, without engine execution."""
import copy,gzip,hashlib,importlib.util,json
from pathlib import Path
spec=importlib.util.spec_from_file_location('sessions',Path(__file__).with_name('audit-truss-sessions.py'))
sessions=importlib.util.module_from_spec(spec);spec.loader.exec_module(sessions)
compressed,summary,original=sessions.inputs()
assert sessions.audit(compressed,summary,original)['cases']==76
rejected=[]
for label,message in [('missing','case count'),('duplicate','case identity'),('rows','prior receipt agreement'),('session','session agreement'),('sql','SQL digest shape'),('archive','archive digest')]:
    changed=copy.deepcopy(summary);reports=json.loads(gzip.decompress(compressed))
    if label=='missing':reports.pop()
    elif label=='duplicate':reports[1]=copy.deepcopy(reports[0])
    elif label=='rows':reports[0]['rows']=[]
    elif label=='session':reports[0]['session']['transactionIsolation']='read committed'
    elif label=='sql':reports[0]['executedSqlSha256']='not-a-hash'
    raw=json.dumps(reports,ensure_ascii=False).encode();packed=gzip.compress(raw,mtime=0)
    changed.update(gzipSha256=hashlib.sha256(packed).hexdigest(),uncompressedSha256=hashlib.sha256(raw).hexdigest(),bytes=len(raw))
    if label=='archive':changed['gzipSha256']='0'*64
    try:sessions.audit(packed,changed,original)
    except AssertionError as error:assert str(error)==message,(label,error);rejected.append(label)
    else:raise AssertionError('corruption admitted: '+label)
print(json.dumps({'status':'passed','corruptionsRejected':len(rejected),'controls':rejected}))
