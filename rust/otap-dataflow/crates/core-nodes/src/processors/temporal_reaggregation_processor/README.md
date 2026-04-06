# Temporal Reaggregation Processor

Status: **WIP**

URN: `urn:otel:processor:temporal_reaggregation`

## Overview

The temporal reaggregation processor decreases telemetry volume by
reaggregating metrics collected at a higher frequency into a lower one.
This is one of the three semantics-preserving transformations outlined in the
[Metrics Data Model specification][metrics-data-model].

This processor is partially modeled after the [Go interval processor][go-interval].

[metrics-data-model]: https://opentelemetry.io/docs/specs/otel/metrics/data-model/#events--data-stream--timeseries
[go-interval]: https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/v0.147.0/processor/intervalprocessor

## Supported metrics

This processor only aggregates a subset of metric types. In particular:

- Cumulative monotonic sums
- Cumulative histograms
- Cumulative exponential histograms
- Gauges
- Summaries

Other metric types are passed through unchanged.

## Limitations

This processor has the following limitations:

- Exemplars for aggregated metrics are dropped, however exemplars for passed through metrics are
preserved.
- Array and Map attribute value types are discarded for the purpose of identifying metrics for
OtapArrowRecords only.

## Configuration

```yaml
temporal-reaggregation:
  type: urn:otel:processor:temporal_reaggregation
  config:
    # The interval at which aggregated metrics are emitted.
    # Default: 60s
    period: 60s
```
