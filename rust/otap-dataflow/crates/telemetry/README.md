# Internal Telemetry System (schema-first, multivariate, NUMA-aware)

Status: draft under active development.

A low-overhead, NUMA-aware telemetry system that turns a declarative schema into
a type-safe Rust API for emitting richly structured, multivariate metrics. It is
designed for engines that run a thread-per-core and require predictable latency
while still exporting high-fidelity operational data.

## Core principles

1. **Schema-first**: You declare a metric schema (attributes + instrument kinds)
   and derive strongly typed metric sets. This eliminates stringly-typed
   lookups, guarantees field ordering, and lets downstream tooling reason about
   the data shape at compile time.
2. **Native multivariate metrics**: A metric set groups multiple instruments
   that share identical attribute tuples and timestamps. Collection exports
   sparse non-zero field/value pairs, avoiding per-field overhead and reducing
   wire size.
3. **Performance focus**: Counter increments are zero-cost in steady state (no
   atomics, no branching beyond range checks) by leveraging per-core ownership
   and cache alignment. The cold path (flush, aggregate, encode) is NUMA-aware
   and batch oriented, separating mutation from collection.
4. **Auto-describing**: From the same schema we generate OpenTelemetry semantic
   descriptors so the system can describe its own telemetry: instrument kinds,
   units, brief docs, and attribute keys. Exporters can attach this metadata
   once, enabling self-describing streams.

## Architectural highlights

- Per-core metric sets: each core mutates only its own instance => no cross-core
  contention.
- Reset-on-flush semantics: values accumulate for a cadence (e.g. 100 ms) then
  are atomically snapshotted and zeroed, yielding deltas by construction.
- Sparse enumeration: only non-zero fields are walked; zeroing touches only
  dirty counters.
- Descriptor & schema statics: each generated metric set exposes a
  `MetricsDescriptor` with an ordered slice of `MetricsField` (name, unit,
  instrument kind, brief). Similarly, a `AttributesDescriptor` provides
  attribute keys and their types.
- Registry & reflection: a global registry tracks live metric sets, enabling
  periodic flush loops without bespoke wiring.
- Transport decoupling (aka bottom half of the SDK): snapshot batches move over
  MPSC queues to aggregation / export workers.

![Architecture Phase 1](assets/Metrics%20Phase%201.svg)

## Metric Macros

The `#[metric_set]` macro generates a strongly typed metric set from a Rust
struct definition.

The `#[attribute_set]` macro generates a strongly typed attribute set from a
Rust struct definition.

See the [telemetry-macros crate](../telemetry-macros) for details.

## Logging Macros

