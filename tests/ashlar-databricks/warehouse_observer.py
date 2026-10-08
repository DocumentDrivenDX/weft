"""Test-only query annotation; keeps raw native receipts unchanged."""
import json
from warehouse_capture import capture

def observe(client,label,statement,parameters,state):
    actual=client.sql(label,capture(statement),parameters)
    columns=client.records[-1]['response']['manifest']['schema']['columns']
    assert (columns[0]['name'],columns[0]['type_name'])==('__weft_warehouse','STRING')
    if actual:
        encoded=[row[0] for row in actual];method='same-statement'
    else:
        probe=client.sql(label+'-empty-warehouse-probe',"SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_warehouse")
        assert len(probe)==1 and len(probe[0])==1
        encoded=[probe[0][0]];method='separate-probe-after-empty-query'
    identities=set()
    for raw in encoded:
        identity=json.loads(raw)
        assert set(identity)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and identity['dbr_version'] is None
        assert all(isinstance(identity[k],str) and identity[k] for k in ['dbsql_version','u_build_hash','r_build_hash'])
        identities.add(json.dumps(identity,sort_keys=True))
    assert len(identities)==1
    state['identities'].update(identities)
    state['captures'].append(dict(label=label,method=method,warehouse=json.loads(next(iter(identities)))))
    return [row[1:] for row in actual],columns[1:]
