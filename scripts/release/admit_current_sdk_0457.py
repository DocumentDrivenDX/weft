"""Authenticate GitHub observations before the frozen current3/42 validator.

Read-only fixed-repository GETs/downloads only. This is not native or publication
admission. Final R is supplied only after freeze; no staging SHA is substituted.
"""
from pathlib import Path
import argparse
from datetime import datetime, timedelta
import base64
import hashlib
import io
import json
import re
import stat
import subprocess
import zipfile

VALIDATOR_SHA = '0479852207b043ae2d8ad63ca337a8abb67633cff8cc9b051e1b856f278d9b80'
_VALIDATOR = Path(__file__).with_name('verify_current_sdk_0457.py')
_VALIDATOR_RAW = _VALIDATOR.read_bytes()
if hashlib.sha256(_VALIDATOR_RAW).hexdigest() != VALIDATOR_SHA:
    raise ValueError('Frozen current42 validator bytes differ')
# Execute exactly the verified pure validator source, never a shadow module or
# a timestamp-matching .pyc selected through sys.path/sys.modules.
_gate = {'__name__': 'frozen_current_sdk_0457', '__file__': str(_VALIDATOR)}
exec(compile(_VALIDATOR_RAW, str(_VALIDATOR), 'exec'), _gate)
validate, need, load_json = (_gate[name] for name in ('validate', 'need', 'load_json'))
PROFILE, VERSION, REPOSITORY, TARGETS = (_gate[name] for name in ('PROFILE', 'VERSION', 'REPOSITORY', 'TARGETS'))

S = 'e3f38ef6f05ae636d2837f9d69f8d6805222664b'
LOCK = '92974ab7df6beb8033fba4d3b535fee1f694229e0a6918fbd3d059d65162121d'
CORPUS = 'c8a13365a0a479d2156bf765a4718999f8e59a8d77f3f88cfb613e427f917c5e'
INPUTS_SHA = '7ed71376f652688640653128533bc562b8a6fec95f05c44bcf8354147103c9d0'
CONTROLLER = 'scripts/release/qualify_current_sdk_0457.py'
INPUTS = 'scripts/release/current-inputs-0457.json'
PREFIX = 'repos/' + REPOSITORY
WORKFLOWS = {'linux': '.github/workflows/release.yml', 'macos': '.github/workflows/release-hosted-macos.yml'}
MAX_ZIP_BYTES = 512 * 1024 * 1024


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n')


def positive(value):
    return type(value) is int and value > 0


class GitHub:
    def __init__(self, out):
        self.out = out

    def json(self, path, label):
        result = subprocess.run(['gh', 'api', '--hostname', 'github.com', '--method', 'GET', path], capture_output=True)
        (self.out / (label + '.json')).write_bytes(result.stdout)
        (self.out / (label + '.stderr')).write_bytes(result.stderr)
        save(self.out / (label + '-transport.json'), {'endpoint': path, 'exitCode': result.returncode})
        need(result.returncode == 0, 'GitHub API failed: ' + label)
        return load_json(result.stdout)

    def download(self, path, filename):
        destination = self.out / filename
        with destination.open('xb') as stream:
            result = subprocess.run(['gh', 'api', '--hostname', 'github.com', '--method', 'GET', path],
                                    stdout=stream, stderr=subprocess.PIPE)
        (self.out / (filename + '.stderr')).write_bytes(result.stderr)
        save(self.out / (filename + '-transport.json'), {'endpoint': path, 'exitCode': result.returncode})
        need(result.returncode == 0, 'GitHub artifact download failed')
        need(destination.stat().st_size <= MAX_ZIP_BYTES, 'Artifact ZIP exceeds byte bound')
        return destination.read_bytes()


def collection(client, path, field, label):
    rows = []
    total = None
    for page in range(1, 101):
        response = client.json(path + '?per_page=100&page=' + str(page), label + '-' + str(page))
        count = response.get('total_count')
        need(type(count) is int and count >= 0 and (total is None or total == count), 'Inconsistent paginated ' + field)
        total = count
        batch = response.get(field)
        need(type(batch) is list and len(batch) <= 100, 'Malformed paginated ' + field)
        rows.extend(batch)
        need(len(rows) <= total, 'Paginated inventory exceeds total')
        if len(rows) == total:
            ids = [item.get('id') for item in rows]
            need(all(positive(i) for i in ids) and len(set(ids)) == len(ids), 'Duplicate/invalid ' + field + ' IDs')
            return rows
        need(len(batch) == 100, 'Incomplete paginated ' + field)
    raise ValueError('Paginated inventory exceeds bound')


