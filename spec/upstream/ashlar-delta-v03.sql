-- Proposed ashlar-delta/0.3, not an in-place migration or a support claim.
-- Adds exact native cursor and direct source-record references to 0.2.
-- Nullable fields accommodate explicitly qualified legacy fixtures.
-- Native-source profiles require cursor/reference and reject scalar flattening.
-- Source delivery ID is resolved by (source_feed,source_epoch,source_delivery_id).
-- UC managed Delta is selected; derived metadata does not redefine producer fields.
-- Edge eligibility statistics below reflect r130/r131/r133 native tuning;
-- the revised complete 13-table DDL has not been re-executed as a package.
-- Candidate L: liquid clustering. Do not combine with PARTITIONED BY or ZORDER.
CREATE TABLE object_current (
  source_system STRING NOT NULL, type_id BIGINT NOT NULL, id BIGINT NOT NULL,
  logical_key_json STRING NOT NULL, schema_revision STRING NOT NULL,
  entity_version BIGINT NOT NULL, props_json STRING NOT NULL,
  retained_json STRING NOT NULL, root_id BIGINT,
  source_feed STRING NOT NULL, source_epoch STRING NOT NULL,
  source_position BIGINT, published_at TIMESTAMP NOT NULL,
  lookup_hash STRING NOT NULL, apply_batch_id STRING,
  source_cursor_json STRING, source_delivery_id STRING
) USING DELTA CLUSTER BY (lookup_hash)
TBLPROPERTIES ('delta.dataSkippingStatsColumns'='lookup_hash,source_system,type_id,id',
'delta.targetFileSize'='67108864','delta.parquet.compression.codec'='zstd');

CREATE TABLE edge_current (
  source_system STRING NOT NULL, rel_type_id BIGINT NOT NULL, id BIGINT NOT NULL,
  source_type BIGINT NOT NULL, source_id BIGINT NOT NULL,
  target_type BIGINT NOT NULL, target_id BIGINT NOT NULL,
  schema_revision STRING NOT NULL, entity_version BIGINT NOT NULL,
  props_json STRING NOT NULL, retained_json STRING NOT NULL, order_key STRING,
  source_feed STRING NOT NULL, source_epoch STRING NOT NULL,
  source_position BIGINT, published_at TIMESTAMP NOT NULL,
  lookup_hash STRING NOT NULL, apply_batch_id STRING,
  source_cursor_json STRING, source_delivery_id STRING
) USING DELTA CLUSTER BY (lookup_hash)
TBLPROPERTIES ('delta.dataSkippingStatsColumns'='lookup_hash,source_system,rel_type_id,id,entity_version,apply_batch_id',
'delta.targetFileSize'='67108864','delta.parquet.compression.codec'='zstd');

CREATE TABLE property_journal (
  source_system STRING NOT NULL, entity_kind STRING NOT NULL,
  type_id BIGINT NOT NULL, id BIGINT NOT NULL, property_id BIGINT,
  entity_version BIGINT NOT NULL, operation STRING NOT NULL,
  old_present BOOLEAN NOT NULL, old_json STRING,
  new_present BOOLEAN NOT NULL, new_json STRING,
  schema_revision STRING NOT NULL, source_feed STRING NOT NULL,
  source_epoch STRING NOT NULL, source_position BIGINT,
  event_ordinal BIGINT NOT NULL, source_time_text STRING,
  published_at TIMESTAMP NOT NULL, apply_batch_id STRING,
  source_cursor_json STRING, source_delivery_id STRING
) USING DELTA CLUSTER BY (source_feed, source_epoch, source_position, id)
-- r100: batch statistics improve pinned-version journal file pruning;
-- no measured freshness gain. Backfill existing statistics separately.
TBLPROPERTIES ('delta.dataSkippingStatsColumns'='source_feed,source_epoch,source_position,id,apply_batch_id');

CREATE TABLE tombstone (
  source_system STRING NOT NULL, entity_kind STRING NOT NULL,
  type_id BIGINT NOT NULL, id BIGINT NOT NULL, entity_version BIGINT NOT NULL,
  source_feed STRING NOT NULL, source_epoch STRING NOT NULL,
  source_position BIGINT,
  source_cursor_json STRING, source_delivery_id STRING
) USING DELTA CLUSTER BY (source_system, type_id, id);

-- One row is the complete publication descriptor. Updated by a single publisher
-- only after table-version validation; never infer multi-table commit atomicity.
CREATE TABLE publication_manifest (
  publication_id STRING NOT NULL, profile_version STRING NOT NULL,
  table_versions_json STRING NOT NULL, source_progress_json STRING NOT NULL,
  schema_revisions_json STRING NOT NULL, validation_report_json STRING NOT NULL,
  recorded_at TIMESTAMP NOT NULL
) USING DELTA;

-- Example hot typed read projection: rebuildable, not a second source of truth.
CREATE TABLE node_type_a (
  node_key STRING NOT NULL, source_system STRING NOT NULL,
  type_id BIGINT NOT NULL, id BIGINT NOT NULL,
  group_value STRING, group_present BOOLEAN NOT NULL,
  rank_value BIGINT, rank_present BOOLEAN NOT NULL,
  props_json STRING NOT NULL, retained_json STRING NOT NULL
) USING DELTA CLUSTER BY (id, group_value)
TBLPROPERTIES ('delta.dataSkippingStatsColumns'='node_key,source_system,id,group_value,rank_value');

