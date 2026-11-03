"""Pure selected Cargo-resolution correspondence; never invokes Cargo or qualifies bytes.

Callers supply the actual --locked metadata for each tree. Archive inventory,
source/lock/resource custody and metadata acquisition remain caller obligations.
"""
from __future__ import annotations

import hashlib
import json
from pathlib import Path, PurePosixPath

LOCAL_ROOTS = {
    'crates/weft-core': ('weft-core', '0.1.0'),
    'crates/weft-databricks': ('weft-databricks', '0.1.0'),
    'crates/weft-postgresql': ('weft-postgresql', '0.1.0'),
    'crates/weft-python': ('weft-python', '0.4.57'),
    'crates/weft-runtime': ('weft-runtime', '0.1.0'),
    'vendor/ahash': ('ahash', '0.8.12'),
}


def _require(condition, message):
    if not condition:
        raise ValueError(message)


def _digest(value):
    return hashlib.sha256(json.dumps(value, sort_keys=True, separators=(',', ':'),
                                    ensure_ascii=False, allow_nan=False).encode()).hexdigest()


def _absolute(value):
    _require(isinstance(value, (str, Path)), 'Path must be a string or Path')
    raw = str(value)
    p = PurePosixPath(raw)
    _require(p.is_absolute() and str(p) == raw and '..' not in p.parts
             and '\\' not in raw and '\x00' not in raw, 'Noncanonical absolute path')
    # Do not resolve away symlinks. Check existing components, even for retained
    # metadata whose original hosted checkout no longer exists on this machine.
    q = Path('/')
    for part in p.parts[1:]:
        q /= part
        _require(not q.is_symlink(), 'Symlink path substitution')
    return p


def _index(items, label):
    _require(isinstance(items, list), f'Malformed {label}')
    result = {}
    for item in items:
        _require(isinstance(item, dict) and isinstance(item.get('id'), str)
                 and item['id'] not in result, f'Duplicate/malformed {label} id')
        result[item['id']] = item
    return result


def _closure(metadata, root):
    root = _absolute(root)
    _require(metadata.get('version') == 1, 'Unsupported metadata version')
    _require(metadata.get('workspace_root') == str(root), 'Foreign workspace root')
    packages = _index(metadata['packages'], 'package')
    resolution = metadata['resolve']
    nodes = _index(resolution['nodes'], 'resolve node')
    selected_root = resolution.get('root')
    _require(selected_root in packages and selected_root in nodes,
             'Missing actual selected root')
    normalized = {}
    local_paths = set()
    identities = {}

    def identity(package_id):
        if package_id in identities:
            return identities[package_id]
        _require(package_id in packages and package_id in nodes, 'Missing selected dependency')
        p = packages[package_id]
        name, version, source = p['name'], p['version'], p['source']
        _require(isinstance(name, str) and isinstance(version, str), 'Invalid package identity')
        if source is None:
            manifest = _absolute(p['manifest_path'])
            _require(manifest.is_relative_to(root), 'External local manifest')
            relative = manifest.relative_to(root).as_posix()
            local = str(PurePosixPath(relative).parent)
            _require(PurePosixPath(relative).name == 'Cargo.toml'
                     and LOCAL_ROOTS.get(local) == (name, version), 'Unknown/substituted local root')
            prefix = 'path+' + Path(str(manifest.parent)).as_uri() + '#'
            _require(package_id.startswith(prefix), 'Local package id/manifest mismatch')
            suffix = package_id[len(prefix):]
            _require(suffix in (version, name + '@' + version), 'Invalid local package id')
            _require(local not in local_paths, 'Duplicate selected local root')
            local_paths.add(local)
            key = ('local', name, version, relative, suffix)
        else:
            _require(isinstance(source, str) and source.startswith(('registry+', 'git+'))
                     and package_id == source.split('#', 1)[0] + '#' + name + '@' + version,
                     'Remote source/id mismatch')
            # Preserve the complete registry/git source, revision and ID. No URL
            # normalization, source substitution or registry equivalence rules.
            key = ('remote', name, version, source, package_id)
        identities[package_id] = key
        return key

    todo = [selected_root]
    visited = set()
    while todo:
        package_id = todo.pop()
        if package_id in visited:
            continue
        key = identity(package_id)
        visited.add(package_id)
        node = nodes[package_id]
        features = node['features']
        _require(isinstance(features, list) and all(isinstance(f, str) for f in features)
                 and len(features) == len(set(features)), 'Invalid selected features')
        _require(isinstance(node['deps'], list), 'Invalid dependency edges')
        edges = []
        all_ids = set()
        for dep in node['deps']:
            _require(isinstance(dep.get('name'), str) and dep['name']
                     and isinstance(dep.get('pkg'), str), 'Malformed dependency alias/id')
            all_ids.add(dep['pkg'])
            kinds = dep['dep_kinds']
            _require(isinstance(kinds, list) and kinds, 'Missing dependency kinds')
            seen = set()
            for item in kinds:
                kind, target = item['kind'], item['target']
                _require(kind in (None, 'build', 'dev')
                         and (target is None or isinstance(target, str)), 'Unknown kind/target')
                _require((kind, target) not in seen, 'Duplicate dependency kind/target')
                seen.add((kind, target))
                if kind == 'dev':
                    continue
                edges.append((dep['name'], identity(dep['pkg']), kind, target))
                todo.append(dep['pkg'])
        dependencies = node['dependencies']
        _require(isinstance(dependencies, list) and all(isinstance(x, str) for x in dependencies)
                 and len(dependencies) == len(set(dependencies))
                 and set(dependencies) == all_ids, 'Inconsistent resolve dependency lists')
        _require(len(edges) == len(set(edges)), 'Duplicate selected dependency edge')
        _require(key not in normalized, 'Duplicate selected package identity')
        normalized[key] = {'identity': key, 'features': sorted(features),
                           'edges': sorted(edges, key=lambda edge: json.dumps(edge))}
    root_identity = identities[selected_root]
    _require(root_identity[:4] == ('local', 'weft-python', '0.4.57',
                                   'crates/weft-python/Cargo.toml'), 'Substituted actual Python root')
    _require(local_paths == set(LOCAL_ROOTS), 'Incomplete selected six local roots')
    return root_identity, sorted(normalized.values(), key=lambda item: item['identity'])


def verify_archived_resolution(original_metadata, archived_metadata, original_root, archive_root):
    """Return checked proof or ValueError; only local checkout relocation is allowed.

    Full input digests preserve original source representation (including ignored
    dev/unselected metadata). They differ on relocation; selected graph digests
    must match. This proof makes no archive-byte or real Cargo-execution claim.
    """
    try:
        original = _closure(original_metadata, original_root)
        archived = _closure(archived_metadata, archive_root)
        _require(original == archived, 'Selected normal/build resolution drift')
        return {'schema': 'weft-archived-resolution/1', 'checked': True,
                'originalMetadataSha256': _digest(original_metadata),
                'archivedMetadataSha256': _digest(archived_metadata),
                'selectedGraphSha256': _digest(original),
                'actualSelectedRoot': original[0], 'selectedPackageCount': len(original[1]),
                'selectedLocalRoots': sorted(LOCAL_ROOTS), 'selectedGraph': original[1],
                'scope': 'metadata correspondence only; no producer qualification'}
    except (KeyError, TypeError, AttributeError, IndexError, RecursionError) as exc:
        raise ValueError('Malformed Cargo metadata') from exc
