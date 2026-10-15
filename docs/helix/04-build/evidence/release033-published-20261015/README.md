# Published Weft0.4.33 artifacts

Public source tag remains cc943a2f0ad87e3742cccdc171184bad7d4dbf60. The repaired
workflow at e98bd1966c0cc902c09ade0cb6f30d73ff7b365a promotes exactly the five
successful tag-run artifacts after source/repository/tag/job verification.

`verify.py` compares actual downloaded release wheels to the retained tag-run
artifacts and checks public pins, smoke reports, embedded modules/shims and
SHA256SUMS. Paths identify the observed local artifact cohorts. The installed
public ARMwheel passes4 original compiler smoke checks. Its local sourceCommit
is null because the smoke runner uses an exported source tree, while the five
published CI receipts retain exact tagged sourceCommit. No fake source pin is
inserted into the local replay.

This qualifies packaging and finite compiler APIs. It does not qualify protected
native backends, Databricks caller authority, complete aggregate enforcement or
B-008 completion. The immutable original132-case catalog is unchanged.
