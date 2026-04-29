# Filter Proccessor

Status: **WIP**

This crate will contain the implementation of the filter processor.
For reference please the golang version of the filter processor.
[GoLang Filter Processor](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/main/processor/filterprocessor/README.md)

## Example Config

```yaml
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
[`configs/fake-metric-filter-debug-noop.yaml`](../../../../configs/fake-metric-filter-debug-noop.yaml).

### Metrics

To filter metrics, define `metrics.include` or `metrics.exclude` with a
`match_type` and `metric_names`. Supported `match_type` values are `strict` and
`regexp`. When both `include` and `exclude` are defined, include filtering runs
first, and exclude filtering is applied to that result.

### Logs

To filter logs you can choose to define logs to `include` or `exclude`.
You can also choose to define both, if both are defined then the result
will be the interesection of the two. Currently we allow you to filter
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
