# Filter Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:filter` (`urn:otel:processor:filter`)
- Feature gate: Default
- Stability: Experimental

## Overview

The filter processor drops logs or traces according to configured include and
exclude rules. Metrics currently pass through unchanged.

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
| `processor.filter.pdata.log_signals_consumed` | `{log}` | Number of log signals consumed. |
| `processor.filter.pdata.span_signals_consumed` | `{span}` | Number of span signals consumed. |
| `processor.filter.pdata.log_signals_filtered` | `{log}` | Number of log signals filtered. |
| `processor.filter.pdata.span_signals_filtered` | `{span}` | Number of span signals filtered. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | No node-specific events are emitted. |

## Limits

- Metric filtering is not currently supported.
- Include and exclude semantics depend on the signal-specific filter type in
  `otap-df-pdata`.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Core node catalog](../../../README.md)
