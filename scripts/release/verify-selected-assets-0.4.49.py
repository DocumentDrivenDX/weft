#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Refuse promotion unless selected wheel bytes and native case evidence match."""
import argparse
import base64
import csv
import hashlib
import io
import json
import re
import uuid
import zipfile
import gzip
from pathlib import Path


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def require(condition, message):
    if not condition:
        raise SystemExit(message)


def is_sha(value):
    return type(value) is str and re.fullmatch(r'[0-9a-f]{64}', value) is not None


def canonical_json(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True,
                      separators=(',', ':'), allow_nan=False)


def wheel_record(path, platform):
    with zipfile.ZipFile(path) as archive:
        names = archive.namelist()
        require(len(names) == len(set(names)), f'{path}: duplicate members')
        record = next(n for n in names if n.endswith('.dist-info/RECORD'))
        rows = list(csv.reader(io.StringIO(archive.read(record).decode())))
        require({r[0] for r in rows} == set(names), f'{path}: RECORD membership mismatch')
        for name, digest, size in rows:
            if name == record:
                require(digest == size == '', f'{path}: invalid RECORD self-entry')
                continue
            raw = archive.read(name)
            expected = 'sha256=' + base64.urlsafe_b64encode(hashlib.sha256(raw).digest()).decode().rstrip('=')
            require(digest == expected and size == str(len(raw)), f'{path}: RECORD mismatch for {name}')
        native = [n for n in names if n.endswith(('.so', '.pyd'))]
        require(len(native) == 1, f'{path}: expected one native module')
        require(platform in Path(path).name, f'{path}: wrong wheel platform')
        return {'native': hashlib.sha256(archive.read(native[0])).hexdigest(),
                'record': hashlib.sha256(archive.read(record)).hexdigest()}


p = argparse.ArgumentParser()
p.add_argument('--source-commit', required=True)
p.add_argument('--linux-run-id', required=True)
p.add_argument('--linux-run-report', type=Path, required=True)
p.add_argument('--linux-wheel', type=Path, required=True)
p.add_argument('--linux-build-report', type=Path, required=True)
p.add_argument('--linux-packaging-report', type=Path, required=True)
p.add_argument('--macos-wheel', type=Path, required=True)
p.add_argument('--macos-build-report', type=Path, required=True)
p.add_argument('--macos-packaging-report', type=Path, required=True)
p.add_argument('--managed-qualification', type=Path, required=True)
p.add_argument('--backend03-evidence', type=Path, required=True)
p.add_argument('--backend03-negative-having-evidence', type=Path, required=True)
p.add_argument('--backend02-page1-evidence', type=Path, required=True)
p.add_argument('--backend02-continuation-evidence', type=Path, required=True)
p.add_argument('--program-parity', type=Path, required=True)
p.add_argument('--prior-local-evidence', type=Path, required=True)
p.add_argument('--prior-grouped-gzip', type=Path, required=True)
p.add_argument('--prior-negative-gzip', type=Path, required=True)
p.add_argument('--prior-pages-gzip', type=Path, required=True)
p.add_argument('--output', type=Path, required=True)
a = p.parse_args()
require(re.fullmatch(r'[0-9a-f]{40}', a.source_commit) is not None, 'source commit must be a full lowercase Git SHA')

run = json.loads(a.linux_run_report.read_text())
require(run.get('conclusion') == 'success' and run.get('event') == 'workflow_dispatch', 'selected Linux Actions run is not a successful manual build')
require(run.get('workflowName') == 'Build selected Weft 0.4.49 Linux wheel', 'selected Linux Actions workflow identity mismatch')
require(run.get('headSha') == a.source_commit, 'selected Linux workflow did not run from the exact source commit')
linux_sha, mac_sha = sha(a.linux_wheel), sha(a.macos_wheel)
linux_native, mac_native = wheel_record(a.linux_wheel, 'manylinux'), wheel_record(a.macos_wheel, 'macosx')

