# Introduction

OpenTelemetry Processing Language (OPL) is a stream-oriented language for
transforming, filtering, and routing OpenTelemetry signals. It is designed
around the OpenTelemetry data model -- pipelines consume and produce valid
Logs, Spans, and Metrics. OPL borrows a readable, pipe-separated syntax that
makes it natural to chain processing steps together.

## Pipeline Syntax

An OPL pipeline starts with a source that identifies the signal type, followed
by one or more operators separated by `|`:

```text
logs | where severity_number >= 17 | set attributes["critical"] = true
```

Operators are applied sequentially, left to right. Each operator receives the
output of the previous one. Pipelines and operator arguments can be written
across multiple lines for readability:

```text
logs |
where
  severity_number >= 17 or
  severity_text == "ERROR" |
set
  attributes["critical"] = true,
  body = concat("ERROR: ", body)
```

## Comments

Line comments start with `//`:

```text
// keep only critical logs and tag them
logs |
where severity_number >= 17 |
set attributes["critical"] = true
```

Block comments start with `/*` and end with `*/`, and can span multiple lines:

```text
/*
Inject kubernetes attributes for:
- namespace
- cluster

As well as service:
- criticality
- version
*/
signals |
  set
    resource.attributes["k8s.namespace.name"] = "test-app",
    resource.attributes["k8s.cluster.name"] = "testing-ca-central1",
    resource.attributes["service.criticality"] = "low",
    resource.attributes["service.version"] = "0.1.23"
```

## Available Sources

The source keyword determines which signal types the pipeline processes:

- `logs` -- log records
- `traces` -- trace spans
- `metrics` -- metric data points
- `signals` -- all signal types

## Guide Overview

This guide covers the currently implemented OPL operators and functions:

- [Data Model & Types](./types.md) -- signal structure, field reference,
  value types, and literals
- [Basic Operators](./basic_operators.md) -- filtering with `where` and
  assigning values with `set`/`extend`
- [Attribute Operators](./attr_operators.md) -- renaming and removing
  attributes with `rename` and `remove`, and bulk-processing attributes
  with `apply`
- [Flow Control](./flow_control.md) -- conditional branching with
  `if`/`else if`/`else`, signal type checks with `is`, and routing
  output with `route_to`
- [Function Reference](./functions.md) -- built-in functions for string
  manipulation, hashing, encoding, math, date/time formatting, and more
