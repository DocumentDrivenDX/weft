"""Independent R/Q/A policy controls; all identities/ZIPs/APIs here are MOCKED.

No fixture is producer evidence. No Cargo, native module, browser or network is
executed. Source corpus and frozen42 roster are read-only independent oracles.
"""
import copy
import base64
import hashlib
import io
import json
from pathlib import Path
import sys
import unittest
import zipfile

HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path.insert(0, str(HERE))
import current_macos_pair_0457 as policy
import admit_controller_pairs_0457 as admission

R = 'c623b94b66b0b46d492d457c62faddc896e2e452'
RTREE = 'cc552d691c92c7abdd44fdf4e4ca585936ae68c5'
S = 'e3f38ef6f05ae636d2837f9d69f8d6805222664b'
LOCK = '92974ab7df6beb8033fba4d3b535fee1f694229e0a6918fbd3d059d65162121d'
MAC, ARM = 'aarch64-apple-darwin', 'aarch64-unknown-linux-gnu'
IDS = ['original-sales-join-truss.postgresql', 'original-empty-global-truss.postgresql',
       'original-empty-grouped-truss.postgresql', 'original-sales-join-ashlar.databricks',
       'original-empty-global-ashlar.databricks', 'original-empty-grouped-ashlar.databricks',
       'original-string-exact-0', 'original-string-exact-1', 'original-string-exact-2']
CONTROLLERS = {
    'scripts/release/current_macos_pair_0457.py', 'scripts/release/run_macos_pair_0457.py',
    'scripts/release/admit_controller_pairs_0457.py', 'scripts/release/publish_macos_pair_0457.py',
    'scripts/release/test_macos_pair_0457.py', 'scripts/release/current_publication_0457.py'}


def raw(value):
    return json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode()


def sha(value):
    return hashlib.sha256(value).hexdigest()


def blob(value):
    return hashlib.sha1(b'blob ' + str(len(value)).encode() + b'\0' + value).hexdigest()


def zipped(files):
    out = io.BytesIO()
    with zipfile.ZipFile(out, 'w', zipfile.ZIP_STORED) as z:
        for name, value in sorted(files.items()):
            info = zipfile.ZipInfo(name, (2026, 1, 1, 0, 0, 0))
            info.external_attr = 0o100644 << 16
            z.writestr(info, value)
    return out.getvalue()


def registration(target=MAC):
    """Synthetic complete unchanged-production tree and exact control delta."""
    arm = target == ARM
    workflow = '.github/workflows/release-arm64-pair-0457.yml' if arm else '.github/workflows/release-macos-pair-0457.yml'
    controllers = CONTROLLERS | ({'scripts/release/run_arm64_pair_0457.py'} if arm else set())
    code = {name: ('# PURE MOCK CONTROL ' + name + '\n').encode() for name in controllers}
    code[workflow] = b'# PURE MOCK WORKFLOW; never dispatched\n'
    production = ['Cargo.lock', 'Cargo.toml', 'pyproject.toml', 'crates/weft-core/src/lib.rs',
                  'crates/weft-python/build.rs', 'crates/weft-python/Cargo.toml',
                  'crates/weft-core/resources/policy.json', 'scripts/release/verify_current_sdk_0457.py']
    before = [dict(path=n, mode='100644', type='blob', sha=blob(('original R '+n).encode())) for n in production]
    after = copy.deepcopy(before) + [dict(path=n, mode='100644', type='blob', sha=blob(b)) for n, b in code.items()]
    changes = [dict(path=n, before=None, after=dict(mode='100644', sha=blob(b))) for n, b in sorted(code.items())]
    record = dict(profile='weft-current-arm64-controller-pair/1' if arm else 'weft-current-macos-controller-pair/1',
                  status='reviewed', sourceCommit=R, sourceTree=RTREE,
                  controllerCommit=('4' if arm else '2')*40, controllerTree=('5' if arm else '3')*40,
                  workflowPath=workflow, workflowSha256=sha(code[workflow]),
                  controllerHashes={n: sha(code[n]) for n in sorted(controllers)}, changedPaths=changes,
                  controllerRef='codex/weft-0457-arm64-controller' if arm else 'codex/weft-0457-macos-controller')
    return record, dict(sha=RTREE, truncated=False, tree=before), dict(sha=record['controllerTree'], truncated=False, tree=after), code


