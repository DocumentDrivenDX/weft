"""Actual installed native extension transport parity; no engine execution.

Inputs are explicit owned regular files. The enclosing bounded launcher owns
process lifetime, closed environment, and opening/closing artifact custody.
"""
import argparse
import hashlib
import importlib.util
import json
import os
import stat
from pathlib import Path
import sys

LIMIT = 4 * 1024 * 1024
REQUEST_LIMIT = 16 * 1024 * 1024
CORPUS_LIMIT = 32 * 1024 * 1024

def read(path, limit=LIMIT):
    if any(p.is_symlink() for p in (path, *path.parents)):
        raise ValueError('regular-input')
    fd = os.open(path, os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW)
    primary = None
    try:
        if not stat.S_ISREG(os.fstat(fd).st_mode):
            raise ValueError('regular-input')
        chunks = []
        remaining = limit + 1
        while remaining:
            chunk = os.read(fd, min(remaining, 65536))
            if not chunk:
                break
            chunks.append(chunk)
            remaining -= len(chunk)
        raw = b''.join(chunks)
    except BaseException as error:
        primary = error
    finally:
        try:
            os.close(fd)
        except BaseException as error:
            if primary is None or (isinstance(primary, Exception) and not isinstance(error, Exception)):
                primary = error
            else:
                primary.cleanup_failed = True
    if primary is not None:
        raise primary
    if len(raw) > limit:
        raise ValueError('input-bound')
    return raw


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--extension', type=Path, required=True)
    parser.add_argument('--cases', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    args = parser.parse_args()
    if not all(p.is_absolute() for p in (args.extension, args.cases, args.output)):
        raise ValueError('absolute-config')
    rows = json.loads(read(args.cases, CORPUS_LIMIT))
    if not isinstance(rows, list) or not 1 <= len(rows) <= 128:
        raise ValueError('case-inventory')
    spec = importlib.util.spec_from_file_location('weft', args.extension)
    if spec is None or spec.loader is None:
        raise ValueError('extension-loader')
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    responses = []
    for row in rows:
        request = row['request']
        if type(request) is not str or len(request.encode('utf8')) > REQUEST_LIMIT:
            raise ValueError('request-bound')
        first = module.compile_paths_keys_json(request)
        second = module.compile_paths_keys_json(request)
        if type(first) is not str or first != second or len(first.encode('utf8')) > LIMIT:
            raise ValueError('repeat-or-response-bound')
        if first != row['expected']:
            raise ValueError('exact-response-parity')
        responses.append({'id': row['id'], 'response': first,
                          'responseSha256': hashlib.sha256(first.encode('utf8')).hexdigest()})
    # The generic entrypoint is intentionally distinct and still uses its old
    # closed version composition. It cannot acquire PathsKeys through this build.
    selected = next(row for row in rows if json.loads(row['request']).get('interfaceVersion') == 'weft-compile/0.4.1' and json.loads(row['expected'])['status'] == 'compiled')
    generic = module.compile_json(selected['request'])
    generic_value = json.loads(generic)
    if generic_value['status'] != 'blocked' or not any(d['code'] == 'WFT-VERSION' for d in generic_value['diagnostics']):
        raise ValueError('generic-entrypoint-profile')
    for value in (None, {}, 1):
        try:
            module.compile_paths_keys_json(value)
        except TypeError:
            continue
        raise ValueError('native-string-transport')
    for value in ('\ud800', '\udc00'):
        try:
            module.compile_paths_keys_json(value)
        except UnicodeEncodeError:
            continue
        raise ValueError('native-scalar-transport')
    result = {'format': 'weft-native-paths-embedding-parity/0.1',
              'cases': responses, 'genericResponse': generic,
              'qualification': 'Actual native extension transport only; no SQL execution or host obligations.'}
    with args.output.open('x', encoding='utf8') as stream:
        json.dump(result, stream, ensure_ascii=False, indent=2)
        stream.write('\n')
    print(json.dumps({'state': 'native-extension-parity-passed', 'cases': len(responses)}))

if __name__ == '__main__':
    main()
