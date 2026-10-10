"""Tiny fake producer ports exercise new mode; no compiler/native qualification."""
from dataclasses import replace
import json
import unittest
from unittest.mock import patch
import test_corpus as historical
m=historical.m


class PathsKeysCorpusTests(unittest.TestCase):
    def setUp(self):
        fixture=historical.CorpusTests('test_full_fake_transport_receipts_and_no_authority')
        fixture.setUp()
        self.addCleanup(fixture.temp.cleanup)
        self.fixture=fixture
        fixture.caps[-1]='relationship.boundedKeys'
        self.profile=('ashlar.databricks.paths-keys','0.4.0-paths-keys-candidate','spark4-delta4-paths-keys-candidate')
        self.backend_identity=dict(zip(('backendId','backendVersion','targetProfile'),self.profile))
        self.backend_identity['interfaceVersion']='weft-backend/0.3.0'
        response={'status':'compiled','backend':self.backend_identity,'targetContext':{'id':self.profile[2]},
                  'logicalPlan':{'requiredCapabilities':fixture.caps}}
        def row(identity,compiled=True):
            expected=response if compiled else {'status':'blocked'}
            return {'id':identity,'role':'valid','requestHex':m.encoded({'case':identity}).hex(),
                    'responseHex':(m.encoded(expected)+b'\n').hex()}
        pairs=(('weft-compile/0.1.0','weft-sql/0.1.0'),('weft-compile/0.2.0','weft-sql/0.2.0'),
               ('weft-compile/0.3.0','weft-sql/0.3.0'),('weft-compile/0.3.0','weft-sql/0.4.0'),
               ('weft-compile/0.5.0','weft-sql/0.5.0'))
        fences=[dict(id=identity,role='namespace' if i<3 else 'invalid',
                     requestHex=m.encoded(dict(interfaceVersion=pair[0],dialect=pair[1])).hex(),
                     responseHex=(m.encoded(fixture.fence)+b'\n').hex())
                for i,(identity,pair) in enumerate(zip(m.NAMESPACE_IDS,pairs))]
        (fixture.source/'original').unlink()
        retained=[];originals=[]
        def retain(name,raw):
            path=fixture.source/name;path.parent.mkdir(parents=True,exist_ok=True);path.write_bytes(raw)
            item=dict(path=name,mode='100644',gitBlob=m.hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest(),sha256=m.sha(raw),bytes=len(raw));retained.append(item);return dict(path=name,sha256=item['sha256'],bytes=len(raw))
        scopes=('columns-native','application-native','key-refusal','unsigned-columns','optional-native','relationship-native','compound-native','compound-boundaries-native','compound-application-native')
        for scope in scopes:
            rows=[dict(id=i,request={'original':i},response={'originalExpected':i}) for i in range(462)] if scope=='columns-native' else []
            raw=b''.join(m.encoded(x)+b'\n' for x in rows)
            name='source-subset/docs/helix/04-build/evidence/B-006-'+scope+'/compile-artifacts.jsonl'
            retain(name,raw);originals.extend(dict(x,id=scope+':'+str(x['id'])) for x in rows)
        for scope in ('scalar-native','global-native'):retain('source-subset/docs/helix/04-build/evidence/B-006-'+scope+'/.keep',b'')
        original=dict(id='cross-module',request={'cross':True},response={'crossExpected':True});originals.append(original)
        retain('source-subset/docs/helix/04-build/evidence/B-006-cross-module-native/compile.json',m.encoded(original))
        historical_entries=[dict(x,path=x['path'][len('source-subset/'):]) for x in retained]
        inventory_raw=m.encoded(dict(sourceCommit='b'*40,files=sorted(historical_entries,key=lambda x:x['path'])))
        inventory_desc=retain('inventory.json',inventory_raw)
        history={'sourceCommit':'b'*40,'sourceInventorySha256':m.sha(inventory_raw),'binarySha256':'e'*64,
                 'backendInput':{'sha256':'f'*64,'bytes':12},'cases':[
                 dict(id='legacy:'+x['id'],scope='legacy',role='namespace',exit=0,stderrHex='',migrations=[],requestHex=m.encoded(x['request']).hex(),originalExpectedHex=(m.encoded(x['response'])+b'\n').hex(),responseHex=(m.encoded(fixture.fence)+b'\n').hex()) for x in originals]}
        manifest={'realizationId':'historical-fixture','source':{'commit':'b'*40,'inventory':{'decodedSha256':m.sha(inventory_raw),'decodedBytes':len(inventory_raw),'trackedFiles':len(historical_entries),'artifact':inventory_desc}},
                  'executable':{'sha256':'e'*64},'backendManifests':[{'sha256':'f'*64,'bytes':12}]}
        descriptors=[retain('history.json',m.encoded(history)),retain('manifest.json',m.encoded(manifest))]
        fixture.inventory.write_bytes(m.encoded({'sourceCommit':'a'*40,'files':sorted(retained,key=lambda x:x['path'])}))
        historical_descriptor=descriptors[0]
        self.bundle={'format':'weft-paths-keys-corpus/0.1','sourceCommit':'a'*40,
             'paths':[row('case-'+str(i)) for i in range(50)],
             'controls':[row(identity,i>=16) for i,identity in enumerate(m.CONTROL_IDS)],
             'namespaceFences':fences,
             'coverage':{c:{'accepted':['paths:case-0'],'refused':[],'scope':'fixture metadata only'} for c in fixture.caps},
             'sources':descriptors,
             'historicalQualification':dict(sourceCommit='b'*40,realizationId='historical-fixture',receipt=dict(historical_descriptor),manifest=descriptors[1],legacyCases=463)}
        fixture.backend.write_bytes(m.encoded(dict(backendId=self.profile[0],backendVersion=self.profile[1],
             interfaceVersion='weft-backend/0.3.0',languageProfiles=[dict(dialectProfile='weft-sql/0.4.0',irVersion='weft-ir/0.4.0')],
             targetProfiles=[{'id':self.profile[2]}],capabilities=[{'id':c} for c in fixture.caps])))
        self.config=replace(fixture.config,profile='paths-keys',maximum_cases=74,
                            backend_sha256=m.sha(fixture.backend.read_bytes()),source_inventory_sha256=m.sha(fixture.inventory.read_bytes()))
        self.calls=[];self.schema_calls=[]
        self.save()

    def save(self):
        self.fixture.cases.write_bytes(m.encoded(self.bundle))
        self.config=replace(self.config,cases_sha256=m.sha(self.fixture.cases.read_bytes()))

    def execute(self,binary,data,mode,config):
        self.calls.append((data,mode))
        for case in self.bundle['namespaceFences']+self.bundle['paths']+self.bundle['controls']:
            if data==bytes.fromhex(case['requestHex']):return 0,bytes.fromhex(case['responseHex']),b''
        return self.fixture.execute(binary,data,mode,config)

    def qualify(self):
        self.fixture.calls=0
        return m.qualify(self.config,execute=self.execute,validate_schema=lambda *args:self.schema_calls.append(args))

    def test_small_profile_retains_historical_reference_not_execution(self):
        result=self.qualify()
        self.assertEqual(result['executedProtocolCases'],74)
        self.assertEqual(len(result['transport']),8)
        self.assertEqual(sum(result['protocolStatusCounts'].values()),74)
        self.assertLessEqual(result['retainedPrechargeBytes'],result['retainedBudgetLimitBytes'])
        self.assertLess(len(m.encoded(result)),result['retainedPrechargeBytes'])
        self.assertFalse(any(c['id'].startswith('legacy:') for c in result['cases']))
        self.assertEqual(result['historicalQualification'],self.bundle['historicalQualification'])
        self.assertEqual(result['producerProvenance']['harnessSha256'],result['harnessSha256'])
        self.assertIn('not asserted',result['producerProvenance']['scope'])
        self.assertEqual(len(self.calls),74*2+8)
        self.assertEqual([c[:2] for c in self.schema_calls[:5]],
                         [('legacy','namespace')]*3+[('controls','invalid')]*2)

    def test_profile_closed_and_old_api_default_preserved(self):
        self.assertEqual(self.fixture.config.profile,'paths')
        for value in ('',None,True,'other'):
            with self.assertRaises(m.Refusal):replace(self.config,profile=value)
        with self.assertRaises(m.Refusal):replace(self.config,declared_capability_count=47)

    def test_exact_fence_pair_and_static_response(self):
        for field,value in (('role','valid'),('responseHex',b'{}\n'.hex()),
                            ('requestHex',b'{"interfaceVersion":"weft-compile/0.4.0","dialect":"weft-sql/0.4.0"}'.hex())):
            original=dict(self.bundle['namespaceFences'][0])
            self.bundle['namespaceFences'][0][field]=value;self.save()
            with self.assertRaises(m.Refusal):self.qualify()
            self.assertFalse(self.config.output.exists())
            self.bundle['namespaceFences'][0]=original

    def test_inventories_and_history_closed_before_execution(self):
        for mutation in ('fence-order','paths-count','history-count','history-unbound','history-source','history-realization','unknown-key'):
            from copy import deepcopy
            original=deepcopy(self.bundle)
            if mutation=='fence-order':self.bundle['namespaceFences'].reverse()
            elif mutation=='paths-count':self.bundle['paths'].pop()
            elif mutation=='history-count':self.bundle['historicalQualification']['legacyCases']=True
            elif mutation=='history-unbound':self.bundle['historicalQualification']['receipt']['sha256']='c'*64
            elif mutation=='history-source':self.bundle['historicalQualification']['sourceCommit']='c'*40
            elif mutation=='history-realization':self.bundle['historicalQualification']['realizationId']='unproven-label'
            else:self.bundle['forged']=True
            self.save();self.calls=[]
            with self.assertRaises(m.Refusal):self.qualify()
            self.assertEqual(self.calls,[])
            self.bundle=original

    def test_self_consistent_resealed_history_cannot_forge_original_pairs(self):
        path=self.fixture.source/'history.json';original=path.read_bytes()
        for field in ('requestHex','originalExpectedHex','responseHex'):
            value=m.document(original);value['cases'][0][field]='7b7d'
            raw=m.encoded(value);path.write_bytes(raw)
            descriptor=dict(path='history.json',sha256=m.sha(raw),bytes=len(raw))
            self.bundle['sources'][0]=descriptor;self.bundle['historicalQualification']['receipt']=descriptor
            inventory=m.document(self.fixture.inventory.read_bytes())
            for item in inventory['files']:
                if item['path']=='history.json':item.update(sha256=m.sha(raw),bytes=len(raw),gitBlob=m.hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest())
            self.fixture.inventory.write_bytes(m.encoded(inventory));self.config=replace(self.config,source_inventory_sha256=m.sha(self.fixture.inventory.read_bytes()))
            self.save();self.calls=[]
            with self.assertRaisesRegex(m.Refusal,'historical-pairs'):self.qualify()
            self.assertEqual(self.calls,[])

    def test_old_backend_or_compiled_profile_cannot_satisfy_new_mode(self):
        backend=m.document(self.fixture.backend.read_bytes())
        original_backend=dict(backend)
        backend['backendId']='ashlar.databricks.paths'
        self.fixture.backend.write_bytes(m.encoded(backend))
        self.config=replace(self.config,backend_sha256=m.sha(self.fixture.backend.read_bytes()))
        with self.assertRaises(m.Refusal):self.qualify()
        self.assertFalse(self.config.output.exists())
        self.fixture.backend.write_bytes(m.encoded(original_backend))
        self.config=replace(self.config,backend_sha256=m.sha(self.fixture.backend.read_bytes()))
        response=m.document(bytes.fromhex(self.bundle['paths'][0]['responseHex']))
        response['backend']['backendId']='ashlar.databricks.paths'
        self.bundle['paths'][0]['responseHex']=(m.encoded(response)+b'\n').hex();self.save()
        with self.assertRaises(m.Refusal):self.qualify()
        self.assertFalse(self.config.output.exists())

    def test_missing_capability_coverage_and_low_budget_refuse(self):
        del self.bundle['coverage']['c0'];self.save()
        with self.assertRaises(m.Refusal):self.qualify()
        self.bundle['coverage']['c0']={'accepted':['paths:case-0'],'refused':[],'scope':'fixture'};self.save()
        self.config=replace(self.config,maximum_receipt_bytes=1)
        self.calls=[]
        with self.assertRaises(m.Refusal):self.qualify()
        self.assertEqual(self.calls,[])

    def test_cli_default_and_explicit_profile_reach_typed_configuration(self):
        from dataclasses import fields
        import contextlib,io
        for profile in ('paths','paths-keys'):
            arguments=['producer']
            for field in fields(self.config):
                if field.name=='profile':continue
                arguments.extend(['--'+field.name.replace('_','-'),str(getattr(self.config,field.name))])
            if profile=='paths-keys':arguments.extend(['--profile',profile])
            with patch('sys.argv',arguments),patch.object(m,'qualify') as qualify:
                m.main()
            self.assertEqual(qualify.call_args.args[0].profile,profile)
        with patch('sys.argv',arguments+['--profile','unknown']),patch.object(m,'qualify') as qualify,contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit) as error:m.main()
        self.assertEqual(error.exception.code,2)
        qualify.assert_not_called()