for label, path, wheel, native, target in (
    ('linux', a.linux_build_report, a.linux_wheel, linux_native, 'x86_64-unknown-linux-gnu'),
    ('macos', a.macos_build_report, a.macos_wheel, mac_native, 'aarch64-apple-darwin'),
):
    build = json.loads(path.read_text())
    package = json.loads((a.linux_packaging_report if label == 'linux' else a.macos_packaging_report).read_text())
    require(build.get('status') == package.get('status') == 'passed', f'{label}: build/packaging report not passed')
    require(build.get('version') == package.get('version') == '0.4.49', f'{label}: wrong version')
    require(build.get('sourceCommit') == a.source_commit, f'{label}: wrong source commit')
    require(build.get('wheel') == package.get('wheel') == wheel.name, f'{label}: report names another wheel')
    require(build.get('wheelSha256') == package.get('wheelSha256') == sha(wheel), f'{label}: wheel digest mismatch')
    require(build.get('nativeModuleSha256') == package.get('nativeModuleSha256') == native['native'], f'{label}: native module digest mismatch')
    require(build.get('recordSha256') == package.get('recordSha256') == native['record'], f'{label}: RECORD digest mismatch')
    require(build.get('target') == target, f'{label}: unexpected build target')
    if label == 'linux':
        require(build.get('manylinux') == '2014', 'Linux build is not manylinux2014')
        require(str(build.get('buildRunId')) == a.linux_run_id, 'Linux report run ID differs from selected artifact run')
        require(build.get('buildWorkflowName') == run['workflowName'], 'Linux report workflow identity mismatch')

proof = json.loads(a.managed_qualification.read_text())
require(proof.get('schema') == 'weft-selected-qualification/0.1', 'unsupported qualification report schema')
require(proof.get('status') == 'passed' and proof.get('version') == '0.4.49', 'managed qualification is not passed for 0.4.49')
require(proof.get('sourceCommit') == a.source_commit, 'managed qualification source commit mismatch')
require(proof.get('linuxWheel') == {'sha256': linux_sha, 'nativeModuleSha256': linux_native['native'], 'recordSha256': linux_native['record']}, 'managed proof does not bind the Linux wheel bytes and RECORD')
require(proof.get('macosWheel') == {'sha256': mac_sha, 'nativeModuleSha256': mac_native['native'], 'recordSha256': mac_native['record']}, 'managed proof does not bind the macOS wheel bytes and RECORD')

