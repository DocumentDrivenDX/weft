"""Finite fake proof controls; never manufacture a qualified compiler receipt."""
import copy
import gzip
import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/'scripts/distribution'))
spec=importlib.util.spec_from_file_location('owning_paths_assembler',ROOT/'scripts/distribution/assemble-paths-distribution.py')
a=importlib.util.module_from_spec(spec);sys.modules[spec.name]=a;spec.loader.exec_module(a)
import paths_keys_distribution as k


class PathsKeysAssemblyTests(unittest.TestCase):
    def setUp(self):
        caps=['cap'+str(i)for i in range(47)]+['relationship.boundedKeys']
        backend_identity=dict(backendId=k.BACKEND_ID,backendVersion=k.VERSION,interfaceVersion='weft-backend/0.3.0',targetProfile=k.TARGET)
        self.backend=dict(backendId=k.BACKEND_ID,backendVersion=k.VERSION,interfaceVersion='weft-backend/0.3.0',languageProfiles=[dict(dialectProfile='weft-sql/0.4.0',irVersion='weft-ir/0.4.0')],targetProfiles=[dict(id=k.TARGET)],capabilities=[dict(id=c)for c in caps])
        def case(identity,compiled,role='valid'):
            response=dict(interfaceVersion='weft-compile/0.4.0',status='compiled'if compiled else'blocked')
            if compiled:response.update(backend=backend_identity,targetContext=dict(id=k.TARGET),logicalPlan=dict(requiredCapabilities=caps))
            return dict(id=identity,role=role,requestHex=a.encoded(dict(fake=identity)).hex(),responseHex=(a.encoded(response)+b'\n').hex())
        fences=[dict(id=i,role='namespace'if n<3 else'invalid',requestHex='7b7d',responseHex=(a.encoded(a.FENCE)+b'\n').hex())for n,i in enumerate(k.NAMESPACE_IDS)]
        self.bundle=dict(format='weft-paths-keys-corpus/0.1',sourceCommit=k.SOURCE,paths=[case('case'+str(i),i<36)for i in range(50)],controls=[case(i,n>=16,'invalid'if i in ('unknown-envelope-member','mismatched-interface-dialect','duplicate-envelope-member')else'valid')for n,i in enumerate(a.CONTROL_IDS)],namespaceFences=fences,coverage={c:dict(accepted=['paths:case0'],refused=[],scope='fake metadata only')for c in caps},sources=[],historicalQualification={'fixtureOnly':True})
        rows=[]
        for scope,items in [('namespace',fences),('paths',self.bundle['paths']),('controls',self.bundle['controls'])]:
            for n,item in enumerate(items):rows.append(dict(item,id=scope+':'+item['id'],scope=('legacy'if n<3 else'controls')if scope=='namespace'else scope,exit=0,stderrHex='',migrations=[]))
        transport=[]
        for identity in a.TRANSPORT_IDS:
            failed=identity in ('oversize','invalid-utf8','split-utf8-at-limit','directory-input','closed-output')
            marker='INPUT_LIMIT'if identity=='oversize'else'UTF8'if'utf8'in identity else'INPUT_IO'if identity=='directory-input'else'OUTPUT_IO'
            transport.append(dict(id=identity,exit=2 if failed else 0,stdoutHex=''if failed else a.encoded(dict(status='blocked')).hex(),stderrHex=('WEFT_CLI_'+marker+'\n').encode().hex()if failed else''))
        self.receipt=dict(sourceCommit=k.SOURCE,binarySha256=k.BINARY,sourceInventorySha256=k.INVENTORY,declaredCapabilityCount=48,profile='paths-keys',executedProtocolCases=74,cases=rows,protocolStatusCounts=dict(compiled=39,blocked=35),coverage=copy.deepcopy(self.bundle['coverage']),transport=transport,historicalQualification=self.bundle['historicalQualification'],harnessSha256=k.HARNESS,schemaCheckerSha256=k.BRIDGE,producerProvenance=dict(harnessSha256=k.HARNESS))

    def verify(self):return k.verify_records(self.receipt,self.bundle,self.backend,a)

    def test_separate_fake_complete_shape(self):
        self.assertEqual(len(self.verify()),74)
        self.assertNotEqual(k.SOURCE,a.SOURCE);self.assertNotEqual(k.HARNESS,a.RECEIPT)

    def test_old_defaults_and_closed_profile(self):
        with tempfile.TemporaryDirectory()as td:
            p=Path(td).resolve();config=a.Config(p,p,p,p/'out','candidate')
            self.assertEqual(config.profile,'paths')
            with self.assertRaises(ValueError):a.Config(p,p,p,p/'out','candidate','unknown')
            with patch.object(k,'assemble_keys',return_value={'inert':True})as call:
                self.assertEqual(a.assemble(a.Config(p,p,p,p/'out','candidate','paths-keys')),{'inert':True})
                self.assertEqual(call.call_args.args[1],a)

    def test_realization_identity_matches_public_manifest_before_effects(self):
        with tempfile.TemporaryDirectory()as td:
            p=Path(td).resolve()
            for profile in ('paths','paths-keys'):
                for identity in ('.','..','_candidate','-candidate','é-candidate','candidate/other'):
                    with self.assertRaises(ValueError):a.Config(p,p,p,p/'output',identity,profile)
                    self.assertFalse((p/'output').exists())
                for identity in ('weft-candidate','3a_candidate','A.b-1'):
                    self.assertEqual(a.Config(p,p,p,p/'output',identity,profile).realization_id,identity)

    def test_profiles_inventory_and_custody_refuse(self):
        for mutation in ('old-profile','row-count','backend','status-count','producer','historical','transport'):
            self.setUp()
            if mutation=='old-profile':self.receipt['profile']='paths'
            elif mutation=='row-count':self.receipt['cases'].pop()
            elif mutation=='backend':self.backend['backendId']='ashlar.databricks.paths'
            elif mutation=='status-count':self.receipt['protocolStatusCounts']['compiled']=40
            elif mutation=='producer':self.receipt['harnessSha256']='0'*64
            elif mutation=='historical':self.receipt['historicalQualification']={}
            else:self.receipt['transport'][0]['stderrHex']=''
            with self.assertRaises(ValueError,msg=mutation):self.verify()

    def test_exact_original_protocol_and_selected_backend_refuse(self):
        for mutation in ('request','response','role','compiled-backend','coverage'):
            self.setUp()
            if mutation in ('request','response'):self.receipt['cases'][5][mutation+'Hex']='7b7d'
            elif mutation=='role':self.receipt['cases'][5]['role']='invalid'
            elif mutation=='coverage':self.bundle['coverage'].pop('relationship.boundedKeys')
            else:
                response=a.document(bytes.fromhex(self.receipt['cases'][5]['responseHex']));response['backend']['backendVersion']='old'
                raw=a.encoded(response).hex();self.receipt['cases'][5]['responseHex']=raw;self.bundle['paths'][0]['responseHex']=raw
            with self.assertRaises(ValueError,msg=mutation):self.verify()

    def test_shared_compression_and_package_bounds_unchanged(self):
        with self.assertRaises(ValueError):a.inflate(gzip.compress(b'0123456789'),9)
        with self.assertRaises(ValueError):a.inflate(gzip.compress(b'a')+gzip.compress(b'b'),10)
        snapshot=a.Snapshot()
        with patch.object(a,'FILE_LIMIT',4),patch.object(a,'TOTAL_LIMIT',6):
            snapshot.generated('a',b'1234')
            with self.assertRaises(ValueError):snapshot.generated('b',b'123')
        self.assertEqual(a.FILE_LIMIT,32*1024*1024);self.assertEqual(a.TOTAL_LIMIT,96*1024*1024)

    def test_shared_publish_executes_only_closed_selected_binary(self):
        with tempfile.TemporaryDirectory()as td:
            root=Path(td).resolve()
            for selected in ('bin/weft-paths','bin/weft-paths-keys'):
                snapshot=a.Snapshot()
                for name in ('bin/weft-paths','bin/weft-paths-keys','metadata.json'):snapshot.generated(name,b'inert bytes')
                output=root/Path(selected).name
                if selected=='bin/weft-paths':a.publish(snapshot,output)
                else:a.publish(snapshot,output,executable=selected)
                for name,raw in snapshot.artifacts.items():
                    self.assertEqual((output/name).read_bytes(),raw)
                    self.assertEqual((output/name).stat().st_mode&0o777,0o555 if name==selected else 0o444)
            with self.assertRaises(ValueError):a.publish(a.Snapshot(),root/'unknown',executable='arbitrary')
            self.assertFalse((root/'unknown').exists())

    def test_both_profiles_write_cancel_survives_close_failure(self):
        class Cancel(KeyboardInterrupt):
            def __setattr__(self,name,value):
                if name=='cleanup_failed':raise RuntimeError('marker refused')
                super().__setattr__(name,value)
        for selected in ('bin/weft-paths','bin/weft-paths-keys'):
            with tempfile.TemporaryDirectory()as td:
                root=Path(td).resolve();snapshot=a.Snapshot();snapshot.generated(selected,b'inert bytes');primary=Cancel()
                class Stream:
                    def write(self,raw):raise primary
                    def close(self):raise OSError('controlled close')
                original=Path.open
                def opening(path,*args,**kwargs):
                    if args==('xb',):return Stream()
                    return original(path,*args,**kwargs)
                with patch.object(Path,'open',opening):
                    with self.assertRaises(Cancel)as caught:a.publish(snapshot,root/'output',executable=selected)
                self.assertIs(caught.exception,primary);self.assertFalse((root/'output').exists())
                self.assertEqual(list(root.iterdir()),[])

    def test_new_binary_version_fence_and_capability_are_mandatory(self):
        self.receipt['cases'][0]['responseHex']=a.encoded(dict(interfaceVersion='weft-compile/0.4.0',status='blocked')).hex()
        self.bundle['namespaceFences'][0]['responseHex']=self.receipt['cases'][0]['responseHex']
        with self.assertRaises(ValueError):self.verify()
        self.setUp();self.backend['capabilities'][-1]['id']='different'
        with self.assertRaises(ValueError):self.verify()

if __name__=='__main__':unittest.main()
