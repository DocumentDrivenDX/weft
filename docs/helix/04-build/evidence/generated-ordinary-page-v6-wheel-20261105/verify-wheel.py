import pathlib
import weft

base = pathlib.Path(__file__).parent
request5 = (base / 'request-v5.json').read_text().strip()
request6 = (base / 'request-v6.json').read_text().strip()
out5 = weft.prepare_generated_read_algebra_v5_json(request5)
out6 = weft.prepare_generated_read_algebra_v6_json(request6)
(base / 'wheel-issued-v5.json').write_text(out5 + '\n')
(base / 'wheel-issued-v6.json').write_text(out6 + '\n')
expected5 = pathlib.Path('docs/helix/04-build/evidence/generated-connection-v5-20261104/issued-v5.json').read_text().strip()
expected6 = pathlib.Path('docs/helix/04-build/evidence/generated-ordinary-page-v6-correction-20261105/issued-v6.json').read_text().strip()
assert out5 == expected5, 'wheel /5 output differs from frozen source vector'
assert out6 == expected6, 'wheel /6 output differs from accepted source vector'
try:
    weft.prepare_generated_read_algebra_v6_json('{"version":"invalid"}')
except Exception:
    pass
else:
    raise AssertionError('invalid /6 request was accepted')
(base / 'verification-result.txt').write_text('PASS: installed 0.4.65.dev1 wheel exports v6; /6 equals accepted source bytes; /5 equals frozen source bytes; invalid /6 request refused.\n')
print((base / 'verification-result.txt').read_text(), end='')
