---
ddx:
  id: CONTRACT-006
  type: contract
  activity: design
  status: draft
  authoring:
    home: repo
  links:
    - id: US-008
      kind: informed_by
    - id: TD-008
      kind: informed_by
    - id: ADR-001
      kind: informed_by
---

# CONTRACT-006: Reliability runner

**Type:** CLI, host configuration and diagnostic protocol. **Version:** `weft-runner/1`. **Owner:** Weft owner.

## Purpose

Make development and CI qualification reproducible, bounded and inspectable without exposing compiler inputs or subprocess text.

## Scope and Boundaries

Host-only Python 3.12 tools own configuration, process execution and safe diagnostic files. Compiler APIs MUST continue to receive explicit inputs. The runner MUST NOT authorize public security compilation, native database execution or credential provisioning.

## Normative Surface

Configuration MUST resolve typed defaults, optional JSON file, allowlisted `WEFT_*` environment, then explicit CLI flags. Unknown file keys, malformed values or unavailable required handles MUST refuse before run-directory creation. `--config` selects an optional file; each below field has a corresponding kebab-case flag and uppercase `WEFT_` environment variable. Configuration input MUST be a regular file, with symlinks/special streams refused before reading. JSON file size MUST be at most 64 KiB; duplicate keys MUST refuse. Empty values MUST NOT silently fall back.

| Field | Shape | Default / rule |
|---|---|---|
| cargo | absolute executable path | required operator handle; `cargo --version` MUST identify 1.90.0 |
| rustup_home, cargo_home | absolute existing directories | required operator handles; command environment MUST use these explicitly |
| temp_root, output_root | absolute directories | required operator locations, no committed defaults; create only after validation |
| timeout_seconds | finite number, 1–3600 seconds | 1800 |
| record_limit | integer, 4–256 records per run | 256 |
| queue_limit | integer, 1–256 records | 16 |
| retained_runs | integer, 1–32 runs | 8 |
| export_timeout_seconds | finite number, 0.05–10 seconds | 2 |
| shutdown_timeout_seconds | finite number, 0.1–15 seconds | 3 |
| endpoint | optional HTTP(S) URL ending `/v1/logs` | absent; no userinfo, query or fragment; no committed destination |

Tool identity MUST record the selected Cargo path's SHA-256, reported Cargo/Rust versions, and repository `rust-toolchain.toml` SHA-256. Rust MUST report 1.90.0. The effective command environment MUST pin RUSTC to the validated sibling executable and disable compiler/workspace wrappers, overriding ambient and Cargo build configuration. Version probes MUST use the bounded process-group boundary, drain without retaining more than256 bytes, reject larger output, and match the entire single-line pinned version grammar. Executable hashing MUST stream bounded chunks. Operational path values MUST NOT appear in diagnostic event bodies/attributes. The environment MAY retain platform linker configuration; compiler modules MUST NOT read runner configuration.

Canonical temp/output roots MUST be outside the source checkout, including symlink aliases; overlap MUST refuse before any directory creation/copy. Mutation execution MUST use fresh isolated copies and a separate configured target directory. Replacement cardinality MUST equal one. Each selected original-source baseline MUST first exit0 with exactly one executed/passed test and zero failed/ignored tests under the same command/environment/filter; its outcome MUST be retained. Qualification MUST require exit 101, exactly one selected test, its failed outcome, a panic and every mutation-specific expected signature. Missing tests, wrong signatures, compilation failures and unapplied mutations MUST fail. Python optimization MUST NOT alter these decisions. Failure MUST NOT produce a passed summary. Raw streams MUST be drained with bounded working memory and MUST NOT be persisted or echoed by default.

## Precedence and Compatibility

`weft-runner/1` is separate from compile transport and OTel versions. Unknown versions and keys refuse. Paths and exporter destinations are operator handles. Existing B007 mutation logs remain historical evidence; new execution produces separate run evidence and never rewrites them.

## Error Semantics

| Condition | Outcome | Recovery |
|---|---|---|
| invalid configuration/tool identity | safe `configuration` failure, nonzero exit before run | correct explicit settings |
| missing replacement, test or expected failure | safe `mutation` failure, nonzero exit | inspect selected source/test locally |
| process timeout | terminate process group, bounded drain/kill, failed run | fix cause or increase admitted timeout |
| capture/export loss | manifest incomplete/loss counts, nonzero qualification result | restore capture/export and rerun |
| invalid/expired retrieval cursor | explicit refusal | start a fresh snapshot |

## Examples

```text
python tests/qualify-and-evolve/source-mutations.py --cargo /opt/toolchain/bin/cargo --rustup-home /opt/toolchain/rustup --cargo-home /opt/toolchain/cargo --temp-root /work/tmp --output-root /work/runs
```

## Telemetry and Diagnostic Surfaces

