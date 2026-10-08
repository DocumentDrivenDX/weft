"""Convert an authored JSON case array to JSON Lines without changing records.
Use before browser-check for corpora exceeding JavaScript's single-string limit.
"""
import json, sys
from pathlib import Path
source, destination = map(Path, sys.argv[1:])
with source.open() as stream:
    records = json.load(stream)
assert isinstance(records, list)
with destination.open('w') as stream:
    for record in records:
        line = json.dumps(record, ensure_ascii=False, separators=(',', ':'))
        assert json.loads(line) == record
        stream.write(line + '\n')
print(f'{len(records)} records converted without changing values')
