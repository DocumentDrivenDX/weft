"""Pure fixtures shaped after retained Cargo metadata; no Cargo/network execution."""
import copy
import importlib.util
from pathlib import Path

import pytest

SPEC = importlib.util.spec_from_file_location('archived_resolution',
    Path(__file__).with_name('verify_archived_resolution_0457.py'))
MOD = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MOD)
verify = MOD.verify_archived_resolution

# Independently declared expected fixture, not derived from helper constants.
ROOTS = [('crates/weft-core', 'weft-core', '0.1.0'),
         ('crates/weft-databricks', 'weft-databricks', '0.1.0'),
         ('crates/weft-postgresql', 'weft-postgresql', '0.1.0'),
         ('crates/weft-python', 'weft-python', '0.4.57'),
         ('crates/weft-runtime', 'weft-runtime', '0.1.0'),
         ('vendor/ahash', 'ahash', '0.8.12')]
S = 'e3f38ef6f05ae636d2837f9d69f8d6805222664b'


def metadata(root):
    packages, nodes = [], []
    for path, name, version in ROOTS:
        package_id = 'path+' + (root / path).as_uri() + '#' + version
        packages.append(dict(id=package_id, name=name, version=version, source=None,
                             manifest_path=str(root / path / 'Cargo.toml')))
        nodes.append(dict(id=package_id, dependencies=[], deps=[], features=['default']))
    git = 'git+https://github.com/DocumentDrivenDX/umf.git?rev=' + S
    for name, version, source, package_id in [
        ('umf', '0.10.8', git + '#' + S, git + '#umf@0.10.8'),
        ('bytes', '1.10.1', 'registry+https://github.com/rust-lang/crates.io-index',
         'registry+https://github.com/rust-lang/crates.io-index#bytes@1.10.1')]:
        packages.append(dict(id=package_id, name=name, version=version, source=source,
                             manifest_path='/cargo/cache/' + name + '/Cargo.toml'))
        nodes.append(dict(id=package_id, dependencies=[], deps=[], features=[]))
    def edge(a, b, kind=None, target=None, alias=None):
        nodes[a]['dependencies'].append(packages[b]['id'])
        nodes[a]['deps'].append(dict(name=alias or packages[b]['name'].replace('-', '_'),
            pkg=packages[b]['id'], dep_kinds=[dict(kind=kind, target=target)]))
    for i in [0, 1, 2, 4, 6]:
        edge(3, i)
    edge(6, 7, 'build', 'cfg(unix)', 'renamed_bytes')
    edge(7, 5, 'build')  # ahash is reached through remote packages, not a direct root edge.
    probe = dict(id='path+' + (root / 'tests/probe').as_uri() + '#probe@0.0.0',
                 name='probe', version='0.0.0', source=None,
                 manifest_path=str(root / 'tests/probe/Cargo.toml'))
    packages.append(probe)
    nodes.append(dict(id=probe['id'], dependencies=[], deps=[], features=['dev-only']))
    edge(3, 8, 'dev')
    return dict(version=1, workspace_root=str(root), packages=packages,
                resolve=dict(root=packages[3]['id'], nodes=nodes))


@pytest.fixture
def pair(tmp_path):
    a, b = tmp_path / 'original', tmp_path / 'archive'
    return metadata(a), metadata(b), a, b


def test_relocation_only_and_original_representation(pair):
    a, b, ra, rb = pair
    before = copy.deepcopy((a, b))
    proof = verify(a, b, ra, rb)
    assert proof['checked'] is True
    assert proof['selectedPackageCount'] == 8
    assert proof['actualSelectedRoot'][1:4] == ('weft-python', '0.4.57', 'crates/weft-python/Cargo.toml')
    assert proof['selectedLocalRoots'] == sorted(x[0] for x in ROOTS)
    assert proof['originalMetadataSha256'] != proof['archivedMetadataSha256']
    assert (a, b) == before
    assert S in repr(proof['selectedGraph'])
    assert 'renamed_bytes' in repr(proof['selectedGraph'])


def test_unselected_and_dev_only_probe_do_not_enter_closure(pair):
    a, b, ra, rb = pair
    b['packages'][-1]['version'] = '99.0.0'
    b['resolve']['nodes'][-1]['features'] = ['unselected-drift']
    assert verify(a, b, ra, rb)['selectedPackageCount'] == 8


