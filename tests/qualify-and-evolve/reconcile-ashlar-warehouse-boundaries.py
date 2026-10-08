"""Replay exact unsigned results with same-statement Databricks SQL identity."""
import os,runpy
from pathlib import Path
root=Path(__file__).resolve().parents[2]
os.environ.update(WEFT_UNSIGNED_ENGINE_RECEIPTS='1',WEFT_UNSIGNED_WAREHOUSE_RECEIPTS='1',WEFT_UNSIGNED_RECONCILE_INPUT=str(root/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-boundaries-native'),WEFT_UNSIGNED_RECONCILE_OUTPUT=str(root/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-boundaries-reconciliation'))
runpy.run_path(str(Path(__file__).with_name('reconcile-unsigned-boundaries.py')))
