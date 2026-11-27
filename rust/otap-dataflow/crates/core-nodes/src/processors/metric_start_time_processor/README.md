# Metric Start Time Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:metric_start_time` (`urn:otel:processor:metric_start_time`)
- Feature gate: Always enabled
- Stability: Experimental

## Overview

The metric start time processor sets missing `start_time_unix_nano` values on
cumulative metric data points directly on columnar OTAP Arrow record batches,
modeled on the Go Collector
[metricstarttimeprocessor](https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/processor/metricstarttimeprocessor).

With the `subtract_initial_point` strategy, the processor handles cumulative
monotonic `Sum`, cumulative `Histogram`, cumulative `ExponentialHistogram`, and
`Summary` data points whose `start_time_unix_nano` is unset (`0`) or equal to
`time_unix_nano`:

- The first observed point per series is cached as the baseline and dropped.
- Subsequent points receive `start_time_unix_nano` from the initial observation
  and have the cached baseline values subtracted. Optional histogram `min`/`max`
  extrema are cleared when non-zero baseline observations are subtracted, and
  optional histogram `sum` is omitted until reset if the initial baseline sum
  was absent (with non-zero count) or `NaN`.
- When a counter, count, sum, bucket count, scale, or explicit bounds reset is
  detected, `start_time_unix_nano` is set to `max(time_unix_nano - 1ms, 0)` and
  the reference baseline resets to zero.
- Non-metric signals (`Logs` and `Traces`), gauges, delta metrics, non-monotonic
  cumulative sums, and points with an existing distinct `start_time_unix_nano`
  pass through untouched.

## Configuration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `strategy` | string | `subtract_initial_point` | Start time adjustment strategy. Currently only `subtract_initial_point` is supported. |
| `gc_interval` | duration | `10m` | Interval between mark-and-sweep passes that evict inactive series and resources. Must be greater than zero. |

## Examples

Minimal configuration using defaults (`subtract_initial_point` and `10m` GC
interval):

```yaml
type: processor:metric_start_time
config: {}
```

Explicit strategy and shorter garbage collection interval:

```yaml
type: processor:metric_start_time
config:
  strategy: subtract_initial_point
  gc_interval: 5m
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `processor.metric_start_time`

All metrics in this set carry registration attribute `signal = "metrics"`.

| Metric | Unit | Description |
| --- | --- | --- |
| `processor.metric_start_time.dropped.items` | `{item}` | Number of initial cumulative data points dropped while seeding per-series reference state. |
| `processor.metric_start_time.adjusted.data_points` | `{metric}` | Number of cumulative data points whose start time and value were adjusted. |
| `processor.metric_start_time.resets.detected` | `{reset}` | Number of cumulative counter, summary, or histogram resets detected. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | No node-specific events are emitted. |

## Limits

- Only the `subtract_initial_point` strategy is implemented today. Unlike the Go
  Collector's `metricstarttimeprocessor` (which defaults to `true_reset_point`),
  omitting `strategy` currently defaults to `subtract_initial_point`.
- Series identity keys on resource attributes, metric type, metric name, and
  data point attributes, matching the Go Collector; `InstrumentationScope` and
  metric `unit` are not included in the cache key.
- Garbage collection runs on incoming metric batches via two-cycle
  mark-and-sweep (clearing cached state before processing after idle gaps longer
  than `2 * gc_interval`); there is currently no hard per-series cardinality cap
  or timer-driven idle sweep.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Core node catalog](../../../README.md)
