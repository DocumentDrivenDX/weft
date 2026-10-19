# SPDX-License-Identifier: MIT OR Apache-2.0
"""Real native Python classes over actual supplied UMF packets; no database."""
import sys,json,hashlib,gc,copy,pickle
from pathlib import Path
sys.path.insert(0,sys.argv[1]);import weft
mode=sys.argv[2];root=Path(__file__).resolve().parents[2]
corpus=json.loads((root/'tests/current-query-presence/fixtures/actual-atomic-transport.json').read_text());emitted=json.loads((root/'tests/current-query-presence/fixtures/actual-atomic-transport-emitted.json').read_text())
def encode(x):return json.dumps(x,ensure_ascii=False,sort_keys=True,separators=(',',':'))
checks=[]
def ok(name,value):assert value,name;checks.append(name)
def refuses(name,fn):
 try:fn()
 except (TypeError,ValueError,AttributeError) as error:checks.append(name);return str(error)
 raise AssertionError(name)
if mode in ['legacy','runtime-only']:
 ok('class absent',not hasattr(weft,'CurrentPresencePreparation'));ok('factory absent',not hasattr(weft.ApplicationLineagePreparation,'prepare_current_presence'))
 for case in corpus['cases'][:5]:
  if mode=='legacy':refuses('old lineage rejects .4',lambda:weft.prepare_application_lineage(encode(case['request'])))
  else:
   parent=weft.prepare_application_lineage(encode(case['request']));ok('Core .4 inspection remains data',json.loads(parent.json())['authority'] is False);refuses('runtime-only old query refuses .4',parent.prepare_current_query)
