"""Host-owned psycopg protocol proof; compiler owns no connection or execution.
Requires the separately owned localhost test database and fresh full corpus.
"""
import json,os
from decimal import Decimal
from pathlib import Path
import psycopg
from psycopg import RawCursor
ROOT=Path(__file__).resolve().parents[2]
reports=json.loads((ROOT/'target/b005/full-corpus-reports.json').read_text())
cases=[r for r in reports if r['id'].startswith('parameter-') and r['response']['status']=='compiled']
receipts=[]
with psycopg.connect(host='127.0.0.1',port=int(os.environ['WEFT_DRIVER_PORT']),dbname='postgres',user='postgres',autocommit=True) as connection:
    version=connection.execute('SHOW server_version').fetchone()[0]
    assert version.startswith('17.9'),version
    assert connection.execute('SHOW server_encoding').fetchone()[0]=='UTF8'
    for case in cases:
        result=case['response'];request=case['request'];binding=json.loads(request['target']['bindingJson'])
        parameter=request['parameters']['boundary'];family=parameter['family'];expected=Decimal(parameter['value'])
        element='customer-id' if family=='integer' else 'order-total'
        prop=next(p for p in binding['properties'] if p['logical']['element']==element)
        ty=int(prop['ownerTypeId']);pid=int(prop['propertyId'])
        with connection.transaction():
            connection.execute('CREATE TEMP TABLE object(id bigint PRIMARY KEY,type_id int NOT NULL,props jsonb NOT NULL) ON COMMIT DROP')
            connection.execute('CREATE TEMP TABLE row_home_state(state_id bigint PRIMARY KEY,owner_kind text,object_id bigint,object_type_id int,property_owner_type_id int,property_id int,root_node_id bigint) ON COMMIT DROP')
            connection.execute('CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,PRIMARY KEY(state_id,node_id)) ON COMMIT DROP')
            connection.execute('CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value boolean,numeric_value numeric,PRIMARY KEY(state_id,node_id)) ON COMMIT DROP')
            connection.execute('INSERT INTO object VALUES (987654321,%s,jsonb_build_object(%s::text,%s::numeric))',(ty,str(pid),expected))
            connection.execute("INSERT INTO row_home_state VALUES (1,'object',987654321,%s,%s,%s,1)",(ty,ty,pid))
            connection.execute("INSERT INTO row_home_node VALUES (1,1,NULL,'scalar')")
            connection.execute('INSERT INTO row_home_scalar VALUES (1,1,%s,NULL,NULL,%s)',(family,expected))
            slots=[Decimal(p['value']) if p['logicalType']['family'] in ['integer','decimal'] else p['value'] for p in result['parameters']]
            guards=[g for o in result['obligations'] if o['id']=='truss.candidate.scalarIntegrity' for g in o['parameters']['checks']]
            with RawCursor(connection) as cursor:
                for guard in guards:
                    cursor.execute(guard['sql'],slots,prepare=True)
                    assert cursor.fetchall()==[('0',)],case['id']
                # Native $n placeholders are sent unchanged; no SQL substitution.
                cursor.execute(result['sql'],slots,prepare=True)
                rows=cursor.fetchall()
                assert len(rows)==1 and isinstance(rows[0][0],str),(case['id'],rows)
                assert Decimal(rows[0][0])==expected,(case['id'],rows,expected)
                assert cursor.description[0].type_code==25,(case['id'],cursor.description)
                receipts.append(dict(id=case['id'],actual=rows[0][0],expected=str(expected),carrierOid=25,prepared=True))
    summary=dict(cases=len(receipts),driver=psycopg.__version__,libpq=psycopg.pq.version(),server=version,nativePlaceholders=True,prepared=True,textCarrier=True)
(ROOT/'target/b005/driver-native-reports.json').write_text(json.dumps(dict(summary=summary,receipts=receipts),indent=2))
print(json.dumps(summary))
