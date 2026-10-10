"""Unix host process boundary: bounded raw-stream scanning, no raw output sinks."""
from __future__ import annotations
import dataclasses, os, pathlib, re, selectors, signal, subprocess, time

@dataclasses.dataclass(frozen=True)
class Result:
    exit_code: int
    timed_out: bool
    found: frozenset[str]
    selected_tests: int
    failed_filters: frozenset[str]
    passed: int
    failed: int
    ignored: int
    filtered: int
    bytes_read: int
    duration_ms: int
    captured: bytes = b""

def run(command:list[str], cwd:pathlib.Path, env:dict[str,str], timeout:float, needles:tuple[str,...]=(), filters:tuple[str,...]=(), capture_limit:int=0, input_data:bytes|None=None, cancel=None) -> Result:
    if os.name!='posix':raise RuntimeError('unsupported host')
    max_tail=max([len(n.encode()) for n in needles]+[1])-1
    if max_tail>4096:raise ValueError('invalid scanner signature')
    if not 0<=capture_limit<=256:raise ValueError('invalid capture limit')
    captured=b''
    if input_data is not None and (type(input_data)!=bytes or len(input_data)>8192):raise ValueError('invalid process input')
    start=time.monotonic(); deadline=start+timeout
    process=subprocess.Popen(command,cwd=cwd,env=env,stdin=subprocess.PIPE if input_data is not None else None,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True)
    selector=selectors.DefaultSelector();selector.register(process.stdout,selectors.EVENT_READ)
    os.set_blocking(process.stdout.fileno(),False)
    tail=b'';line=b'';found=set();failed_filters=set();selected=passed=failed=ignored=filtered=total=0
    timed_out=False;kill_deadline=None
    def finish_line(raw):
        nonlocal selected,passed,failed,ignored,filtered
        text=raw.decode('utf8',errors='replace')
        match=re.fullmatch(r'running (\d+) tests?',text.strip())
        if match:selected+=int(match[1])
        for name in filters:
            if re.fullmatch(r'test (?:\S+::)?'+re.escape(name)+r' \.\.\. FAILED',text.strip()):failed_filters.add(name)
        match=re.search(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored; (?:\d+ measured; )?(\d+) filtered out',text)
        if match:
            a,b,c,d=map(int,match.groups());passed+=a;failed+=b;ignored+=c;filtered+=d
    try:
        if input_data is not None:
            try:process.stdin.write(input_data);process.stdin.close()
            except BrokenPipeError:pass
        while selector.get_map() or process.poll() is None:
            now=time.monotonic()
            if (now>=deadline or (cancel is not None and cancel.is_set())) and kill_deadline is None:
                timed_out=True;kill_deadline=now+1
                try:os.killpg(process.pid,signal.SIGTERM)
                except ProcessLookupError:pass
            if kill_deadline is not None and now>=kill_deadline:
                try:os.killpg(process.pid,signal.SIGKILL)
                except ProcessLookupError:pass
                # A detached descendant may retain stdout. Close our pipe boundedly.
                if now>=kill_deadline+1:break
            for key,_ in selector.select(.05):
                chunk=os.read(key.fileobj.fileno(),8192)
                if not chunk:selector.unregister(key.fileobj);continue
                if capture_limit:captured=(captured+chunk)[:capture_limit]
                total+=len(chunk);window=tail+chunk
                for needle in needles:
                    if needle.encode() in window:found.add(needle)
                tail=window[-max_tail:] if max_tail else b''
                for part in chunk.splitlines(keepends=True):
                    line=(line+part)[-65536:]
                    if part.endswith(b'\n'):finish_line(line.rstrip(b'\r\n'));line=b''
        if line:finish_line(line)
        if process.poll() is None:
            try:os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:pass
        exit_code=process.wait(timeout=1)
    finally:
        selector.close();process.stdout.close()
        if process.poll() is None:
            try:os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            process.wait(timeout=1)
    return Result(exit_code,timed_out,frozenset(found),selected,frozenset(failed_filters),passed,failed,ignored,filtered,total,round((time.monotonic()-start)*1000),captured)