There are internal macros defined in `otap_df_telemetry` with names
`otel_info!`, `otel_warn!`, `otel_error!`, and `otel_debug!`. These
macros all require a constant event-name string as the first argument;
the event name must follow
[OpenTelemetry Event naming conventions](../../docs/telemetry/events-guide.md#event-naming)
(lowercase, dot-separated, stable, low-cardinality). The `target`
(equivalent to OpenTelemetry `InstrumentationScope.name`) is set to the
component target established by `otel_component_scope!`, or to the Cargo
package name outside registered component modules. Otherwise, the macros follow
Tokio `tracing` syntax for key-value expressions.

Registered components bind their target once in the component root, before
declaring child modules:

```rust
pub const TRANSFORM_PROCESSOR_URN: &str = "urn:otel:processor:transform";

otap_df_telemetry::otel_component_scope!(
    urn = TRANSFORM_PROCESSOR_URN,
    target = "otel.processor.transform",
);

mod config;
mod routing;
```

The component and all child modules can then use `otel_info!`, `otel_debug!`,
and the other event macros without repeating a target. The target is the
component URN without the `urn:` prefix and with colons replaced by dots. The
macro validates that projection at compile time, so the target above cannot
drift from `urn:otel:processor:transform`.

The component target applies to instrumentation owned by that module subtree.
Events emitted by shared libraries retain the shared library's target, as is
normal for `tracing`; enabling a caller's target does not automatically enable
targets used by its dependencies. Component modules should use the unqualified
local `otel_*` macros because fully qualified calls to the base macros retain
the Cargo package target.

For example:

```rust
use otap_df_telemetry::otel_info;

otel_info!(
    "syslog_cef_receiver.start",
    protocol = "TCP",
    listening_addr = tcp_config.listening_addr.to_string()
);
```

## Internal telemetry collection

The dataflow engine supports multiple ways to configure internal logs and
metrics. Log provider modes determine how logging is configured in different
parts of the code.

All modes use a [`tracing_subscriber::EnvFilter`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html).
The `engine.telemetry.logs.level` field accepts either a severity such as
`warn` or a complete target directive string. Successful full-engine
reconciliation applies changes to this field to existing tracing subscribers,
so an OpAMP or admin control plane can temporarily increase verbosity without
restarting the engine. Failed reconciliation preserves the active filter.

`EnvFilter` target directives use prefix matching. A directive for
`<namespace>.<kind>.<name>` also matches another component whose target begins
with that complete string. For example, `otel.processor.transform` also
matches a hypothetical `otel.processor.transform_extra` target.

When an investigation also needs diagnostics from a shared library, enable
both targets. For example, OTLP receiver HTTP diagnostics include events owned
by `otap-df-otap`:

```text
warn,otel.receiver.otlp=debug,otap-df-otap=debug
```

At startup, a valid `RUST_LOG` environment variable takes precedence over
`logs.level`. After startup, a successful full-engine reconciliation makes the
reconciled `logs.level` authoritative and replaces the environment-derived
filter. This lets an OpAMP or admin control plane reliably change verbosity
even when the process was launched with `RUST_LOG`.

### Limitation: active span state is not reconstructed

`EnvFilter` supports span-scoped directives such as
`warn,[pipeline_thread]=debug`, which raise verbosity only while a matching
span is entered. Those directives work when supplied at startup through
`logs.level` or `RUST_LOG`, but reconciliation cannot apply them to spans that
are already entered.

Reconciliation installs a newly built `EnvFilter` into each live dispatcher.
`EnvFilter` tracks span scopes through `on_new_span` and `on_enter`, so a
replacement filter has no record of spans entered before it was installed and
never pushes them onto its scope stack. A reconciled span directive applies to
matching spans created after the update, including spans created by replacement
pipeline threads. Events inside a long-lived `pipeline_thread` span that was
already entered fall back to the new filter's non-span directives until that
pipeline thread and its span are recreated.
The non-span part of the reconciled `logs.level` takes effect immediately.

There are four aspects that can be configured:

- `engine`: logging for pipeline threads that run dataflow processing
  (receivers, processors, exporters)
- `global`: fallback logging for code outside engine/admin contexts
  (e.g., libraries, startup code)
- `admin`: logging for administrative threads (metrics aggregation,
  observed state store, controller tasks)
- `internal`: logging for the engine observability pipeline itself;
  restricted to `console_direct` or `noop` to avoid feedback loops

These modes are configured through the `engine.telemetry.logs.providers`
field, with the following choices:

- `its`: send logs to the Internal Telemetry System for consumption by the
  internal telemetry receiver in `engine.observability.pipeline`.
- `console_async`: configure asynchronous console logging. In this
  mode log records are printed to the console by the
  observed-state-store thread, avoiding blocking the caller.
- `console_direct`: configure synchronous logging. This mode blocks
  the calling thread to print each log statement immediately.
- `noop`: disables logging.

Periodic internal metrics always flow through the engine observability
pipeline. When configuration omits that pipeline, the engine installs a
built-in pipeline that consumes metrics with a noop exporter. Global and engine
logs continue to use `console_async` by default; the built-in pipeline's console
route is used only when a log provider explicitly selects `its`. A custom
internal telemetry receiver can override the default registry drain and export
interval and apply metric views through its `metrics` block. Its `signals` field
defaults to `[logs, metrics]`; either signal can be omitted. When metric
emission is disabled, the receiver commits the ITS export accumulator without
OTLP conversion or downstream delivery, while leaving the admin metric view
intact.

ITS metric export and admin endpoint reads use independent registry views, so
an admin reset does not consume metrics waiting for pipeline export.
The bridge projects multivariate metric sets into standard univariate OTLP
metrics that normal OTLP or OTAP exporters can consume. This is a transitional
representation pending native multivariate metric-set support in OTAP.

Prometheus scraping is provided by the admin server at the fixed
`/api/v1/metrics` path. Receiver views do not apply to that endpoint.

For more on this design, see the [self-tracing architecture
document](../../docs/self_tracing_architecture.md). See a sample
configuration in
[configs/internal-telemetry-metrics.yaml](../../configs/internal-telemetry-metrics.yaml).

## Roadmap

- Generate OpenTelemetry Semantic Registry from the schema.
- Generate Telemetry client SDK from custom registry and Weaver.
- Structured events and spans.
- NUMA-aware aggregation.

The Internal Telemetry System (ITS) is moving in the direction
depicted below:

![Architecture Phase 2](assets/ITS.svg)

Our own OTAP Dataflow Engine will be configured to consume our internal
telemetry streams, and will be used to export to external backends such as
Prometheus, OTLP-compatible systems, or OTAP-compatible systems.

Note: The recent telemetry guidelines defined in `/docs/telemetry` are
still being implemented in this SDK. Expect changes and improvements
over time.