def payload(target=MAC):
    """Tiny header-only native wheel and synthetic SDK reports; NEVER loaded."""
    arm = target == ARM
    tag = 'manylinux_2_17_aarch64.manylinux2014_aarch64' if arm else 'macosx_11_0_arm64'
    name = 'umf_weft-0.4.57-cp39-abi3-' + tag + '.whl'
    native = bytearray(32)
    if arm:
        native[:6] = b'\x7fELF\x02\x01'; native[18:20] = (183).to_bytes(2, 'little')
    else:
        native[:4] = b'\xcf\xfa\xed\xfe'; native[4:8] = (0x100000c).to_bytes(4, 'little')
    shim = b'# PURE MOCK shim, not importable SDK evidence\n'
    wheel = zipped({'weft/_native.abi3.so': bytes(native), 'weft/__init__.py': shim})
    checks = json.loads((HERE/'fixtures/current-sdk-0457/actual42-checks.json').read_bytes())
    pins = dict(wheelSha256=sha(wheel), nativeModuleSha256=sha(native), shimSha256=sha(shim))
    sdk = dict(version='0.4.57', sourceCommit=R, wheel=name, pinsBeforeAfter=pins,
               controls=42, checks=checks, nativeAdmission=False, securityReady=False, temporalQualification='held')
    packaging = dict(status='passed', version='0.4.57', wheel=name, wheelSha256=pins['wheelSha256'],
                     nativeModuleSha256=pins['nativeModuleSha256'], python='PURE MOCK CPython',
                     scope='exact installed archive, RECORD, metadata, ABI tag and notices; native semantics separately gated')
    return {name: wheel, 'sdk-smoke.json': raw(sdk), 'packaging.json': raw(packaging)}


def parity(files):
    source = json.loads((ROOT/'tests/compile/fixtures/cases.json').read_bytes())
    by_id = {c['id']: c for c in source}
    cases = [copy.deepcopy(by_id[n]) for n in IDS]
    rows = []
    for c in cases:
        code = 'WFT-BACKEND-VERSION' if c['id'].endswith('ashlar.databricks') else 'WFT-BACKEND-MISSING'
        rows.append(dict(id=c['id'], raw=raw(dict(status='blocked', diagnostics=[dict(code=code)])).decode()))
    files['compiler-cases.json'] = raw(cases)
    files['compiler-roster.json'] = raw(dict(profile='nine-frozen-refusal-responses/1', cases=IDS,
                                            selectedCasesSha256=sha(files['compiler-cases.json'])))
    files['compiler-reports.json'] = raw(rows)
    sdk = json.loads(files['sdk-smoke.json'])
    files['python-compiler-parity.json'] = raw(dict(status='passed', distribution='umf-weft',
        distributionVersion='0.4.57', nativeModuleSha256=sdk['pinsBeforeAfter']['nativeModuleSha256'],
        cases=[dict(id=c['id'], status='passed') for c in cases]))
    summary = dict(byteParity=True, cases=9)
    files['browser-compiler-parity.json'] = raw(summary)
    files['browser-compiler-receipts.json'] = raw(dict(summary=summary, cases=[dict(id=c['id'],
        requestSha256=sha(raw(c['request'])), actualSha256=sha(r['raw'].encode()),
        expectedSha256=sha(r['raw'].encode())) for c, r in zip(cases, rows)]))
    return dict(profile='nine-frozen-refusal-responses-rust-python-chromium/1',
                cases=IDS, rust='passed', installedPython='passed', chromiumWasm='passed')


