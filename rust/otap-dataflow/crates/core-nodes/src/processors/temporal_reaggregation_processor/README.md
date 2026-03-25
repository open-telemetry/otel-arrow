# Temporal Reaggregation Processor

Status: **WIP**

URN: `urn:otel:processor:temporal_reaggregation`

## Overview

The temporal reaggregation processor decreases telemetry volume by
reaggregating metrics collected at a higher frequency into a lower one.
This is one of the three semantics-preserving transformations outlined in the
[Metrics Data Model specification][metrics-data-model].

Reaggregation works by collecting metrics for a fixed interval and emitting a
single data point per stream when the interval expires. For each stream, only
the most recent data point (by timestamp) is kept.

This processor is modeled after the [Go interval processor][go-interval].

[metrics-data-model]: https://opentelemetry.io/docs/specs/otel/metrics/data-model/#events--data-stream--timeseries
[go-interval]: https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/v0.147.0/processor/intervalprocessor

## Supported Metric Types

The following metric types are aggregated (kept for the interval, latest
value emitted):

- Monotonically increasing, cumulative sums
- Monotonically increasing, cumulative histograms
- Monotonically increasing, cumulative exponential histograms
- Gauges (unless `pass_through.gauge` is set)
- Summaries (unless `pass_through.summary` is set)

The following metric types are passed through unchanged:

- All delta metrics
- Non-monotonically increasing sums

Non-metrics signals (logs, traces) are always passed through unchanged.

> **Note:** Aggregating over an interval is inherently lossy. For cumulative
> sums, histograms, and exponential histograms, you lose precision but not
> overall data. For gauges and summaries, aggregation represents actual data
> loss (e.g., a value that increased then decreased back would appear
> unchanged). Use the `pass_through` options if this is undesirable.

## Configuration

```yaml
temporal-reaggregation:
  type: urn:otel:processor:temporal_reaggregation
  config:
    # The interval at which aggregated metrics are emitted.
    # Default: 60s
    period: 60s

    pass_through:
      # Whether gauge metrics should be passed through unchanged.
      # Default: false
      gauge: false

      # Whether summary metrics should be passed through unchanged.
      # Default: false
      summary: false
```

## Example

Given the following incoming sum metrics:

<!-- markdownlint-disable MD013 -->

| Timestamp | Metric Name | Temporality | Attributes   | Value |
|-----------|-------------|-------------|--------------|-------|
| 0         | test_metric | Cumulative  | labelA: foo  | 4.0   |
| 2         | test_metric | Cumulative  | labelA: bar  | 3.1   |
| 4         | other       | Delta       | type: orange | 77.4  |
| 6         | test_metric | Cumulative  | labelA: foo  | 8.2   |
| 8         | test_metric | Cumulative  | labelA: foo  | 12.8  |
| 10        | test_metric | Cumulative  | labelA: bar  | 6.4   |

<!-- markdownlint-enable MD013 -->

The processor immediately passes through:

| Timestamp | Metric Name | Temporality | Attributes   | Value |
|-----------|-------------|-------------|--------------|-------|
| 4         | other       | Delta       | type: orange | 77.4  |

At the next interval, the processor emits:

| Timestamp | Metric Name | Temporality | Attributes  | Value |
|-----------|-------------|-------------|-------------|-------|
| 8         | test_metric | Cumulative  | labelA: foo | 12.8  |
| 10        | test_metric | Cumulative  | labelA: bar | 6.4   |

After emitting, internal state is cleared. If no new metrics arrive, the
next interval emits nothing.
