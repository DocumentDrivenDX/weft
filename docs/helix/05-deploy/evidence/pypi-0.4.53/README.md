# Weft 0.4.53 public PyPI distribution

[Actions run 40386978860](https://github.com/DocumentDrivenDX/weft/actions/runs/40386978860) passed all three gates: source verification, trusted OIDC publication with exact PyPI-served-byte checks, and a fresh public-PyPI-only installation/import. The main workflow driver was `89882adf09290e53c8f1867d2973653a026ae221`. It consumed the existing `v0.4.53` release at `726a840e956e25217d6aeea2674fdfcf3f342a24`, with `SHA256SUMS` digest `b1785570c5995a52dfa276ed4964c309a96177da31884b97d32bf924487c2d26`.

The three original Linux x86_64, Linux aarch64 and macOS arm64 wheels were uploaded unchanged. No build, repack, new version or tag change occurred. This supplies the exact existing `weft-sql==0.4.53` dependency selected by Ashlar; it does not change Ashlar's pins or establish completion of its separate consumer qualification.

The raw [source](source-receipt.json), [publication](publication-receipt.json) and [consumer](consumer-receipt.json) receipts equal the Actions ZIP members. [verification-record.json](verification-record.json) retains all three API artifact digests, receipt hashes and selected wheel digests. The source ZIP's three wheel hashes match the original release assets; independent public PyPI metadata also reports exactly that file/hash cohort.

This record establishes registry distribution and installed compiler import only. Existing semantic, native and platform qualification limits remain unchanged. It adds no new native/provider campaign or support claim.
