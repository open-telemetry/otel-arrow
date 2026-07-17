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

Declare it inside `engine.observability.pipeline` and select the `its` metrics
provider:

The observability pipeline is defined under `engine.observability` rather than
inside a user pipeline group, so it is owned by the engine and cannot be
referenced from groups.

```yaml
engine:
  telemetry:
    reporting_interval: 1s
    metrics:
      provider: its
  observability:
    pipeline:
      nodes:
        internal:
          type: receiver:internal_telemetry
          config:
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

The receiver can override the metric emission interval and apply a supported
subset of OpenTelemetry metric views. If `metrics.interval` is omitted, it
inherits `engine.telemetry.reporting_interval`.

```yaml
type: receiver:internal_telemetry
config:
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
- The `its` metrics provider requires exactly one internal telemetry receiver
  in the observability pipeline.
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
