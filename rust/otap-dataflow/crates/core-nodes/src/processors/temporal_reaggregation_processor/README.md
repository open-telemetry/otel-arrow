# Temporal Reaggregation Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:temporal_reaggregation` (`urn:otel:processor:temporal_reaggregation`)
- Feature gate: Default
- Stability: Experimental

## Overview

The temporal reaggregation processor decreases telemetry volume by reaggregating
metrics collected at a higher frequency into a lower one. This is one of the
three semantics-preserving transformations outlined in the [Metrics Data Model
specification][metrics-data-model].

This processor is partially modeled after the [Go interval
processor][go-interval].

## Getting Started

Set the aggregation period and request tracking limits:

```yaml
type: processor:temporal_reaggregation
config:
  period: 60s
  inbound_request_limit: 1024
  outbound_request_limit: 2048
  max_stream_cardinality: 16384
```

## Configuration

```yaml
type: processor:temporal_reaggregation
config:
  # Interval at which aggregated metrics are emitted (default: 60s,
  # minimum: 100ms).
  period: 60s

  # Maximum number of inbound request contexts buffered for ack/nack tracking
  # (default: 1024).
  inbound_request_limit: 1024

  # Maximum number of outbound request contexts buffered for ack/nack tracking
  # (default: 2048, minimum: 3).
  #
  # Set this higher than inbound_request_limit when batches usually contain a
  # mix of aggregable and non-aggregable metrics.
  #
  # Set this closer to inbound_request_limit when batches usually contain only
  # aggregable metrics or only non-aggregable metrics.
  outbound_request_limit: 2048

  # Maximum number of individual metric streams tracked while aggregating a
  # single batch (default: 16383). When this limit is hit, data is flushed
  # early.
  max_stream_cardinality: 16384
```

[metrics-data-model]:
  https://opentelemetry.io/docs/specs/otel/metrics/data-model/#events--data-stream--timeseries
[go-interval]:
  https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/v0.147.0/processor/intervalprocessor

## Supported metrics

This processor only aggregates a subset of metric types. In particular:

- Cumulative monotonic sums
- Cumulative histograms
- Cumulative exponential histograms
- Gauges
- Summaries

Other metric types are passed through unchanged.

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `processor.temporal_reaggregation.pdata`

| Metric | Unit | Description |
| --- | --- | --- |
| `processor.temporal_reaggregation.pdata.flushes_timer` | `{flush}` | Number of flushes triggered by the regular timer. |
| `processor.temporal_reaggregation.pdata.flushes_overflow` | `{flush}` | Number of flushes triggered by exceeding the maximum stream count. |
| `processor.temporal_reaggregation.pdata.batches_rejected` | `{batch}` | Number of incoming batches rejected because they individually exceed some specified limit or fail to be processed into a view. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `temporal_reaggregation.view.creation_failed` | `warn` | A view could not be created over input data. |
| `temporal_reaggregation.attribute.encode_failed` | `warn` | One or more attributes could not be encoded. |
| `temporal_reaggregation.calldata.invalid` | `warn` | Returned calldata was invalid for the processor return path. |
| `temporal_reaggregation.ack.erroneous` | `warn` | An erroneous ACK/NACK event was observed. |

## Limits

This processor has the following limitations:

- Exemplars for aggregated metrics are dropped, however exemplars for passed
  through metrics are preserved.
- Array and Map attribute values are discarded for the purpose of identifying
  metrics due to a current limitation with OTAP views.
- During a shutdown sequence, any in-flight metrics will be aggregated one final
  time until either a stream/id limit is reached or the shutdown message is
  received by the processor.

## Pipeline placement

It is recommended to place this processor:

1. Before any batch processor in the same pipeline, as this processor will
   resize batches and generally produces a larger number of smaller output
   batches than were input.
2. Before any retry processors in the same pipeline because this processor does
   not support returning pdata for similar reasons to the batch processor in
   that it's memory expensive to hold them.

A typical pipeline ordering with batch, temporal_reaggregation, and retry
processors would be:
`receivers -> temporal_reaggregation -> batch -> retry -> exporters`

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Core node catalog](../../../README.md)