def component(target=MAC, attempt=1):
    p, a, b, code = registration(target)
    files = payload(target)
    if target == MAC:
        sdk = json.loads(files['sdk-smoke.json'])
        files['packaging-macos-arm64.json'] = raw(dict(status='passed', version='0.4.57',
            distribution='umf-weft', sourceCommit=R, target=target, wheel=sdk['wheel'],
            wheelSha256=sdk['pinsBeforeAfter']['wheelSha256'],
            nativeModuleSha256=sdk['pinsBeforeAfter']['nativeModuleSha256']))
    par = parity(files) if target == MAC else {'scope':'original ARM platform has no separate nine-refusal Mac parity; shared raw temporal Node/Chromium proof remains separately required'}
    sdk = json.loads(files['sdk-smoke.json']); name = sdk['wheel']
    c = dict(profile=p['profile'], status='passed', sourceCommit=R, sourceTree=RTREE,
             controllerCommit=p['controllerCommit'], workflowCommit=p['controllerCommit'],
             controllerTree=p['controllerTree'], producerRunId=901, producerRunAttempt=attempt,
             producerJobName='macos-pair' if target == MAC else 'arm64-pair', target=target,
             nativeAdmission=False, publicationAdmission=False, cargoLockSha256=LOCK, canonicalUmf=S,
             sdkStages=[dict(name=n, exitCode=0) for n in
                ['venv','normal-install','pip-check','packaging','native-origin','temporal14','routing5','sdk42']],
             wheel=name, **sdk['pinsBeforeAfter'], sdkReportSha256=sha(files['sdk-smoke.json']),
             packagingReportSha256=sha(files['packaging.json']), parity=par)
    return c, files, p


def actual_run(p):
    return dict(id=901, repository=dict(id=31, full_name='DocumentDrivenDX/weft'),
        head_repository=dict(id=31), head_sha=p['controllerCommit'],
        head_commit=dict(id=p['controllerCommit'], tree_id=p['controllerTree']),
        head_branch=p['controllerRef'], event='workflow_dispatch', path=p['workflowPath'],
        status='completed', conclusion='success', workflow_id=88, run_attempt=2)


class FakeGitHub:
    """Closed route map: any unexpected endpoint fails, never falls back to API."""
    def __init__(self, target=MAC, attempt=1):
        self.p, before, after, code = registration(target)
        c, files, _ = component(target, attempt)
        files['component.json'] = raw(c)
        self.zip = zipped(files)
        self.run = actual_run(self.p)
        self.original = dict(self.run, run_attempt=attempt, conclusion='failure')
        # Overall attempt1 may have failed elsewhere. This platform job succeeded.
        self.job = dict(id=601, name='macos-pair' if target == MAC else 'arm64-pair',
            head_sha=self.p['controllerCommit'], run_id=901, run_attempt=attempt,
            status='completed', conclusion='success', labels=['macos-14' if target == MAC else 'ubuntu-24.04-arm'],
            steps=[dict(name='Retain exact pair qualification and partials', conclusion='success',
                        started_at='2026-11-03T12:00:00Z', completed_at='2026-11-03T12:01:00Z')])
        self.artifact = dict(id=701, name=f'weft-pair-{target}-{R}-{self.p["controllerCommit"]}-901-attempt-{attempt}',
            created_at='2026-11-03T12:00:30Z', expired=False, digest='sha256:'+sha(self.zip),
            workflow_run=dict(id=901, head_sha=self.p['controllerCommit']))
        prefix = 'repos/DocumentDrivenDX/weft'
        self.routes = {
            prefix+'/git/trees/'+RTREE+'?recursive=1':before,
            prefix+'/git/commits/'+self.p['controllerCommit']:dict(sha=self.p['controllerCommit'], tree=dict(sha=self.p['controllerTree'])),
            prefix+'/git/trees/'+self.p['controllerTree']+'?recursive=1':after,
            prefix+'/actions/runs/901':self.run,
            prefix+'/actions/runs/901/artifacts?per_page=100&page=1':dict(total_count=1, artifacts=[self.artifact]),
            prefix+f'/actions/runs/901/attempts/{attempt}':self.original,
            prefix+f'/actions/runs/901/attempts/{attempt}/jobs?per_page=100&page=1':dict(total_count=1, jobs=[self.job])}
        for n, value in code.items():
            self.routes[prefix+'/contents/'+n+'?ref='+self.p['controllerCommit']] = dict(
                path=n, encoding='base64', type='file', content=base64.b64encode(value).decode(), sha=blob(value))
        self.calls = []

    def json(self, path, label):
        self.calls.append(path)
        if path not in self.routes:
            raise AssertionError('Unexpected mock API route: '+path)
        return copy.deepcopy(self.routes[path])

    def download(self, path, filename):
        if path != 'repos/DocumentDrivenDX/weft/actions/artifacts/701/zip':
            raise AssertionError('Unexpected mock download: '+path)
        return self.zip


