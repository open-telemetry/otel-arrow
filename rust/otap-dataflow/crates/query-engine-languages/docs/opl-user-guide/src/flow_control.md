# Flow Control

These operators control how records flow through a pipeline -- branching
processing logic based on conditions or directing output to named ports.

## Conditional (`if`)

The `if` operator applies different operators to different records based on a
condition. Records that match the condition are processed by the operators
inside the block; records that do not match are either passed through unchanged
or handled by subsequent `else if`/`else` branches.

The `else if`/`else` branches are optional, and if not provided, no changes
will be applied to the telemetry signals that do not match any condition.

The output of this operator is the result of each nested branch unioned
together.

### Basic usage

```
// rename an attribute only for logs with ERROR severity
logs | if (severity_text == "ERROR") {
    rename attributes["http.request.method"] = attributes["http.method"]
}
```

Records that do not match the condition pass through unchanged. In the example
above, non-ERROR logs keep their original `http.method` attribute.

### `if`/`else`

An `else` block handles all records that did not match the `if` condition:

```
logs | if (severity_text == "ERROR") {
    set attributes["error.priority"] = "high"
} else {
    set attributes["error.priority"] = "low"
}
```

### `if`/`else if`/`else`

Multiple conditions can be chained with `else if`. Each record is processed by
the first branch whose condition it matches. If no condition matches, the
`else` branch runs (if present), otherwise the record passes through unchanged:

```
logs |
if (severity_text == "ERROR") {
    set attributes["log.tier"] = "critical"
} else if (severity_text == "WARN") {
    set attributes["log.tier"] = "warning"
} else {
    set attributes["log.tier"] = "info"
}
```

Each record is evaluated against branches in order, top to bottom. A record is
handled by at most one branch -- once a condition matches, later branches are
skipped for that record.

### Operators inside branches

The body of each branch can contain any operators chained with `|`, just like a
top-level pipeline. This includes `where`, `set`, `rename`, `remove`,
`route_to`, and even nested `if` blocks:

```
logs |
if (severity_text == "ERROR") {
    rename attributes["http.request.method"] = attributes["http.method"] |
    set attributes["error.flagged"] = true
} else {
    remove attributes["debug.detail"]
}
```

A `where` filter inside a branch drops records that don't match, while
records in other branches are unaffected:

```
logs |
if (severity_text == "ERROR") {
    // among ERROR logs, only keep those from the payments service
    where resource.attributes["service.name"] == "payments" |
    set attributes["error.escalate"] = true
}
```

In this example, ERROR logs from the payments service are kept and tagged.
ERROR logs from other services are dropped. All non-ERROR logs pass through
unchanged.

### Row ordering

The order of records in the output may not match their original order. Records
from different branches are unioned together after processing, however the
resulting row order is not specified.

## Signal Type Checks (`is`)

When a pipeline targets all signal types using the `signals` source, the `is`
operator can test what kind of record is being processed:

- `is Log` -- matches log records
- `is Metric` -- matches metric data points
- `is Span` -- matches trace spans

This is most useful inside `if` conditions to apply signal-specific
transformations within a single pipeline:

```
signals |
if (is Log) {
    set attributes["signal.source"] = "logs"
} else if (is Metric) {
    set attributes["signal.source"] = "metrics"
} else if (is Span) {
    set attributes["signal.source"] = "traces"
}
```

The `is` check can also be used in `where` filters:

```
// process only logs, drop metrics and spans
signals | where is Log
```

## Route Output (`route_to`)

The `route_to` operator sends the current batch to a named output port instead
of the default output. This is useful for directing different subsets of
telemetry to different downstream destinations.

### Basic usage

```
logs | route_to "error_sink"
```

The port name must be a static string literal -- it cannot be a computed
expression.

After `route_to` executes, the default output receives an empty batch. This
means `route_to` effectively diverts the data away from the normal output path.

### Runtime configuration

The `route_to` operator names a destination, but the actual wiring of that
destination to a downstream consumer is orthogonal to the OPL language itself.
The named output port must be configured in the runtime that hosts the query
engine. For example, when using the transform processor, each route name must
correspond to a configured output port on the processor node so the dataflow
engine knows where to deliver the routed data.

### Combining with `if` for conditional routing

`route_to` is most powerful when combined with `if` to route different records
to different destinations based on conditions:

```
logs |
if (severity_text == "ERROR") {
    route_to "error_sink"
} else if (severity_text == "WARN") {
    route_to "warning_sink"
}
```

In this example, ERROR logs are sent to `error_sink`, WARN logs to
`warning_sink`, and all other logs flow through the default output unchanged.

You can also apply transformations before routing:

```
logs |
if (severity_text == "ERROR") {
    set attributes["error.routed"] = true |
    route_to "error_sink"
}
// all non-routed log records continue to next pipeline operator
```
