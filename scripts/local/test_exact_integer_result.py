import copy,unittest
from exact_integer_result import decode_rows
def artifact(nullable=False):return {'status':'compiled','backend':{'backendId':'ashlar.databricks.mathematical-integer','targetProfile':'dbsql-mathematical-integer-candidate','backendVersion':'0.1.0-candidate'},'columns':[{'position':1,'outputName':'n','nullable':nullable,'representation':{'kind':'scalar','carrier':'text','decoder':'exact-integer','logicalType':{'family':'integer','facets':{},'nullable':nullable}}}]}
class ExactResult(unittest.TestCase):
 def test_original_exact_carriers_and_nullable_empty_sum(self):
  for token in ['9007199254740993','9'*38,'-'+'9'*38,'-0']:
   self.assertEqual(decode_rows(artifact(),[{'n':token}]),[{'n':{'exactInteger':str(int(token)),'originalCarrier':token}}])
  self.assertEqual(decode_rows(artifact(True),[{'n':None}]),[{'n':None}])
 def test_rounding_coercion_width_and_profile_refuse(self):
  for token in ['1'+'0'*38,'1.0','1e3','+1','1 ','١',1,True,None]:
   with self.assertRaises(ValueError):decode_rows(artifact(),[{'n':token}])
  for edit in [lambda a:a['backend'].update(targetProfile='dbsql-candidate'),lambda a:a['columns'][0]['representation']['logicalType'].update(facets={'integerWidth':{'bits':64,'signed':True}}),lambda a:a['columns'][0].update(nullable=1)]:
   a=artifact();edit(a)
   with self.assertRaises(ValueError):decode_rows(a,[{'n':'1'}])
  for row in [{},{'n':'1','extra':'2'}]:
   with self.assertRaises(ValueError):decode_rows(artifact(),[row])
if __name__=='__main__':unittest.main()
