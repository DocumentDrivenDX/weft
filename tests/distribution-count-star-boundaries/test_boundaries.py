"""Inert independent controls; never invoke the compiler."""
import importlib.util,json,unittest
from pathlib import Path
spec=importlib.util.spec_from_file_location('boundary',Path(__file__).resolve().parents[2]/'scripts/distribution/check-count-star-cli-boundaries.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Controls(unittest.TestCase):
    def test_closing_cancel_outranks_body_error(self):
        for typ in (KeyboardInterrupt,SystemExit,GeneratorExit):
            closing=typ()
            with self.assertRaises(typ) as caught:m.raise_cleanup(ValueError(),closing)
            self.assertIs(caught.exception,closing)
    def test_body_cancel_identity_retained(self):
        body=KeyboardInterrupt();m.raise_cleanup(body,SystemExit())
        self.assertTrue(body.cleanup_failed)
    def test_exact_error_and_preselection_semantics(self):
        for identity,marker in (('oversize','INPUT_LIMIT'),('invalid-utf8','UTF8'),('split-utf8-at-limit','UTF8'),('directory-input','INPUT_IO'),('closed-output','OUTPUT_IO')):
            row={'id':identity,'exit':2,'stdoutHex':'','stderrHex':('WEFT_CLI_'+marker+'\n').encode().hex()};m.check_observation(row)
            row['exit']=0
            with self.assertRaises(m.Refusal):m.check_observation(row)
        out={'diagnostics':[{'code':'WFT-INPUT','message':'Invalid request JSON','phase':'input','recoverability':'correct-input','severity':'error'}],'interfaceVersion':'weft-compile/0.4.0','status':'blocked'}
        for identity in ('exact-limit','exact-limit-multibyte','malformed-json'):
            row={'id':identity,'exit':0,'stdoutHex':(json.dumps(out,sort_keys=True,separators=(',',':'))+'\n').encode().hex(),'stderrHex':''};m.check_observation(row)
            out['interfaceVersion']='weft-compile/0.4.1';row['stdoutHex']=(json.dumps(out,sort_keys=True,separators=(',',':'))+'\n').encode().hex()
            with self.assertRaises(m.Refusal):m.check_observation(row)
            out['interfaceVersion']='weft-compile/0.4.0'
if __name__=='__main__':unittest.main()
