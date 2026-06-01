# Data Model & Types

## Signal Types

OPL pipelines begin with a source that identifies which telemetry signal type
the pipeline operates on:

- `logs` -- log records
- `traces` -- trace spans
- `metrics` -- metric data points
- `signals` -- all signal types (see
  [Signal Type Checks](./flow_control.md#signal-type-checks-is))

```
logs | where severity_text == "ERROR"
traces | where attributes["http.status_code"] > 499
metrics | where name == "http.server.request.duration"
```

## Record Structure

OpenTelemetry data follows a hierarchical structure. Each signal type has a
similar layout:

**Resource** -- describes the entity producing telemetry (e.g., a service
instance). Contains:

- `resource.schema_url` -- schema URL for the resource
- `resource.attributes` -- key-value pairs describing the resource (e.g.,
  `service.name`, `service.version`, `k8s.namespace.name`)

**Instrumentation Scope** -- describes the instrumentation library. Contains:

- `instrumentation_scope.name` -- scope name
- `instrumentation_scope.version` -- scope version
- `instrumentation_scope.schema_url` -- schema URL
- `instrumentation_scope.attributes` -- key-value pairs for the scope

**Signal Records** -- the individual telemetry items. Fields vary by signal
type.

### Log fields

| Field | Type | Description |
|-------|------|-------------|
| `time_unix_nano` | Timestamp | Time the event occurred |
| `observed_time_unix_nano` | Timestamp | Time the event was observed |
| `severity_number` | Integer | Numeric severity (1-24) |
| `severity_text` | String | Severity text (e.g., "ERROR") |
| `body` | Any | Log message body |
| `event_name` | String | Event name |
| `attributes` | Map | Log-level attributes |

### Span fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Span name |
| `trace_id` | Bytes | Trace identifier |
| `span_id` | Bytes | Span identifier |
| `parent_span_id` | Bytes | Parent span identifier |
| `kind` | Integer | Span kind |
| `start_time_unix_nano` | Timestamp | Span start time |
| `end_time_unix_nano` | Timestamp | Span end time |
| `status` | Struct | Span status |
| `attributes` | Map | Span-level attributes |

### Metric fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | String | Metric name |
| `description` | String | Metric description |
| `unit` | String | Unit of measurement |
| `attributes` | Map | Data point attributes |

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

```
logs | where attributes["http.target"] is String
```

The following type names are available for `is` checks:

| Type Name | Description |
|-----------|-------------|
| `String` | Text string |
| `Integer` | 64-bit signed integer |
| `Double` | Double-precision floating point |
| `Boolean` | `true` or `false` |
| `Array` | Array of values |
| `Map` | Key-value map |
| `Null` | Null / empty value |

Type checks are commonly used to guard function calls that only accept
specific types:

```
// lower_case only accepts strings
logs | where attributes["http.target"] is String and
    contains(lower_case(attributes["http.target"]), "/api/")
```

### Null checks

An attribute can be compared to `null` to test whether it exists:

```
// keep only logs that have the "error.type" attribute
logs | where not(attributes["error.type"] == null)
```

## Literals

OPL supports the following literal types in expressions:

| Literal | Syntax | Example |
|---------|--------|---------|
| String | `"..."` | `"hello"` |
| Integer | digits | `42` |
| Float | digits with decimal | `3.14` |
| Boolean | `true` / `false` | `true` |
| Regex | `r"..."` | `r".*error.*"` |
| Timestamp | `date_time"..."` | `date_time"2026-06-01T00:00:00.0"` |

## Comparison Operators

| Operator | Description |
|----------|-------------|
| `==` | Equal to |
| `!=` | Not equal to |
| `>` | Greater than |
| `>=` | Greater than or equal to |
| `<` | Less than |
| `<=` | Less than or equal to |
| `=~` | Case-insensitive equality |

## Logical Operators

| Operator | Description |
|----------|-------------|
| `and` | Logical AND |
| `or` | Logical OR |
| `not(...)` | Logical negation |
