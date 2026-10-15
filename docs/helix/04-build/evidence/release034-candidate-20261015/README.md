# Weft 0.4.34 pre-release wheel qualification

Build-only run 38817410581 passed all five platform jobs. Each exact installed
wheel passed the four PostgreSQL current-query controls, ten native68 compiler
controls and 32 Databricks bridge controls. Both persisted smoke reports match
source/version/archive/native-extension/shim bytes in every downloaded wheel.
These are candidate artifacts, not a claim about later published archives.

The downloaded macOS ARM wheel also passed fresh native68 public acceptance,
create, ordinary SELECT and COUNT against unchanged original named products.
The ordinary caller lacked direct object SELECT; the disposable PostgreSQL
exporter context exited without cleanup error. This is not protected policy,
ownership or aggregate-security qualification. The frozen Truss source and
3588cf03 runtime manifest are the exact prototype already pinned under
../native068-compiler-20261015/.

The v0.4.34 tag points to 86273dd3; its sole difference from the verified build
source e4ccbd9d is release notes. Release run 38818298197 was queued at capture;
publication and final public archive/extension cohorts require separate proof.
All original 132 B-008 cases remain required; none are promoted by this receipt.
