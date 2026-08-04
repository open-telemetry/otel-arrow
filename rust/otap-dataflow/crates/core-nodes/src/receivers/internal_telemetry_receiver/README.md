# Internal Telemetry Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:internal_telemetry` (`urn:otel:receiver:internal_telemetry`)
- Feature gate: Default
- Stability: Experimental

## Overview

The internal telemetry receiver consumes the engine's own logs and aggregated
metrics from the pipeline context and emits them as OTLP pdata. It is intended
for the engine observability pipeline rather than normal user ingest.

For metrics, the receiver projects each multivariate metric set into standard
univariate OTLP metrics. Normal OTLP and OTAP exporters can consume that pdata.
This bridge is transitional pending native multivariate metric-set support in
OTAP.

## Getting Started

The engine supplies an observability pipeline by default. Its internal
telemetry receiver emits logs and metrics, a type router sends logs to the
console exporter, and metrics go to the noop exporter. Override
`engine.observability.pipeline` to export either signal elsewhere. The pipeline
is engine-owned and cannot be referenced from user groups.

```yaml
engine:
  telemetry:
    reporting_interval: 1s
  observability:
    pipeline:
      nodes:
        internal:
          type: receiver:internal_telemetry
          config:
            signals: [logs, metrics]
            metrics:
              interval: 2s
        debug:
          type: processor:debug
          config:
            verbosity: detailed
            signals: [metrics]
        noop:
          type: exporter:noop
          config: {}
      connections:
        - from: internal
          to: debug
        - from: debug
          to: noop
```

## Configuration

`signals` defaults to `[logs, metrics]`. Either signal can be selected
independently, but the list cannot be empty. A metrics-only receiver requires
the global, engine, and admin log providers to use modes other than `its`.

With `signals: [logs]`, the receiver does not convert or emit OTLP metrics. It
still periodically commits the registry's private ITS export accumulator so
that retired metric sets can be released. This cleanup does not consume the
independent metric view used by the admin endpoint.

The `metrics` block can override the cold-path registry drain and emission
interval and apply a supported subset of OpenTelemetry metric views. When the
block or its `interval` field is omitted, the receiver uses
`engine.telemetry.reporting_interval`. That engine setting also controls the
metric-set snapshot cadence. Setting `metrics` to `null` is invalid.

```yaml
type: receiver:internal_telemetry
config:
  signals: [logs, metrics]
  metrics:
    interval: 2s
    views:
      - selector:
          scope_name: pipeline
          scope_attributes:
            pipeline.group.id: production
          instrument_name: uptime
        stream:
          name: process_uptime
          description: Uptime of a production pipeline process.
```

View selectors use exact matches. `scope_name` selects the metric-set
descriptor, `scope_attributes` requires the metric-set entity to contain every
configured key-value pair, and `instrument_name` selects a field in that set.
Scope attribute selectors support string, integer, floating-point, and boolean
values; values must have the same type and value as the entity attribute.
Non-negative integer selectors also match internal unsigned integer attributes.
The supported stream overrides are `name` and `description`.

This receiver is normally declared inside `engine.observability.pipeline`, not
inside a user ingest pipeline.

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

| Metric | Unit | Description |
| --- | --- | --- |
| *None* | N/A | This node does not register a node-specific metric set. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | This receiver consumes internal telemetry and does not emit additional node-specific events during normal operation. |

## Limits

- The receiver requires internal telemetry settings in the pipeline context.
- It is normally used under `engine.observability.pipeline`.
- The engine observability pipeline must contain exactly one connected internal
  telemetry receiver.
- Logs and metrics can be selected independently with a non-empty `signals`
  list.
- Internal metric batches use the receiver's `metrics.interval`, or
  `engine.telemetry.reporting_interval` when no receiver interval is set.
- Receiver-local views support exact scope name, scalar scope attribute, and
  instrument selectors with metric name and description overrides.
- The receiver drains an export-specific registry view; admin endpoint reads
  and resets do not consume its pending metrics.

## Related Docs

- [Configuration model observability pipeline](../../../../../docs/configuration-model.md#observability-pipeline)
- [Telemetry guide](../../../../../docs/telemetry/README.md)
- [Core node catalog](../../../README.md)