def actual_run(run, expected_id, R, tree, path, *, final_success=True):
    repo, head_repo = run.get('repository', {}), run.get('head_repository', {})
    need(type(run.get('id')) is int and run['id'] == expected_id and repo.get('full_name') == REPOSITORY
         and positive(repo.get('id')) and head_repo.get('id') == repo['id']
         and head_repo.get('full_name') == REPOSITORY, 'Producer API repository/run differs')
    need(run.get('head_sha') == R and run.get('head_commit', {}).get('id') == R
         and run['head_commit'].get('tree_id') == tree and run.get('head_branch') == 'main'
         and run.get('event') == 'workflow_dispatch', 'Producer API is not exact-main R/tree dispatch')
    allowed = ('success',) if final_success else ('success', 'failure', 'cancelled', 'timed_out')
    need(run.get('path') == path and run.get('status') == 'completed' and run.get('conclusion') in allowed
         and positive(run.get('run_attempt')) and positive(run.get('workflow_id')), 'Producer API workflow/run not successful')
    return dict(repository=REPOSITORY, runId=expected_id, workflowPath=path, sourceCommit=R,
                workflowCommit=R, headBranch='main', event='workflow_dispatch', conclusion=run['conclusion'])


def actual_job(jobs, target, run, R, *, successful=True):
    mac = target == 'aarch64-apple-darwin'
    runner = 'macos-14' if mac else ('ubuntu-24.04-arm' if target.startswith('aarch64') else 'ubuntu-24.04')
    name = 'macos-arm64' if mac else f'wheels ({runner}, {target}, 2014)'
    selected = [job for job in jobs if job.get('name') == name]
    need(len(selected) == 1, 'Exact current SDK job missing/duplicate: ' + target)
    job = selected[0]
    need(positive(job.get('id')) and type(job.get('run_id')) is int and job.get('run_id') == run['id'] and job.get('head_sha') == R
         and job.get('status') == 'completed' and (not successful or job.get('conclusion') == 'success')
         and type(job.get('run_attempt', run['run_attempt'])) is int
         and job.get('run_attempt', run['run_attempt']) == run['run_attempt'], 'Actual SDK job identity/status differs')
    need(runner in job.get('labels', []), 'Actual SDK job runner platform differs')
    if not successful and job.get('conclusion') != 'success':
        need(job.get('conclusion') in ('failure', 'cancelled', 'skipped', 'timed_out', 'neutral', 'action_required'), 'Original job conclusion unavailable')
        return job
    steps = job.get('steps', [])
    names = ['Current42 SDK and actual installed final-R correspondence',
             'Emit current final-R Mac wheel and SDK evidence immediately' if mac else 'Emit current final-R wheel and SDK evidence immediately']
    for name in names:
        matches = [step for step in steps if step.get('name') == name]
        need(len(matches) == 1 and matches[0].get('status') == 'completed'
             and matches[0].get('conclusion') == 'success', 'Actual SDK/upload step not successful')
    return job


def artifact_job_binding(artifact, job, target):
    """Artifacts lack an attempt ID: bind their API creation to the upload step."""
    name = ('Emit current final-R Mac wheel and SDK evidence immediately' if target == 'aarch64-apple-darwin'
            else 'Emit current final-R wheel and SDK evidence immediately')
    upload = next(step for step in job['steps'] if step['name'] == name)
    def stamp(value):
        need(type(value) is str, 'Actual artifact/upload API timestamp absent')
        parsed = datetime.fromisoformat(value.replace('Z', '+00:00'))
        need(parsed.utcoffset() == timedelta(0), 'Actual artifact/upload API timestamp not UTC')
        return parsed
    need(stamp(upload.get('started_at')) <= stamp(artifact.get('created_at')) <= stamp(upload.get('completed_at')),
         'Artifact creation is outside its claimed job/attempt upload window')


def artifact_candidates(rows, run, R, target):
    prefix = f'current-sdk-{target}-{R}-{run["id"]}-attempt-'
    selected = []
    for row in rows:
        name = row.get('name', '')
        if type(name) is str and name.startswith(prefix):
            attempt = name[len(prefix):]
            need(re.fullmatch('[1-9][0-9]*', attempt) is not None, 'Artifact name attempt malformed')
            selected.append((int(attempt), row))
    need(selected and len({n for n, _ in selected}) == len(selected), 'Exact attempt-specific SDK artifact missing/duplicate')
    return sorted(selected, key=lambda pair: pair[0])


