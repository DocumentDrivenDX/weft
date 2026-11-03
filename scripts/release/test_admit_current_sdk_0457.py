"""Mocked GitHub-to-frozen-validator controls. No real API or native execution."""
from pathlib import Path
import base64
import copy
import hashlib
import io
import json
import subprocess
import runpy
import sys
from types import SimpleNamespace
import tempfile
import unittest
from unittest.mock import patch
import zipfile
import admit_current_sdk_0457 as admit
import test_verify_current_sdk_0457 as frozen_fixtures

R = 'a' * 40
TREE = 'b' * 40


def raw_json(value):
    return json.dumps(value).encode()


def blob(raw):
    return hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()


def zipped(files):
    stream = io.BytesIO()
    with zipfile.ZipFile(stream, 'w') as archive:
        for name, raw in files.items():
            archive.writestr(name, raw)
    return stream.getvalue()


class AdmitCurrentSDKTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(); self.addCleanup(self.tmp.cleanup)
        self.out = Path(self.tmp.name) / 'admission'
        self.documents, self.downloads, self.calls = {}, {}, []
        source = {
            admit.CONTROLLER: b'# explicitly synthetic source API fixture, never executed\n',
            'scripts/release/verify_current_sdk_0457.py': admit._VALIDATOR.read_bytes(),
            'Cargo.lock': b'# synthetic frozen lock\n',
            'tests/engine-catalog/temporal-native-495-corpus.json': b'{"synthetic":"corpus"}',
        }
        resources = []
        for index in range(9):
            name = 'crates/weft-core/resources/umf-temporal/resource-' + str(index) + '.json'
            source[name] = raw_json({'syntheticResource': index})
            resources.append({'path': name, 'sha256': admit.sha(source[name])})
        self.manifest = {
            'canonicalSourceID': f'git+https://github.com/DocumentDrivenDX/umf.git?rev={admit.S}#{admit.S}',
            'CargoLockSHA256': admit.sha(source['Cargo.lock']), 'resources': resources,
        }
        source[admit.INPUTS] = raw_json(self.manifest)
        for path in admit.WORKFLOWS.values(): source[path] = b'# synthetic workflow source\n'
        self.source = source
        self.code_hash = admit.sha(source[admit.CONTROLLER])
        patches = [patch.object(admit, 'LOCK', admit.sha(source['Cargo.lock'])),
                   patch.object(admit, 'CORPUS', admit.sha(source['tests/engine-catalog/temporal-native-495-corpus.json'])),
                   patch.object(admit, 'INPUTS_SHA', admit.sha(source[admit.INPUTS]))]
        for p in patches: p.start(); self.addCleanup(p.stop)
        self.endpoint('/git/ref/heads/main', {'ref': 'refs/heads/main', 'object': {'type': 'commit', 'sha': R}})
        self.endpoint('/git/commits/' + R, {'sha': R, 'tree': {'sha': TREE}})
        self.endpoint('/git/trees/' + TREE + '?recursive=1', {'sha': TREE, 'truncated': False,
                      'tree': [{'path': name, 'mode': '100644', 'type': 'blob', 'sha': blob(raw)} for name, raw in source.items()]})
        for name, raw in source.items():
            self.endpoint('/contents/' + name + '?ref=' + R, {'type': 'file', 'path': name, 'encoding': 'base64',
                          'content': base64.b64encode(raw).decode(), 'sha': blob(raw)})
        self.runs = {}
        for role, run_id in (('linux', 1), ('macos', 2)):
            run = {'id': run_id, 'repository': {'id': 42, 'full_name': admit.REPOSITORY},
                   'head_repository': {'id': 42, 'full_name': admit.REPOSITORY},
                   'head_sha': R, 'head_branch': 'main', 'head_commit': {'id': R, 'tree_id': TREE},
                   'event': 'workflow_dispatch', 'path': admit.WORKFLOWS[role], 'status': 'completed',
                   'conclusion': 'success', 'run_attempt': 1, 'workflow_id': run_id + 10}
            self.runs[role] = run
            self.endpoint('/actions/runs/' + str(run_id), run)
            self.endpoint('/actions/runs/' + str(run_id) + '/attempts/1', copy.deepcopy(run))
            self.endpoint('/actions/workflows/' + str(run_id + 10), {'id': run_id + 10, 'path': admit.WORKFLOWS[role]})
        self.files, self.artifacts, self.jobs = {}, {'linux': [], 'macos': []}, {'linux': [], 'macos': []}
        for index, (envelope, report, wheel, packaging) in enumerate(frozen_fixtures.fixtures()):
            target = envelope['target']; role = 'macos' if target == 'aarch64-apple-darwin' else 'linux'
            run_id = self.runs[role]['id']; artifact_id = index + 101
            runner = 'macos-14' if role == 'macos' else ('ubuntu-24.04-arm' if target.startswith('aarch64') else 'ubuntu-24.04')
            job = {'id': index + 201, 'name': 'macos-arm64' if role == 'macos' else f'wheels ({runner}, {target}, 2014)',
                   'run_id': run_id, 'head_sha': R, 'run_attempt': 1, 'status': 'completed', 'conclusion': 'success', 'labels': [runner],
                   'steps': [{'name': name, 'status': 'completed', 'conclusion': 'success',
                              'started_at': '2026-11-03T10:00:00Z', 'completed_at': '2026-11-03T10:01:00Z'} for name in [
                       'Current42 SDK and actual installed final-R correspondence',
                       'Emit current final-R Mac wheel and SDK evidence immediately' if role == 'macos' else 'Emit current final-R wheel and SDK evidence immediately']]}
            self.jobs[role].append(job)
            component = {
                'profile': 'weft-current-sdk-platform-observation/1', 'status': 'passed', 'sourceCommit': R, 'sourceTree': TREE,
                'workflowCommit': R, 'producerRunId': run_id, 'producerRunAttempt': '1',
                'producerJobName': 'macos-arm64' if role == 'macos' else 'wheels',
                'workflowRef': admit.REPOSITORY + '/' + admit.WORKFLOWS[role] + '@refs/heads/main',
                'target': target, 'distribution': 'umf-weft', 'version': '0.4.57', 'canonicalUmf': admit.S,
                'cargoLockSha256': admit.LOCK, 'controllerScript': admit.CONTROLLER, 'controllerCodeSha256': self.code_hash,
                'python': 'synthetic Python observation, never executed',
                'platform': 'macOS-synthetic-arm64' if role == 'macos' else 'Linux-synthetic-' + ('aarch64' if target.startswith('aarch64') else 'x86_64'),
                'nativeAdmission': False, 'publicationAdmission': False, 'temporalCases': 14, 'routingControls': 5, 'sdkControls': 42,
                'wheel': envelope['wheel'], 'wheelSha256': admit.sha(wheel), 'sdkReportSha256': admit.sha(report),
                'packagingReportSha256': admit.sha(packaging), 'nativeModuleSha256': envelope['nativeModuleSha256'], 'shimSha256': envelope['shimSha256'],
                'stageResults': [{'name': name, 'exitCode': 0} for name in ['venv', 'normal-install', 'pip-check', 'packaging', 'native-origin', 'temporal14', 'routing5', 'sdk42']],
            }
            files = {'component.json': raw_json(component), 'sdk-smoke.json': report, 'packaging.json': packaging, envelope['wheel']: wheel}
            self.files[target] = files
            artifact = {'id': artifact_id, 'created_at': '2026-11-03T10:00:30Z', 'name': f'current-sdk-{target}-{R}-{run_id}-attempt-1', 'expired': False,
                        'workflow_run': {'id': run_id, 'head_sha': R, 'head_branch': 'main', 'repository_id': 42, 'head_repository_id': 42}}
            self.artifacts[role].append(artifact)
            self.rezip(target)
        for role in self.runs:
            run_id = self.runs[role]['id']
            self.endpoint(f'/actions/runs/{run_id}/attempts/1/jobs?per_page=100&page=1', {'total_count': len(self.jobs[role]), 'jobs': self.jobs[role]})
            self.endpoint(f'/actions/runs/{run_id}/artifacts?per_page=100&page=1', {'total_count': len(self.artifacts[role]), 'artifacts': self.artifacts[role]})
        self.mock = patch.object(admit.subprocess, 'run', side_effect=self.transport)
        self.mock.start(); self.addCleanup(self.mock.stop)

    def endpoint(self, suffix, value): self.documents[admit.PREFIX + suffix] = value

    def artifact(self, target):
        return next(a for rows in self.artifacts.values() for a in rows if target in a['name'])

    def rezip(self, target):
        artifact = self.artifact(target)
        raw = zipped(self.files[target])
        self.downloads[admit.PREFIX + f'/actions/artifacts/{artifact["id"]}/zip'] = raw
        artifact['digest'] = 'sha256:' + admit.sha(raw)

    def component_change(self, **fields):
        target = next(iter(admit.TARGETS))
        component = json.loads(self.files[target]['component.json']); component.update(fields)
        self.files[target]['component.json'] = raw_json(component); self.rezip(target)

    def transport(self, argv, **kwargs):
        self.assertEqual(argv[:6], ['gh', 'api', '--hostname', 'github.com', '--method', 'GET'])
        self.assertEqual(len(argv), 7)
        path = argv[6]; self.calls.append(path)
        if path in self.downloads:
            kwargs['stdout'].write(self.downloads[path])
            return subprocess.CompletedProcess(argv, 0, stderr=b'')
        self.assertIn(path, self.documents, 'Unmocked API request (no real request permitted)')
        return subprocess.CompletedProcess(argv, 0, stdout=raw_json(self.documents[path]), stderr=b'')

    def admit(self): return admit.authenticate(R, 1, 2, self.out)

    def reject(self):
        with self.assertRaises((ValueError, KeyError, zipfile.BadZipFile)): self.admit()
        self.assertFalse((self.out / 'current-sdk-admission.json').exists())
        self.assertEqual(json.loads((self.out / 'admission-error.json').read_text())['status'], 'refused')

    def test_mocked_end_to_end_exact_three_by_42(self):
        result = self.admit()
        self.assertEqual(len(result['wheels']), 3)
        self.assertEqual(result['controllerCodeSha256'], self.code_hash)
        self.assertEqual({r['actualJob'] for r in result['actualGitHubObservations']}, {201, 202, 203})
        self.assertFalse(result['nativeAdmission']); self.assertFalse(result['publicationAdmission'])
        self.assertTrue((self.out / 'R-controller-code.source').exists())
        self.assertEqual(len(list(self.out.glob('*.zip'))), 3)

    def test_wrong_run_id(self): self.runs['linux']['id'] = 99; self.reject()
    def test_wrong_repository(self): self.runs['linux']['repository']['full_name'] = 'fork/weft'; self.reject()
    def test_wrong_head_source(self): self.runs['linux']['head_sha'] = 'c' * 40; self.reject()
    def test_wrong_run_tree(self): self.runs['linux']['head_commit']['tree_id'] = 'c' * 40; self.reject()
    def test_not_main_dispatch(self): self.runs['linux']['event'] = 'push'; self.reject()
    def test_incomplete_run(self): self.runs['linux']['status'] = 'in_progress'; self.reject()
    def test_wrong_job_head(self): self.jobs['linux'][0]['head_sha'] = 'c' * 40; self.reject()
    def test_wrong_artifact_head(self): self.artifacts['linux'][0]['workflow_run']['head_sha'] = 'c' * 40; self.reject()
    def test_wrong_component_tree(self): self.component_change(sourceTree='c' * 40); self.reject()
    def test_wrong_job_run(self): self.jobs['linux'][0]['run_id'] = 2; self.reject()
    def test_skipped_sdk_step(self): self.jobs['linux'][0]['steps'][0]['conclusion'] = 'skipped'; self.reject()
    def test_wrong_job_platform(self): self.jobs['linux'][0]['labels'] = ['macos-14']; self.reject()
    def test_wrong_workflow_api(self): self.documents[admit.PREFIX + '/actions/workflows/11']['path'] = '.github/workflows/other.yml'; self.reject()
    def test_wrong_artifact_run(self): self.artifacts['linux'][0]['workflow_run']['id'] = 2; self.reject()
    def test_expired_artifact(self): self.artifacts['linux'][0]['expired'] = True; self.reject()
    def test_missing_artifact_digest(self): del self.artifacts['linux'][0]['digest']; self.reject()
    def test_wrong_artifact_digest(self): self.artifacts['linux'][0]['digest'] = 'sha256:' + '0' * 64; self.reject()

    def test_tampered_download_zip_retained(self):
        path = next(iter(self.downloads)); self.downloads[path] += b'tampered'
        self.reject(); self.assertTrue(list(self.out.glob('*.zip')))

    def test_controller_contents_not_bound_to_R_tree(self):
        item = self.documents[admit.PREFIX + '/contents/' + admit.CONTROLLER + '?ref=' + R]
        raw = b'# different caller-chosen controller\n'
        item.update(content=base64.b64encode(raw).decode(), sha=blob(raw))
        self.reject()

    def test_controller_content_blob_mismatch(self):
        self.documents[admit.PREFIX + '/contents/' + admit.CONTROLLER + '?ref=' + R]['content'] = base64.b64encode(b'changed').decode()
        self.reject()

    def test_component_cannot_supply_expected_controller(self): self.component_change(controllerCodeSha256='d' * 64); self.reject()
    def test_component_wrong_source(self): self.component_change(sourceCommit='c' * 40); self.reject()
    def test_component_split_linux_run(self): self.component_change(producerRunId=2); self.reject()
    def test_component_wrong_attempt(self): self.component_change(producerRunAttempt='2'); self.reject()
    def test_component_wrong_S(self): self.component_change(canonicalUmf='c' * 40); self.reject()
    def test_component_wrong_lock(self): self.component_change(cargoLockSha256='0' * 64); self.reject()
    def test_component_wrong_platform(self): self.component_change(platform='Linux-aarch64'); self.reject()
    def test_component_wrong_report_byte_hash(self): self.component_change(sdkReportSha256='0' * 64); self.reject()

    def test_invalid_native_platform_reaches_unchanged_validator(self):
        target = next(iter(admit.TARGETS)); files = self.files[target]
        wheel_name = json.loads(files['component.json'])['wheel']
        with zipfile.ZipFile(io.BytesIO(files[wheel_name])) as archive: members = {n: archive.read(n) for n in archive.namelist()}
        native = bytearray(members['weft/weft.abi3.so']); native[18:20] = (183).to_bytes(2, 'little')
        members['weft/weft.abi3.so'] = bytes(native); files[wheel_name] = zipped(members)
        report = json.loads(files['sdk-smoke.json']); report['pinsBeforeAfter'].update(wheelSha256=admit.sha(files[wheel_name]), nativeModuleSha256=admit.sha(native))
        files['sdk-smoke.json'] = raw_json(report)
        packaging = json.loads(files['packaging.json']); packaging.update(wheelSha256=admit.sha(files[wheel_name]), nativeModuleSha256=admit.sha(native)); files['packaging.json'] = raw_json(packaging)
        self.component_change(wheelSha256=admit.sha(files[wheel_name]), nativeModuleSha256=admit.sha(native), sdkReportSha256=admit.sha(files['sdk-smoke.json']), packagingReportSha256=admit.sha(files['packaging.json']))
        with self.assertRaisesRegex(ValueError, 'native ELF architecture'): self.admit()
        self.assertFalse((self.out / 'current-sdk-admission.json').exists())

    def test_resource_contents_tampered(self):
        path = self.manifest['resources'][0]['path']; item = self.documents[admit.PREFIX + '/contents/' + path + '?ref=' + R]
        raw = b'changed'; item.update(content=base64.b64encode(raw).decode(), sha=blob(raw))
        tree = self.documents[admit.PREFIX + '/git/trees/' + TREE + '?recursive=1']['tree']
        next(row for row in tree if row['path'] == path)['sha'] = blob(raw)
        self.reject()

    def test_truncated_tree(self): self.documents[admit.PREFIX + '/git/trees/' + TREE + '?recursive=1']['truncated'] = True; self.reject()
    def test_incomplete_jobs_inventory(self): self.documents[admit.PREFIX + '/actions/runs/1/attempts/1/jobs?per_page=100&page=1']['total_count'] = 3; self.reject()

    def test_frozen_validator_loading_rejects_import_shadow(self):
        with patch.dict(sys.modules, {'verify_current_sdk_0457': SimpleNamespace(validate=lambda *a: {'forged': True})}):
            module = runpy.run_path(str(Path(admit.__file__)), run_name='pure_shadow_control')
        self.assertEqual(module['validate'].__code__.co_filename, str(admit._VALIDATOR))
        self.assertEqual(module['VALIDATOR_SHA'], admit.VALIDATOR_SHA)

    def test_real_pagination_path_is_walked_and_retained(self):
        first = admit.PREFIX + '/actions/runs/1/attempts/1/jobs?per_page=100&page=1'
        jobs = copy.deepcopy(self.jobs['linux'])
        jobs += [{'id': 1000 + n, 'name': 'unrelated-observation-' + str(n)} for n in range(99)]
        self.documents[first] = {'total_count': 101, 'jobs': jobs[:100]}
        self.documents[admit.PREFIX + '/actions/runs/1/attempts/1/jobs?per_page=100&page=2'] = {'total_count': 101, 'jobs': jobs[100:]}
        self.assertEqual(len(self.admit()['wheels']), 3)
        self.assertTrue((self.out / 'linux-attempt-1-jobs-2.json').exists())

    def test_duplicate_named_artifact_refuses(self):
        duplicate = copy.deepcopy(self.artifacts['linux'][0]); duplicate['id'] = 999
        self.artifacts['linux'].append(duplicate)
        self.documents[admit.PREFIX + '/actions/runs/1/artifacts?per_page=100&page=1']['total_count'] = 3
        self.reject()

    def test_unsafe_artifact_member_refuses_even_with_actual_digest(self):
        target = next(iter(admit.TARGETS)); self.files[target]['../unsafe'] = b'not extracted'
        self.rezip(target); self.reject()

    def resume_fixture(self):
        # Original x86 job passed while the ARM job failed; retry only ARM.
        self.runs['linux']['run_attempt'] = 2
        original = self.documents[admit.PREFIX + '/actions/runs/1/attempts/1']
        original['conclusion'] = 'failure'
        second = copy.deepcopy(self.runs['linux'])
        self.endpoint('/actions/runs/1/attempts/2', second)
        arm_job = copy.deepcopy(self.jobs['linux'][1])
        arm_job.update(id=302, run_attempt=2)
        for step in arm_job['steps']:
            step.update(started_at='2026-11-03T11:00:00Z', completed_at='2026-11-03T11:01:00Z')
        failed_arm = copy.deepcopy(self.jobs['linux'][1]); failed_arm['conclusion'] = 'failure'
        failed_arm['steps'][0]['conclusion'] = 'failure'
        self.documents[admit.PREFIX + '/actions/runs/1/attempts/1/jobs?per_page=100&page=1']['jobs'] = [self.jobs['linux'][0], failed_arm]
        self.endpoint('/actions/runs/1/attempts/2/jobs?per_page=100&page=1', {'total_count': 1, 'jobs': [arm_job]})
        target = 'aarch64-unknown-linux-gnu'
        component = json.loads(self.files[target]['component.json']); component['producerRunAttempt'] = '2'
        self.files[target]['component.json'] = raw_json(component)
        self.artifact(target)['created_at'] = '2026-11-03T11:00:30Z'
        self.artifact(target)['name'] = f'current-sdk-{target}-{R}-1-attempt-2'
        self.rezip(target)

    def test_unchanged_R_reuses_successful_original_attempt_without_rewriting(self):
        original_x86 = self.files['x86_64-unknown-linux-gnu']['component.json']
        original_sdk = self.files['x86_64-unknown-linux-gnu']['sdk-smoke.json']
        original_zip = self.downloads[admit.PREFIX + '/actions/artifacts/101/zip']
        self.resume_fixture()
        result = self.admit()
        observed = {row['target']: row for row in result['actualGitHubObservations']}
        first = observed['x86_64-unknown-linux-gnu']
        self.assertEqual((first['runAttempt'], first['actualJobAttempt'], first['finalRunAttempt']), (1, 1, 2))
        self.assertEqual(first['producerAttemptConclusion'], 'failure')
        self.assertEqual(first['finalRunConclusion'], 'success')
        self.assertEqual(first['actualJob'], 201)
        self.assertEqual(observed['aarch64-unknown-linux-gnu']['runAttempt'], 2)
        self.assertEqual(self.files['x86_64-unknown-linux-gnu']['component.json'], original_x86)
        self.assertEqual(self.files['x86_64-unknown-linux-gnu']['sdk-smoke.json'], original_sdk)
        self.assertEqual((self.out / '101.zip').read_bytes(), original_zip)
        self.assertTrue((self.out / 'linux-attempt-1-jobs-1.json').exists())
        self.assertTrue((self.out / 'linux-attempt-2-jobs-1.json').exists())

    def test_resume_wrong_claimed_attempt_refuses(self):
        self.resume_fixture()
        self.component_change(producerRunAttempt='2')
        with self.assertRaisesRegex(ValueError, 'claimed attempt differs'): self.admit()

    def test_old_artifact_cannot_claim_new_successful_job_attempt(self):
        self.resume_fixture(); self.component_change(producerRunAttempt='2')
        copied_job = copy.deepcopy(self.jobs['linux'][0]); copied_job.update(id=301, run_attempt=2)
        for step in copied_job['steps']:
            step.update(started_at='2026-11-03T11:00:00Z', completed_at='2026-11-03T11:01:00Z')
        page = self.documents[admit.PREFIX + '/actions/runs/1/attempts/2/jobs?per_page=100&page=1']
        page['jobs'].append(copied_job); page['total_count'] = 2
        with self.assertRaisesRegex(ValueError, 'claimed attempt differs'): self.admit()

    def test_old_artifact_cannot_be_renamed_to_a_new_attempt(self):
        self.resume_fixture(); self.component_change(producerRunAttempt='2')
        first = self.artifacts['linux'][0]; first['name'] = f'current-sdk-x86_64-unknown-linux-gnu-{R}-1-attempt-2'
        copied_job = copy.deepcopy(self.jobs['linux'][0]); copied_job.update(id=301, run_attempt=2)
        for step in copied_job['steps']:
            step.update(started_at='2026-11-03T11:00:00Z', completed_at='2026-11-03T11:01:00Z')
        page = self.documents[admit.PREFIX + '/actions/runs/1/attempts/2/jobs?per_page=100&page=1']
        page['jobs'].append(copied_job); page['total_count'] = 2
        with self.assertRaisesRegex(ValueError, 'claimed job/attempt upload window'): self.admit()

    def test_non_attempt_artifact_name_is_not_admitted(self):
        self.artifacts['linux'][0]['name'] = f'current-sdk-x86_64-unknown-linux-gnu-{R}-1'
        self.reject()

    def test_resume_changed_controller_cannot_reuse_old_cohort(self):
        self.resume_fixture(); self.component_change(controllerCodeSha256='e' * 64)
        self.reject()

    def test_resume_changed_R_attempt_cannot_reuse_old_cohort(self):
        self.resume_fixture()
        self.documents[admit.PREFIX + '/actions/runs/1/attempts/1']['head_sha'] = 'c' * 40
        self.reject()

    def test_failed_prior_attempt_artifact_does_not_poison_success_reuse(self):
        self.resume_fixture()
        target = 'aarch64-unknown-linux-gnu'
        failed = copy.deepcopy(self.artifact(target))
        failed.update(id=998, name=f'current-sdk-{target}-{R}-1-attempt-1', expired=True,
                      created_at='2026-11-03T10:00:30Z')
        self.artifacts['linux'].append(failed)
        self.documents[admit.PREFIX + '/actions/runs/1/artifacts?per_page=100&page=1']['total_count'] = 3
        result = self.admit()
        self.assertEqual(result['ignoredFailedArtifacts'][0]['artifactID'], 998)
        self.assertEqual(result['ignoredFailedArtifacts'][0]['conclusion'], 'failure')
        self.assertFalse((self.out / '998.zip').exists())
        self.assertEqual(len(result['wheels']), 3)

    def test_artifact_creation_must_match_original_job_attempt(self):
        self.artifacts['linux'][0]['created_at'] = '2026-11-03T12:00:00Z'
        self.reject()

    def test_main_moves_during_admission(self):
        original = self.transport
        def moving(argv, **kwargs):
            if argv[6] == admit.PREFIX + '/git/ref/heads/main' and argv[6] in self.calls:
                self.documents[argv[6]]['object']['sha'] = 'c' * 40
            return original(argv, **kwargs)
        with patch.object(admit.subprocess, 'run', side_effect=moving): self.reject()
        self.assertEqual(len(list(self.out.glob('*.zip'))), 3)

    def test_producer_attempt_changes_during_admission(self):
        original = self.transport
        def moving(argv, **kwargs):
            if argv[6] == admit.PREFIX + '/actions/runs/1' and argv[6] in self.calls:
                self.runs['linux']['run_attempt'] = 2
            return original(argv, **kwargs)
        with patch.object(admit.subprocess, 'run', side_effect=moving): self.reject()

    def test_frozen_validator_at_R_cannot_be_replaced(self):
        path = 'scripts/release/verify_current_sdk_0457.py'
        item = self.documents[admit.PREFIX + '/contents/' + path + '?ref=' + R]
        raw = b'# replacement validator, never executed'
        item.update(content=base64.b64encode(raw).decode(), sha=blob(raw))
        next(row for row in self.documents[admit.PREFIX + '/git/trees/' + TREE + '?recursive=1']['tree'] if row['path'] == path)['sha'] = blob(raw)
        self.reject()

    def test_failed_download_partial_bytes_retained(self):
        original = self.transport
        def failing(argv, **kwargs):
            if argv[6] in self.downloads:
                kwargs['stdout'].write(b'partial ZIP')
                return subprocess.CompletedProcess(argv, 1, stderr=b'mocked download failure')
            return original(argv, **kwargs)
        with patch.object(admit.subprocess, 'run', side_effect=failing): self.reject()
        self.assertIn(b'partial ZIP', [p.read_bytes() for p in self.out.glob('*.zip')])

    def test_failed_api_snapshot_is_retained(self):
        def failed(argv, **kwargs): return subprocess.CompletedProcess(argv, 1, stdout=b'partial API response', stderr=b'mocked failure')
        with patch.object(admit.subprocess, 'run', side_effect=failed): self.reject()
        self.assertEqual((self.out / 'main-ref.json').read_bytes(), b'partial API response')
        self.assertEqual((self.out / 'main-ref.stderr').read_bytes(), b'mocked failure')


if __name__ == '__main__': unittest.main()
