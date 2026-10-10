"""Fresh fixed 5dd CLI boundaries only; no corpus, installation or index authority.

Each completed raw observation is retained before its expected outcome is checked.
Inputs are deterministic transport probes, not graph or native SQL performance tests.
"""
import argparse,hashlib,json,os,selectors,signal,stat,subprocess,time
from pathlib import Path
from types import SimpleNamespace

SOURCE='5ddcebd6941c572ddd62b78925fdb1fe78b1c688'
BINARY='0e8199a8953f46d7151c87ec001e6c8edff414c0af9cecc6c13e0ea8d23ede80'
BINARY_BYTES=8532672
LIMIT=16*1024*1024
IDS=('oversize','invalid-utf8','split-utf8-at-limit','exact-limit',
     'exact-limit-multibyte','malformed-json','directory-input','closed-output')
class Refusal(ValueError):pass

def raise_cleanup(primary,error):
    """Keep body cancellation identity; closing cancellation outranks ordinary error."""
    if primary is not None:
        if isinstance(primary,Exception) and not isinstance(error,Exception):raise error
        try:primary.cleanup_failed=True
        except Exception:pass
    elif not isinstance(error,Exception):raise error
    else:raise Refusal('transport-cleanup')

def binary_bytes(path):
    if not path.is_absolute() or any(p.is_symlink() for p in (path,*path.parents)):raise Refusal('contained-binary')
    fd=os.open(path,os.O_RDONLY|os.O_NONBLOCK|os.O_NOFOLLOW);primary=None
    try:
        info=os.fstat(fd)
        if not stat.S_ISREG(info.st_mode) or not info.st_mode&0o111:raise Refusal('executable-required')
        raw=bytearray()
        while len(raw)<=BINARY_BYTES:
            part=os.read(fd,min(65536,BINARY_BYTES+1-len(raw)))
            if not part:break
            raw.extend(part)
        if len(raw)!=BINARY_BYTES or hashlib.sha256(raw).hexdigest()!=BINARY:raise Refusal('fixed-product-pin')
    except BaseException as exc:primary=exc;raise
    finally:
        try:os.close(fd)
        except BaseException as exc:raise_cleanup(primary,exc)

def probes():
    yield b' '*(LIMIT+1)
    yield b'\xff'
    yield b' '*(LIMIT-1)+b'\xc3'
    yield b' '*LIMIT
    yield b' '*(LIMIT-2)+'é'.encode()
    yield b'{'
    yield b''
    yield b'{}'

def check_observation(row):
    identity=row['id'];out=bytes.fromhex(row['stdoutHex']);err=bytes.fromhex(row['stderrHex'])
    if identity in ('oversize','invalid-utf8','split-utf8-at-limit','directory-input','closed-output'):
        marker='INPUT_LIMIT' if identity=='oversize' else 'UTF8' if 'utf8' in identity else 'INPUT_IO' if identity=='directory-input' else 'OUTPUT_IO'
        if row['exit']!=2 or out or err!=('WEFT_CLI_'+marker+'\n').encode():raise Refusal('boundary:'+identity)
    else:
        expected={'diagnostics':[{'code':'WFT-INPUT','message':'Invalid request JSON','phase':'input','recoverability':'correct-input','severity':'error'}],'interfaceVersion':'weft-compile/0.4.0','status':'blocked'}
        expected_raw=(json.dumps(expected,sort_keys=True,separators=(',',':'))+'\n').encode()
        if row['exit'] or err or out!=expected_raw:raise Refusal('boundary:'+identity)

