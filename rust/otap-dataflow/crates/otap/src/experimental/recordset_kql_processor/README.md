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

## LogRecord structure and accessing data

Log record structure follows the [OpenTelemetry
Specification](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/logs/data-model.md#logs-data-model).

|Field            | Type        |
|-----------------|-------------|
|Timestamp        |DateTime(UTC)|
|ObservedTimestamp|DateTime(UTC)|
|TraceId          |Byte[]       |
|SpanId           |Byte[]       |
|TraceFlags       |Integer      |
|SeverityText     |String       |
|SeverityNumber   |Integer      |
|Body             |Any          |
|Attributes       |Map          |
|EventName        |String       |

## Associated data

Each log record is associated to a
[Resource](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/resource/README.md)
and an [Instrumentation
Scope](https://github.com/open-telemetry/opentelemetry-specification/blob/main/specification/common/instrumentation-scope.md).

### Resource

|Field            | Type        |
|-----------------|-------------|
|Attributes       |Map          |

### Instrumentation Scope

|Field            | Type        |
|-----------------|-------------|
|Attributes       |Map          |
|Name             |String       |
|Version          |String       |

## Accessing data

When querying data on a log record we use the `source` identifier.

The following query will remove the `TraceId` and `SpanId` fields:

```yaml
query: "source | project-away TraceId, SpanId"
```

Top-level fields can be cleared and modified but new top-level fields cannot be
introduced. To support the addition of data a special top-level field named
`Attributes` is used. `Attributes` is a map type which contains values tied to a
string key.

The following query will add `TimeProcessed` key/value to `Attributes`:

```yaml
query: "source | extend Attributes['TimeProcessed'] = now()"
```

To simplify accessing custom data the query engine will automatically resolve
anything unknown using the `Attributes` map.

This is equivalent to the previous query:

```yaml
query: "source | extend TimeProcessed = now()"
```

Nested data may also be accessed.

The following query will add the value of the `Body.name` field as
`Attributes.Name`. `Body.name` will then be removed. Note: If `Body.name` cannot
be found a `null` value will be set.

```yaml
query: |
 source
  | extend Name = Body.name
  | project-away Body.name
```

To access `Resource` and/or `Instrumentation Scope` data the `resource` and/or
`scope` identifiers may be used.

```yaml
query: |
 source
  | extend
     ProcessName = resource.Attributes['process.name'],
     Category = scope.name
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