def actual_artifact(artifact, run, R, target, attempt, *, require_live):
    name = f'current-sdk-{target}-{R}-{run["id"]}-attempt-{attempt}'
    binding = artifact.get('workflow_run', {})
    need(artifact.get('name') == name and positive(artifact.get('id'))
         and (not require_live or artifact.get('expired') is False)
         and type(binding.get('id')) is int and binding.get('id') == run['id'] and binding.get('head_sha') == R
         and binding.get('head_branch') == 'main' and binding.get('repository_id') == run['repository']['id']
         and binding.get('head_repository_id') == run['repository']['id'], 'Artifact API source/run/expiration differs')
    need(type(artifact.get('digest')) is str and re.fullmatch('sha256:[0-9a-f]{64}', artifact['digest']), 'Actual artifact API digest absent')
    return artifact


def archive_files(raw):
    need(len(raw) <= MAX_ZIP_BYTES, 'Artifact ZIP exceeds byte bound')
    files = {}
    with zipfile.ZipFile(io.BytesIO(raw)) as archive:
        infos = archive.infolist()
        need(len(infos) <= 2048 and sum(i.file_size for i in infos) <= MAX_ZIP_BYTES, 'Artifact payload exceeds bound')
        need(len({i.filename for i in infos}) == len(infos), 'Duplicate SDK artifact member')
        for info in infos:
            name = info.filename
            need(not name.startswith('/') and '\\' not in name and '\x00' not in name
                 and all(p not in ('', '.', '..') for p in name.rstrip('/').split('/')), 'Unsafe SDK artifact path')
            mode = stat.S_IFMT(info.external_attr >> 16)
            need(mode in (0, stat.S_IFREG, stat.S_IFDIR) and (mode != stat.S_IFDIR or info.is_dir()), 'Unsafe SDK artifact member type')
            if not info.is_dir():
                files[name] = archive.read(info)
    return files


def source_files(client, R, tree, out):
    response = client.json(PREFIX + '/git/trees/' + tree + '?recursive=1', 'R-tree')
    need(response.get('sha') == tree and response.get('truncated') is False, 'R source tree incomplete/different')
    entries = response.get('tree', [])
    need(type(entries) is list and len({e['path'] for e in entries}) == len(entries), 'Duplicate R source path')
    source = {e['path']: e for e in entries}
    def content(path, label):
        need(path in source and source[path].get('type') == 'blob' and source[path].get('mode') in ('100644', '100755'), 'R source file missing/unsafe: ' + path)
        item = client.json(PREFIX + '/contents/' + path + '?ref=' + R, label)
        need(item.get('type') == 'file' and item.get('path') == path and item.get('encoding') == 'base64', 'Source contents API file differs')
        raw = base64.b64decode(''.join(item['content'].split()), validate=True)
        blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
        need(blob == item.get('sha') == source[path].get('sha'), 'Source contents/tree Git blob differs')
        (out / (label + '.source')).write_bytes(raw)
        return raw
    code = content(CONTROLLER, 'R-controller-code')
    need(sha(content('scripts/release/verify_current_sdk_0457.py', 'R-validator')) == VALIDATOR_SHA, 'R frozen validator differs')
    manifest = content(INPUTS, 'R-inputs')
    need(sha(manifest) == INPUTS_SHA, 'Frozen canonical S input manifest differs')
    inputs = load_json(manifest)
    need(inputs['canonicalSourceID'] == f'git+https://github.com/DocumentDrivenDX/umf.git?rev={S}#{S}'
         and inputs['CargoLockSHA256'] == LOCK, 'Canonical S/lock identity differs')
    need(sha(content('Cargo.lock', 'R-lock')) == LOCK, 'Frozen R Cargo.lock differs')
    need(sha(content('tests/engine-catalog/temporal-native-495-corpus.json', 'R-corpus')) == CORPUS, 'Frozen R corpus differs')
    resources = inputs['resources']
    need(len(resources) == 9 and len({r['path'] for r in resources}) == 9, 'Canonical S resource roster differs')
    for index, resource in enumerate(resources):
        need(resource['path'].startswith('crates/weft-core/resources/umf-temporal/'), 'Unexpected canonical resource path')
        need(sha(content(resource['path'], 'R-resource-' + str(index))) == resource['sha256'], 'Canonical S resource differs')
    for path in WORKFLOWS.values():
        need(path in source and source[path].get('type') == 'blob', 'Frozen workflow source absent')
    return sha(code)


