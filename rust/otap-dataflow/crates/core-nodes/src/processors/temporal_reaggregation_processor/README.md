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

## Configuration

```yaml
temporal-reaggregation:
  type: urn:otel:processor:temporal_reaggregation
  config:
    # The interval at which aggregated metrics are emitted.
    # Default: 60s
    period: 60s
```
