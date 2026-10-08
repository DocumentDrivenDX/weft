"""Reuse engine/result corruption controls against warehouse identity receipts."""
import os,runpy
from pathlib import Path
root=Path(__file__).resolve().parents[2]
os.environ.update(WEFT_UNSIGNED_WAREHOUSE_RECEIPTS='1',WEFT_ASHLAR_ENGINE_CONTROL_INPUT=str(root/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-boundaries-native'))
runpy.run_path(str(Path(__file__).with_name('ashlar-engine-controls.py')))
