"""Start only after the coordinator explicitly grants the native lease."""
from __future__ import annotations
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys

import psycopg
from psycopg.conninfo import make_conninfo

prepared = Path(__file__).resolve().parent
out = Path(sys.argv[1])
out.mkdir()
(out / "socket").mkdir()
(out / "source").mkdir()
for name in ("native_controls.py", "README.md", "alternate-request.json", "alternate-program.json",
             "inverse-unique-request.json", "inverse-unique-program.json", "degree-request.json", "degree-program.json"):
    shutil.copyfile(prepared / name, out / "source" / name)
shutil.copyfile(__file__, out / "owned_runner.py")
pg = Path("/opt/homebrew/opt/postgresql@18/bin")
checks = []
started = False
completed = False
postmaster = None


def run(name: str, command: list, env: dict | None = None, *, required: bool = True) -> int:
    with (out / f"{name}.log").open("w") as log:
        result = subprocess.run(list(map(str, command)), env={**os.environ, **(env or {})},
                                stdout=log, stderr=subprocess.STDOUT)
    checks.append({"name": name, "command": list(map(str, command)), "exitCode": result.returncode})
    if required and result.returncode:
        raise RuntimeError(f"{name} failed; retained log")
    return result.returncode


try:
    run("version", [pg / "postgres", "--version"])
    run("init", [pg / "initdb", "-D", out / "data", "-A", "trust", "-U", "sdk_admin",
                 "--no-sync", "--encoding=UTF8", "--locale=C"])
    run("start", [pg / "pg_ctl", "-D", out / "data", "-l", out / "server.log", "-o",
                  f"-F -k {out / 'socket'} -c listen_addresses='' -p 55507", "-w", "start"])
    started = True
    postmaster = int((out / "data" / "postmaster.pid").read_text().splitlines()[0])
    admin_dsn = make_conninfo(host=str(out / "socket"), port=55507, dbname="postgres", user="sdk_admin")
    reader_dsn = make_conninfo(host=str(out / "socket"), port=55507, dbname="postgres", user="weft_r13_reader")
    with psycopg.connect(admin_dsn, autocommit=True) as admin:
        admin.execute("CREATE ROLE weft_r13_reader LOGIN NOSUPERUSER NOBYPASSRLS")
        admin.execute("GRANT CREATE ON DATABASE postgres TO weft_r13_reader")
    run("campaign", [sys.executable, "-I", "-B", out / "source" / "native_controls.py"],
        {"UMF_PG_TEST_DSN": reader_dsn, "UMF_PG_ADMIN_DSN": admin_dsn})
    completed = True
finally:
    if started:
        run("stop", [pg / "pg_ctl", "-D", out / "data", "-m", "fast", "-w", "stop"], required=False)
    alive = False
    if postmaster is not None:
        try:
            os.kill(postmaster, 0)
            alive = True
        except ProcessLookupError:
            pass
    stopped = not alive and not (out / "data" / "postmaster.pid").exists()
    sources = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in (out / "source").iterdir() if p.is_file()}
    result = {"status": "passed" if completed and stopped else "failed", "checks": checks,
              "serverStopped": stopped, "postmasterAlive": alive, "sourceSha256": sources,
              "temporal": "HELD", "scope": "original owner admissions and native query; not SDK E2E"}
    (out / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    assert stopped, "owned PostgreSQL cleanup failed"
print(json.dumps(result))
