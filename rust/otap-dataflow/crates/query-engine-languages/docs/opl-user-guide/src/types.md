# Data Model & Types

This page describes the OpenTelemetry data model as it applies to OPL, the
fields available on each signal type, the type system for attribute values and
expressions, and the literal and operator syntax used in pipelines.

## Signal Types

OPL pipelines begin with a source that identifies which telemetry signal type
the pipeline operates on:

- `logs` -- log records
- `traces` -- trace spans
- `metrics` -- metric data points

```text
logs | where severity_text == "ERROR"
traces | where attributes["http.status_code"] > 499
metrics | where name == "http.server.request.duration"
```

The `signals` source targets all signal types simultaneously. The fields
available on `signals` are the intersection of fields available across logs,
traces, and metrics (e.g., `attributes`, `resource.attributes`). To access
signal-specific fields, use the `is` operator to narrow to a concrete signal
type within an `if` block -- see
[Signal Type Checks](./flow_control.md#signal-type-checks-is).

```text
// tag all telemetry with the processing environment
signals | set resource.attributes["deployment.environment.name"] = "production"
```

## Record Structure

OpenTelemetry data follows a hierarchical structure. Each signal type has a
similar layout:

**Resource** -- describes the entity producing telemetry (e.g., a service
instance). Contains:

- `resource.schema_url` -- schema URL for the resource
- `resource.dropped_attributes_count` -- number of dropped resource attributes
- `resource.attributes` -- key-value pairs describing the resource (e.g.,
  `service.name`, `service.version`, `k8s.namespace.name`)

**Instrumentation Scope** -- describes the instrumentation library. Contains:

- `instrumentation_scope.name` -- scope name
- `instrumentation_scope.version` -- scope version
- `instrumentation_scope.dropped_attributes_count` -- number of dropped scope
  attributes
- `instrumentation_scope.attributes` -- key-value pairs for the scope

**Signal Records** -- the individual telemetry items. Fields vary by signal
type.

### Log fields

| Field | Type | Description |
| ------- | ------ | ------------- |
| `time_unix_nano` | Timestamp | Time the event occurred |
| `observed_time_unix_nano` | Timestamp | Time the event was observed |
| `severity_number` | Integer | Numeric severity level |
| `severity_text` | String | Severity text (e.g., "ERROR") |
| `body` | AnyValue | Log message body |
| `event_name` | String | Event name |
| `trace_id` | Bytes | Trace identifier for correlation |
| `span_id` | Bytes | Span identifier for correlation |
| `flags` | Integer | Trace flags |
| `dropped_attributes_count` | Integer | Number of dropped log attributes |
| `attributes` | Map | Log-level attributes |

### Span fields

| Field | Type | Description |
| ------- | ------ | ------------- |
| `name` | String | Span name |
| `trace_id` | Bytes | Trace identifier |
| `span_id` | Bytes | Span identifier |
| `parent_span_id` | Bytes | Parent span identifier |
| `trace_state` | String | W3C trace state |
| `kind` | Integer | Span kind enum |
| `start_time_unix_nano` | Timestamp | Span start time |
| `duration_time_unix_nano` | Duration | Span duration |
| `status` | Struct | Span status |
| `flags` | Integer | Span flags |
| `dropped_attributes_count` | Integer | Number of dropped span attributes |
| `dropped_events_count` | Integer | Number of dropped span events |
| `dropped_links_count` | Integer | Number of dropped span links |
| `attributes` | Map | Span-level attributes |

### Metric fields

| Field | Type | Description |
| ------- | ------ | ------------- |
| `name` | String | Metric name |
| `description` | String | Metric description |
| `unit` | String | Unit of measurement |
| `metric_type` | Integer | Metric type enum (Gauge, Sum, Histogram, etc.) |
| `aggregation_temporality` | Integer | Aggregation temporality enum |
| `is_monotonic` | Boolean | Whether the metric is monotonic |
| `attributes` | Map | Metric-level attributes (metadata) |

### Future signal fields

Support for additional OpenTelemetry structures is planned for the future,
including metric data points, exemplars, span events, span links, and
entities. These will become accessible in OPL as the query engine evolves.

## Attributes

Attributes are key-value pairs present at three levels of the hierarchy:

- `attributes["key"]` -- attributes on the signal record itself
- `resource.attributes["key"]` -- attributes on the resource
- `instrumentation_scope.attributes["key"]` -- attributes on the scope

Attribute values can hold different types. The `is` operator tests the type
of a value (see [Value Types](#value-types) below).

## Value Types

Attribute values and expressions have types. The `is` operator tests the type
of a value at runtime:

```text
logs | where attributes["http.target"] is String
```

The following type names are available for `is` checks:

| Type Name | Description |
| ----------- | ------------- |
| `String` | Text string |
| `Integer` | 64-bit signed integer |
| `Double` | Double-precision floating point |
| `Boolean` | `true` or `false` |
| `Bytes` | Binary data |
| `Array` | Array of values |
| `Map` | Key-value map |
| `Null` | Null / empty value |

`Array` and `Map` types can be detected with `is` checks, but there is
currently no support for indexing into them or using their contents in
expressions. This means you can test whether an attribute is an `Array` or
`Map`, but you cannot access individual elements within them.

Type checks are commonly used to guard function calls that only accept
specific types:

```text
// lower_case only accepts strings -- guard with a type check
logs | where attributes["http.target"] is String and
    contains(lower_case(attributes["http.target"]), "/api/")
```

### Null checks

An attribute can be compared to `null` to test whether it exists:

```text
// keep only logs that have the "error.type" attribute
logs | where not(attributes["error.type"] == null)
```

## Literals

OPL supports the following literal types in expressions:

| Literal | Syntax | Example |
| --------- | -------- | --------- |
| String | `"..."` | `"hello"` |
| Integer | digits | `42` |
| Float | digits with decimal | `3.14` |
| Boolean | `true` / `false` | `true` |
| Regex | `r"..."` | `r".*error.*"` |
| Timestamp | `timestamp"..."` | `timestamp"2026-06-01T00:00:00.0"` |

Regex and timestamp literals are tagged literals -- their values are parsed
and **validated at compile time**. An invalid regex pattern or timestamp will
produce an error before the pipeline runs. Timestamp values must be in ISO
8601 format.

## Comparison Operators

| Operator | Description |
| ---------- | ------------- |
| `==` | Equal to |
| `!=` | Not equal to |
| `>` | Greater than |
| `>=` | Greater than or equal to |
| `<` | Less than |
| `<=` | Less than or equal to |
| `=~` | Case-insensitive equality |

## Logical Operators

| Operator | Description |
| ---------- | ------------- |
| `and` | Logical AND |
| `or` | Logical OR |
| `not(...)` | Logical negation |