def test_array_order_is_not_semantics(pair):
    a, b, ra, rb = pair
    b['packages'].reverse()
    b['resolve']['nodes'].reverse()
    for n in b['resolve']['nodes']:
        n['deps'].reverse()
        n['dependencies'].reverse()
    assert verify(a, b, ra, rb)['checked']


@pytest.mark.parametrize('mutation', [
    'missing-root', 'substituted-root', 'missing-local', 'extra-local',
    'local-version', 'git-source', 'registry-source', 'remote-version',
    'edge-kind', 'edge-target', 'edge-alias', 'edge-package', 'missing-edge',
    'feature', 'external-manifest', 'traversal-manifest', 'substituted-local-id',
    'duplicate-package', 'duplicate-node', 'missing-node', 'inconsistent-dependencies',
    'unknown-kind', 'foreign-workspace', 'malformed', 'duplicate-feature',
])
def test_refuses_selected_drift(pair, mutation):
    a, b, ra, rb = pair
    ns, ps = b['resolve']['nodes'], b['packages']
    if mutation == 'missing-root': b['resolve']['root'] = None
    elif mutation == 'substituted-root': b['resolve']['root'] = ps[0]['id']
    elif mutation == 'missing-local': ps.pop(5)
    elif mutation == 'extra-local': ns[3]['deps'][-1]['dep_kinds'][0]['kind'] = None
    elif mutation == 'local-version': ps[0]['version'] = '0.2.0'
    elif mutation == 'git-source': ps[6]['source'] = ps[6]['source'].replace(S, 'f' * 40)
    elif mutation == 'registry-source': ps[7]['source'] += '/different'
    elif mutation == 'remote-version': ps[6]['version'] = '0.10.9'
    elif mutation == 'edge-kind': ns[6]['deps'][0]['dep_kinds'][0]['kind'] = None
    elif mutation == 'edge-target': ns[6]['deps'][0]['dep_kinds'][0]['target'] = 'cfg(windows)'
    elif mutation == 'edge-alias': ns[6]['deps'][0]['name'] = 'other_alias'
    elif mutation == 'edge-package': ns[6]['deps'][0]['pkg'] = ps[0]['id']; ns[6]['dependencies'] = [ps[0]['id']]
    elif mutation == 'missing-edge': ns[3]['deps'].pop(0); ns[3]['dependencies'].pop(0)
    elif mutation == 'feature': ns[6]['features'] = ['new-feature']
    elif mutation == 'external-manifest': ps[0]['manifest_path'] = '/external/weft-core/Cargo.toml'
    elif mutation == 'traversal-manifest': ps[0]['manifest_path'] = str(rb) + '/crates/../crates/weft-core/Cargo.toml'
    elif mutation == 'substituted-local-id': ps[0]['manifest_path'] = str(rb / 'vendor/ahash/Cargo.toml')
    elif mutation == 'duplicate-package': ps.append(copy.deepcopy(ps[0]))
    elif mutation == 'duplicate-node': ns.append(copy.deepcopy(ns[0]))
    elif mutation == 'missing-node': ns.pop(6)
    elif mutation == 'inconsistent-dependencies': ns[3]['dependencies'] = []
    elif mutation == 'unknown-kind': ns[6]['deps'][0]['dep_kinds'][0]['kind'] = 'mystery'
    elif mutation == 'foreign-workspace': b['workspace_root'] = '/foreign'
    elif mutation == 'malformed': del b['resolve']
    elif mutation == 'duplicate-feature': ns[0]['features'] = ['default', 'default']
    with pytest.raises(ValueError): verify(a, b, ra, rb)


def test_refuses_symlink_substitution(pair):
    a, b, ra, rb = pair
    rb.mkdir()
    external = rb.parent / 'outside'
    external.mkdir()
    (rb / 'vendor').symlink_to(external, target_is_directory=True)
    with pytest.raises(ValueError, match='Symlink'):
        verify(a, b, ra, rb)


def test_consistent_git_source_substitution_still_refused(pair):
    a, b, ra, rb = pair
    old = b['packages'][6]['id']
    new = old.replace(S, 'f' * 40)
    b['packages'][6]['id'] = new
    b['packages'][6]['source'] = b['packages'][6]['source'].replace(S, 'f' * 40)
    for n in b['resolve']['nodes']:
        if n['id'] == old: n['id'] = new
        n['dependencies'] = [new if x == old else x for x in n['dependencies']]
        for d in n['deps']:
            if d['pkg'] == old: d['pkg'] = new
    with pytest.raises(ValueError, match='resolution drift'):
        verify(a, b, ra, rb)
