# Weft 0.4.52 selected-assets evidence

This packet preserves the compiler-only functional receipt and every referenced
log/fixture/result file. It does not claim native execution. The receipt is
SHA-256 `16b987ebb2e0ad5e419676e5a63cd827ca3eb94716fd75d3afff31374c10e441`
and binds source `32ec02f0c3fd24a97b7726500dc5992c6475908d`, the selected macOS
ARM64 wheel SHA-256
`598509e981e3238961d33d430c711b481e430adc945e692a1784ab758f7b6d46`, and UMF
core `0.10.0` / access `0.1.0.dev6` pins.

The packet contains four managed-page compiler cases, two current-read compiler
controls, and two selected public 0.4.51 Source-F controls. See the receipt for
exact commands, fixture/output hashes, and limits. The Source-F comparison covers
selected opaque-payload output and unsupported-version refusal only.

The parent directory also retains the offline verification result for both
selected wheels, including the live run identity from GitHub Actions. Linux
functional/compiler execution is outside that result: the Linux run only checked
archive, RECORD, metadata, ABI tag and notices.

The release publisher expects the macOS evidence staged as
`mac-functional-evidence-0.4.52.tar.gz`, with the top-level folder
`runtime-32ec02f/` preserved. It validates the receipt against every referenced
file before using the selected wheel pair. The Linux artifact run is independently
looked up through the GitHub Actions API by the promotion workflow; the selected
artifact report here is not treated as authentication of that run.
