"""Actual public UMF receipts, original lexical token and corruption controls."""
import json,os,subprocess,tempfile,unittest,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
class SourceTokens(unittest.TestCase):
 def test_public_validation_and_exact_json_custody(self):
  repo=os.environ.get('ASHLAR_PUBLIC_UMF','/private/tmp/ashlar-umf-dataset-45473')
  source=(ROOT/'tests/fixtures/original-commerce-0.8/ontology.json').read_text()
  field={'documentId':json.loads(source)['id'],'revision':'original-1','module':'domain','element':'order_lines.quantity'}
  cases=[('9007199254740993',True,None),('9'*38,True,None),('1'+'0'*38,True,None),('9'*50,True,None),('1e3',False,'backend-capability'),('12.5',False,'source-integrity'),('true',False,'source-integrity'),('null',False,'source-integrity'),('"123"',False,'source-integrity'),('10,"q":11',False,'source-integrity'),('"\\u0031"',False,'source-integrity')]
  rows=[{'source_system':'probe','type_id':'3','id':str(i),'schema_revision':'original-1','props_json':'{"q":'+token+'}','extracted_token':'1000.0'if token=='1e3'else token}for i,(token,*_)in enumerate(cases)]
  with tempfile.TemporaryDirectory(prefix='public-integer-')as t:
   request=Path(t)/'request.json';receipt=Path(t)/'receipt.json';data={'sourceText':source,'modelPins':[{'documentId':field['documentId'],'revision':'original-1','umfVersion':'0.8.0','sha256':hashlib.sha256(source.encode()).hexdigest()}],'checks':[{'check':{'field':field,'record':dict(field,element='order_lines'),'propertyId':'q'},'rows':rows}]}
   request.write_text(json.dumps(data))
   subprocess.run(['bun',str(ROOT/'scripts/local/public_integer_source.ts'),repo,str(request),str(receipt)],check=True)
   actual=json.loads(receipt.read_text());self.assertEqual(actual['originalRequestText'],request.read_text());self.assertFalse(actual['admitted'])
   for observed,(token,admitted,failure)in zip(actual['observations'][0]['rows'],cases):
    self.assertEqual(observed['admitted'],admitted);self.assertEqual(observed['failureClass'],failure);self.assertEqual(observed['original'],rows[int(observed['original']['id'])])
    if admitted:self.assertEqual(observed['token'],token);self.assertIs(observed['publicResult']['valid'],True);self.assertIs(observed['publicResult']['complete'],True)
   for scope,key,value in [('pin','documentId','swapped-document'),('pin','revision','swapped-revision'),('pin','sha256','0'*64),('pin','umfVersion','0.7.0'),('field','documentId','swapped-document'),('record','revision','swapped-revision')]:
    changed=json.loads(json.dumps(data));target=changed['modelPins'][0]if scope=='pin'else changed['checks'][0]['check'][scope];target[key]=value;request.write_text(json.dumps(changed));refusal=Path(t)/(scope+'-'+key+'.json')
    run=subprocess.run(['bun',str(ROOT/'scripts/local/public_integer_source.ts'),repo,str(request),str(refusal)],capture_output=True);self.assertNotEqual(run.returncode,0);self.assertFalse(refusal.exists())
   exponent=actual['observations'][0]['rows'][4];self.assertIs(exponent['publicResult']['valid'],True);self.assertEqual(exponent['token'],'1e3')
if __name__=='__main__':unittest.main()
