# Weft 0.4.51 final Linux parity gate

This packet records the successful Linux entity/facts native and Chromium
producer gate, run `39822650070` at source commit
`cc694344da31af76ef6b97e572d20a8347989bdd`. It is a separate completed run
from the earlier Linux parity run `39811016270` at `c5a4537...`; neither should
be relabeled as the other. `producer-custody.json` preserves the original
producer paths, exact byte counts, SHA-256 values and run identities.

The run includes the 33-case entity and 5-case facts native reports and their
Chromium reports/receipts. The native report SHA-256 values are
`ff8803dda03a4ef376e44984c1a010ead8065b817a002d6178dba8b0343256bf` (entity)
and `c45b385d21c64d4abec74849a7cad50229a6eaa054b2a14a192b13c2566b8982`
(facts); they match the original selected Mac parity report bytes. The browser
receipts report expected/actual byte parity. They use the 28,344,868-byte Linux
debug-profile WASM module, SHA-256
`b1a573acb81e45e345357dc8b6820d6765479c17827b9d7a8b689957c141266f`, distinct
from the selected Mac release-profile WASM module
`73012141c317c5eea4feabe81f448b8df832822267cd3ca83b4b1420fe875d30`.

This packet only closes the named Linux parity producer gate. The separate full
workspace conformance run `39822650250` remains in progress and is not covered
by this evidence. The prior c5 producer snapshot and custody packet remain
unchanged in `../release051-final-packet-20261022/`.
