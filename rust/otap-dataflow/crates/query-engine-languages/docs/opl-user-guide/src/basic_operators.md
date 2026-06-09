# Basic Operators

These operators handle the most common pipeline tasks -- filtering records and
assigning values to fields and attributes.

## Filter (`where`)

The `where` operator filters telemetry data. Records that match the logical
expression are kept; everything else is dropped.

```text
// keep only logs with "ERROR" severity level
logs | where severity_text == "ERROR"
```

```text
// discard access logs for the health endpoint
logs | where attributes["url.path"] != "/v1/health"
```

```text
// keep only logs from the "observability" namespace
logs | where resource.attributes["k8s.namespace.name"] == "observability"
```

Comparison operators `>`, `>=`, `<` and `<=` are available for numeric and
timestamp types:

```text
logs | where severity_number >= 17
```

```text
logs | where time_unix_nano < timestamp"2026-06-01T00:00:00.0"
```

The `=~` comparison operator performs case-insensitive equality:

```text
// will match "http.request", "HTTP.request", "HTTP.REQUEST", etc.
logs | where event_name =~ "http.request"
```

`and`, `or`, `not` and parentheses `(`/`)` can be used to combine filter
conditions:

```text
logs | where severity_number > 4 and severity_number <= 8
```

```text
logs |
where
    severity_text == "WARN" or
    severity_text == "ERROR" or
    severity_text == "FATAL"
```

```text
logs | where not(
    attributes["url.path"] == "/v1/health" or
    attributes["url.path"] == "/v1/metrics"
)
```

### String functions

Various string functions are available for filtering, including `contains`,
`matches`, `starts_with` and `ends_with`:

```text
// keep logs where body is a string containing text "error"
logs | where contains(body, "error")
```

```text
// discard logs where the kubernetes pod name contains "testing"
logs | where not(matches(resource.attributes["k8s.pod.name"], r".*testing.*"))
```

### Value type checks (`is`)

Attribute values can hold different types (strings, integers, floats, booleans,
etc.). The `is` operator tests the type of a value, which is useful for
guarding function calls that only accept specific types:

```text
// lower_case only accepts strings -- guard with a type check
logs |
where
    attributes["http.target"] is String and
    contains(lower_case(attributes["http.target"]), "/api/")
```

Without the type guard, `lower_case` may fail if
`attributes["http.target"]` is not a string.

The `is` check works with any attribute scope:

```text
logs | where resource.attributes["service.version"] is String
```

## Assign (`set`)

The `set` operator modifies or assigns a field value:

```text
// set event_name field
logs | set event_name = "event.happened"
```

It can also set the value of an attribute. If no attribute exists with the
given key, a new attribute will be created:

```text
// ensure each log has attribute "exception.type" with value "OSError"
logs | set attributes["exception.type"] = "OSError"
```

```text
logs | set resource.attributes["k8s.cluster.name"] = "dev-ca-central1"
```

Many expressions can be used to compute the value being assigned, such as
function invocations and arithmetic:

```text
// compute log body from other fields
logs | set body = concat("[", severity_text, "]: ", event_name)
```

```text
// redact an attribute's value with its hash
logs | set attributes["sensitive"] = encode(sha256(attributes["sensitive"]), "hex")
```

### Multiple assignments

Multiple assignments can be made in a single invocation of the `set` operator
by separating each assignment with a comma:

```text
logs |
set
    attributes["user.name"] = "alice",
    attributes["user.role"] = "admin",
    attributes["user.active"] = true,
    body = "hello world"
    // etc.
```

### Cardinality constraints

When setting fields or attributes on a resource or scope, the expression must
not reference fields with a higher cardinality than the destination.

This means that an expression targeting a resource field cannot reference
log/span/metric or instrumentation scope fields. Similarly, an expression
targeting an instrumentation scope field cannot reference log/span/metric
fields.

For example, the following expressions are not allowed and will produce an
error:

```text
logs | set resource.schema_url = schema_url // log schema URL

logs | set resource.attributes["service.name"] = instrumentation_scope.attributes["service.name"]

logs | set resource.attributes["service.name"] = attributes["service.name"]

logs | set resource.attributes["service.name"] = concat(attributes["service.name"], "-suffix")

logs | set instrumentation_scope.name = attributes["otel.scope.name"]

// etc.
```
