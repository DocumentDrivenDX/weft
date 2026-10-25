#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Run one release-prep command inside the original monotonic budget."""
import argparse
import json
import os
import pathlib
import selectors
import signal
import subprocess
import sys
import time

MAX_LOG = 256 * 1024
UPLOAD_RESERVE = 300
TRUNCATION_MARKER = b"\n[output truncated; command failed]\n"

def signal_group(pid, selected_signal):
    try:os.killpg(pid,selected_signal)
    except ProcessLookupError:pass

def main():
    parser=argparse.ArgumentParser()
    parser.add_argument('--stage',required=True)
    parser.add_argument('--seconds',type=int,required=True)
    parser.add_argument('--grace-seconds',type=int,default=10)
    parser.add_argument('--log',type=pathlib.Path,required=True)
    parser.add_argument('--receipt',type=pathlib.Path,required=True)
    parser.add_argument('command',nargs=argparse.REMAINDER)
    a=parser.parse_args()
    command=a.command[1:] if a.command[:1]==['--'] else a.command
    if not command or a.seconds<1 or a.grace_seconds<1:raise SystemExit('invalid bounded command request')
    try:deadline=float(os.environ['WEFT_RELEASE_DEADLINE_MONOTONIC'])
    except (KeyError,ValueError):raise SystemExit('original release deadline is absent or invalid')
    start=time.monotonic();remaining=deadline-start-UPLOAD_RESERVE
    allowed=min(a.seconds,remaining)
    if allowed<=a.grace_seconds:raise SystemExit('release acquisition gate closed: insufficient command time remains before evidence/upload reserve')
    a.log.parent.mkdir(parents=True,exist_ok=True);a.receipt.parent.mkdir(parents=True,exist_ok=True)
    process=subprocess.Popen(command,stdin=subprocess.DEVNULL,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,start_new_session=True,close_fds=True)
    selector=selectors.DefaultSelector();selector.register(process.stdout,selectors.EVENT_READ)
    written=0;truncated=False;expired=False;overflow=False
    with a.log.open('wb') as output:
        while process.poll() is None or selector.get_map():
            now=time.monotonic()
            if now-start>=allowed-a.grace_seconds and process.poll() is None:
                expired=True;signal_group(process.pid,signal.SIGTERM)
                try:process.wait(timeout=a.grace_seconds)
                except subprocess.TimeoutExpired:signal_group(process.pid,signal.SIGKILL)
            for key,_ in selector.select(timeout=0.1):
                chunk=os.read(key.fd,65536)
                if not chunk:
                    selector.unregister(key.fileobj);continue
                room=MAX_LOG-written-(len(TRUNCATION_MARKER) if len(chunk)>MAX_LOG-written else 0)
                if room>0:
                    part=chunk[:room];output.write(part);written+=len(part)
                if len(chunk)>room:
                    overflow=True;truncated=True
                    if process.poll() is None:signal_group(process.pid,signal.SIGTERM)
            if expired or overflow:
                if process.poll() is None:
                    try:process.wait(timeout=a.grace_seconds)
                    except subprocess.TimeoutExpired:signal_group(process.pid,signal.SIGKILL)
                if not selector.get_map():break
            if process.poll() is not None and not selector.get_map():break
        if truncated:output.write(TRUNCATION_MARKER)
    code=process.wait()
    receipt={'schema':'weft-bounded-command/1','stage':a.stage,'argv':command,'exitCode':code,'timedOut':expired,'logOverflow':overflow,'logBytesRetained':written+(len(TRUNCATION_MARKER) if truncated else 0),'logLimitBytes':MAX_LOG,'durationMs':int((time.monotonic()-start)*1000),'allowedSeconds':allowed,'originalDeadlineMonotonic':deadline}
    a.receipt.write_text(json.dumps(receipt,indent=2)+'\n')
    print(json.dumps({k:receipt[k] for k in ('stage','exitCode','timedOut','logOverflow','durationMs')}))
    return 0 if code==0 and not expired and not overflow else 1

if __name__=='__main__':sys.exit(main())