case_files = {
    'backend03GroupedHaving': ('backend03-grouped-having', a.backend03_evidence, '0.1.0-managed-current-core-candidate', 'weft-backend/0.3.0'),
    'backend03NegativeHaving': ('backend03-negative-having', a.backend03_negative_having_evidence, '0.1.0-managed-current-core-candidate', 'weft-backend/0.3.0'),
    'backend02Page1': ('backend02-page1', a.backend02_page1_evidence, '0.1.0-managed-current-core-pages-candidate', 'weft-backend/0.2.7'),
    'backend02Continuation': ('backend02-continuation', a.backend02_continuation_evidence, '0.1.0-managed-current-core-pages-candidate', 'weft-backend/0.2.7'),
}
case_refs = proof['managedUc']['caseEvidence']
require(set(case_refs) == set(case_files), 'exact four native case evidence references required')
runtime = None
publication = None
for case_name, (case_id, path, backend_version, interface_version) in case_files.items():
    evidence = json.loads(path.read_text())
    reference = case_refs[case_name]
    require(reference == {'asset': path.name, 'sha256': sha(path)}, f'{case_name}: case evidence asset digest mismatch')
    require(evidence.get('case') == case_id and evidence.get('status') == 'passed', f'{case_name}: native evidence not passed')
    require(evidence.get('sourceCommit') == a.source_commit, f'{case_name}: source commit mismatch')
    require(evidence.get('linuxWheelSha256') == linux_sha and evidence.get('linuxNativeModuleSha256') == linux_native['native'] and evidence.get('linuxRecordSha256') == linux_native['record'], f'{case_name}: evidence is not from selected Linux wheel and RECORD')
    runtime_identity = evidence.get('runtimeIdentity')
    require(type(runtime_identity) is dict and type(runtime_identity.get('opening')) is dict and runtime_identity.get('opening') == runtime_identity.get('closing'), f'{case_name}: matching opening/closing runtime custody required')
    for key in ('runtimeName', 'sparkVersion', 'principal', 'catalog', 'schema', 'protocol'):
        require(type(runtime_identity['opening'].get(key)) is str and runtime_identity['opening'][key], f'{case_name}: observed {key} required')
    require(runtime_identity['opening']['runtimeName'] == 'databricks-serverless', f'{case_name}: serverless runtime required')
    require(any(type(runtime_identity['opening'].get(key)) is str and runtime_identity['opening'][key] for key in ('sparkBuildText', 'runtimeTag')), f'{case_name}: observed Databricks build/runtime identity required')
    require(type(evidence.get('publicationReceipt')) is dict, f'{case_name}: observed publication receipt required')
    receipt = evidence['publicationReceipt']
    require(type(receipt.get('manifest')) is dict and is_sha(receipt.get('manifestSha256')), f'{case_name}: exact manifest receipt required')
    manifest_bytes = canonical_json(receipt['manifest']).encode('utf-8')
    require(hashlib.sha256(manifest_bytes).hexdigest() == receipt['manifestSha256'], f'{case_name}: publication manifest digest mismatch')
    binding = receipt.get('weftBindingInputs')
    require(type(binding) is dict and type(binding.get('dataRoles')) is dict and set(binding['dataRoles']) == {'object_current', 'edge_current', 'tombstone', 'whole_source_history'}, f'{case_name}: exact four-role data vector required')
    require(type(binding.get('manifestUuid')) is str and binding['manifestUuid'], f'{case_name}: publication manifest UUID required')
    try:
        require(str(uuid.UUID(binding['manifestUuid'])) == binding['manifestUuid'], f'{case_name}: canonical publication manifest UUID required')
    except (ValueError, AttributeError, TypeError):
        raise SystemExit(f'{case_name}: valid publication manifest UUID required')
    for role, item in binding['dataRoles'].items():
        require(type(item) is dict and type(item.get('name')) in (list, tuple) and len(item['name']) == 3 and item['name'][2] == role, f'{case_name}: qualified three-part name for {role} required')
        try:
            require(type(item.get('uuid')) is str and str(uuid.UUID(item['uuid'])) == item['uuid'], f'{case_name}: canonical table UUID for {role} required')
        except (ValueError, AttributeError, TypeError):
            raise SystemExit(f'{case_name}: valid table UUID for {role} required')
        require(type(item.get('version')) is int and item['version'] >= 0, f'{case_name}: observed head for {role} required')
    expected_vector = {'.'.join(binding['dataRoles'][role]['name']): {'uuid': binding['dataRoles'][role]['uuid'], 'version': binding['dataRoles'][role]['version']} for role in binding['dataRoles']}
    require(receipt.get('dataRoleVector') == expected_vector, f'{case_name}: manifest vector differs from four-role observed heads')
    manifest = binding.get('manifest')
    require(manifest == receipt['manifest'] and manifest.get('publication_id') == receipt.get('publicationId'), f'{case_name}: manifest/publication receipt identity mismatch')
    try:
        manifest_vector = json.loads(manifest['table_versions_json'])
    except (KeyError, TypeError, json.JSONDecodeError):
        raise SystemExit(f'{case_name}: manifest four-role version vector is malformed')
    require(manifest_vector == {name: entry['version'] for name, entry in expected_vector.items()}, f'{case_name}: reopened manifest does not contain exact four-role version vector')
    require(type(receipt.get('installationInventory')) is dict and set(receipt['installationInventory']) == {'object_current', 'edge_current', 'tombstone', 'whole_source_history', 'publication_manifest', 'publication_attempt_phase'}, f'{case_name}: separate six-role installation inventory required')
    for role, item in receipt['installationInventory'].items():
        require(type(item) is dict and set(('name', 'uuid', 'version', 'schema', 'protocol')) <= set(item), f'{case_name}: complete six-role native inventory required')
        require(type(item['name']) in (str, list, tuple) and type(item['uuid']) is str and type(item['version']) is int and type(item['schema']) is list and type(item['protocol']) is dict and item['protocol'], f'{case_name}: malformed native inventory item {role}')
        if type(item['name']) in (list, tuple):
            require(len(item['name']) == 3 and list(item['name'][:2]) == [runtime_identity['opening']['catalog'], runtime_identity['opening']['schema']], f'{case_name}: observed UC destination differs from runtime identity for {role}')
            require(item['name'][2] == role, f'{case_name}: observed installation role/name mismatch for {role}')
    for role, item in binding['dataRoles'].items():
        installed = receipt['installationInventory'][role]
        require(list(item['name']) == list(installed['name']) and item['uuid'] == installed['uuid'] and item['version'] == installed['version'], f'{case_name}: four-role binding differs from observed installation inventory for {role}')
    require(receipt['installationInventory']['publication_manifest']['uuid'] == binding['manifestUuid'], f'{case_name}: manifest UUID differs from six-role inventory')
    require(type(evidence.get('oracle')) is dict, f'{case_name}: accepted source oracle evidence required')
    for key in ('sha256', 'authorityReceiptSha256', 'catalogHeadSha256', 'modelSha256', 'cursorSha256', 'scheduleSha256'):
        require(is_sha(evidence['oracle'].get(key)), f'{case_name}: oracle {key} digest required')
    require(type(evidence['oracle'].get('sourceAuthority')) is str and evidence['oracle']['sourceAuthority'] == 'external-real-truss-holder', f'{case_name}: accepted Truss source authority required')
    require(type(evidence['oracle'].get('capturedAt')) is str and evidence['oracle']['capturedAt'], f'{case_name}: source capture time required')
    require(evidence['oracle'].get('profile') == 'ashlar-managed-current-source-oracle/0.1' and type(evidence['oracle'].get('scheduleStep')) is int, f'{case_name}: schedule-bound accepted source oracle required')
    request_json, artifact_json = evidence.get('requestJson'), evidence.get('artifactJson')
    require(type(request_json) is str and type(artifact_json) is str, f'{case_name}: exact compiler request and artifact bytes required')
    require(hashlib.sha256(request_json.encode('utf-8')).hexdigest() == evidence.get('requestSha256'), f'{case_name}: request digest mismatch')
    require(hashlib.sha256(artifact_json.encode('utf-8')).hexdigest() == evidence.get('artifactSha256'), f'{case_name}: artifact digest mismatch')
    request, artifact = json.loads(request_json), json.loads(artifact_json)
    require(canonical_json(request) == request_json and canonical_json(artifact) == artifact_json, f'{case_name}: compiler request/artifact bytes are not canonical JSON')
    require(request.get('target', {}).get('backendId') == 'ashlar.databricks.managed-uc-spark' and request['target'].get('backendVersion') == backend_version and request['target'].get('targetProfile') == 'ashlar-managed-current-core/0.1', f'{case_name}: exact managed request target required')
    backend = artifact.get('backend')
    require(type(backend) is dict and backend == {'backendId': 'ashlar.databricks.managed-uc-spark', 'backendVersion': backend_version, 'interfaceVersion': interface_version, 'targetProfile': 'ashlar-managed-current-core/0.1'}, f'{case_name}: compiled artifact target/interface mismatch')
    require(artifact.get('status') == 'compiled' and evidence.get('artifactStatus') == 'compiled', f'{case_name}: public compiler artifact must be compiled')
    require(type(artifact.get('sql')) is str and hashlib.sha256(artifact['sql'].encode('utf-8')).hexdigest() == evidence.get('compilerSqlSha256'), f'{case_name}: emitted compiler SQL digest mismatch')
    require(type(request.get('sql')) is str and hashlib.sha256(request['sql'].encode('utf-8')).hexdigest() == evidence.get('sqlSha256'), f'{case_name}: original request SQL digest mismatch')
    slots = artifact.get('parameters')
    require(type(slots) is list and all(type(item) is dict and item.get('position') == index and type(item.get('value')) is str for index, item in enumerate(slots, 1)), f'{case_name}: exact ordered compiler parameter slots required')
    canonical_slots = canonical_json(slots)
    require(hashlib.sha256(canonical_slots.encode('utf-8')).hexdigest() == evidence.get('orderedParameterSha256'), f'{case_name}: original ordered parameter digest mismatch')
    require(type(evidence.get('expectedRows')) is list and evidence.get('actualRows') == evidence.get('expectedRows'), f'{case_name}: expected and actual native output differ')
    expected_rows_digest = hashlib.sha256(canonical_json(evidence['expectedRows']).encode('utf-8')).hexdigest()
    require(evidence.get('expectedRowsSha256') == evidence.get('oracle', {}).get('expectedRowsSha256') == expected_rows_digest, f'{case_name}: expected rows are not bound to the frozen source oracle')
    require(type(evidence.get('runId')) in (str, int) and str(evidence['runId']).isdigit() and type(evidence.get('taskReceiptId')) is str and evidence['taskReceiptId'], f'{case_name}: bounded Databricks Jobs run and task receipt IDs required')
    if case_name == 'backend02Page1':
        require(evidence.get('pageSize') == 2 and evidence.get('hasContinuation') is True and is_sha(evidence.get('continuationSha256')), 'Backend02 initial page must prove page-size-2 continuation')
    if case_name == 'backend02Continuation':
        first = json.loads(a.backend02_page1_evidence.read_text())
        require(evidence.get('pageSize') == 2 and evidence.get('hasContinuation') is False and evidence.get('incomingContinuationSha256') == first.get('continuationSha256'), 'Backend02 continuation must consume the exact first-page cursor and terminate')
    if case_name == 'backend03NegativeHaving':
        require(evidence['expectedRows'] == [] and 'HAVING COUNT(*) > 2' in request['sql'], 'Backend03 negative HAVING case must compile the >2 predicate and expect the empty oracle result')
    if case_name == 'backend03GroupedHaving':
        require(evidence['expectedRows'] and 'HAVING COUNT(*) > 0' in request['sql'], 'Backend03 positive HAVING case must compile the >0 predicate and expect positive grouped rows')
    if runtime is None:
        runtime, publication = evidence['runtimeIdentity'], receipt
    else:
        require(evidence['runtimeIdentity'] == runtime, f'{case_name}: runtime identity differs across native cases')
        require(receipt == publication, f'{case_name}: publication custody differs across native cases')

