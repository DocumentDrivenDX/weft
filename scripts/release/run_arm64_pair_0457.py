"""ARM-only cold acquisition/action boundary; starts before action checkout/pull."""
from pathlib import Path
import argparse,json,os,subprocess,sys
from current_macos_pair_0457 import need
PIN='e83996d129638aa358a18fbd1dfb82f0b0fb5d3b'

def action_environment(root):
 # Explicitly name the same workspace target directory. This also prevents the
 # pinned action's implicit host `cargo metadata` (without --locked) just to
 # discover that directory; this is an empty cold target, not cache restoration.
 env=os.environ.copy();env.update({'GITHUB_WORKSPACE':str(root),'CARGO_TARGET_DIR':str(root/'target'),'CARGO_BUILD_JOBS':'1','INPUT_COMMAND':'build','INPUT_ARGS':'--manifest-path crates/weft-python/Cargo.toml --release --locked --out dist','INPUT_MATURIN-VERSION':'1.9.6','INPUT_RUST-TOOLCHAIN':'1.90.0','INPUT_TARGET':'aarch64-unknown-linux-gnu','INPUT_MANYLINUX':'2014','INPUT_SCCACHE':'false','INPUT_CONTAINER':'quay.io/pypa/manylinux2014_aarch64@sha256:db68ee9e5e879da5945dfbc8cab446f02245da6522aca0a9e4cda9256c911d2f','INPUT_WORKING-DIRECTORY':str(root),'INPUT_BEFORE-SCRIPT-LINUX':'''set -eu
mkdir -p dist/current-native-tools
rustc --version --verbose > dist/current-native-tools/rust.stdout
cargo --version --verbose > dist/current-native-tools/cargo.stdout
maturin --version > dist/current-native-tools/maturin.stdout
grep -q '^rustc 1.90.0 ' dist/current-native-tools/rust.stdout
grep -q '^cargo 1.90.0 ' dist/current-native-tools/cargo.stdout
grep -qx 'maturin 1.9.6' dist/current-native-tools/maturin.stdout'''})
 # No token is persisted in a receipt or printed; same Action API handle.
 env['INPUT_TOKEN']=env.get('GH_TOKEN','');return env

def acquire_and_build(root,out):
 action=out.parent/'pinned-maturin-action';need(not action.exists(),'Fresh action checkout required')
 # Entire sequence is a child of the outer bounded stage, including canonical
 # action acquisition and its Docker image/tool setup; no unbounded pre-stage.
 subprocess.run(['git','init',str(action)],check=True)
 subprocess.run(['git','-C',str(action),'remote','add','origin','https://github.com/PyO3/maturin-action.git'],check=True)
 subprocess.run(['git','-C',str(action),'fetch','--depth=1','origin',PIN],check=True)
 subprocess.run(['git','-C',str(action),'checkout','--detach',PIN],check=True)
 need(subprocess.check_output(['git','-C',str(action),'rev-parse','HEAD'],text=True).strip()==PIN,'Pinned action identity differs')
 need("using: 'node24'" in (action/'action.yml').read_text() and "main: 'dist/index.js'" in (action/'action.yml').read_text(),'Pinned Action entrypoint differs')
 env=action_environment(root);receipt={k:v for k,v in env.items() if k.startswith('INPUT_') and k!='INPUT_TOKEN'};receipt.update(actionCommit=PIN,sourceCommit=__import__('current_macos_pair_0457').R,controllerCommit=os.environ['GITHUB_SHA'],scope='Cold source acquisition and genuine pinned Action; no cache restore')
 (out/'arm-action-inputs.json').write_text(json.dumps(receipt,indent=2)+'\n')
 subprocess.run(['node',str(action/'dist/index.js')],cwd=root,env=env,check=True)

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--product-root',type=Path,required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args();acquire_and_build(a.product_root.resolve(),a.out.resolve())
