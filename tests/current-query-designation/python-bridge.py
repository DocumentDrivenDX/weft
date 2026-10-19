# SPDX-License-Identifier: MIT OR Apache-2.0
"""Actual native bridge correspondence, not native source/current authority.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import hashlib,json,pathlib
import weft
root=pathlib.Path(__file__).resolve().parent
checks=[]
def refuses(label,call):
 try: call()
 except (TypeError,ValueError,AttributeError): checks.append({'id':label,'refused':True})
 else: raise AssertionError(label)
for path in sorted((root/'fixtures').glob('*.request.json')):
 raw=path.read_text();prepared=weft.prepare_application_lineage(raw)
 actual=prepared.current_query_designations_json().encode()
 assert actual==path.with_name(path.name.replace('.request.','.report.')).read_bytes()
 assert prepared.source_request_json()==raw
 report=json.loads(actual);assert report['authority'] is False and report['securityReady'] is False
 checks.append({'id':path.stem,'sha256':hashlib.sha256(actual).hexdigest(),'passed':True})
 refuses('no-override:'+path.stem,lambda:prepared.current_query_designations_json({'originalAction':'read'}))
raw=(root/'fixtures/mixed-original-uses.request.json').read_text()
prepared=weft.prepare_application_lineage(raw)
for value in [raw,json.loads(raw),object()]:
 refuses('native-receiver:'+type(value).__name__,lambda value=value:weft.ApplicationLineagePreparation.current_query_designations_json(value))
request=json.loads(raw);request['opaqueSources']=[s for s in request['opaqueSources'] if s['role']!='query-designation']
missing=weft.prepare_application_lineage(json.dumps(request))
refuses('issued-object-without-profile',missing.current_query_designations_json)
assert prepared.source_request_json()==raw
print(json.dumps({'scope':'Native immutable producer bridge only; not permission, SQL or native current-source admission','controls':len(checks),'checks':checks},indent=2))
