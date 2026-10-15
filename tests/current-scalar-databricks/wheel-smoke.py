"""Exact standard-wheel Databricks API checks; no native security qualification."""
import argparse,contextlib,hashlib,io,json,pathlib,runpy,subprocess,sys,zipfile
import weft
p=argparse.ArgumentParser();p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
def sha(b):return hashlib.sha256(b).hexdigest()
def pins():
 modules=[m for n,m in sys.modules.items() if n.startswith('weft') and getattr(m,'__file__','').endswith(('.so','.pyd'))];assert len(modules)==1
 native=pathlib.Path(modules[0].__file__).read_bytes();shim=pathlib.Path(weft.__file__).read_bytes()
 with zipfile.ZipFile(a.wheel) as z:
  names=[n for n in z.namelist() if n.endswith(('.so','.pyd'))];assert len(names)==1 and native==z.read(names[0]) and shim==z.read('weft/__init__.py')
 return {'wheelSha256':sha(a.wheel.read_bytes()),'nativeModuleSha256':sha(native),'shimSha256':sha(shim)}
before=pins();capture=io.StringIO()
with contextlib.redirect_stdout(capture):runpy.run_path(str(pathlib.Path(__file__).with_name('python-bridge.py')),run_name='__main__')
result=json.loads(capture.getvalue());assert result['controls']==32 and pins()==before
root=pathlib.Path(__file__).resolve().parents[2];commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip() if (root/'.git').exists() else None
result.update(version=weft.__version__,sourceCommit=commit,wheel=a.wheel.name,pinsBeforeAfter=before,nativeAdmission=False,securityReady=False)
a.output.write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'status':'passed','version':weft.__version__,'controls':32}))
