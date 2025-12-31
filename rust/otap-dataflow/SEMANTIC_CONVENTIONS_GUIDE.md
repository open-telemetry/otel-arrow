# OpenTelemetry Semantic Conventions – Contributor Guide

This document summarizes the **core rules and guidelines contributors must
follow in this project** when defining **metric names**, **units**,
**attributes**, and **event metadata**.

Its goal is to **ease contributor work** by providing a clear, concise, and
opinionated reference tailored to this project.

**Important**
All rules and conventions described here are **derived directly from the
official OpenTelemetry Semantic Conventions**.
The OpenTelemetry specifications are the **ultimate source of truth**, and this
document must never be considered authoritative on its own.

Primary references:

* General naming conventions:
  [https://opentelemetry.io/docs/specs/semconv/general/naming/](https://opentelemetry.io/docs/specs/semconv/general/naming/)
* Metric semantic conventions:
  [https://opentelemetry.io/docs/specs/semconv/general/metrics/](https://opentelemetry.io/docs/specs/semconv/general/metrics/)

Contributors are expected to **consult the upstream OTel documentation**
whenever ambiguity exists or when introducing new semantics.

---

## Table of Contents

1. General Naming Conventions
2. Metric Naming and Semantics
3. Units Guidelines
4. Events and Attributes
5. Examples and Best Practices
6. Contributor Checklist

---

## 1. General Naming Conventions

These rules apply to **metric names**, **attribute names**, **event names**, and
other semantic identifiers.

### Core Rules

* Names **must be lowercase**.
* Use **dot (`.`) separators** to express hierarchy and namespaces.
* Use **underscores (`_`) only inside a single namespace segment** to separate
  words.
* Names must:

    * Start with a letter
    * End with an alphanumeric character
    * Not contain consecutive delimiters (`..`, `__`)
* Avoid ambiguous, overloaded, or generic names.
* Abbreviations are allowed **only when widely understood** (e.g. `http`, `cpu`,
  `db`).
* A semantic identifier must have **one clear meaning** and must not conflict
  with existing conventions.

Source:
[https://opentelemetry.io/docs/specs/semconv/general/naming/](https://opentelemetry.io/docs/specs/semconv/general/naming/)

---

## 2. Metric Naming and Semantics

### Metric Names

* Metric names **must follow general naming conventions**.
* Names should represent **what is being measured**, not how it is aggregated.
* Prefer **nouns** or **noun phrases**.
* Do **not encode units in metric names** when unit metadata is available.
* Do **not append `_total`** or other backend-specific suffixes in OTel metrics.
* Do **not pluralize** metric names unless they represent a count of discrete
  entities.

Examples:

```
http.server.request.duration
system.cpu.time
process.memory.usage
```

Source:
[https://opentelemetry.io/docs/specs/semconv/general/metrics/](https://opentelemetry.io/docs/specs/semconv/general/metrics/)

---

### Metric Attributes

* Attributes add **dimensions**, not meaning.
* Reuse existing semantic attributes whenever possible.
* Attribute names must follow the same naming rules as metrics.
* **Avoid attributes that introduce high cardinality unless explicitly
  required.**
* Attribute sets must remain meaningful under aggregation.

Example:

```
http.server.request.duration{http.method="GET", http.status_code=200}
```

---

### Instrument Semantics

* Counters represent **monotonically increasing values**.
* UpDownCounters represent values that may increase or decrease.
* Gauges represent **instantaneous measurements**.
* Histograms represent **distributions of measurements**.

The instrument type must align with the semantic meaning of the metric.

---

## 3. Units Guidelines

### General Rules

* Units **must not be embedded in metric names**.
* Units must be provided as metric metadata.
* Units should follow **UCUM conventions**.
* Units must be **unambiguous and self-contained**.

Examples:

* `s` for seconds
* `By` for bytes
* `1` for dimensionless ratios

Source:
[https://opentelemetry.io/docs/specs/semconv/general/metrics/#units](https://opentelemetry.io/docs/specs/semconv/general/metrics/#units)

---

### Duration and Time

* Durations should be expressed in **seconds (`s`)**.
* Time counters should also use `s`.

Example:

```
process.cpu.time   unit: s
```

---

### Ratios and Utilization

* Ratios and utilization metrics are **dimensionless**.
* Use unit `1`.

Example:

```
system.cpu.utilization   unit: 1
```

---

### Counts

* Count metrics should use **curly-brace units** when applicable.
* Use singular semantic units.

Examples:

```
{request}
{batch}
{signal}
{error}
{connection}
```

---

## 4. Events and Attributes

### Event Naming

* Event names must be **low cardinality** and stable.
* Names must follow general naming conventions.
* Events represent **discrete occurrences**, not continuous measurements.

Examples:

```
http.request.start
pipeline.config.reload
connection.closed
```

---

### Event Attributes

* Attributes provide structured context for events.
* Attribute naming rules are identical to metric attribute rules.
* Use arrays for multiple values when appropriate.
* Avoid duplicating information already present in metric streams unless
  required.

Source:
[https://opentelemetry.io/docs/specs/semconv/general/naming/](https://opentelemetry.io/docs/specs/semconv/general/naming/)

---

## 5. Examples and Best Practices

### Good Metric Examples

```
http.server.request.duration   unit: s
system.memory.usage            unit: By
system.cpu.utilization         unit: 1
```

### Good Attribute Examples

```
http.method = "GET"
http.status_code = 200
network.transport = "tcp"
```

### Anti-Patterns

Avoid:

* Units in names: `http_request_duration_seconds`
* Backend-specific suffixes: `_total`, `_count`
* Overloaded names with multiple meanings
* High-cardinality attributes by default

---

## 6. Contributor Checklist

Before introducing a new metric or event, verify:

* The name follows OTel naming rules.
* Existing semantic conventions do not already cover the use case.
* Units are expressed via metadata and follow UCUM.
* Instrument type matches semantic intent.
* Attributes are reusable, well-scoped, and low cardinality.
* Meaning remains clear under aggregation.

When in doubt, **refer to the upstream OpenTelemetry Semantic Conventions**,
which remain the authoritative source.