class PairPolicyControls(unittest.TestCase):
    def test_fixed_source_and_frozen_validator_are_not_relabelled(self):
        self.assertEqual(policy.R, R); self.assertEqual(policy.RTREE, RTREE)
        self.assertEqual(sha((HERE/'verify_current_sdk_0457.py').read_bytes()),
            '0479852207b043ae2d8ad63ca337a8abb67633cff8cc9b051e1b856f278d9b80')
        self.assertEqual(sha((HERE/'fixtures/current-sdk-0457/actual42-checks.json').read_bytes()),
            'd95f2253c651bd409e5f02aeaa77eedda14b1c1d1a457cd5d0e3fe0a4e36f090')
        self.assertEqual(sha((ROOT/'tests/compile/fixtures/cases.json').read_bytes()),
            'd0fe7b7592ab21d105cbf4e2a3f68e22e97a77e507c70395daa35dd899dd7851')

    def test_truthful_mac_and_separate_arm_registration(self):
        for target in [MAC, ARM]:
            p, a, b, _ = registration(target)
            self.assertEqual(policy.selected_pair(p,target),p)
            self.assertTrue(policy.closed_closure(p,a,b,target)['productionInputsUnchanged'])
        for target, wrong in [(MAC, ARM),(ARM,MAC)]:
            with self.assertRaises(ValueError):policy.selected_pair(registration(wrong)[0],target)
        for target in ['x86_64-unknown-linux-gnu','wasm32-unknown-unknown','other','',None]:
            with self.assertRaises(ValueError):policy.selected_pair(registration()[0],target)

    def test_registration_rejects_collapsed_wrong_and_old_identities(self):
        for target in [MAC, ARM]:
            for key,value in [('sourceCommit','f'*40),('sourceTree','f'*40),('controllerCommit',R),
                              ('controllerTree',RTREE),('workflowPath','.github/workflows/release.yml'),
                              ('controllerRef','main'),('profile','weft-current-sdk-platform-observation/1'),
                              ('status','draft')]:
                with self.subTest(target=target,key=key):
                    p=registration(target)[0];p[key]=value
                    with self.assertRaises(ValueError):policy.selected_pair(p,target)

    def test_complete_closure_rejects_production_bytes_modes_and_unlisted_controls(self):
        for target in [MAC,ARM]:
            for path in ['Cargo.lock','Cargo.toml','pyproject.toml','crates/weft-core/src/lib.rs',
                         'crates/weft-python/Cargo.toml','crates/weft-python/build.rs',
                         'crates/weft-core/resources/policy.json','scripts/release/unreviewed.py']:
                with self.subTest(target=target,path=path):
                    p,a,b,_=registration(target)
                    row=next((x for x in b['tree'] if x['path']==path),None)
                    if row:row['sha']='f'*40
                    else:b['tree'].append(dict(path=path,type='blob',mode='100644',sha='f'*40))
                    with self.assertRaises(ValueError):policy.closed_closure(p,a,b,target)
            for mode,kind in [('100755','blob'),('120000','blob'),('160000','commit')]:
                p,a,b,_=registration(target);b['tree'][0].update(mode=mode,type=kind)
                with self.assertRaises(ValueError):policy.closed_closure(p,a,b,target)

    def test_tree_truncation_duplicates_and_paths_refuse(self):
        for mutation in ['truncated','duplicate','traversal','absolute','wrong-tree']:
            with self.subTest(mutation=mutation):
                p,a,b,_=registration()
                if mutation=='truncated':b['truncated']=True
                elif mutation=='duplicate':b['tree'].append(copy.deepcopy(b['tree'][0]))
                elif mutation=='traversal':b['tree'][0]['path']='a/../Cargo.lock'
                elif mutation=='absolute':b['tree'][0]['path']='/Cargo.lock'
                else:b['sha']='f'*40
                with self.assertRaises(ValueError):policy.closed_closure(p,a,b,MAC)

    def test_git_entry_type_cannot_be_erased_by_closure_comparison(self):
        p,a,b,_=registration();b['tree'][0]['type']='commit'
        with self.assertRaises(ValueError):policy.closed_closure(p,a,b,MAC)

    def test_malformed_unchanged_tree_entry_is_not_authoritative(self):
        p,a,b,_=registration()
        for tree in [a,b]:tree['tree'][0]['mode']='100999'
        with self.assertRaises(ValueError):policy.closed_closure(p,a,b,MAC)

    def test_producer_actual_head_tree_ref_workflow_repository_must_match(self):
        for target in [MAC,ARM]:
            p=registration(target)[0]
            admission.producer_identity(actual_run(p),901,p)
            for field,value in [('head_sha',R),('head_branch','main'),('path','.github/workflows/release.yml'),
                                ('event','push'),('status','in_progress'),('conclusion','failure'),('id',902)]:
                with self.subTest(target=target,field=field):
                    run=actual_run(p);run[field]=value
                    with self.assertRaises(ValueError):admission.producer_identity(run,901,p)
            for nested,key,value in [('head_commit','tree_id',RTREE),('head_commit','id',R),
                                     ('repository','full_name','attacker/weft'),('head_repository','id',99)]:
                run=actual_run(p);run[nested][key]=value
                with self.assertRaises(ValueError):admission.producer_identity(run,901,p)


