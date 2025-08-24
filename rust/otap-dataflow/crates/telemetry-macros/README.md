# Telemetry Macros

These macros help you define metric sets and attribute sets with minimal
boilerplate.

- metric_set: declare a metrics container struct and auto-derive the handler and
  descriptor
- attribute_set: declare an attribute container struct and auto-derive its
  handler and descriptor

Below is a quick guide for defining and using a metric set.

## Define a metric set

- Import instrument types from otap-df-telemetry and the macro from this crate.
- Annotate your struct with `#[metric_set(name = "<metrics.group.name>")]`.
- For each metric field, choose one of the supported instruments and add
  `#[metric(unit = "{unit}")]`.
    - Supported instruments: `Counter<u64>` (`UpDownCounter<u64>`, `Gauge<u64>`
      soon).
    - Units follow a simple string convention (e.g., `{msg}`, `{record}`,
      `{span}`).
- Optional: Document each field with a Rust doc comment; it becomes the metric
  "brief" in the descriptor.
- Optional: Override a field metric name with
  `#[metric(name = "custom.name", unit = "{unit}")]`.
    - If `name` is omitted, the field identifier is converted by replacing `_`
      with `.`.

Example (from the OTAP Perf Exporter):

```rust
use otap_df_telemetry::instrument::{Counter, UpDownCounter, Gauge};
use otap_df_telemetry_macros::metric_set;

/// Pdata-oriented metrics for the OTAP PerfExporter.
#[metric_set(name = "perf.exporter.pdata.metrics")]
#[derive(Debug, Default, Clone)]
pub struct PerfExporterPdataMetrics {
    /// Number of pdata batches received.
    #[metric(unit = "{msg}")]
    pub batches: Counter<u64>,

    /// Number of invalid pdata batches received.
    #[metric(unit = "{msg}")]
    pub invalid_batches: Counter<u64>,

    /// Number of Arrow records received.
    #[metric(unit = "{record}")]
    pub arrow_records: Counter<u64>,

    /// Number of logs received.
    #[metric(unit = "{log}")]
    pub logs: Counter<u64>,

    /// Number of spans received.
    #[metric(unit = "{span}")]
    pub spans: Counter<u64>,

    /// Number of metrics received.
    #[metric(unit = "{metric}")]
    pub metrics: Counter<u64>,
}
```

Notes:

- The macro injects `#[repr(C, align(64))]` for better cache-line isolation.
- The macro also derives the required handler to integrate with the metrics
  registry.
- Support status: `UpDownCounter` and `Gauge` exist but are not yet fully
  supported end-to-end.

### Macro expansion (simplified)

The `#[metric_set]` macro expands your struct by adding attributes and a handler
implementation roughly like this:

```rust
#[repr(C, align(64))]
#[derive(Debug, Default, Clone, otap_df_telemetry_macros::MetricSetHandler)]
#[metrics(name = "perf.exporter.pdata.metrics")]
pub struct PerfExporterPdataMetrics {
    // same fields as above
    pub batches: otap_df_telemetry::instrument::Counter<u64>,
    pub invalid_batches: otap_df_telemetry::instrument::Counter<u64>,
    // ...
}

impl otap_df_telemetry::metrics::MetricSetHandler for PerfExporterPdataMetrics {
    fn descriptor(&self) -> &'static otap_df_telemetry::descriptor::MetricsDescriptor {
        static DESCRIPTOR: otap_df_telemetry::descriptor::MetricsDescriptor = otap_df_telemetry::descriptor::MetricsDescriptor {
            name: "perf.exporter.pdata.metrics",
            metrics: &[
                otap_df_telemetry::descriptor::MetricsField {
                    name: "batches",
                    unit: "{msg}",
                    brief: "Number of pdata batches received.",
                    instrument: otap_df_telemetry::descriptor::Instrument::Counter,
                },
                otap_df_telemetry::descriptor::MetricsField {
                    name: "invalid.batches",
                    unit: "{msg}",
                    brief: "Number of invalid pdata batches received.",
                    instrument: otap_df_telemetry::descriptor::Instrument::Counter,
                },
                // ... other fields
            ],
        };
        &DESCRIPTOR
    }

    fn snapshot_values(&self) -> Vec<u64> {
        let mut out = Vec::with_capacity(self.descriptor().metrics.len());
        out.push(self.batches.get());
        out.push(self.invalid_batches.get());
        // ... other fields
        out
    }

    fn clear_values(&mut self) {
        self.batches.reset();
        self.invalid_batches.reset();
        // ... other fields
    }

    fn needs_flush(&self) -> bool {
        if self.batches.get() != 0 { return true; }
        if self.invalid_batches.get() != 0 { return true; }
        // ... other fields
        false
    }
}
```

Note: For performance reasons, we could introduce in the future an unsafe
implementation of the `snapshot_values` and `clear_values`.

## Register and use a metric set

- In a node or component that has a `PipelineContext`, register your metric set
  to get a `MetricSet<T>` handle.
- Use the instrument methods to record values. Common methods:
    - Counter: `inc()`, `add(u64)`
    - UpDownCounter: `inc()`, `dec()`, `add(u64)`, `sub(u64)`
    - Gauge: `set(u64)`, `inc()`, `dec()`, `add(u64)`, `sub(u64)`
- The engine’s telemetry cycle will snapshot and clear values as needed.

Example usage in a node:

```rust
use otap_df_engine::context::PipelineContext;
use otap_df_telemetry::metrics::MetricSet;

pub struct MyNode {
    metrics: MetricSet<PerfExporterPdataMetrics>,
}

impl MyNode {
    pub fn new(pipeline_ctx: PipelineContext) -> Self {
        let metrics = pipeline_ctx.register_metrics::<PerfExporterPdataMetrics>();
        Self { metrics }
    }

    pub fn on_batch(&mut self, records: u64) {
        self.metrics.batches.inc();
        self.metrics.arrow_records.add(records);
    }
}
```

Collecting on control messages (periodic telemetry):

```
Message::Control(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
    // Report and clear current metrics snapshot
    let _ = metrics_reporter.report(&mut self.metrics).await;
}
```