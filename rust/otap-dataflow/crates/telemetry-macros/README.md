# Telemetry Macros

These macros help you define metric sets and attribute sets with minimal
boilerplate.

- metric_set: declare a metrics container struct and auto-derive the handler and
  descriptor
- attribute_set: declare an attribute container struct and auto-derive its
  handler and descriptor

Below is a quick guide for defining and using a metric set.

## Metric and attribute set guidance

The [telemetry documentation](../../docs/telemetry/README.md) is the canonical
source for metric and attribute-set design:

- The [metrics guide](../../docs/telemetry/metrics-guide.md) defines metric-set
  scope, naming, units, instrument semantics, and bounded dimensions.
- The [item attributes guide](../../docs/telemetry/item-attributes.md) defines
  registration and measurement attributes, including complete declaration and
  recording examples.

## Macro API

- Import instrument types from otap-df-telemetry and the macro from this crate.
- Annotate your struct with `#[metric_set(name = "<metrics.group.name>")]`.
- For each metric field, choose one of the supported instruments and add
  `#[metric(unit = "<unit>")]`.
  - Supported instruments: `Counter<u64|f64>`, `UpDownCounter<u64|f64>`,
    `ObserveCounter<u64|f64>`, `ObserveUpDownCounter<u64|f64>`,
    `Gauge<u64|f64>`, and `Mmsc`.
  - Units follow UCUM conventions, for example `By`, `s`, `1`, and annotation
    units such as `{item}`.
- Optional: Document each field with a Rust doc comment; it becomes the metric
  "brief" in the descriptor.
- Optional: Override a field metric name with
  `#[metric(name = "custom.name", unit = "<unit>")]`.
  - If `name` is omitted, the field identifier is converted by replacing `_`
    with `.`.
- The macro injects `#[repr(C, align(64))]` for better cache-line isolation.
- The macro also derives the required handler to integrate with the metrics
  registry.
