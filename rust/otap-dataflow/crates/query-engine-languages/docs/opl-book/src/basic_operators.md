# Basic Operators

You can filter the things using `where`

## Filter (`where`)

The `where` operator can be used to filter telemetry data. Any fields telemetry
item that matches the logical expression will be kept, and anything not 
matching will be dropped.

```
// keep only logs with "ERROR" severity level
logs | where severity_text == "ERROR"
```

```
// discard any logs from kubernetes namespace "testing"
logs | where attributes["k8s.namespace.name"] != "testing"
```

Comparison operators `>`, `>=`, `<` and `<=` are available numeric and 
timestamp types:

```
logs | where severity_number >= 17
```

```
logs | where time_unix_nano < date_time"2026-06-01T00:00:00.0"
```

The `=~` comparison operator performs case-insensitive equality
```
// will match "http.request", "HTTP.request", "HTTP.REQUEST", etc.
logs | where event_name =~ "http.request"
```

`and`, `or`, `not` and parentheses `(`/`)` keywords can be used to combine
filter conditions.

```
logs | where severity_number > 4 and severity_number <= 8
```

```
logs | 
where 
    severity_text == "WARN" or
    severity_text == "ERROR" or
    severity_text == "FATAL"
```

```
logs | where not(
    attributes["k8s.namespace.name"] == "testing"
) or (
    attributes["k8s.namespace.name"] != "testing" and severity_text == "ERROR"
)
```

Various functions are available for filtering strings such as `contains`, 
`matches`, `starts_with` and `ends_with`
```
// keep logs where body is a string containing text "error"
logs | where contains(body, "error")
```

```
// discard logs where the kubernetes pod name contains "testing"
logs | where not(matches(attributes["k8s.pod.name"], r".*testing.*"))
```

## Assign (`set`)