CREATE TABLE edge_ab (
  edge_key STRING NOT NULL, source_system STRING NOT NULL,
  rel_type_id BIGINT NOT NULL, id BIGINT NOT NULL,
  src STRING NOT NULL, dst STRING NOT NULL,
  source_id BIGINT NOT NULL, target_id BIGINT NOT NULL,
  score DOUBLE, score_present BOOLEAN NOT NULL,
  props_json STRING NOT NULL, retained_json STRING NOT NULL
) USING DELTA CLUSTER BY (source_id, target_id)
TBLPROPERTIES ('delta.dataSkippingStatsColumns'='edge_key,src,dst,source_id,target_id,score');

-- Native singleton read: use the manifest's fixed table version (literal supplied
-- by validated execution adapter), exact source/type/id; no Fabric dependency.
-- SELECT * FROM object_current VERSION AS OF 42
-- WHERE source_system='truss-pilot' AND type_id=1 AND id=123;

-- Candidate Z is a DIFFERENT table; type partition only when each partition is
-- sufficiently large. Sparse types remain shared, not one tiny partition each.
-- CREATE TABLE object_current_z (...) USING DELTA PARTITIONED BY (type_id);
-- OPTIMIZE object_current_z ZORDER BY (source_system, id);
-- CREATE TABLE edge_current_z (...) USING DELTA PARTITIONED BY (rel_type_id);
-- OPTIMIZE edge_current_z ZORDER BY (source_id, target_id);
-- Separate experiment: source-clustered narrow forward adjacency and
-- target-clustered reverse adjacency, carrying edge identity (never dedup pairs).


-- Optional derived traversal tables, built at the same publication boundary.
-- One row per independent edge. Never DISTINCT endpoint pairs.
CREATE TABLE adjacency_forward (
  source_system STRING NOT NULL, rel_type_id BIGINT NOT NULL, edge_id BIGINT NOT NULL,
  source_type BIGINT NOT NULL, source_id BIGINT NOT NULL,
  target_type BIGINT NOT NULL, target_id BIGINT NOT NULL,
  structural_version BIGINT NOT NULL
) USING DELTA CLUSTER BY (source_system, source_type, source_id, rel_type_id)
TBLPROPERTIES ('delta.dataSkippingStatsColumns'='source_system,source_type,source_id,rel_type_id,target_type,target_id,edge_id');

-- Materialize only if reverse traversal warrants its storage/refresh cost.
CREATE TABLE adjacency_reverse (
  source_system STRING NOT NULL, rel_type_id BIGINT NOT NULL, edge_id BIGINT NOT NULL,
  source_type BIGINT NOT NULL, source_id BIGINT NOT NULL,
  target_type BIGINT NOT NULL, target_id BIGINT NOT NULL,
  structural_version BIGINT NOT NULL
) USING DELTA CLUSTER BY (source_system, target_type, target_id, rel_type_id)
TBLPROPERTIES ('delta.dataSkippingStatsColumns'='source_system,target_type,target_id,rel_type_id,source_type,source_id,edge_id');

-- Publisher enforces direction in out/in and nonnegative counts; native CHECK
-- was rejected in this workspace DDL probe.
-- Counts independent edges, including parallel edges. A self-loop contributes
-- once to each direction. Absence means zero only with declared full coverage.
CREATE TABLE degree_summary (
  source_system STRING NOT NULL, type_id BIGINT NOT NULL, id BIGINT NOT NULL,
  rel_type_id BIGINT NOT NULL, direction STRING NOT NULL, edge_count BIGINT NOT NULL
) USING DELTA CLUSTER BY (source_system, type_id, id, rel_type_id);

-- Optional coordination profile; not producer journal/checkpoint identity.
-- DDL alone does not enforce fencing, uniqueness or transactional membership.
CREATE TABLE publisher_fence (
  stream STRING NOT NULL, fence_epoch BIGINT NOT NULL, owner STRING NOT NULL,
  pending_batch_id STRING, sequence BIGINT NOT NULL
) USING DELTA;

CREATE TABLE apply_receipt (
  apply_batch_id STRING NOT NULL, stream STRING NOT NULL,
  fence_epoch BIGINT NOT NULL, sequence BIGINT NOT NULL,
  stage_id STRING NOT NULL, payload_digest STRING NOT NULL,
  expected_count BIGINT NOT NULL, source_progress_json STRING NOT NULL,
  schema_revisions_json STRING NOT NULL, recorded_at TIMESTAMP NOT NULL
) USING DELTA;

-- lookup_hash = lower-case SHA-256 hex of UTF-8 compact JSON named_struct:
-- object: source_system,type_id,id; edge: source_system,rel_type_id,id,
-- in that order, exact source string and signed integer values. Publisher checks
-- exact hash before publication. Reader retains all native predicates as well.
-- apply_batch_id is nullable for imported/legacy rows; an apply profile must
-- issue a unique publisher ID and validate it independently of producer origins.
-- 64MiB is a tuning target, not a guaranteed maximum file size.
-- No high-cardinality partitions; no automatic per-batch full OPTIMIZE policy.
-- Catalog-managed atomic profile must explicitly enable/qualify its features
-- on every participant. These ordinary tables do not imply atomic commit.


CREATE TABLE source_record (
  source_feed STRING NOT NULL,
  source_epoch STRING NOT NULL,
  delivery_id STRING NOT NULL,
  record_kind STRING NOT NULL,
  source_cursor_json STRING NOT NULL,
  payload_json STRING NOT NULL,
  payload_digest STRING NOT NULL,
  schema_revision STRING,
  received_at TIMESTAMP NOT NULL,
  apply_batch_id STRING
) USING DELTA CLUSTER BY (source_feed, source_epoch, delivery_id)
TBLPROPERTIES ('delta.dataSkippingStatsColumns'='source_feed,source_epoch,delivery_id');