else:
 ok('distinct class',weft.CurrentPresencePreparation is not weft.CurrentQueryPreparation)
 by_id={c['id']:c for c in corpus['cases']}
 for row in emitted:
  request=encode(by_id[row['id']]['request']);original=weft.prepare_application_lineage(request);parent_id=id(original);child=original.prepare_current_presence()
  ok(row['id']+':same parent',child.source_preparation() is original);ok(row['id']+':distinct old class',not isinstance(child,weft.CurrentQueryPreparation));refuses('old designation .4',original.prepare_current_query)
  report=child.json();ok('actual preparation hash',hashlib.sha256(report.encode()).hexdigest()==row['binding']['preparationSha256']);native=json.loads(child.native_request_json());ok('exact original bytes',native['sourceRequestJson']==request);ok('native required',native['authority'] is False and native['nativeAdmission']=='required')
  del original;gc.collect();ok('parent retained',id(child.source_preparation())==parent_id)
  binding=encode(row['binding']);compiled=child.lower_postgresql_current_presence_candidate_json(binding);ok('actual artifact',json.loads(compiled)==row['compiled']);checked=child.check_postgresql_current_presence_candidate_json(binding,encode(row['frame']));ok('actual transport',json.loads(checked)==row['report'])
 first=emitted[0];original=weft.prepare_application_lineage(encode(by_id[first['id']]['request']));child=original.prepare_current_presence();binding=encode(first['binding']);frame=encode(first['frame'])
 refuses('class substitution',lambda:setattr(child,'__class__',weft.CurrentQueryPreparation));refuses('instance dictionary',lambda:getattr(child,'__dict__'));refuses('deepcopy',lambda:copy.deepcopy(child));
 refuses('constructor',lambda:weft.CurrentPresencePreparation());refuses('report constructor',lambda:weft.CurrentPresencePreparation(child.json()));refuses('object constructor',lambda:object.__new__(weft.CurrentPresencePreparation));refuses('copy',lambda:copy.copy(child));refuses('pickle',lambda:pickle.dumps(child));refuses('subclass',lambda:type('Forged',(weft.CurrentPresencePreparation,),{}));refuses('field override',lambda:setattr(child,'original',original));refuses('factory wrong self',lambda:weft.ApplicationLineagePreparation.prepare_current_presence({}))
 if mode=='combined':
  ok('legacy lower descriptor enabled',hasattr(weft.CurrentQueryPreparation,'lower_postgresql_current_rows_candidate_json'));ok('legacy check descriptor enabled',hasattr(weft.CurrentQueryPreparation,'check_postgresql_current_rows_candidate_json'))
 legacy_raw=(root/'tests/current-query-designation/fixtures/disclosed-token-equality.request.json').read_text();legacy_parent=weft.prepare_application_lineage(legacy_raw);legacy=legacy_parent.prepare_current_query()
 refuses('old parent new factory',legacy_parent.prepare_current_presence)
 refuses('new descriptor old context',lambda:weft.CurrentPresencePreparation.lower_postgresql_current_presence_candidate_json(legacy,binding))
 refuses('new checker old context',lambda:weft.CurrentPresencePreparation.check_postgresql_current_presence_candidate_json(legacy,binding,frame))
 ok('new object no old method',not hasattr(child,'lower_postgresql_current_rows_candidate_json'))
 ok('old object no new method',not hasattr(legacy,'lower_postgresql_current_presence_candidate_json'))
 refuses('legacy compiler new object',lambda:weft.compile_json(child));legacy_report=json.loads(weft.compile_json(child.json()));ok('legacy compiler new report',legacy_report.get('status')!='compiled' and legacy_report.get('sql') is None)
 if hasattr(weft.CurrentQueryPreparation,'lower_postgresql_current_rows_candidate_json'):
  refuses('legacy lower new binding',lambda:legacy.lower_postgresql_current_rows_candidate_json(binding))
 if hasattr(weft.CurrentQueryPreparation,'lower_postgresql_current_rows_candidate_json'):
  refuses('old descriptor new self',lambda:weft.CurrentQueryPreparation.lower_postgresql_current_rows_candidate_json(child,binding))
 if hasattr(weft.CurrentQueryPreparation,'check_postgresql_current_rows_candidate_json'):
  refuses('old check new self',lambda:weft.CurrentQueryPreparation.check_postgresql_current_rows_candidate_json(child,binding,frame))
 # Exact PyString checks precede coercion or user callbacks.
 calls=[]
 class Trap:
  def __str__(self):calls.append('str');raise AssertionError('must not coerce')
  def __len__(self):calls.append('len');raise AssertionError('must not inspect')
 class Text(str):
  def __len__(self):calls.append('subclass-len');raise AssertionError('must not inspect')
 for value in [None,{},b'{}',bytearray(b'{}'),Trap(),Text(binding),'\ud800',' '*4_000_001]:
  refuses('nonexact or bounded binding',lambda value=value:child.lower_postgresql_current_presence_candidate_json(value))
 for value in [None,{},b'{}',Trap(),Text(frame),'\ud800']:
  refuses('nonexact frame',lambda value=value:child.check_postgresql_current_presence_candidate_json(binding,value))
 ok('no coercion callbacks',not calls)
 ok('UTF8 binding gate before parser',refuses('UTF8 binding byte bound',lambda:child.lower_postgresql_current_presence_candidate_json('é'*2_000_001)).startswith('WFT-CURRENT-PRESENCE-BRIDGE:'))
 ok('UTF8 frame gate before parser',refuses('UTF8 frame byte bound',lambda:child.check_postgresql_current_presence_candidate_json(binding,'é'*8_000_001)).startswith('WFT-CURRENT-PRESENCE-BRIDGE:'))
 changed=json.loads(binding);changed['version']='weft.postgresql.current-values/0.5-candidate';refuses('old binding version',lambda:child.lower_postgresql_current_presence_candidate_json(encode(changed)))
 changed=json.loads(frame);changed['resultRows'][0][0]='null';refuses('weak scalar',lambda:child.check_postgresql_current_presence_candidate_json(binding,encode(changed)))
 changed=json.loads(frame);changed['resultRows']=[];refuses('missing eligible row',lambda:child.check_postgresql_current_presence_candidate_json(binding,encode(changed)))
receipt={'scope':'Real Python native issued-type/binding/data correspondence; supplied logical producer fixtures; no native database/authority','mode':mode,'checks':len(checks),'passed':True,'checksPassed':checks,'modulePath':weft.__file__}
Path(sys.argv[3]).write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'mode':mode,'checks':len(checks),'passed':True,'nativeDatabase':False}))