def authenticate(R, linux_run, macos_run, out):
    need(type(R) is str and re.fullmatch('[0-9a-f]{40}', R), 'Malformed final R')
    need(positive(linux_run) and positive(macos_run) and linux_run != macos_run, 'Producer run IDs invalid')
    out = Path(out).absolute()
    need(not out.exists() and not out.is_symlink(), 'Fresh external output directory required')
    out = out.resolve()
    need(not out.is_relative_to(Path(__file__).resolve().parents[2]), 'Output must be outside the source checkout')
    out.mkdir(parents=True)
    client = GitHub(out)
    try:
        main_ref = client.json(PREFIX + '/git/ref/heads/main', 'main-ref')
        need(main_ref.get('ref') == 'refs/heads/main' and main_ref.get('object', {}).get('type') == 'commit'
             and main_ref['object'].get('sha') == R, 'Final main has moved; review before proceeding')
        commit = client.json(PREFIX + '/git/commits/' + R, 'R-commit')
        tree = commit.get('tree', {}).get('sha')
        need(commit.get('sha') == R and type(tree) is str and re.fullmatch('[0-9a-f]{40}', tree), 'Commit API differs')
        controller_hash = source_files(client, R, tree, out)
        runs, artifacts, identities, attempts, jobs = {}, {}, {}, {}, {}
        for role, run_id in (('linux', linux_run), ('macos', macos_run)):
            run = client.json(PREFIX + '/actions/runs/' + str(run_id), role + '-run')
            identities[role] = actual_run(run, run_id, R, tree, WORKFLOWS[role])
            workflow = client.json(PREFIX + '/actions/workflows/' + str(run['workflow_id']), role + '-workflow')
            need(workflow.get('id') == run['workflow_id'] and workflow.get('path') == WORKFLOWS[role], 'Workflow API identity differs')
            runs[role] = run
            artifacts[role] = collection(client, PREFIX + '/actions/runs/' + str(run_id) + '/artifacts', 'artifacts', role + '-artifacts')
        rows, observations, ignored = [], [], []
        def original_attempt(role, number):
            run = runs[role]
            need(number <= run['run_attempt'], 'Artifact claims an unobserved future attempt')
            key = (run['id'], number)
            if key not in attempts:
                label = role + '-attempt-' + str(number)
                original = client.json(PREFIX + f'/actions/runs/{run["id"]}/attempts/{number}', label)
                actual_run(original, run['id'], R, tree, WORKFLOWS[role], final_success=False)
                need(original['run_attempt'] == number and original['workflow_id'] == run['workflow_id'], 'Original attempt API identity differs')
                attempts[key] = original
                jobs[key] = collection(client, PREFIX + f'/actions/runs/{run["id"]}/attempts/{number}/jobs', 'jobs', label + '-jobs')
            return key

        for target in TARGETS:
            role = 'macos' if target == 'aarch64-apple-darwin' else 'linux'
            run, producer = runs[role], identities[role]
            selected = []
            for number, candidate in artifact_candidates(artifacts[role], run, R, target):
                key = original_attempt(role, number)
                actual_artifact(candidate, run, R, target, number, require_live=False)
                candidate_job = actual_job(jobs[key], target, attempts[key], R, successful=False)
                if candidate_job['conclusion'] != 'success':
                    ignored.append(dict(target=target, artifactID=candidate['id'], artifactDigest=candidate['digest'],
                                        runAttempt=number, actualJob=candidate_job['id'], conclusion=candidate_job['conclusion']))
                    continue
                actual_artifact(candidate, run, R, target, number, require_live=True)
                artifact_job_binding(candidate, candidate_job, target)
                selected.append((number, key, candidate, candidate_job))
            need(len(selected) == 1, 'One uniquely successful current SDK artifact/job per target required')
            attempt, attempt_key, artifact, job = selected[0]
            raw = client.download(PREFIX + '/actions/artifacts/' + str(artifact['id']) + '/zip', str(artifact['id']) + '.zip')
            need(sha(raw) == artifact['digest'][7:], 'Downloaded artifact differs from actual GitHub digest')
            files = archive_files(raw)
            component = load_json(files['component.json'])
            need(component.get('producerRunAttempt') == str(attempt), 'Component/artifact/job claimed attempt differs')
            need(component.get('profile') == 'weft-current-sdk-platform-observation/1' and component.get('status') == 'passed'
                 and component.get('sourceCommit') == R and component.get('sourceTree') == tree, 'Component is not successful final R/tree')
            need(component.get('workflowCommit') == R and type(component.get('producerRunId')) is int
                 and component['producerRunId'] == run['id'] and component.get('producerRunAttempt') == str(attempt)
                 and component.get('producerJobName') == ('macos-arm64' if role == 'macos' else 'wheels')
                 and component.get('workflowRef') == REPOSITORY + '/' + WORKFLOWS[role] + '@refs/heads/main', 'Component producer/job/attempt binding differs')
            need(component.get('target') == target and component.get('distribution') == 'umf-weft'
                 and component.get('version') == VERSION and component.get('canonicalUmf') == S
                 and component.get('cargoLockSha256') == LOCK, 'Component target/S/lock differs')
            platform = component.get('platform', '')
            cpu = 'arm64' if role == 'macos' else ('aarch64' if target.startswith('aarch64') else 'x86_64')
            need(type(platform) is str and platform.startswith('macOS-' if role == 'macos' else 'Linux-')
                 and cpu in platform and type(component.get('python')) is str and bool(component['python']), 'Component actual platform differs')
            need(component.get('controllerScript') == CONTROLLER and component.get('controllerCodeSha256') == controller_hash, 'Component controller is not code from frozen R')
            need(component.get('nativeAdmission') is False and component.get('publicationAdmission') is False
                 and [component.get(k) for k in ('temporalCases', 'routingControls', 'sdkControls')] == [14, 5, 42], 'Component scope/counts differ')
            stages = component.get('stageResults', [])
            need([s.get('name') for s in stages] == ['venv', 'normal-install', 'pip-check', 'packaging', 'native-origin', 'temporal14', 'routing5', 'sdk42']
                 and all(type(s.get('exitCode')) is int and s['exitCode'] == 0 for s in stages), 'Component stage results incomplete/failed')
            wheel = component['wheel']
            need(type(wheel) is str and wheel in files and '/' not in wheel, 'Actual wheel absent/unsafe')
            report_raw, packaging_raw, wheel_raw = files['sdk-smoke.json'], files['packaging.json'], files[wheel]
            for key, body in (('wheelSha256', wheel_raw), ('sdkReportSha256', report_raw), ('packagingReportSha256', packaging_raw)):
                need(component.get(key) == sha(body), 'Component byte binding differs: ' + key)
            envelope = dict(profile=PROFILE, sourceCommit=R, sourceTree=tree, distribution='umf-weft', version=VERSION,
                            target=target, wheel=wheel, wheelSha256=sha(wheel_raw), nativeModuleSha256=component['nativeModuleSha256'],
                            shimSha256=component['shimSha256'], sdkReportSha256=sha(report_raw), packagingReportSha256=sha(packaging_raw),
                            controllerCodeSha256=controller_hash, producer=producer, controller=dict(producer))
            save(out / (target + '-envelope.json'), envelope)
            rows.append((envelope, report_raw, wheel_raw, packaging_raw))
            observations.append(dict(target=target, artifactID=artifact['id'], artifactDigest=artifact['digest'], artifactCreatedAt=artifact['created_at'],
                                     actualRun=run['id'], actualJob=job['id'], runAttempt=attempt, actualJobAttempt=attempt,
                                     producerAttemptConclusion=attempts[attempt_key]['conclusion'], finalRunAttempt=run['run_attempt'], finalRunConclusion=run['conclusion']))
        result = validate(rows, R, tree, controller_hash)
        for role, run in runs.items():
            again = client.json(PREFIX + '/actions/runs/' + str(run['id']), role + '-run-after')
            actual_run(again, run['id'], R, tree, WORKFLOWS[role])
            need(again['run_attempt'] == run['run_attempt'], 'Producer attempt changed during admission')
        need(client.json(PREFIX + '/git/ref/heads/main', 'main-ref-after') == main_ref, 'Final main changed during admission')
        need(sha(_VALIDATOR.read_bytes()) == VALIDATOR_SHA, 'Frozen validator changed during admission')
        result.update(actualGitHubObservations=observations, ignoredFailedArtifacts=ignored, controllerCodeSource='GitHub contents/tree API at immutable R, verified Git blob ID', controllerCodeSha256=controller_hash)
        save(out / 'current-sdk-admission.json', result)
        return result
    except Exception as error:
        save(out / 'admission-error.json', {'status': 'refused', 'errorType': type(error).__name__, 'message': str(error), 'nativeAdmission': False, 'publicationAdmission': False})
        raise


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source-sha', required=True)
    parser.add_argument('--linux-run', type=int, required=True)
    parser.add_argument('--macos-run', type=int, required=True)
    parser.add_argument('--out', type=Path, required=True)
    args = parser.parse_args()
    authenticate(args.source_sha, args.linux_run, args.macos_run, args.out)


if __name__ == '__main__':
    main()
