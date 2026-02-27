# otap-df-pdata-views

Zero-dependency, backend-agnostic view traits for OTLP/OTAP telemetry data.

## Overview

This crate provides read-only view traits for traversing hierarchical telemetry data structures
(logs, traces, resources) without any external dependencies. It is designed to be consumed both
within the `otap-dataflow` workspace and by external crates that need a lightweight integration
point without pulling in the full `otap-df-pdata` stack.

## Traits

- `views::logs` — `LogsDataView`, `ResourceLogsView`, `ScopeLogsView`, `LogRecordView`
- `views::trace` — `TracesView`, `ResourceSpansView`, `ScopeSpansView`, `SpanView`, `EventView`, `LinkView`, `StatusView`
- `views::resource` — `ResourceView`
- `views::common` — `AnyValueView`, `AttributeView`, `InstrumentationScopeView`, `ValueType`, `Str`

## Usage

Implement the relevant view trait over your existing data structure to plug into
any pipeline that consumes these traits (e.g. `geneva-uploader`).

## Dependencies

Intentionally none.
