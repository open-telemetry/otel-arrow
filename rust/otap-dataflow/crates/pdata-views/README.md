# otel-arrow-dfe-pdata-views

This crate is currently pre-1.0. Its public API may evolve between minor
releases.

Zero-dependency, backend-agnostic view traits for OTLP/OTAP telemetry data.

## Overview

This crate provides read-only view traits for traversing hierarchical
telemetry data structures (logs, metrics, traces, resources) without external
dependencies. It is designed to be consumed both within the
`otap-dataflow` workspace and by external crates that need a lightweight
integration point without pulling in the full `otel-arrow-dfe-pdata` stack.

## Traits

- `views::logs`: `LogsDataView`, `ResourceLogsView`, `ScopeLogsView`,
  `LogRecordView`
- `views::metrics`: `MetricsView`, resource and scope views, metric data
  views, data-point views, exemplars, and histogram bucket views
- `views::trace`: `TracesView`, `ResourceSpansView`, `ScopeSpansView`,
  `SpanView`, `EventView`, `LinkView`, `StatusView`
- `views::resource`: `ResourceView`
- `views::common`: `AnyValueView`, `AttributeView`,
  `InstrumentationScopeView`, `ValueType`, `Str`

## Usage

```sh
cargo add otel-arrow-dfe-pdata-views
```

Implement the relevant view trait over your existing data structure to
plug into any pipeline that consumes these traits (e.g. `geneva-uploader`).
Consumers can write generic code against those traits:

```rust
use otel_arrow_dfe_pdata_views::views::metrics::MetricsView;

fn resource_group_count(metrics: &impl MetricsView) -> usize {
    metrics.resources().count()
}
```

## Dependencies

Intentionally none.
