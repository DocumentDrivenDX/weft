# SPDX-License-Identifier: MIT OR Apache-2.0
import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from retained_output import EVIDENCE, replay_output


class ReplayOutput(unittest.TestCase):
    def test_replay_redirects_archival_outputs_only(self):
        requested = EVIDENCE/'packet/summary'
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with patch.dict(os.environ,{'WEFT_REPLAY_OUTPUT_ROOT':directory}):
                self.assertEqual(replay_output(requested),root/'packet/summary')
                self.assertEqual(replay_output(root/'control/out'),root/'control/out')
            with patch.dict(os.environ,{},clear=True):
                self.assertEqual(replay_output(requested),requested)
            with patch.dict(os.environ,{'WEFT_REPLAY_OUTPUT_ROOT':'relative'}), self.assertRaises(ValueError):
                replay_output(requested)


if __name__ == '__main__':
    unittest.main()
