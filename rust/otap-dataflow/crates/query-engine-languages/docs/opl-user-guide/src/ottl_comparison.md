# OTTL Comparison

This page helps users familiar with OTTL (the OpenTelemetry Transformation
Language) understand how the same operations are expressed in OPL.

The two languages take different approaches: OTTL uses individual statement
functions like `set(destination, source)`, each optionally guarded by a
`where` clause. OPL chains operators in a pipeline separated by `|`, where
each operator transforms the stream for the next.

## Syntax Overview

OTTL processes telemetry with a list of statements, each acting independently:

```text
set(severity_text, "ERROR")
set(attributes["tier"], "critical") where severity_number >= 17
delete_key(attributes, "internal.debug")
```

OPL expresses the same logic as a single pipeline:

```text
logs |
set severity_text = "ERROR" |
if (severity_number >= 17) {
    set attributes["tier"] = "critical"
} |
remove attributes["internal.debug"]
```

A key difference is how conditional logic works: OTTL attaches `where` clauses
to individual statements, while OPL uses `if` blocks that can contain multiple
operators. See [Conditional Logic](#conditional-logic) for more details.

## Field Assignment

OTTL's `set()` editor maps directly to OPL's `set` operator. The main
syntactic difference is that OPL uses `=` assignment syntax instead of
function-call syntax:

| OTTL | OPL |
|------|-----|
| `set(severity_text, "ERROR")` | `set severity_text = "ERROR"` |
| `set(event_name, "request")` | `set event_name = "request"` |
| `set(attributes["k"], "v")` | `set attributes["k"] = "v"` |
| `set(resource.attributes["k"], "v")` | `set resource.attributes["k"] = "v"` |

Multiple assignments in OPL can be combined into a single `set` invocation:

```text
logs |
set
    severity_text = "ERROR",
    attributes["tier"] = "critical"
```

## Filtering

OTTL uses `where` clauses on individual statements to conditionally apply
them. OPL uses `where` as a standalone pipeline operator that drops
non-matching records entirely:

| OTTL | OPL |
|------|-----|
| (drop records) `where severity_number < 9` | `where severity_number >= 9` |
| (no direct equivalent) | `where contains(body, "error")` |

To conditionally apply an operator without dropping records, use `if` blocks
instead of `where` (see [Conditional Logic](#conditional-logic)).

## Attribute Operations

OTTL provides several editor functions for manipulating attribute maps. OPL
offers equivalent capabilities through dedicated operators and the `apply`
block:

### Deleting attributes

OTTL:

```text
delete_key(attributes, "http.method")
```

OPL:

```text
logs | remove attributes["http.method"]
```

### Deleting attributes by pattern

OTTL's `delete_matching_keys` removes keys matching a regex. In OPL, use
`apply` with a `where` filter to achieve the same result:

OTTL:

```text
delete_matching_keys(attributes, "internal\\..*")
```

OPL:

```text
logs | apply attributes {
    where not(matches(key, r"internal\..*"))
}
```

### Keeping only specific attributes

OTTL's `keep_keys` retains only named keys. In OPL, use `apply` with a
`where` filter:

OTTL:

```text
keep_keys(attributes, "http.method", "url.path")
```

OPL:

```text
logs | apply attributes {
    where key == "http.method" or key == "url.path"
}
```

OTTL's `keep_matching_keys` retains keys matching a pattern:

OTTL:

```text
keep_matching_keys(attributes, "http\\..*")
```

OPL:

```text
logs | apply attributes {
    where matches(key, r"http\..*")
}
```

### Renaming attributes

OTTL:

```text
rename_key(attributes, "http.method", "http.request.method")
```

OPL:

```text
logs | rename attributes["http.request.method"] = attributes["http.method"]
```

OPL supports renaming multiple keys in a single invocation:

```text
logs |
rename
    attributes["http.request.method"] = attributes["http.method"],
    attributes["url.path"] = attributes["http.target"]
```

## Value Transformation Functions

OTTL converters (functions that return values) map to OPL's built-in
functions. In OTTL, converters are used as arguments to editors. In OPL,
functions are used directly in expressions:

| OTTL Converter | OPL Function |
|----------------|--------------|
| `Concat(values, delim)` | `concat_ws(delim, ...)` |
| `ConvertCase(v, "lower")` / `ToLowerCase(v)` | `lower_case(v)` |
| `ConvertCase(v, "upper")` / `ToUpperCase(v)` | `upper_case(v)` |
| `Substring(v, start, length)` | `substring(v, start [, length])` |
| `Trim(v)` | `ltrim(rtrim(v))` |
| `Log(v)` | `log10(v)` |
| `Coalesce(a, b, ...)` | `coalesce(a, b, ...)` |
| `UUID()` | `uuid()` |
| `UUIDv7()` | `uuidv7()` |

For more complex transformations, here is how a full OTTL statement maps to
an OPL pipeline operator:

OTTL:

```text
set(attributes["hash"], SHA256(attributes["secret"]))
```

OPL:

```text
logs | set attributes["hash"] = encode(sha256(attributes["secret"]), "hex")
```

### Pattern matching and string tests

Note: in OTTL, these predicates appear inside `where` clauses that guard
individual statements. In OPL, they are used as standalone expressions in
`where` filters or `if` conditions.

| OTTL Predicate | OPL Predicate |
|----------------|---------------|
| `IsMatch(body, ".*error.*")` | `matches(body, r".*error.*")` |
| `Index(body, "error") >= 0` | `contains(body, "error")` |
| `HasPrefix(a["url.path"], "/api/")` | `starts_with(attributes["url.path"], "/api/")` |
| `HasSuffix(a["url.path"], "/health")` | `ends_with(attributes["url.path"], "/health")` |

### Hashing functions

| OTTL | OPL |
|------|-----|
| `SHA1(value)` | `sha1(value)` |
| `SHA256(value)` | `sha256(value)` |
| `SHA512(value)` | `sha512(value)` |
| `MD5(value)` | `md5(value)` |
| `FNV(value)` | `fnv(value)` |
| `Murmur3Hash(value)` | `murmur3(value)` |
| `XXH3(value)` | `xxh3(value)` |
| `XXH128(value)` | `xxh128(value)` |

## Conditional Logic

OTTL applies conditions per-statement using `where` clauses:

```text
set(attributes["tier"], "critical") where severity_text == "ERROR"
set(attributes["tier"], "warning") where severity_text == "WARN"
set(attributes["tier"], "info") where severity_text == "INFO"
```

OPL uses `if`/`else if`/`else` blocks, which are more expressive -- each
record is handled by exactly one branch, avoiding the need to write mutually
exclusive conditions:

```text
logs |
if (severity_text == "ERROR") {
    set attributes["tier"] = "critical"
} else if (severity_text == "WARN") {
    set attributes["tier"] = "warning"
} else {
    set attributes["tier"] = "info"
}
```

OPL branches can also contain multiple chained operators:

```text
logs |
if (severity_text == "ERROR") {
    set attributes["tier"] = "critical" |
    set attributes["error.flagged"] = true
}
```

OTTL conditions can use `and`, `or`, and `not`. OPL supports the same logical
operators in both `where` filters and `if` conditions:

OTTL:

```text
set(attributes["tier"], "critical") where severity_number >= 17 and resource.attributes["service.name"] == "payments"
```

OPL:

```text
logs |
if (severity_number >= 17 and resource.attributes["service.name"] == "payments") {
    set attributes["tier"] = "critical"
}
```

## Routing

OPL's `route_to` operator directs records to named output ports. There is no
direct OTTL equivalent -- routing in the OpenTelemetry Collector is typically
handled by connector components or separate pipelines rather than within the
transformation language itself:

```text
logs |
if (severity_text == "ERROR") {
    route_to "error_sink"
} else if (severity_text == "WARN") {
    route_to "warning_sink"
}
```

## Bulk Attribute Processing

OPL's `apply` operator treats attributes as a stream of `key`/`value` pairs,
enabling bulk operations that have no single OTTL equivalent. Instead of
writing multiple OTTL statements for each attribute, `apply` handles them
all at once:

```text
// hash all attribute values in one operation
logs | apply attributes {
    set value = encode(sha256(value), "hex")
}
```

```text
// conditionally transform specific attributes
logs | apply attributes {
    if (key == "user.email" or key == "user.ip") {
        set value = encode(sha256(value), "hex")
    }
}
```

## Signal Type Handling

In OTTL, different signal types are handled by configuring separate statement
lists (`log_statements`, `trace_statements`, `metric_statements`). In OPL,
the signal type is part of the pipeline source, and the `signals` keyword
with `is` checks can handle multiple signal types in a single pipeline:

```text
signals |
if (is Log) {
    set attributes["signal.type"] = "log"
} else if (is Span) {
    set attributes["signal.type"] = "span"
} else if (is Metric) {
    set attributes["signal.type"] = "metric"
}
```

## OTTL Capabilities Not Yet in OPL

OPL is actively working toward covering the full utility of OTTL. The
following sections list OTTL capabilities that do not yet have OPL equivalents
but are planned for future releases.

### Editor functions

| OTTL Editor | Description |
|-------------|-------------|
| `append(target, value)` | Append values to an array |
| `flatten(target)` | Flatten nested maps |
| `merge_maps(target, source, strategy)` | Merge two maps |
| `limit(target, limit, priority_keys...)` | Limit number of attribute keys |
| `truncate_all(target, limit)` | Truncate all string values to a maximum length |
| `replace_all_matches(target, pattern, replacement)` | Replace matching attribute values |
| `replace_all_patterns(target, mode, pattern, replacement)` | Replace regex patterns in attribute values |
| `replace_match(target, pattern, replacement)` | Replace a single match |
| `replace_pattern(target, pattern, replacement)` | Replace a single regex pattern |

### Converter functions

| OTTL Converter | Description |
|----------------|-------------|
| `ParseJSON(value)` | Parse a JSON string into a structured value |
| `ParseKeyValue(value, ...)` | Parse key-value pair strings |
| `ParseCSV(value, ...)` | Parse CSV strings |
| `ExtractGrokPatterns(value, pattern)` | Extract fields using Grok patterns |
| `IsValidLuhn(value)` | Luhn checksum validation |
| `Split(value, delimiter)` | Split a string into an array |
| `Len(target)` | Get the length of a string or collection |
| `Keys(target)` / `Values(target)` | Extract keys or values from a map |
| `Int(value)` / `Double(value)` / `Bool(value)` | Type conversion functions |
| `Time(value, format)` / `TruncateTime(...)` | Time parsing and truncation |
| `Duration(value)` / `Nanoseconds(...)` / `Microseconds(...)` | Duration functions |
| `Now()` / `Unix(...)` / `UnixNano(...)` | Timestamp functions |
| `SpanID(bytes)` / `TraceID(bytes)` / `ProfileID(bytes)` | ID construction |
| `IsRootSpan()` | Check if span is root |
| `IsInCIDR(value, cidr)` | CIDR range check |
| `Sort(target, order)` | Sort a collection |
| `SliceToMap(target, ...)` | Convert a slice to a map |
| `URL(value)` / `UserAgent(value)` | Parse URLs and user agent strings |
| `Format(format, ...)` | String formatting |

### Structural capabilities

The following capabilities are available in OTTL but not yet supported in OPL:

- **Date/time arithmetic** -- OTTL provides functions for extracting and
  manipulating date/time components (`Day`, `Hour`, `Minute`, `Month`,
  `Year`, `Weekday`, etc.)
- **Temporary variables / caching** -- OTTL supports the concept of temporary
  variables within statement execution; OPL does not have an equivalent
- **Processing data points and exemplars** -- OTTL can operate on metric data
  points and exemplars via dedicated contexts (`ottldatapoint`)
- **Processing span events and span links** -- OTTL can operate on span events
  and links via dedicated contexts (`ottlspanevent`)
- **Accessing Array/Map attribute values** -- OTTL can index into and
  manipulate Array and Map attribute values; OPL can detect these types with
  `is` but cannot access their contents
- **XML processing** -- OTTL provides XML manipulation functions
  (`GetXML`, `InsertXML`, `RemoveXML`, etc.)
