"""@covers US-006-AC2 @covers US-006-AC4 @covers US-008-AC4
Temporary fail-closed source mutants; active source is never edited.
"""
import subprocess,json,os,pathlib,sys,uuid
ROOT=pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts'))
from reliability.config import load_config,ConfigurationError
from reliability.mutations import execute,MutationError

def main():
 try:
  config=load_config(sys.argv[1:]);report=execute(config,ROOT)
  path=config.output_root/('source-mutations-'+uuid.uuid4().hex+'.json')
  with open(path,'x',opener=lambda name,flags:os.open(name,flags,0o600)) as stream:
   json.dump(report,stream,indent=2);stream.write('\n')
  print(json.dumps({'version':'weft-runner/1','status':'passed','detected':report['detected']}));return 0
 except (ConfigurationError,MutationError,OSError,ValueError,UnicodeError,RuntimeError,subprocess.SubprocessError):
  print('weft-runner: source mutation qualification failed',file=sys.stderr);return 1
if __name__=='__main__':sys.exit(main())
