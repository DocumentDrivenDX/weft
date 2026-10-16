# Release work recovery

Original open PR12 head `0e2d76d94f0c1accb449bf9dc703618ceaaa0906`
is preserved byte-for-byte in the source Git graph and retained packaging packet.
Its 0.4.2 version/feature defaults and historical corpus expectations cannot be
merged over the newer compiler. The current release workflow already owns five
platforms and the newer complete native ABI/Date/Decimal/refinement gates.

Ported original project/package MIT-or-Apache licensing, Python distribution
metadata and README, locked native dependency license collection (111 current
dependencies), archive RECORD/ABI/notice and exact installed-wheel checks. The
current five-platform publisher additionally requires five matching packaging
reports; promotion cannot omit them. Existing semantic release gates are retained.

The old six-interpreter-by-five-platform workflow, historical 2181/174 corpus
expectations and 0.4.2-specific publisher remain archived rather than being
silently substituted for current qualification. No claim extends their historic
results to the current sources. Temporal SDK qualification remains HELD.

Final installed packaged ARM wheel passes archive/installed packaging checks, two
negative controls, Rust9 engine tests and the actual Truss native14 campaign.
The release workflow parses with pinned Bun1.4.2; closed11-package boundaries pass.
All local owned native runtimes stopped. Hosted full CI and five-wheel release
remain required before public combined release or Truss default adoption.
