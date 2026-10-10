"""Pinned host-only Cargo entrypoint; raw streams never become diagnostics."""
import subprocess,json,pathlib,sys
if __package__ in (None,''):
 sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[1]))
from reliability.config import load_config,ConfigurationError
from reliability.process import run
ROOT=pathlib.Path(__file__).resolve().parents[2]
def main():
 try:
  separator=sys.argv.index('--');config=load_config(sys.argv[1:separator]);args=sys.argv[separator+1:]
  if not args or args[0] not in ('test','build','check','metadata','run'):raise ConfigurationError()
  identity=config.tool_identity(ROOT)
  result=run([str(config.cargo)]+args,ROOT,config.command_environment(),config.timeout_seconds)
  print(json.dumps({'version':'weft-runner/1','toolIdentity':identity,'exitCode':result.exit_code,'timedOut':result.timed_out,'passed':result.passed,'failed':result.failed,'ignored':result.ignored,'filtered':result.filtered,'durationMs':result.duration_ms}))
  return 1 if result.timed_out or result.exit_code else 0
 except (ConfigurationError,ValueError,OSError,RuntimeError,subprocess.SubprocessError):
  print('weft-runner: configuration or operation failed',file=sys.stderr);return 1
if __name__=='__main__':sys.exit(main())
