# KQL Processor

An experimental otap-dataflow processor that filters and transforms
OpenTelemetry data using Kusto Query Language (KQL) expressions.

## Overview

This processor integrates the experimental KQL "recordset" engine from
`rust/experimental/query_engine` to enable powerful data
transformations within OTAP pipelines.  This was developed as a prototype
as we prepare for a direct column-oriented implementation, it is functional
and production quality however not an optimized implementation.

## Features

- **Filters**: Use KQL `where` clauses to filter logs, traces, and metrics
- **Transformations**: Apply KQL `extend` and `project` operations to modify data
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

Enable the `kql-processor` feature flag:

```bash
cargo build --features kql-processor
```

## Running the Demo

A complete demo configuration is available at `configs/fake-kql-debug-noop.yaml`. Run it with:

```bash
cargo run --features kql-processor --bin df_engine -- --pipeline ./configs/fake-kql-debug-noop.yaml --num-cores 1
```

This demonstrates:
- Fake log data generation
- KQL-based Body field enrichment with vendor-specific URLs
- Debug processor output showing modified logs
