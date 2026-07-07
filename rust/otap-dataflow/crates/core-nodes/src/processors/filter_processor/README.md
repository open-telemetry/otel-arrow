# Filter Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:filter` (`urn:otel:processor:filter`)
- Feature gate: Default
- Stability: Experimental

## Overview

The filter processor drops metrics, logs, or traces according to configured
include and exclude rules.

For reference, compare the Go Collector
[filter processor](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/processor/filterprocessor/README.md).

## Getting Started

Start with a signal-specific include or exclude rule:

```yaml
type: processor:filter
config:
  logs:
    include:
      match_type: strict
      severity_texts:
        - WARN
        - ERROR
```

## Configuration

The node-level config can define independent filter rules for metrics, logs,
and traces:

```yaml
type: processor:filter
config:
  metrics:
    include:
      match_type: strict
      metric_names:
        - http.server.request.count
        - process.cpu.utilization
    exclude:
      match_type: regexp
      metric_names:
        - ^internal\..*$
  logs:
    include:
      match_type: strict
      resource_attributes:
        - key: deployment.environment
          value: prod
      record_attributes: []
      severity_texts:
        - WARN
        - ERROR
      severity_number:
        min: 13
        match_undefined: false
      bodies:
        - checkout started
        - failed to write to socket
    exclude:
      match_type: strict
      resource_attributes:
        - key: deployment.environment
          value: staging
      record_attributes:
        - key: component
          value: db
        - key: retryable
          value: true
      severity_texts:
        - WARN
      severity_number: null
      bodies:
        - checkout started
    log_record: []
  traces:
    include:
      match_type: strict
      resource_attributes:
        - key: deployment.environment
          value: prod
      span_attributes: []
      span_names:
        - checkout-warn
        - checkout-error
      event_names:
        - checkout-event
      event_attributes: []
      link_attributes: []
    exclude:
      match_type: strict
      resource_attributes:
        - key: deployment.environment
          value: staging
      span_attributes:
        - key: component
          value: db
      span_names:
        - payment-warn
        - payment-error
      event_names:
        - payment-event
      event_attributes:
        - key: success
          value: false
      link_attributes:
        - key: correlation
          value: false
```

For a runnable metric-name filter pipeline, see
[`configs/trafficgen-metric-filter-debug-noop.yaml`](../../../../../configs/trafficgen-metric-filter-debug-noop.yaml).

### Metrics

To filter metrics, define `metrics.include` or `metrics.exclude` with a
`match_type` and `metric_names`. Supported `match_type` values are `strict` and
`regexp`. When both `include` and `exclude` are defined, include filtering runs
first, and exclude filtering is applied to that result.

### Logs

To filter logs you can choose to define logs to `include` or `exclude`.
You can also choose to define both, if both are defined then the result
will be the intersection of the two. Currently we allow you to filter
based on `resource_attributes` (all the attributes must match),
`record_attributes` (only one in the list has to match), `severity_texts`,
`severity_number`, and `bodies`. When defining the `severity_number` you set
the min acceptable `severity_number` you can also choose whether to match
on undefined

### Traces

To filter traces, just like logs, you define the `include` or `exclude` fields.
You can filter based on `resource_attributes` (all the attributes must match,
for each of the remaining fields only one entry has to match),
`span_attributes`, `span_names`, `event_names`, `event_attributes` and
`link_attributes`.

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `processor.filter.pdata`

| Metric | Unit | Description |
| --- | --- | --- |
| `processor.filter.pdata.log.batches.seen` | `{batch}` | Number of log batches received by the filter processor. Counted on receipt, before conversion/decode, so batches that later fail conversion or decode are still included. |
| `processor.filter.pdata.log.include.configured.batches` | `{batch}` | Number of log batches received while an include filter was configured. Counted on receipt alongside `log.batches.seen`, before conversion/decode and filtering. It does not imply filtering ran successfully or that any records matched the include rule. |
| `processor.filter.pdata.log.exclude.configured.batches` | `{batch}` | Number of log batches received while an exclude filter was configured. Counted on receipt alongside `log.batches.seen`, before conversion/decode and filtering. It does not imply filtering ran successfully or that any records matched the exclude rule. |
| `processor.filter.pdata.log.signals.consumed` | `{log}` | Number of log signals consumed. |
| `processor.filter.pdata.log.signals.kept` | `{log}` | Number of log signals kept (consumed minus filtered). |
| `processor.filter.pdata.log.signals.filtered` | `{log}` | Number of log signals filtered. |
| `processor.filter.pdata.metric.batches.seen` | `{batch}` | Number of metric batches received by the filter processor. Counted on receipt, before conversion/decode, so batches that later fail conversion or decode are still included. |
| `processor.filter.pdata.metric.include.configured.batches` | `{batch}` | Number of metric batches received while an include filter was configured. Counted on receipt alongside `metric.batches.seen`, before conversion/decode and filtering. It does not imply filtering ran successfully or that any records matched the include rule. |
| `processor.filter.pdata.metric.exclude.configured.batches` | `{batch}` | Number of metric batches received while an exclude filter was configured. Counted on receipt alongside `metric.batches.seen`, before conversion/decode and filtering. It does not imply filtering ran successfully or that any records matched the exclude rule. |
| `processor.filter.pdata.metric.signals.consumed` | `{metric}` | Number of metric signals consumed. |
| `processor.filter.pdata.metric.signals.kept` | `{metric}` | Number of metric signals kept (consumed minus filtered). |
| `processor.filter.pdata.metric.signals.filtered` | `{metric}` | Number of metric signals filtered. |
| `processor.filter.pdata.span.batches.seen` | `{batch}` | Number of span batches received by the filter processor. Counted on receipt, before conversion/decode, so batches that later fail conversion or decode are still included. |
| `processor.filter.pdata.span.include.configured.batches` | `{batch}` | Number of span batches received while an include filter was configured. Counted on receipt alongside `span.batches.seen`, before conversion/decode and filtering. It does not imply filtering ran successfully or that any records matched the include rule. |
| `processor.filter.pdata.span.exclude.configured.batches` | `{batch}` | Number of span batches received while an exclude filter was configured. Counted on receipt alongside `span.batches.seen`, before conversion/decode and filtering. It does not imply filtering ran successfully or that any records matched the exclude rule. |
| `processor.filter.pdata.span.signals.consumed` | `{span}` | Number of span signals consumed. |
| `processor.filter.pdata.span.signals.kept` | `{span}` | Number of span signals kept (consumed minus filtered). |
| `processor.filter.pdata.span.signals.filtered` | `{span}` | Number of span signals filtered. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | No node-specific events are emitted. |

## Limits

- Include and exclude semantics depend on the signal-specific filter type in
  `otap-df-pdata`.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Core node catalog](../../../README.md)
