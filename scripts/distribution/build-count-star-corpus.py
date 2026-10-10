"""Author additive 0.4.1 expectations; never execute or normalize a producer."""
import argparse,copy,gzip,hashlib,json,re
from pathlib import Path
OLD_BACKEND='0.4.0-paths-keys-candidate'
NEW_BACKEND='0.4.1-count-star-having-candidate'
PAIRS={'interfaceVersion':('weft-compile/0.4.0','weft-compile/0.4.1'),
       'dialect':('weft-sql/0.4.0','weft-sql/0.4.1'),
       'backendVersion':(OLD_BACKEND,NEW_BACKEND)}
RESPONSE_PAIRS={**PAIRS,'irVersion':('weft-ir/0.4.0','weft-ir/0.4.1'),
                'dialectProfile':('weft-sql/0.4.0','weft-sql/0.4.1')}
MESSAGES={'Request is outside the closed 0.4 envelope':'Request is outside the closed 0.4.1 envelope',
          'This entrypoint requires the exact 0.4 compile and dialect pair':'This entrypoint requires the exact 0.4.1 compile and dialect pair'}
NEW_IDS=('original-replay','joined-filtered-full-bag','expansion-count-owner',
         'old-dialect-new-interface','new-dialect-old-interface',
         'new-language-old-backend','old-language-new-backend','candidate-optout',
         'unprojected-count','no-groups','output-alias-not-source-field')
def sha(raw):return hashlib.sha256(raw).hexdigest()
def encoded(value):return json.dumps(value,ensure_ascii=False,separators=(',',':')).encode()
def migrate(raw,response=False):
    # Match complete unescaped key/value tokens only; escaped model/binding/SQL
    # content and malformed duplicate-object bytes remain untouched.
    changes=[];pairs=RESPONSE_PAIRS if response else PAIRS
    edits=[]
    for key,(old,new) in pairs.items():
        pattern=re.compile(rb'(?<!\\)"'+key.encode()+rb'"\s*:\s*"'+re.escape(old.encode())+rb'"')
        for match in pattern.finditer(raw):
            start=match.end()-len(old)-1;end=match.end()-1
            edits.append((start,end,new.encode(),key,old,new))
    if response:
        for old,new in MESSAGES.items():
            pattern=re.compile(rb'(?<!\\)"message"\s*:\s*"'+re.escape(old.encode())+rb'"')
            for match in pattern.finditer(raw):
                start=match.end()-len(old)-1;end=match.end()-1
                edits.append((start,end,new.encode(),'diagnostic.message',old,new))
    for start,end,new,key,old,value in sorted(edits,reverse=True):
        changes.append({'offset':start,'end':end,'field':key,'old':old,'new':value})
        raw=raw[:start]+new+raw[end:]
    return raw,list(reversed(changes))
def migrate_response(identity,raw):
    if identity=='controls:duplicate-envelope-member':
        # paths_keys::compile_json selects041 only after checked_json succeeds.
        # Duplicate JSON cannot select any profile; retain original040 refusal.
        return raw,[]
    return migrate(raw,True)
def author(old_raw,new_raw,backend_raw,source_commit):
    old=json.loads(old_raw);new=json.loads(new_raw);backend=json.loads(backend_raw)
    if len(old['paths'])!=50 or len(old['controls'])!=19 or len(old['coverage'])!=48:raise ValueError('historical-inventory')
    if backend['backendVersion']!=NEW_BACKEND or len(backend['capabilities'])!=49:raise ValueError('new-profile')
    cases=[];ledger=[]
    for scope in ('paths','controls'):
        for c in old[scope]:
            q=bytes.fromhex(c['requestHex']);a=bytes.fromhex(c['responseHex'])
            mq,qchanges=migrate(q);ma,achanges=migrate_response(scope+':'+c['id'],a)
            oq=json.loads(q);nq=json.loads(mq)
            # These opaque authored values include their original internal bytes.
            for key in ('modules','modelJson','sql','modelSha256'):
                if oq.get(key)!=nq.get(key):raise ValueError('authored-source-change')
            for key in ('bindingJson','bindingSha256','targetProfile','backendId'):
                if oq.get('target',{}).get(key)!=nq.get('target',{}).get(key):raise ValueError('binding-change')
            identity=scope+':'+c['id']
            cases.append({'id':identity,'role':c['role'],'requestHex':mq.hex(),'responseHex':ma.hex(),'expectation':'exact-migrated-bytes'})
            ledger.append({'id':identity,'originalRequestSha256':sha(q),'migratedRequestSha256':sha(mq),'originalResponseSha256':sha(a),'migratedResponseSha256':sha(ma),'requestChanges':qchanges,'responseChanges':achanges,'responseMigrationReason':('Duplicate JSON is refused by checked_json before041 profile selection; retain original040 malformed refusal byte-for-byte.' if identity=='controls:duplicate-envelope-member' else 'Exact explicit version-marker migration.')})
    additions=[c for c in new['checks'] if 'request' in c]
    if tuple(c['case'] for c in additions)!=NEW_IDS:raise ValueError('independent-new-inventory')
    for c in additions:cases.append({'id':'count-star:'+c['case'],'role':'valid','requestHex':encoded(c['request']).hex(),'expectedResponse':c['response'],'expectation':'independent-semantic-response'})
    coverage=copy.deepcopy(old['coverage'])
    coverage['aggregate.havingCountStarGreater']={'accepted':['count-star:'+i for i in NEW_IDS[:3]],'refused':['count-star:'+i for i in ('candidate-optout','unprojected-count','no-groups','output-alias-not-source-field')], 'scope':'Original explicit COUNT(*) HAVING requests: independent expected complete responses, projected/grouped count correspondence and candidate refusal. Native bag/capacity/source execution remains separate.'}
    if set(coverage)!={c['id'] for c in backend['capabilities']}:raise ValueError('complete-new-capability-inventory')
    inputs={'historicalCorpus':{'bytes':len(old_raw),'sha256':sha(old_raw),'sourceCommit':old['sourceCommit']},'independentCountStarCases':{'bytes':len(new_raw),'sha256':sha(new_raw)},'backendManifest':{'bytes':len(backend_raw),'sha256':sha(backend_raw)}}
    corpus={'format':'weft-count-star-having-corpus/0.1','sourceCommit':source_commit,'backendVersion':NEW_BACKEND,'inputs':inputs,'cases':cases,'coverage':coverage,'migrationPolicy':'Explicit keyed protocol/backend/language declaration markers and two exact version diagnostic messages only. Duplicate JSON keeps its original040 preselection refusal envelope because checked_json cannot select041. Original SQL/model/binding values, invalid control structure, query result meanings, host obligations and SQL artifacts are unchanged. Retained original 0.4 corpus is separate and immutable; no general normalization or index authority.'}
    return corpus,{'format':'weft-count-star-having-migration-ledger/0.1','inputs':inputs,'cases':ledger,'originalNamespaceFencesRetainedSeparately':5,'newIndependentCases':list(NEW_IDS)}
def main():
    p=argparse.ArgumentParser()
    for key in ('historical','new-cases','backend','output','ledger'):p.add_argument('--'+key,type=Path,required=True)
    p.add_argument('--source-commit',required=True);a=p.parse_args()
    corpus,ledger=author(a.historical.read_bytes(),a.new_cases.read_bytes(),a.backend.read_bytes(),a.source_commit)
    for path,value in ((a.output,corpus),(a.ledger,ledger)):
        raw=encoded(value)+b'\n'
        with path.open('xb') as f:f.write(gzip.compress(raw,mtime=0) if path.suffix=='.gz' else raw)
if __name__=='__main__':main()