R4 implements the following surface; receiver/negative-control evidence MUST pass before claiming diagnostic qualification. Adopt OTel Logs Data Model 1.61.0, resource conventions 1.44.0 and OTLP 1.11.0 HTTP JSON; pin the actual Python bridge, SDK and receiver versions in executable dependencies. Python `logging` is the established emission path. A single bridge MUST feed independent safe file/console/export projections; never double-ingest file records and logger exports.

A local record MUST contain `schema_version:1`, `timestamp_ns`, `observed_timestamp_ns` (decimal Unix nanosecond strings), `severity_number` (INFO9/WARN13/ERROR17), `severity_text`, `event_name`, safe catalog `body`, fixed `resource` (`service.name=weft-reliability`, `service.version=0.1.0`, `deployment.environment.name=development`), fixed `scope` (`weft.reliability`, version1), and typed `attributes`: `weft.run.id`, `weft.attempt.id`, `weft.sequence`, `weft.operation`, `weft.outcome`, `weft.duration_ms`, `weft.dropped` as applicable. Operation names MUST come from a repository-owned enum. Body text MUST come from a fixed event catalog: `run.started`, `operation.started`, `operation.completed`, `operation.failed`, `capture.loss`, `run.completed`, `run.failed`. Arbitrary message, argument, source, parameter, query, model, binding, environment and raw subprocess fields MUST be discarded before every sink. Record size MUST be at most 4096 UTF-8 bytes.

The exact wire mapping is Timestamp→`timeUnixNano`, ObservedTimestamp→`observedTimeUnixNano`, Severity→`severityNumber`/`severityText`, Body→`body.stringValue`, EventName→`eventName`, Resource→`resourceLogs[].resource.attributes`, Scope→`scopeLogs[].scope`, Attributes→`logRecords[].attributes`. Typed strings, booleans and integers map to `stringValue`, `boolValue`, `intValue` (decimal string). Local JSONL MUST NOT be called OTLP. Optional `trace_id`/`span_id`/`trace_flags` map to `traceId`/`spanId`/`flags` only from valid actual SDK context; otherwise omit. Span without trace MUST refuse; namespaced run IDs MUST never become trace IDs. The pilot MUST emit a real SDK span and export it to the receiver separately from logs, plus an event outside any span.

Each run directory MUST have restrictive 0700 access and files0600. An atomic `manifest.json` MUST declare run/attempt identity, input revision, capture start/end, safe source name, access/retention, unsampled local capture, emitted/captured/dropped counts, export successes/failures, flush state and outcome (`pending`, `passed`, `failed`, `incomplete`). Pending manifests MUST remain distinguishable after interruption; atomic rename is not an fsync durability claim. At most configured retained closed runs plus currently pending runs MAY exist. The runner MUST rotate/delete only its own validated closed run directories; never follow symlinks or delete arbitrary caller files. Each run's single JSONL source MUST be bounded by record limit×4096. Queue overflow, record overflow, exporter timeout/outage, denied local capture and shutdown timeout MUST produce visible counts/incomplete outcome. Console uses stderr and milestones only; stdout remains protocol/output.

Retrieval MUST be read-only and scoped to one explicitly selected run directory owned by the caller. It MUST reject symlinks, malformed manifest/records and unknown fields/versions. Page input limits: at most50 records, 64KiB output, 1MiB scanned input and one second; exact source line references and sequence ordering. The first page MUST fix a byte/sequence snapshot. Appended events in a pending run MUST NOT enter an existing snapshot. Cursor binds run ID, file identity, snapshot length, offset and expiry (60 seconds); replacement/rotation/deletion/truncation/expiry MUST refuse. Output MUST include records, continuation, source references, coverage/capture/outcome/loss and truncation. Zero matches means only zero matches in that recorded coverage. No cross-process total timestamp order is promised.

Pilot ground truth: identify failed operation from safe records with100% accuracy for three scripted interleaved attempts, at most two pages and128KiB retrieved output. The pilot MUST report emitted/captured/received unique counts, elapsed export/shutdown bounds and run overhead. Pass limits: no duplicate/sentinel leakage; zero unexplained loss; each unavailable export completes within configured operation/shutdown deadlines plus one second; retrieval respects every bound. Timing is scoped to the recorded synthetic pilot and MUST NOT be generalized as a production performance/SLO claim.

## Non-Normative Notes

This is a development/CI runner, with optional explicit local receiver/exporter use. Public backend selection, native authorization, service operation and full security acceptance remain separately governed.

### R4 executable schema and host protocol

The host dependency lock pins Python OTel API/SDK/proto/common encoder1.45.1,
semantic conventions0.66b1, protobuf7.36.2 and typing-extensions4.16.0. The real
SDK encoder is version-pinned because its internal API is used. HTTP JSON MUST
encode enum values as integers, int64 values as decimal strings and trace/span
IDs as hexadecimal. The configured `/v1/logs` endpoint also determines the same
origin `/v1/traces` endpoint for actual safe SDK span export. Transport MUST NOT
retry, follow redirects or adopt ambient HTTP proxies. Any `OTEL_*` environment
override MUST refuse through the configuration provider before output creation.
SDK/proto/common-encoder version mismatch MUST refuse. Python's standard logger
bridge MUST never call formatting on arbitrary message/arguments.

