# `core.telemetry` - Observability & Telemetry Layer

The `core.telemetry` package provides structured support for observability within the test orchestration framework. It integrates **OpenTelemetry** standards with a framework-native telemetry model, offering unified access to **metrics**, **traces (spans)**, **events**, and **logs**.

This package is designed to enable **pluggable, extensible telemetry pipelines** for performance monitoring, distributed tracing, and debugging-supporting both in-memory and remote backends.

---

## Overview of Modules

### `framework_event.py` - **Framework Lifecycle Events**

Defines `FrameworkEvents`, which represent key lifecycle transitions (e.g., setup, teardown, execution start) in the orchestration framework.

- Emitted automatically by the framework during context changes.
- Can be consumed by:
  - Reporting strategies
  - Telemetry instrumentation
  - Debugging tools (e.g., distributed traces)

> These events provide semantic markers for monitoring and analysis.

---

### `telemetry_client.py` - **Telemetry Aggregation Interface**

Defines `TelemetryClient`, a lightweight container that aggregates telemetry sources such as:

- Metrics retrievers
- Span retrievers

> Provides unified access to all structured telemetry data via a single object.

---

### `telemetry_runtime.py` - **Telemetry Runtime Environment**

Defines `TelemetryRuntime`, the orchestration point for telemetry providers and clients.

- Initializes OpenTelemetry components:
  - `TracerProvider` for traces
  - `MeterProvider` for metrics
- Hosts the `TelemetryClient` for telemetry access.
- Manages local or remote backends for data persistence.

> Central access point for telemetry tooling across the framework.

---

### `signal_retriever.py` - **Abstract Signal Access Layer**

Defines `SignalRetriever`, the abstract base interface for accessing telemetry signals.

- Base for:
  - `MetricsRetriever`
  - `SpanRetriever`
- Supports retrieval of telemetry data via filters and time ranges.

> Provides a uniform API to build concrete retrievers for different signal types.

---

### `log.py` - **Structured Logging Integration**

Extends Python's built-in logging to integrate with OpenTelemetry spans.

- Custom log handler that:
  - Emits log records as span events.
  - Associates logs with distributed trace context.

> Enables trace-aware structured logging with minimal overhead.

---

### `metric.py` - **Metric Storage, Querying & Exporting**

Implements a full pipeline for structured metrics telemetry, based on OpenTelemetry.

**Key Components:**

- `MetricRow`: Typed schema for a single metric record.
- `MetricDataFrame`: `pandas.DataFrame` subclass with schema validation and rich filtering.
- `MetricDataBackend`: Interface for providing normalized metric data.
- `MetricsRetriever`: Extends `SignalRetriever` to support attribute/time-range queries.
- `FrameworkMetricBackend`: In-memory backend for metrics with caching and transformation.
- `FrameworkMetricsRetriever`: Pulls metrics from the in-memory backend.
- `FrameworkMetricExporter`: Exports OpenTelemetry metrics into the in-memory store.

> Supports both local metrics collection and future remote backends (e.g., Prometheus, OTLP collectors).

---

### `span.py` - **Span Storage, Querying & Exporting**

Implements a full pipeline for trace span and event telemetry.

**Key Components:**

- `SpanRow` / `SpanEventRow`: Typed schemas for spans and events.
- `SpanDataFrame` / `SpanEventDataFrame`: DataFrame subclasses with validation and filtering.
- `SpanDataBackend`: Interface for span data access from various sources.
- `SpanRetriever`: Extends `SignalRetriever` to support rich span filtering (by attributes, duration, etc.).
- `FrameworkSpanBackend`: In-memory span store with transformation and caching.
- `FrameworkSpanRetriever`: Accesses spans from the in-memory backend.
- `FrameworkSpanExporter`: Exports spans into the framework's backend.

> Like metrics, this design supports extensibility to remote trace storage (e.g., Jaeger, Zipkin, OTLP receivers).

---

## Component Relationships

```plaintext
TelemetryRuntime
|-- TracerProvider (spans)
|-- MeterProvider (metrics)
|-- TelemetryClient
     |-- MetricsRetriever
     |    |-- MetricDataBackend
     |-- SpanRetriever
          |-- SpanDataBackend
```

- **Exporters** push data into in-memory backends.
- **Retrievers** pull and filter structured data from those backends.
- **Client** unifies access for reporting, analysis, or monitoring hooks.

---

## Summary

The `core.telemetry` package brings observability into the test orchestration framework, enabling:

- **Metrics collection** and analysis
- **Distributed tracing** for execution insight
- **Lifecycle event tracking**
- **Trace-aware structured logging**

By abstracting telemetry access and supporting both local and remote backends, this layer empowers the framework to provide deep insights into system behavior and performance during test execution.