parity = json.loads(a.program_parity.read_text())
parity_ref = proof['localMacProgramParity']
require(parity_ref == {'asset': a.program_parity.name, 'sha256': sha(a.program_parity)}, 'local parity asset digest mismatch')
require(parity.get('schema') == 'weft-local-program-parity/0.2' and parity.get('sourceCommit') == a.source_commit and parity.get('status') == 'passed', 'local program parity report not bound to selected source')
require(parity.get('localWheel') == {'sha256': mac_sha, 'nativeModuleSha256': mac_native['native'], 'recordSha256': mac_native['record']}, 'local parity compile is not bound to selected macOS wheel')
baseline_sha = sha(a.prior_local_evidence)
baseline = json.loads(a.prior_local_evidence.read_text())
prior_assets = {
    'positive_grouped_graph_query': (a.prior_grouped_gzip, 'positive_grouped_graph_query'),
    'negative_having_query': (a.prior_negative_gzip, 'negative_having_query'),
    'sdk_pages_2_plus_1': (a.prior_pages_gzip, 'sdk_pages_2_plus_1'),
}
prior_reports = {}
for key, (path, packet_key) in prior_assets.items():
    ref = baseline.get('files', {}).get(packet_key)
    try:
        decompressed = gzip.decompress(path.read_bytes())
        require(type(ref) is dict and ref.get('file') == path.name and ref.get('sha256') == hashlib.sha256(decompressed).hexdigest() and ref.get('bytes') == len(decompressed), f'prior {packet_key} gzip payload does not match retained native packet')
        prior_reports[key] = json.loads(decompressed)['query']
    except (OSError, KeyError, json.JSONDecodeError):
        raise SystemExit(f'prior {packet_key} native report is not a valid compressed JSON report')
