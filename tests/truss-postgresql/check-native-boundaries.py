"""Check independent PostgreSQL observations; stdin is psql's single JSON row."""
import json,sys
from decimal import Decimal,localcontext
r=json.load(sys.stdin)
assert r['serverEncoding']=='UTF8',r
with localcontext() as context:
    context.prec=100
    assert Decimal(r['exactSum'])==Decimal('18446744073709551614.0000000000000000000000000001'),r
assert r['stringOrder']==['a','a ','é','😀'],r
assert r['trailingSpaceDistinct'] is True,r
assert r['numericOrder']==['-1','2','10'],r
assert r['lexicalOrder']==['-1','10','2'],r
assert r['states']==[{'present':False,'kind':None},{'present':True,'kind':'null'},{'present':True,'kind':'array'}],r
assert r['emptyCount']=='0' and r['emptySumIsNull'] is True,r
print(json.dumps({'serverVersion':r['serverVersion'],'nativeBoundaries':9,'qualifiedAdapter':False}))
