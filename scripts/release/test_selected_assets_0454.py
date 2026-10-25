# SPDX-License-Identifier: MIT OR Apache-2.0
"""Narrow negative controls for the 0.4.54 selected asset verifier."""
import importlib.util
import unittest
from pathlib import Path

path=Path(__file__).with_name('verify_selected_assets_0454.py')
spec=importlib.util.spec_from_file_location('verify_selected_assets_0454',path)
verifier=importlib.util.module_from_spec(spec);spec.loader.exec_module(verifier)

class ReleaseGateTests(unittest.TestCase):
    def test_wrong_workflow_run_refuses(self):
        with self.assertRaises(ValueError):
            verifier.validate_run({'repository':{'full_name':'DocumentDrivenDX/weft'},'path':'.github/workflows/release.yml','conclusion':'success','status':'completed','head_sha':'b'*40,'event':'workflow_dispatch','head_branch':'main'},repository='DocumentDrivenDX/weft',workflow='.github/workflows/release.yml',source='a'*40)
    def test_failed_mac_check_refuses(self):
        receipt={'schema':'weft-selected-macos-host/1','status':'passed','version':'0.4.54','sourceCommit':'a'*40,'checks':{'PREFLIGHT':'success','WHEEL':'success','VECTORS':'failure','PARITY':'success'}}
        with self.assertRaises(ValueError): verifier.validate_mac(receipt,source='a'*40)
    def test_local_only_receipt_refuses(self):
        receipt={'schema':'weft-selected-macos-host/1','status':'passed','version':'0.4.54','sourceCommit':'a'*40,'functionalQualification':'local-only','checks':{'PREFLIGHT':'success','WHEEL':'success','VECTORS':'success','PARITY':'success'}}
        with self.assertRaises(ValueError): verifier.validate_mac(receipt,source='a'*40)
    def test_wrong_source_and_extra_target_refuse(self):
        with self.assertRaises(ValueError):
            verifier.validate_run({'repository':{'full_name':'DocumentDrivenDX/weft'},'path':'w.yml','conclusion':'success','status':'completed','head_sha':'b'*40,'event':'workflow_dispatch','head_branch':'main'},repository='DocumentDrivenDX/weft',workflow='w.yml',source='a'*40)
        with self.assertRaises(ValueError): verifier.classify_wheels([Path('weft_sql-0.4.54-cp39-abi3-win_amd64.whl')]*3)

if __name__ == '__main__': unittest.main()
