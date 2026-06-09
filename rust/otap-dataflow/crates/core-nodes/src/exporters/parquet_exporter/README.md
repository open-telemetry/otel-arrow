# Parquet Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:parquet` (`urn:otel:exporter:parquet`)
- Feature gate: Default; cloud backends require crate features
- Stability: Experimental

## Overview

The Parquet exporter writes OTAP batches as Parquet files through the shared
object-store abstraction. It can partition output using schema metadata and can
flush files by approximate row count or age.

## Getting Started

Write Parquet files to a local directory with a file storage backend:

```yaml
type: exporter:parquet
config:
  storage:
    file:
      base_uri: "/tmp/otap-parquet"
  writer_options:
    flush_when_older_than: 300s
    target_rows_per_file: 1000000
```

## Configuration

```yaml
type: exporter:parquet
config:
  # Object-store backend and base URI (required).
  storage:
    file:
      base_uri: "/tmp/otap-parquet"

  # Optional partition strategies.
  partitioning_strategies:
    - schema_metadata: ["_part_id"]

  writer_options:
    # Approximate row-count flush target (default: 100000000).
    target_rows_per_file: 1000000

    # Flush files older than this interval (optional).
    flush_when_older_than: 300s
```

The default build supports local file storage. Enable the top-level `azure`
feature for Azure Blob Storage and the top-level `aws` feature for S3. See
[`configs/fake-parquet-azure.yaml`](../../../../../configs/fake-parquet-azure.yaml)
and
[`configs/fake-parquet-s3.yaml`](../../../../../configs/fake-parquet-s3.yaml)
for backend-specific configuration examples.

## Examples

Partition by schema metadata:

```yaml
type: exporter:parquet
config:
  storage:
    file:
      base_uri: "/tmp/otap-parquet"
  partitioning_strategies:
    - schema_metadata: ["_part_id"]
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `exporter.pdata`

| Metric | Unit | Description |
| --- | --- | --- |
| `exporter.pdata.metrics_consumed` | `{msg}` | Number of pdata metrics consumed by this exporter. |
| `exporter.pdata.metrics_exported` | `{msg}` | Number of pdata metrics successfully exported. |
| `exporter.pdata.metrics_failed` | `{msg}` | Number of pdata metrics that failed to be exported. |
| `exporter.pdata.logs_consumed` | `{msg}` | Number of pdata logs consumed by this exporter. |
| `exporter.pdata.logs_exported` | `{msg}` | Number of pdata logs successfully exported. |
| `exporter.pdata.logs_failed` | `{msg}` | Number of pdata logs that failed to be exported. |
| `exporter.pdata.traces_consumed` | `{msg}` | Number of pdata traces consumed by this exporter. |
| `exporter.pdata.traces_exported` | `{msg}` | Number of pdata traces successfully exported. |
| `exporter.pdata.traces_failed` | `{msg}` | Number of pdata traces that failed to be exported. |

#### `otap.exporter.parquet`

| Metric | Unit | Description |
| --- | --- | --- |
| `otap.exporter.parquet.files_created` | `{file}` | Number of Parquet files created (across all payload types and partitions). |
| `otap.exporter.parquet.files_closed` | `{file}` | Number of Parquet files successfully closed (flushed and visible to readers). |
| `otap.exporter.parquet.rows_written` | `{row}` | Total number of rows written into Parquet writers (appended, not necessarily flushed yet). |
| `otap.exporter.parquet.flush_scheduled_max_rows` | `{file}` | Files scheduled for flush due to reaching target rows per file. |
| `otap.exporter.parquet.flush_scheduled_max_age` | `{file}` | Files scheduled for flush due to exceeding max age threshold. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | No node-specific events are emitted. |

## Limits

- Row-count flushing is approximate and does not split a single incoming batch
  across multiple output files.
- Very small `flush_when_older_than` values can produce many small files.
- Cloud backends depend on optional compile-time features.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Core node catalog](../../../README.md)
