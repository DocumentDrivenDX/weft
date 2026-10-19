"""Replay all native unsigned widths; no database execution or profile promotion."""
import json,os,pathlib,runpy
ROOT=pathlib.Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-007-unsigned-all-widths-native'
summary=json.loads((BASE/'summary.json').read_text())
assert summary['status']=='passed' and summary['allWidths'] is True and summary['cases']==64 and summary['nativeStatements']==190
os.environ.update(WEFT_UNSIGNED_ALL_WIDTHS='1',WEFT_UNSIGNED_ENGINE_RECEIPTS='1',WEFT_UNSIGNED_WAREHOUSE_RECEIPTS='1',WEFT_UNSIGNED_RECONCILE_INPUT=str(BASE),WEFT_UNSIGNED_RECONCILE_OUTPUT=str(ROOT/'docs/helix/04-build/evidence/B-007-unsigned-all-widths-reconciliation'))
runpy.run_path(str(pathlib.Path(__file__).with_name('reconcile-unsigned-boundaries.py')))
