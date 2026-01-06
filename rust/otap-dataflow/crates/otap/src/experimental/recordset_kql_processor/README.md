# EXPERIMENTAL KQL "RecordSet" Processor

An OTAP-Dataflow processor that filters and transforms OpenTelemetry data using
Kusto Query Language (KQL) expressions over OpenTelemetry data in an opinionated
way.

## Caveats and notes

The underlying engine does not use Apache Arrow or DataFusion. This is not an
optimized code path in the OTAP Dataflow engine and performance (throughput)
will be lower.

## Overview

This processor integrates the experimental KQL "recordset" engine from
`rust/experimental/query_engine` to enable powerful data transformations within
OTAP pipelines.  This was developed as a prototype as we prepare for a direct
column-oriented implementation, it is functional and production quality however
not an optimized implementation.

## Features

- **Filters**: Use KQL `where` clauses to filter logs
- **Transformations**: Apply KQL `extend` and `project` operations to modify
  data
- **Aggregations**: Perform KQL `summarize` operations for data aggregation

## Configuration

```yaml
processors:
  kql:
    query: "source | where SeverityText == 'ERROR' | extend processed_time = now()"
```

## Examples

### Filter logs by severity

```yaml
query: "source | where SeverityNumber >= 17"  # ERROR and above
```

### Add computed fields

```yaml
query: "source | extend hour = bin(Timestamp, 1h)"
```

### Keep only specific fields

```yaml
query: "source | project-keep Body, SeverityText, Timestamp"
```

### Aggregate logs

```yaml
query: "source | summarize Count = count() by SeverityText"
```

## Building

Enable the `recordset-kql-processor` feature flag:

```bash
cargo build --features recordset-kql-processor
```

## Running the Demo

A complete demo configuration is available at
`configs/fake-kql-debug-noop.yaml`. Run it with:

```bash
cargo run --features recordset-kql-processor --bin df_engine -- --pipeline ./configs/fake-kql-debug-noop.yaml --num-cores 1
```

This demonstrates:

- Fake log data generation
- KQL-based Body field enrichment with vendor-specific URLs
- Debug processor output showing modified logs