`manifest.json` has exactly schema_version1, owner=weft-reliability, run_id,
attempt_id (32hex each), operation, revision (latest emitted sequence), started_ns,
ended_ns (null only while pending), outcome, counts, loss, access=owner-only,
retention_runs, sources=[events.jsonl], export_enabled. Counts are emitted,
captured, dropped, export_queued, exported, export_failed, export_dropped,
export_pending, operations, operations_failed. Local record counts differ from
export request counts: each log or span export is one request. Export pending
counts unresolved requests at the retained manifest snapshot, including work
still running after a shutdown timeout. Loss reasons are rejected, capture_failed,
record_limit, queue_limit, export_failed, shutdown_timeout. Failed operation events
MUST make the final run failed unless capture/export loss makes it incomplete.
Counters are unsampled. A local capture gap MUST remain visible in loss and in
monotonic sequence gaps; retrieval MUST retain exact physical line references.

A run owns events.jsonl, manifest.json, cursor.key and optional qualification.json.
The32-byte owner-only cursor key authenticates cursor state; it MUST NOT appear
in records, console, retrieval or exports. Retrieval MUST NOT write a key or
mutate any file. Duplicate JSON keys MUST refuse. Retention deletes only validated
closed run directories containing these known files and no symlinks. At most
retained_runs pending runs are admitted; a full pending allowance refuses new
runs. Crashed pending runs remain explicitly pending for owner inspection; no
silent recovery or automatic deletion is claimed. A run-shaped directory with a missing/malformed manifest or unsafe entry MUST refuse further creation for owner inspection. A closed capture-denied run MAY lack events.jsonl only when captured=0 and capture_failed is explicit; retention may delete its validated remaining regular files. Existing pre-R4 qualification
reports remain historical records outside this rotation policy. Atomic manifest
replacement supplies lifecycle visibility, without an fsync/durable-storage claim.

Run creation and final outcome produce fixed safe stderr milestones. stdout stays
one protocol result/page. Arbitrary errors MUST become a fixed safe message and
nonzero exit. A failed mutation runner or incomplete diagnostics MUST NOT emit a
passed qualification file. `scripts/reliability/retrieve.py --run-directory <run>`
returns one bounded page; pass its cursor explicitly to continue. Explicit run
selection permits owner-scoped inspection and does not grant native authority.

### R4 review refinements

To reconcile full snapshot integrity with the1MiB retrieval input budget,
record_limit is bounded4..256. Whole capture files are at most256×4096 bytes.
Each page hashes the full recorded snapshot prefix within that budget, validates
terminal captured counts/sequence and refuses closed truncation or rewritten
prefixes. Pending appends preserve the existing prefix digest. Closed incomplete
captures may have a partial tail only when capture_failed explicitly records it;
complete terminal records still reconcile exactly. First pages fix both byte and
end-sequence boundaries. Read-time file mutation refuses rather than guessing.

An owner-scoped `.weft-run.lock` uses nonblocking POSIX flock for a single active
writer per output root, across threads/processes. Admission and retention execute
under that ownership until close. Crashed pending runs remain for inspection and
count against the allowance; malformed own run-shaped entries refuse creation.
A retention failure makes the manifest incomplete with retention_failed. This is
a macOS/Linux host contract; Windows runner diagnostics remain unqualified.

Span export MUST strip inherited trace-state text while preserving actual SDK
trace/span IDs and flags. Local/retrieved contexts MUST reject all-zero IDs.
Exporter HTTP runs in a killable bounded host subprocess, including DNS/connect,
headers and response drain under the same total deadline. No arbitrary transport
output is retained. Shutdown cancellation prevents continued queue processing
past the bounded flush budget; unresolved requests remain export_pending at the
terminal snapshot. Requests without acknowledged success are export_failed;
that does not assert a receiver never accepted their bytes.

Manifests separately retain input_revision and flush_state. Input revision is
exactly kind=sha256, value64hex, scope=compiler-and-host-source, or explicit
kind=unknown/value=null/scope=not-supplied. Validate before creating outputs.
Production entrypoints supply a streamed source fingerprint over Rust/Cargo,
host-runner sources/requirements and module map. Vendored sources, spikes outside
the crates tree, embedded non-Rust inputs, fixtures and native evidence custody
are excluded here and governed by R6 full qualification. Flush state is pending,
complete, failed or timeout. Terminal outcome vocabulary is passed, failed or
incomplete; completed denotes an event/milestone, not another manifest state.

Export success is deliberately scoped to HTTP200, application/json and a parsed
empty object (partialSuccess unset). Duplicate keys, empty/malformed bodies,
wrong content type and every partialSuccess/warning refuse qualification without
retrying. This follows the full-success wire contract in
[OTLP1.11.0](https://opentelemetry.io/docs/specs/otlp/); it is not a claim to
implement all collector response extensions, compression or retry behavior.
