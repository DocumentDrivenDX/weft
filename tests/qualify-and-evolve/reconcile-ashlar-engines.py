"""Audit same-statement engine identity plus independent unsigned expectations."""
import os,runpy
from pathlib import Path
root=Path(__file__).resolve().parents[2]
os.environ['WEFT_UNSIGNED_ENGINE_RECEIPTS']='1'
os.environ['WEFT_UNSIGNED_RECONCILE_INPUT']=str(root/'docs/helix/04-build/evidence/B-007-ashlar-engine-native')
os.environ['WEFT_UNSIGNED_RECONCILE_OUTPUT']=str(root/'docs/helix/04-build/evidence/B-007-ashlar-engine-reconciliation')
runpy.run_path(str(Path(__file__).with_name('reconcile-unsigned-boundaries.py')))
