"""Real Cargo dependency checker and Rust visibility controls, without installs."""
import json,os,subprocess,tempfile,unittest
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];CHECK=ROOT/'scripts/checks/check-module-boundaries.py'
class Boundaries(unittest.TestCase):
    def fixture(self,root,forbidden=False,private=False):
        names=list(json.loads((ROOT/'scripts/checks/module-boundaries.json').read_text())['modules'])
        (root/'Cargo.toml').write_text('[workspace]\nresolver="2"\nmembers='+json.dumps(names)+'\n')
        for name in names:
            d=root/name;(d/'src').mkdir(parents=True)
            deps={'weft-databricks':['weft-core'],'weft-python':['weft-runtime']}.get(name,[])
            if forbidden and name=='weft-core':deps=['weft-postgresql']
            text='[package]\nname='+json.dumps(name)+'\nversion="0.0.0"\nedition="2021"\n[dependencies]\n'
            text+=''.join(t+'={path="../'+t+'"}\n'for t in deps);(d/'Cargo.toml').write_text(text)
            code='fn hidden() {}\npub fn public() {}\n'if name=='weft-core'else ''
            if private and name=='weft-databricks':code='pub fn illegal(){weft_core::hidden();}\n'
            (d/'src/lib.rs').write_text(code)
        subprocess.run(['cargo','generate-lockfile','--offline','--manifest-path',str(root/'Cargo.toml')],check=True,capture_output=True,text=True)
    def invoke(self,root):return subprocess.run([os.sys.executable,str(CHECK),'--manifest-path',str(root/'Cargo.toml')],capture_output=True,text=True)
    def test_actual_allowed_dependency_passes(self):
        with tempfile.TemporaryDirectory()as temp:
            root=Path(temp);self.fixture(root);r=self.invoke(root);self.assertEqual(r.returncode,0,r.stderr)
            self.assertIn(['weft-databricks','weft-core'],json.loads(r.stdout)['edges'])
    def test_actual_forbidden_core_to_backend_fails(self):
        with tempfile.TemporaryDirectory()as temp:
            root=Path(temp);self.fixture(root,forbidden=True);r=self.invoke(root);self.assertNotEqual(r.returncode,0)
            self.assertIn('Forbidden Cargo dependency: weft-core -> weft-postgresql',r.stderr)
    def test_actual_rust_private_symbol_refuses(self):
        with tempfile.TemporaryDirectory()as temp:
            root=Path(temp);self.fixture(root,private=True);self.assertEqual(self.invoke(root).returncode,0)
            r=subprocess.run(['cargo','check','--offline','--locked','--manifest-path',str(root/'Cargo.toml'),'-p','weft-databricks'],capture_output=True,text=True)
            self.assertNotEqual(r.returncode,0);self.assertIn('is private',r.stderr)
if __name__=='__main__':unittest.main()
