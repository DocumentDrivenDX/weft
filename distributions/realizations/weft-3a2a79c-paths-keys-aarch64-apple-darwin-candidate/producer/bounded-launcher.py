"""Bounded local launcher; an explicit review/grant remains external."""
import argparse, hashlib, json, os, pathlib, selectors, signal, subprocess, time

def digest(path):
    h=hashlib.sha256(); size=0
    with open(path,'rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''): h.update(b); size+=len(b)
    return h.hexdigest(),size

def verify(plan):
    for d in plan['resources']:
        if digest(d['path'])!=(d['sha256'],d['bytes']): raise ValueError('Resource custody mismatch')
    inv=json.loads(pathlib.Path(plan['sourceInventory']['path']).read_bytes())
    if inv['sourceCommit']!=plan['sourceCommit']: raise ValueError('Source commit mismatch')
    root=pathlib.Path(plan['sourceExport']); expected=set()
    for d in inv['files']:
        p=root/d['path']; expected.add(d['path'])
        if p.is_symlink() or not p.is_file() or digest(p)!=(d['sha256'],d['bytes']): raise ValueError('Source custody mismatch')
        executable=bool(p.stat().st_mode&0o111)
        if executable!=(d['mode']=='100755'): raise ValueError('Source mode mismatch')
        h=hashlib.sha1(); h.update(('blob '+str(d['bytes'])+'\0').encode())
        with p.open('rb') as f:
            for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
        if h.hexdigest()!=d['gitBlob']: raise ValueError('Source Git blob mismatch')
    actual=set()
    for parent,dirs,files in os.walk(root,followlinks=False):
        for name in dirs:
            if (pathlib.Path(parent)/name).is_symlink(): raise ValueError('Source symlink')
        for name in files: actual.add((pathlib.Path(parent)/name).relative_to(root).as_posix())
    if actual!=expected: raise ValueError('Source inventory mismatch')

def capture(argv,cwd,env,stdout_path,stderr_path,timeout,maximum):
    # No inherited environment. Every stream/process shares the same finite deadline.
    opened=[]; proc=None; sel=None; primary=None; cleanup_error=None; totals=[0,0]
    try:
        for path in [stdout_path,stderr_path]: opened.append(open(path,'xb'))
        proc=subprocess.Popen(argv,cwd=cwd,env=dict(env),stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
        sel=selectors.DefaultSelector()
        for i,s in enumerate([proc.stdout,proc.stderr]): os.set_blocking(s.fileno(),False); sel.register(s,selectors.EVENT_READ,i)
        deadline=time.monotonic()+timeout
        while sel.get_map() or proc.poll() is None:
            left=deadline-time.monotonic()
            if left<=0: raise TimeoutError('Build deadline exceeded')
            for key,_ in sel.select(min(left,.1)):
                b=os.read(key.fileobj.fileno(),65536)
                if not b: sel.unregister(key.fileobj); continue
                i=key.data; totals[i]+=len(b)
                if totals[i]>maximum: raise ValueError('Build output bound exceeded')
                opened[i].write(b)
        code=proc.wait(timeout=max(.001,deadline-time.monotonic()))
    except BaseException as e: primary=e
    finally:
        if proc is not None:
            try: os.killpg(proc.pid,signal.SIGKILL)
            except ProcessLookupError: pass
            except BaseException as e:
                if cleanup_error is None: cleanup_error=e
            try: proc.wait(timeout=1)
            except BaseException as e:
                if cleanup_error is None: cleanup_error=e
            for stream in [proc.stdout,proc.stderr]:
                try: stream.close()
                except BaseException as e:
                    if cleanup_error is None: cleanup_error=e
        if sel is not None:
            try: sel.close()
            except BaseException as e:
                if cleanup_error is None: cleanup_error=e
        for f in opened:
            try: f.close()
            except BaseException as e:
                if cleanup_error is None: cleanup_error=e
    if primary is not None:
        try: primary.cleanup_failed=cleanup_error is not None
        except BaseException: pass
        raise primary
    if cleanup_error is not None: raise cleanup_error
    return {'exitCode':code,'stdoutBytes':totals[0],'stderrBytes':totals[1]}

def main():
    p=argparse.ArgumentParser(); p.add_argument('--command',required=True,type=pathlib.Path); p.add_argument('--sha256',required=True); p.add_argument('--phase',required=True,choices=['cli-build','source-metadata']); p.add_argument('--receipt',required=True,type=pathlib.Path); a=p.parse_args()
    with a.command.open('rb') as f: raw=f.read(1000001)
    if len(raw)>1000000 or hashlib.sha256(raw).hexdigest()!=a.sha256: raise ValueError('Command pin mismatch')
    plan=json.loads(raw)
    own=plan['launcher']
    if pathlib.Path(own['path']).resolve()!=pathlib.Path(__file__).resolve() or digest(__file__)!=(own['sha256'],own['bytes']): raise ValueError('Launcher pin mismatch')
    verify(plan); c=next(c for c in plan['commands'] if c['phase']==a.phase)
    stdout=c.get('stdout',c.get('log')); stderr=c.get('stderr',stdout+'.stderr')
    primary=None; result=None
    try:
        result=capture(c['argv'],c['cwd'],plan['environment'],stdout,stderr,plan['launcherLimits']['timeoutSeconds'],plan['launcherLimits']['maximumStreamBytes'])
    except BaseException as e:
        primary=e
    finally:
        try: verify(plan)
        except BaseException as e:
            if primary is None: primary=e
            else:
                try: primary.closing_custody_failed=True
                except BaseException: pass
    if primary is not None: raise primary
    result.update({'commandSha256':a.sha256,'phase':a.phase,'openingClosingCustody':True,'environmentInherited':False})
    if result['exitCode']==0 and c.get('retainedBinary'):
        src=pathlib.Path(c['builtBinary']); dst=pathlib.Path(c['retainedBinary'])
        with src.open('rb') as f,dst.open('xb') as out:
            for b in iter(lambda:f.read(1024*1024),b''): out.write(b)
        dst.chmod(0o555); result['binary']={'path':str(dst),'sha256':digest(dst)[0],'bytes':digest(dst)[1]}
    with a.receipt.open('xb') as f: f.write((json.dumps(result,sort_keys=True,indent=2)+'\n').encode())
    if result['exitCode']: raise SystemExit(result['exitCode'])
if __name__=='__main__': main()
