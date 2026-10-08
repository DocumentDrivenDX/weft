"""Reconcile saved same-transaction session observations; no native execution.
@covers US-006-AC1 @covers US-006-AC2
"""
import gzip, hashlib, json, re
from pathlib import Path
from evidence_audit import strict
ROOT = Path(__file__).resolve().parents[2]
EVIDENCE = ROOT / 'docs/helix/04-build/evidence'

def audit(compressed, summary, original):
    assert hashlib.sha256(compressed).hexdigest() == summary['gzipSha256'], 'archive digest'
    raw = gzip.decompress(compressed)
    assert hashlib.sha256(raw).hexdigest() == summary['uncompressedSha256'], 'payload digest'
    assert len(raw) == summary['bytes'], 'payload size'
    reports = strict(raw.decode())
    expected = {r['id']: r for r in original}
    assert len(expected) == len(original) == len(reports) == 76, 'case count'
    seen = set()
    keys = {'engine','serverVersion','serverEncoding','clientEncoding','standardConformingStrings','transactionIsolation','lcCollate','lcCtype'}
    for report in reports:
        case = report['id']
        assert case in expected and case not in seen, 'case identity'
        seen.add(case)
        for key in ['raw','response','rows','topology']:
            assert report[key] == expected[case][key], 'prior receipt agreement'
        session = report['session']
        assert set(session) == keys and all(isinstance(v,str) and v for v in session.values()), 'session shape'
        assert session == summary['session'], 'session agreement'
        assert session['serverEncoding'] == session['clientEncoding'] == 'UTF8', 'encoding'
        assert session['standardConformingStrings'] == 'on', 'string setting'
        assert session['transactionIsolation'] == 'repeatable read', 'isolation'
        assert session['lcCollate'] == session['lcCtype'] == 'C', 'locale'
        assert re.fullmatch('[0-9a-f]{64}', report['executedSqlSha256']), 'SQL digest shape'
    assert seen == set(expected), 'case coverage'
    return {'status':'passed','cases':76,'scope':'Retained native session/result reconciliation; no new engine execution or support promotion.'}

def inputs():
    directory = EVIDENCE / 'B-007-truss-session-native'
    original = strict(gzip.decompress((EVIDENCE / 'B-007-truss-application-native/reports.json.gz').read_bytes()).decode())
    return (directory/'reports.json.gz').read_bytes(), strict((directory/'summary.json').read_text()), original

if __name__ == '__main__':
    print(json.dumps(audit(*inputs())))
