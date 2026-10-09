# Original commerce mathematical Integer local compatibility

This bounded development experiment compiles unchanged core0.8 commerce through
an explicitly opted-in `ashlar.databricks.mathematical-integer` candidate. Logical
Integer keeps its original empty facets and exact literal tokens. DECIMAL(38,0)
is a finite backend representation, not a source width or logical Decimal type.
The experiment does not qualify native Databricks, Unity Catalog, publication,
authorization, ACK or general arbitrary-precision computation. Other existing
profiles refuse this source domain before binding dispatch.

The retained original source/graph fixtures remain in
`tests/fixtures/original-commerce-0.8/`. Actual public UMF at
`c7c95e1c4ea5b72541f47fa0350ca467ff02f395` validates all11 original record values.
Their separate relational/context incompleteness is retained. Every consumed
integer row is independently read with original compiler-emitted snapshot SQL;
raw property bytes, original numeric token/span and row/source/type/revision
identities are retained with actual public Field validation receipts. A pinned
YAML AST only tokenizes strict JSON; its numeric values never enter validation
or arithmetic. Revision/container/carrier checks remain scalar integrity;
exact raw-token capacity is a separate capability obligation. Unknown or
unfulfilled obligations prevent this local runner from executing user SQL.

The final native receipt is `/private/tmp/ashlar-original-commerce08-integer-spark4-d/`:
full requests/artifacts, source requests/receipts, raw materialization and report.
It contains five actual unchanged generated queries: quantity projection10,
literal filter10, self join10/10, COUNT1 and SUM10, with11 original scalar/capacity
checks. The original11 commerce records were written once to a local Delta table;
actual table UUID/version are retained. Compiler-shaped development locators
convey no production manifest authority. Runtime is Spark4.0.1, Delta4.0.0,
JDK21, `local[1]`,512MiB, single shuffle/snapshot partition.

Actual controls demonstrate:

- `9007199254740993` and38-digit maximum preserve exact integer text.
- A source-valid39-digit integer passes actual UMF and native integrity,
  fails only finite capacity, and never executes user SQL.
- A39-digit literal refuses during compile with WFT-CAPABILITY.
- Boolean, null, quoted numeric, fractional and duplicate-member corruption
  refuse source integrity; wrong revision yields integrity1/capacity0.
- Public UMF admits `1e3` as an integer. Native extraction changes its lexical
  carrier to `1000.0`, so this candidate refuses backend token representation;
  the receipt retains public validity rather than declaring invalid source.
- Two individually representable38-digit maximum inputs make SUM fail explicitly
  with WFT-CAPABILITY; no partial/null/rounded result is accepted. Adding negative
  maximum yields the exact remaining maximum in the cancellation control.

After native d, model-pin/field/record identity admission was tightened. All16
retained source requests were replayed read-only under the new guard with
identical original receipts; `source-custody-replay/replay.json` records this
boundary. The original native run did not execute the added guard. Swapped
document/revision/hash/version controls refuse before a receipt is written.

A final profile-scoped SUM correction classifies finite overflow as capability
for bounded Integer and Decimal inputs in this explicit math profile; old
profiles retain their original numeric-domain disposition. Independent compiler
controls cover both families. All19 retained native requests recompile identically
after this correction (`sum-profile-correction-replay.json`); the original native
run did not execute the added bounded-input controls.

The read-only native probe `/private/tmp/ashlar-math-token-probe.json` shows exact
38/39/50-digit extraction, while VARIANT39/50 fallback reports DOUBLE. Arithmetic
uses raw token DECIMAL conversion, never that DOUBLE value. Exponent/fraction and
JSON string/boolean/null observations are retained. The durable reproduction is
`scripts/local/exact_integer_token_spark4.py --output FRESH_RECEIPT`.

Reproduce using explicit local runtime paths and a fresh output:

```sh
CARGO_HOME=/private/tmp/weft-toolchain/cargo RUSTUP_HOME=/private/tmp/weft-toolchain/rustup CARGO_TARGET_DIR=/private/tmp/ashlar-weft-math-build /private/tmp/weft-toolchain/cargo/bin/cargo build -p weft-runtime --features ashlar-databricks-candidate,ashlar-databricks-mathematical-integer
JAVA_HOME=/opt/homebrew/opt/openjdk@21/libexec/openjdk.jdk/Contents/Home SPARK_LOCAL_IP=127.0.0.1 PYSPARK_PYTHON=/Users/erik/Projects/tablespec/.venv/bin/python PYSPARK_DRIVER_PYTHON=/Users/erik/Projects/tablespec/.venv/bin/python /Users/erik/Projects/tablespec/.venv/bin/python scripts/local/original-commerce08-integer-spark4.py --compiler /private/tmp/ashlar-weft-math-build/debug/weft-runtime --output FRESH_OUTPUT --jars /private/tmp/ashlar-delta4-jars --umf /private/tmp/ashlar-umf-dataset-45473
python3 -m unittest discover -s scripts/local -p 'test_*integer*.py'
```

The source checkout and UMF pin are explicit clean dependencies. Python exact
integer decoding retains original text and exact host integer values; it does
not claim a canonical UMF Value wire. Required original commerce/other pack and
engine work remains broader than this finite local scalar proof.
