# Telemetry SDK (schema‑first, multivariate, NUMA‑aware)

A low-overhead, NUMA-aware telemetry SDK that turns a declarative schema into a
type-safe Rust API for emitting richly structured, multivariate metrics. It is
designed for engines that run a thread-per-core and require predictable latency
while still exporting high-fidelity operational data.

## Core principles

1. **Schema‑first**: You declare a metric schema (attributes + instrument kinds)
   and derive strongly typed metric sets. This eliminates stringly‑typed
   lookups, guarantees field ordering, and lets downstream tooling reason about
   the data shape at compile time.
2. **Native multivariate metrics**: A metric set groups multiple instruments
   that share identical attribute tuples and timestamps. Collection exports
   sparse non‑zero field/value pairs, avoiding per‑field overhead and reducing
   wire size.
3. **Performance focus**: Counter increments are zero‑cost in steady state (no
   atomics, no branching beyond range checks) by leveraging per‑core ownership
   and cache alignment. The cold path (flush, aggregate, encode) is NUMA‑aware
   and batch oriented, separating mutation from collection.
4. **Auto‑describing**: From the same schema we generate OpenTelemetry semantic
   descriptors so the system can describe its own telemetry: instrument kinds,
   units, brief docs, and attribute keys. Exporters can attach this metadata
   once, enabling self‑describing streams.

## Architectural highlights

- Per‑core metric sets: each core mutates only its own instance => no cross‑core
  contention.
- Reset‑on‑flush semantics: values accumulate for a cadence (e.g. 100 ms) then
  are atomically snapshotted and zeroed, yielding deltas by construction.
- Sparse enumeration: only non‑zero fields are walked; zeroing touches only
  dirty counters.
- Descriptor & schema statics: each generated metric set exposes a
  `MetricsDescriptor` with an ordered slice of `MetricsField` (name, unit,
  instrument kind, brief). Similarly, a `AttributesDescriptor` provides
  attribute keys and their types.
- Registry & reflection: a global registry tracks live metric sets, enabling
  periodic flush loops without bespoke wiring.
- Transport decoupling: snapshot batches move over MPSC queues to
  aggregation / export workers.

## Roadmap

- HTTP (pull) telemetry endpoint. 
- OTLP (push) export via Rust OpenTelemetry SDK.
- Generate OpenTelemetry Semantic Registry from the schema.
- Generate Telemetry client SDK from custom registry and Weaver.
- NUMA-aware aggregation.
- Structured events and spans.
