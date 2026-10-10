"""Offline, byte-pinned schema port for the finite c6 Paths corpus producer.

This validates protocol shapes, not compiler semantics or release authority.
Malformed controls remain raw inputs to the compiler; every response is checked.
"""
import argparse
from dataclasses import dataclass
from decimal import Decimal
import hashlib
import json
import os
from pathlib import Path
import stat
import sys

SCHEMAS = {
    'compile-request.schema.json': '8916022eb698864f5580319848307a1e0e90699dbaac1dafd6b8f2e11119d3e5',
    'compile-request-v0.2.schema.json': '2a9a4264a9d8926176aa3809d613e103f35c8f634b0d8839ddf2bbcf598cacde',
    'compile-request-v0.3.schema.json': '7345c424b1bbf8da15e6f7b4efb81da7eeb85a1e08fb91319eeb7b19ec129047',
    'compile-request-v0.4.schema.json': '51f9793fb7ca32fdde778e2ff8cacd1018e4142fb04dea610005895aacec3932',
    'compile-response-v0.4.schema.json': 'c177b9bbb9fff3d69a833aa5bf05dbe752a1f76e65833cae8ea35fd4b1373140',
    'logical-plan-v0.4.schema.json': 'a6f28858646e4795019037233ef0e8d6b0aeb45e7440cbcf4e7a477891a1b3d5',
}
REQUESTS = {
    'weft-compile/0.1.0': 'compile-request.schema.json',
    'weft-compile/0.2.0': 'compile-request-v0.2.schema.json',
    'weft-compile/0.3.0': 'compile-request-v0.3.schema.json',
    'weft-compile/0.4.0': 'compile-request-v0.4.schema.json',
}


class Refusal(ValueError):
    pass


@dataclass(frozen=True)
class Config:
    schemas: Path
    maximum_input_bytes: int
    maximum_document_bytes: int

    def __post_init__(self):
        if not isinstance(self.schemas, Path) or not self.schemas.is_absolute():
            raise Refusal('absolute-schema-root')
        for value in (self.maximum_input_bytes, self.maximum_document_bytes):
            if type(value) is not int or not 0 < value <= 64 * 1024 * 1024:
                raise Refusal('finite-byte-limit')


def document(raw):
    try:
        text = raw.decode('utf-8', errors='strict')
    except UnicodeError:
        raise Refusal('utf8-json') from None
    def pairs(items):
        result = {}
        for key, value in items:
            if key in result:
                raise Refusal('duplicate-member')
            result[key] = value
        return result
    def integer(token):
        if len(token.lstrip('-')) > 20:
            raise Refusal('integer-limit')
        return int(token)
    def constant(token):
        raise Refusal('nonfinite-number')
    def decimal(token):
        if len(token) > 256:
            raise Refusal('decimal-limit')
        return Decimal(token)
    return json.loads(text, object_pairs_hook=pairs, parse_int=integer,
                      parse_float=decimal, parse_constant=constant)


def regular_read(path, maximum):
    if any(p.is_symlink() for p in (path, *path.parents)):
        raise Refusal('schema-symlink')
    fd = os.open(path, os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW)
    primary = None
    try:
        if not stat.S_ISREG(os.fstat(fd).st_mode):
            raise Refusal('regular-schema-required')
        result = bytearray()
        while len(result) <= maximum:
            chunk = os.read(fd, min(65536, maximum + 1 - len(result)))
            if not chunk:
                break
            result.extend(chunk)
        if len(result) > maximum:
            raise Refusal('schema-byte-limit')
        return bytes(result)
    except BaseException as error:
        primary = error
        raise
    finally:
        try:
            os.close(fd)
        except BaseException:
            if primary is None:
                raise
            primary.cleanup_failed = True


def validate_envelope(raw, config, validate):
    if len(raw) > config.maximum_input_bytes:
        raise Refusal('input-limit')
    envelope = document(raw)
    if type(envelope) is not dict or set(envelope) != {'scope', 'role', 'requestHex', 'responseHex'}:
        raise Refusal('closed-envelope')
    scope, role = envelope['scope'], envelope['role']
    if (scope, role) not in {('legacy', 'namespace'), ('paths', 'valid'),
                              ('controls', 'valid'), ('controls', 'invalid')}:
        raise Refusal('case-role')
    def decode(name):
        value = envelope[name]
        if type(value) is not str or len(value) % 2 or len(value) > 2 * config.maximum_document_bytes:
            raise Refusal('hex-limit')
        if any(c not in '0123456789abcdef' for c in value):
            raise Refusal('hex-shape')
        return bytes.fromhex(value)
    request_raw, response_raw = decode('requestHex'), decode('responseHex')
    response = document(response_raw)
    if validate('compile-response-v0.4.schema.json', response) is not None:
        raise Refusal('schema-port-return')
    if role == 'invalid':
        return
    request = document(request_raw)
    version = request.get('interfaceVersion') if type(request) is dict else None
    if version not in REQUESTS or (role == 'valid' and version != 'weft-compile/0.4.0') or (role == 'namespace' and version == 'weft-compile/0.4.0'):
        raise Refusal('request-version-role')
    if validate(REQUESTS[version], request) is not None:
        raise Refusal('schema-port-return')


class OfflineSchemas:
    def __init__(self, config):
        from jsonschema import Draft202012Validator
        from referencing import Registry, Resource
        self.config = config
        self.raw = {}
        resources = []
        parsed = {}
        for name, digest in SCHEMAS.items():
            path = config.schemas / name
            raw = regular_read(path, config.maximum_document_bytes)
            if len(raw) > config.maximum_document_bytes or hashlib.sha256(raw).hexdigest() != digest:
                raise Refusal('schema-pin')
            schema = document(raw)
            Draft202012Validator.check_schema(schema)
            self.raw[name] = raw
            parsed[name] = schema
            resources.append((schema['$id'], Resource.from_contents(schema)))
        def no_retrieval(uri):
            raise Refusal('schema-retrieval')
        registry = Registry(retrieve=no_retrieval).with_resources(resources)
        self.validators = {name: Draft202012Validator(schema, registry=registry)
                           for name, schema in parsed.items()}

    def validate(self, name, value):
        self.validators[name].validate(value)

    def close_check(self):
        for name, original in self.raw.items():
            path = self.config.schemas / name
            raw = regular_read(path, self.config.maximum_document_bytes)
            if raw != original:
                raise Refusal('closing-schema-drift')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--schemas', type=Path, required=True)
    parser.add_argument('--maximum-input-bytes', type=int, required=True)
    parser.add_argument('--maximum-document-bytes', type=int, required=True)
    args = parser.parse_args()
    try:
        config = Config(args.schemas, args.maximum_input_bytes, args.maximum_document_bytes)
        raw = sys.stdin.buffer.read(config.maximum_input_bytes + 1)
        schemas = OfflineSchemas(config)
        validate_envelope(raw, config, schemas.validate)
        schemas.close_check()
    except Exception:
        sys.stderr.write('WEFT_CORPUS_SCHEMA_REFUSED\n')
        return 2
    return 0


if __name__ == '__main__':
    sys.exit(main())
