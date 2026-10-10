"""Reviewed preparation only; root execution grant remains external."""
import argparse,hashlib,json,pathlib,types

def main():
    p=argparse.ArgumentParser();p.add_argument('--command',required=True);p.add_argument('--sha256',required=True);a=p.parse_args()
    with open(a.command,'rb') as f:raw=f.read(1000001)
    if len(raw)>1000000 or hashlib.sha256(raw).hexdigest()!=a.sha256:raise ValueError('command-pin')
    plan=json.loads(raw);d=plan['launcher']
    with open(d['path'],'rb') as f:snapshot=f.read(d['bytes']+1)
    if len(snapshot)!=d['bytes'] or hashlib.sha256(snapshot).hexdigest()!=d['sha256']:raise ValueError('launcher-pin')
    launcher=types.ModuleType('reviewed_bounded_launcher');exec(compile(snapshot,d['path'],'exec'),launcher.__dict__)
    launcher.verify(plan)
    result=None;primary=None;closed=False
    try:
        c=plan['commands'][0]
        result=launcher.capture(c['argv'],c['cwd'],plan['environment'],c['stdout'],c['stderr'],plan['launcherLimits']['timeoutSeconds'],plan['launcherLimits']['maximumStreamBytes'])
        if result['exitCode']!=0:raise ValueError('producer-refused')
    except BaseException as error:primary=error
    finally:
        try:launcher.verify(plan);closed=True
        except BaseException as error:
            if primary is None:primary=error
            else:
                try:primary.closing_custody_failed=True
                except BaseException:pass
        receipt={'commandSha256':a.sha256,'result':result,'openingClosingCustody':closed,'completed':primary is None,'scope':'Candidate compiler corpus observation; independent acceptance/index/native authority remain separate.'}
        try:
            payload=(json.dumps(receipt,sort_keys=True)+'\n').encode()
            stream=None;failure=None
            try:stream=open(plan['outcome'],'xb');stream.write(payload)
            except BaseException as error:failure=error
            if stream is not None:
                try:stream.close()
                except BaseException as error:
                    if failure is None:failure=error
            if failure is not None:raise failure
        except BaseException as error:
            if primary is None:primary=error
            else:
                try:primary.cleanup_failed=True
                except BaseException:pass
    if primary is not None:raise primary
if __name__=='__main__':main()