class PayloadControls(unittest.TestCase):
    def test_complete_synthetic_mac_and_arm_components(self):
        for target in [MAC,ARM]:
            c,files,p=component(target)
            proof=policy.validate_component(c,files,p,target,901,1)
            self.assertEqual(proof['controllerCommit'],p['controllerCommit'])
            self.assertFalse(proof['nativeAdmission']);self.assertFalse(proof['publicationAdmission'])

    def test_sdk42_missing_order_status_and_authority_refuse(self):
        for mutation in ['missing','reverse','wrong-status','count','native','security','temporal']:
            with self.subTest(mutation=mutation):
                files=payload();sdk=json.loads(files['sdk-smoke.json'])
                if mutation=='missing':sdk['checks'].pop()
                elif mutation=='reverse':sdk['checks'].reverse()
                elif mutation=='wrong-status':sdk['checks'][0]['status']='blocked'
                elif mutation=='count':sdk['controls']=41
                elif mutation=='native':sdk['nativeAdmission']=True
                elif mutation=='security':sdk['securityReady']=True
                else:sdk['temporalQualification']='passed'
                files['sdk-smoke.json']=raw(sdk)
                with self.assertRaises(ValueError):policy.validate_payload(MAC,files)

    def test_wheel_native_shim_and_report_binding_refuse(self):
        for field in ['wheelSha256','nativeModuleSha256','shimSha256','sdkReportSha256','packagingReportSha256']:
            c,files,p=component();c[field]='f'*64
            with self.subTest(field=field),self.assertRaises(ValueError):policy.validate_component(c,files,p,MAC,901,1)
        for target in [MAC,ARM]:
            files=payload(target);name=next(n for n in files if n.endswith('.whl'))
            with zipfile.ZipFile(io.BytesIO(files[name])) as z:items={n:z.read(n) for n in z.namelist()}
            for member in ['weft/_native.abi3.so','weft/__init__.py']:
                changed=dict(items);changed[member]+=b'tampered';bad=dict(files);bad[name]=zipped(changed)
                with self.subTest(target=target,member=member),self.assertRaises(ValueError):policy.validate_payload(target,bad)

    def test_preflight_old_receipt_wrong_attempt_and_promoted_scope_refuse(self):
        for key,value in [('status','preflight-passed'),('profile','weft-current-sdk-platform-observation/1'),
                          ('controllerCommit',R),('workflowCommit',R),('controllerTree',RTREE),
                          ('producerRunAttempt',2),('producerRunId',902),('producerJobName','wheels'),
                          ('cargoLockSha256','f'*64),('canonicalUmf','f'*40),
                          ('nativeAdmission',True),('publicationAdmission',True),('target',ARM)]:
            c,files,p=component();c[key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):policy.validate_component(c,files,p,MAC,901,1)
        c,files,p=component();c['sdkStages']=c['sdkStages'][:1]
        with self.assertRaises(ValueError):policy.validate_component(c,files,p,MAC,901,1)

    def test_missing_sdk_or_parity_material_refuses_without_success(self):
        for name in ['sdk-smoke.json','packaging.json','compiler-cases.json','compiler-roster.json',
                     'compiler-reports.json','python-compiler-parity.json','browser-compiler-parity.json',
                     'browser-compiler-receipts.json']:
            c,files,p=component();del files[name]
            # Missing material may propagate original lookup failure, never success.
            with self.subTest(name=name),self.assertRaises((ValueError,KeyError)):
                policy.validate_component(c,files,p,MAC,901,1)

    def test_all9_ordered_refusals_and_request_response_byte_receipts(self):
        for mutation in ['missing','order','non-refusal','request','response','missing-receipt','receipt-id','python-missing','browser-failed']:
            c,files,p=component()
            if mutation in ['missing','order','non-refusal']:
                rows=json.loads(files['compiler-reports.json'])
                if mutation=='missing':rows.pop()
                elif mutation=='order':rows.reverse()
                else:rows[0]['raw']=raw(dict(status='compiled',sql='SELECT 1')).decode()
                files['compiler-reports.json']=raw(rows)
            elif mutation in ['request','response','missing-receipt','receipt-id']:
                r=json.loads(files['browser-compiler-receipts.json'])
                if mutation=='missing-receipt':r['cases'].pop()
                elif mutation=='receipt-id':r['cases'][0]['id']='other'
                else:r['cases'][0]['requestSha256' if mutation=='request' else 'actualSha256']='f'*64
                files['browser-compiler-receipts.json']=raw(r)
            elif mutation=='python-missing':
                r=json.loads(files['python-compiler-parity.json']);r['cases'].pop();files['python-compiler-parity.json']=raw(r)
            else:
                r=json.loads(files['browser-compiler-parity.json']);r['byteParity']=False;files['browser-compiler-parity.json']=raw(r)
            with self.subTest(mutation=mutation),self.assertRaises(ValueError):policy.validate_component(c,files,p,MAC,901,1)

    def test_old_receipt_cannot_be_relabelled_under_altered_request(self):
        c,files,p=component();cases=json.loads(files['compiler-cases.json'])
        cases[0]['request']['sql']='SELECT changed_input_without_new_execution'
        files['compiler-cases.json']=raw(cases)
        roster=json.loads(files['compiler-roster.json']);roster['selectedCasesSha256']=sha(files['compiler-cases.json']);files['compiler-roster.json']=raw(roster)
        receipts=json.loads(files['browser-compiler-receipts.json']);receipts['cases'][0]['requestSha256']=sha(raw(cases[0]['request']));files['browser-compiler-receipts.json']=raw(receipts)
        with self.assertRaises(ValueError):policy.validate_component(c,files,p,MAC,901,1)


