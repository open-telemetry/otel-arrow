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

Declare it with an empty config inside `engine.observability.pipeline`:

The observability pipeline is defined under `engine.observability` rather than
inside a user pipeline group, so it is owned by the engine and cannot be
referenced from groups.

```yaml
engine:
  telemetry:
    metrics:
      provider: its
  observability:
    pipeline:
      nodes:
        internal:
          type: receiver:internal_telemetry
          config: {}
        console:
          type: exporter:console
          config: {}
      connections:
        - from: internal
          to: console
```

## Configuration

This receiver has no node-specific configuration.

```yaml
type: receiver:internal_telemetry
config: {}
```

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
- Internal metric batches are emitted on `engine.telemetry.reporting_interval`.
- The receiver drains an export-specific registry view; admin endpoint reads
  and resets do not consume its pending metrics.

## Related Docs

- [Configuration model observability pipeline](../../../../../docs/configuration-model.md#observability-pipeline)
- [Telemetry guide](../../../../../docs/telemetry/README.md)
- [Core node catalog](../../../README.md)