def transport(binary,data,mode,config):
    """One deadline covers process, descendants and every pipe; no threads."""
    if mode not in ('normal','directory-input','closed-output'):raise Refusal('transport-mode')
    outer=getattr(config,'process_group_owner','transport')=='outer'
    if outer and (os.getpid()!=os.getpgrp() or os.getpid()!=os.getsid(0)):raise Refusal('outer-process-group-required')
    owned=None;process=None;primary=None;cleanup_failed=False
    cleanup_error=None
    def failed(error):
        nonlocal cleanup_failed,cleanup_error
        cleanup_failed=True
        if cleanup_error is None or isinstance(cleanup_error,Exception) and not isinstance(error,Exception):cleanup_error=error
    selector=selectors.DefaultSelector();streams=[];out=bytearray();err=bytearray()
    deadline=time.monotonic()+config.timeout_seconds
    try:
        stdin=subprocess.PIPE
        if mode=='directory-input':owned=os.open(config.source,os.O_RDONLY);stdin=owned
        process=subprocess.Popen([str(binary)],stdin=stdin,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=not outer,env={'LANG':'C','LC_ALL':'C','TZ':'UTC'})
        streams=[p for p in (process.stdout,process.stderr,process.stdin) if p is not None]
        if owned is not None:os.close(owned);owned=None
        for stream,label in ((process.stdout,'out'),(process.stderr,'err')):
            if label=='out' and mode=='closed-output':stream.close();continue
            os.set_blocking(stream.fileno(),False);selector.register(stream,selectors.EVENT_READ,label)
        remaining=memoryview(data)
        if process.stdin is not None:
            os.set_blocking(process.stdin.fileno(),False)
            if remaining:selector.register(process.stdin,selectors.EVENT_WRITE,'in')
            else:process.stdin.close()
        while selector.get_map() or process.poll() is None:
            available=deadline-time.monotonic()
            if available<=0:raise Refusal('transport-deadline')
            for key,event in selector.select(min(available,0.05)):
                if key.data=='in':
                    try:written=os.write(key.fd,remaining[:65536]);remaining=remaining[written:]
                    except BrokenPipeError:remaining=memoryview(b'')
                    if not remaining:selector.unregister(key.fileobj);key.fileobj.close()
                else:
                    dest,maximum=(out,config.maximum_response_bytes) if key.data=='out' else (err,4096)
                    chunk=os.read(key.fd,min(65536,maximum+1-len(dest)))
                    if not chunk:selector.unregister(key.fileobj);key.fileobj.close();continue
                    dest.extend(chunk)
                    if len(dest)>maximum:raise Refusal('capture-limit')
        return process.returncode,bytes(out),bytes(err)
    except BaseException as error:
        primary=error
        raise
    finally:
        if owned is not None:
            try:os.close(owned)
            except BaseException as error:failed(error)
        # Kill the whole group even if the leader exited but descendants hold pipes.
        if process is not None:
            try:
                if outer:process.kill()
                else:os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            except BaseException as error:failed(error)
        for stream in streams:
            try:stream.close()
            except BaseException as error:failed(error)
        try:selector.close()
        except BaseException as error:failed(error)
        if process is not None:
            try:process.wait(timeout=1)
            except BaseException as error:failed(error)
        if cleanup_failed:
            raise_cleanup(primary,cleanup_error)

def qualify(binary,output):
    if not output.is_absolute() or output.exists() or output.is_symlink() or any(p.is_symlink() for p in output.parents):raise Refusal('fresh-output')
    binary_bytes(binary)
    config=SimpleNamespace(timeout_seconds=15,maximum_response_bytes=4*1024*1024,source=binary.parent,process_group_owner='transport')
    rows=[];primary=None
    # Exclusive, closed raw JSONL records survive an ordinary later semantic failure.
    with output.open('xb'):pass
    try:
        for identity,data in zip(IDS,probes()):
            mode=identity if identity in ('directory-input','closed-output') else 'normal'
            code,out,err=transport(binary,data,mode,config)
            row={'id':identity,'inputBytes':len(data),'inputSha256':hashlib.sha256(data).hexdigest(),'exit':code,'stdoutHex':out.hex(),'stderrHex':err.hex()}
            line=(json.dumps(row,separators=(',',':'))+'\n').encode()
            if len(line)>9*1024*1024:raise Refusal('observation-bound')
            with output.open('ab') as stream:stream.write(line)
            rows.append(row);check_observation(row)
    except BaseException as exc:primary=exc;raise
    finally:
        try:binary_bytes(binary)
        except BaseException as exc:raise_cleanup(primary,exc)
    print(json.dumps({'format':'weft-count-star-cli-boundary-evidence/0.1','sourceCommit':SOURCE,'binarySha256':BINARY,'observedControls':len(rows),'observationsSha256':hashlib.sha256(output.read_bytes()).hexdigest(),'scope':'Eight actual CLI boundary outcomes only; no installed package, native SQL or index admission.'}))

def main():
    p=argparse.ArgumentParser()
    p.add_argument('--binary',type=Path,required=True);p.add_argument('--output',type=Path,required=True)
    a=p.parse_args();qualify(a.binary,a.output)
if __name__=='__main__':main()