def prior_program(key, page_index=None):
    report = prior_reports[key]
    if key == 'sdk_pages_2_plus_1':
        programs = report.get('evidence')
        require(type(programs) is list and page_index in (0, 1) and len(programs) > page_index, 'prior SDK report lacks both recorded page programs')
        program = programs[page_index].get('program')
        require(type(program) is dict and program.get('status') == 'compiled', 'prior SDK report lacks a compiled public program')
        request = program.get('request')
        artifact = program.get('artifact')
    else:
        request = {'sql': report.get('sql')}
        try:
            artifact = json.loads(report['protocol']['recompiled'])
        except (KeyError, TypeError, json.JSONDecodeError):
            raise SystemExit(f'prior {key} report lacks the retained compiler artifact')
    require(type(request) is dict and type(request.get('sql')) is str and type(artifact) is dict and type(artifact.get('parameters')) is list, f'prior {key} report lacks SQL/ordered parameter evidence')
    return request, artifact
prior_case_key = {
    'backend03GroupedHaving': ('positive_grouped_graph_query', None),
    'backend03NegativeHaving': ('negative_having_query', None),
    'backend02Page1': ('sdk_pages_2_plus_1', 0),
    'backend02Continuation': ('sdk_pages_2_plus_1', 1),
}
parity_cases = parity.get('cases')
require(type(parity_cases) is dict and set(parity_cases) == set(case_files), 'route-specific parity cases required')
for case_name in case_files:
    item = parity_cases[case_name]
    require(item.get('priorEvidence') == {'asset': a.prior_local_evidence.name, 'sha256': baseline_sha}, f'{case_name}: prior local evidence asset digest mismatch')
    prior_request, prior_artifact = prior_program(*prior_case_key[case_name])
    local49 = item.get('local49')
    require(type(local49) is dict, f'{case_name}: .49 local compile evidence required')
    request_json, artifact_json = local49.get('requestJson'), local49.get('artifactJson')
    require(type(request_json) is str and type(artifact_json) is str, f'{case_name}: exact .49 local compiler request/artifact required')
    require(hashlib.sha256(request_json.encode('utf-8')).hexdigest() == local49.get('requestSha256'), f'{case_name}: .49 local request digest mismatch')
    require(hashlib.sha256(artifact_json.encode('utf-8')).hexdigest() == local49.get('artifactSha256'), f'{case_name}: .49 local artifact digest mismatch')
    request, artifact = json.loads(request_json), json.loads(artifact_json)
    require(canonical_json(request) == request_json and canonical_json(artifact) == artifact_json, f'{case_name}: .49 local request/artifact bytes are not canonical JSON')
    require(artifact.get('status') == 'compiled' and artifact.get('backend') == prior_artifact.get('backend'), f'{case_name}: .49 local compilation changed the previously qualified backend tuple')
    require(type(request.get('sql')) is str and request['sql'] == prior_request['sql'], f'{case_name}: .49 compile did not replay the exact prior authored request SQL')
    require(type(artifact.get('sql')) is str and artifact['sql'] == prior_artifact.get('sql'), f'{case_name}: .49 emitted local target SQL differs from retained .48 native output')
    require(artifact.get('parameters') == prior_artifact.get('parameters'), f'{case_name}: .49 full ordered parameter slots differ from retained .48 native output')
    require(artifact.get('bindingSha256') == prior_artifact.get('bindingSha256'), f'{case_name}: .49 local compile changed the retained target binding')
    require(hashlib.sha256(artifact['sql'].encode('utf-8')).hexdigest() == local49.get('sqlSha256'), f'{case_name}: .49 emitted SQL digest mismatch')
    slots_sha = hashlib.sha256(canonical_json(artifact['parameters']).encode('utf-8')).hexdigest()
    require(slots_sha == local49.get('orderedParameterSha256'), f'{case_name}: .49 full ordered parameter digest mismatch')

a.output.parent.mkdir(parents=True, exist_ok=True)
a.output.write_text(json.dumps({
    'status': 'passed', 'tag': 'v0.4.49', 'sourceCommit': a.source_commit,
    'linuxBuildRunId': a.linux_run_id, 'linuxBuildWorkflow': run['workflowName'],
    'linuxWheel': {'name': a.linux_wheel.name, 'sha256': linux_sha, 'nativeModuleSha256': linux_native['native'], 'recordSha256': linux_native['record']},
    'macosWheel': {'name': a.macos_wheel.name, 'sha256': mac_sha, 'nativeModuleSha256': mac_native['native'], 'recordSha256': mac_native['record']},
    'managedQualification': {'name': a.managed_qualification.name, 'sha256': sha(a.managed_qualification)},
    'nativeCases': {name: {'name': case[1].name, 'sha256': sha(case[1])} for name, case in case_files.items()},
    'localProgramParity': {'name': a.program_parity.name, 'sha256': sha(a.program_parity)},
    'priorLocalEvidence': {'name': a.prior_local_evidence.name, 'sha256': baseline_sha},
    'promotion': 'selected existing bytes only; this validator does not build or replace assets',
}, indent=2) + '\n')
