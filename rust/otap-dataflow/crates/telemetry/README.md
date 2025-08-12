# NUMA‑aware telemetry SDK exposing a type‑safe API

A low-overhead, NUMA-aware telemetry infrastructure exposing a type-safe API
used by the thread-per-core OTAP dataflow pipelines.

## Goals and constraints

- Type-safe metric API with zero-cost counters on the hot path (no atomics
  on steady-state per-core updates).
- NUMA locality by construction: per-core metrics live with the node and are
  mutated only by that core.
- Reset-on-flush semantics with configurable aggregation cadence (e.g. every 100
  ms), accepting brief staleness between
  flushes.
- Minimal synchronization for collection: SPSC handoff; global aggregation off
  the hot path.

## Design overview

- Pipeline nodes maintain strongly typed, per-core metrics structs with
  lightweight, single-threaded counters.
- Metrics are reset on flush, not on every update, and aggregated by the
  consumer.
- A `Metrics` trait and global registry enable type-safe reflection and export.
- Periodic snapshotting copies metrics to a queue for off-path aggregation.
- Lock-free, single-producer/single-consumer queues transport snapshots to a
  collector.
- Engine integration supports separate controls for reporting and telemetry
  flushing, managed by a shared timer and exporters.

## Future directions

- Expose the aggregated metrics with the Rust OpenTelemetry SDK.
- Expose the aggregated metrics with a basic HTTP endpoint.
- Use OpenTelemetry semantic conventions plus OTEL Weaver to generate this
  NUMA-aware, type-safe SDK.
- Add a centralized, NUMA-aware collector thread per socket.
- Add support for structured events and spans.