class ActualProvenanceMockControls(unittest.TestCase):
    def authenticate(self,c,target=MAC):
        return admission.authenticate_platform(c,c.p,target,901,Path('/unused-pure-output'))

    def test_original_attempt1_success_retained_under_final_attempt2(self):
        for target in [MAC,ARM]:
            c=FakeGitHub(target)
            before=copy.deepcopy(c.routes)
            proof,files,observation=self.authenticate(c,target)
            self.assertEqual(observation['originalAttempt'],1)
            self.assertEqual(proof['producerRunAttempt'],1)
            self.assertEqual(observation['actualJob'],601)
            self.assertEqual(c.original['conclusion'],'failure')
            self.assertEqual(c.run['run_attempt'],2)
            self.assertEqual(c.routes,before)  # no rewriting old attempt receipts
            self.assertIn('/attempts/1/jobs?per_page=100&page=1',' '.join(c.calls))

    def test_wrong_original_job_head_attempt_runner_and_failure_refuse(self):
        for key,value in [('head_sha',R),('run_id',902),('run_attempt',2),('conclusion','failure'),
                          ('status','in_progress'),('labels',['ubuntu-latest'])]:
            c=FakeGitHub();c.job[key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):self.authenticate(c)

    def test_wrong_attempt_api_or_claimed_attempt_refuse(self):
        c=FakeGitHub();c.original['run_attempt']=2
        with self.assertRaises(ValueError):self.authenticate(c)
        c=FakeGitHub();files=admission.archive_files(c.zip);r=json.loads(files['component.json']);r['producerRunAttempt']=2;files['component.json']=raw(r)
        c.zip=zipped(files);c.artifact['digest']='sha256:'+sha(c.zip)
        with self.assertRaises(ValueError):self.authenticate(c)

    def test_upload_window_artifact_digest_tamper_and_expiry_refuse(self):
        for key,value in [('created_at','2026-11-03T11:59:59Z'),('digest','sha256:'+'f'*64),('expired',True)]:
            c=FakeGitHub();c.artifact[key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):self.authenticate(c)
        c=FakeGitHub();c.zip+=b'tamperedZIPbytes'
        with self.assertRaises(ValueError):self.authenticate(c)

    def test_wrong_Q_A_code_blob_or_actual_commit_tree_refuse(self):
        for target in [MAC,ARM]:
            for changed in ['commit','tree','code']:
                c=FakeGitHub(target)
                if changed=='commit':c.routes[admission.PREFIX+'/git/commits/'+c.p['controllerCommit']]['sha']='f'*40
                elif changed=='tree':c.routes[admission.PREFIX+'/git/commits/'+c.p['controllerCommit']]['tree']['sha']='f'*40
                else:
                    path=next(n for n in c.routes if '/contents/' in n)
                    c.routes[path]['content']=base64.b64encode(b'changed controller').decode()
                with self.subTest(target=target,changed=changed),self.assertRaises(ValueError):self.authenticate(c,target)

    def test_preflight_only_and_old_component_zip_never_enrolled(self):
        for mutation in ['preflight','old-profile','missing-sdk','missing-parity']:
            c=FakeGitHub();files=admission.archive_files(c.zip)
            if mutation=='missing-sdk':del files['sdk-smoke.json']
            elif mutation=='missing-parity':del files['browser-compiler-receipts.json']
            else:
                value=json.loads(files['component.json'])
                value['status' if mutation=='preflight' else 'profile']='preflight-passed' if mutation=='preflight' else 'weft-current-sdk-platform-observation/1'
                files['component.json']=raw(value)
            c.zip=zipped(files);c.artifact['digest']='sha256:'+sha(c.zip)
            with self.subTest(mutation=mutation),self.assertRaises((ValueError,KeyError)):self.authenticate(c)


if __name__ == '__main__':
    unittest.main()
