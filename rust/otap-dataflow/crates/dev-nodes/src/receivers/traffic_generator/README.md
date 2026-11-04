# Traffic Generator -- SUT Saturation Benchmarks

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:traffic_generator` (`urn:otel:receiver:traffic_generator`)
- Feature gate: `dev-tools`
- Stability: Experimental

## Overview

The traffic generator receiver emits synthetic or semantic-convention-derived
telemetry for tests, demos, and benchmark scenarios. It is compiled only when
the `dev-tools` feature is enabled.

## Getting Started

Generate synthetic logs with pregenerated payloads:

```yaml
type: receiver:traffic_generator
config:
  data_source: synthetic
  generation_strategy: pre_generated
  traffic_config:
    production_mode: open
    max_batch_size: 1000
    metric_weight: 0
    trace_weight: 0
    log_weight: 1
    log_body_size_bytes: 1024
```

## Configuration

```yaml
type: receiver:traffic_generator
config:
  # Source of generated telemetry shape (default: semantic_conventions).
  data_source: synthetic

  # Semantic convention registry path when data_source is semantic_conventions
  # (default: the vendored, annotated registry in vendor/semconv/model).
  # See "Annotations" below.
  registry_path: ./semantic-conventions

  # Batch generation strategy: "fresh" or "pre_generated" (default: fresh).
  generation_strategy: pre_generated

  # Enables generated pdata ACK/NACK interests and waits for outstanding
  # completions during graceful drain (default: false).
  enable_ack_nack: false

  # Resource attributes rotated across batches (default: []).
  resource_attributes:
    - service.name: load-generator

  # Fixed or generated transport headers (default: {}).
  transport_headers:
    x-tenant-id: demo

  # Signal rate, batch size, and signal weights (required).
  traffic_config:
    production_mode: open
    signals_per_second: null
    max_signal_count: null
    max_batch_size: 1000
    metric_weight: 0
    trace_weight: 0
    log_weight: 1
    log_body_size_bytes: 1024
```

Transport-header names preserve their configured spelling. Names that differ
only by case remain distinct stored context entries.

## Annotations

With `data_source: semantic_conventions`, the receiver emits one signal per
registry group (spans from `span` groups, metrics from `metric` groups, logs
from `event` groups) and fills attribute values, metric values, histogram
buckets, span durations and status, and log severities from
`traffic-generator` annotations in the registry. Without annotations, values
fall back to enum members, `examples`, and type defaults.

### Registry

When `registry_path` is not set, the receiver loads the vendored registry in
[`vendor/semconv/model`](../../../../../vendor/semconv/README.md), which is
annotated for every metric, span, and event and every int, double, boolean,
and int[] attribute. The default path is resolved from the source tree at
build time, so set `registry_path` when running the binary elsewhere. Any
registry set through `registry_path` (a local folder, an archive, or a git
repository) is used as is; registries without annotations fall back to enum
members, examples, and type defaults.

### Placement

Annotations live under the `traffic-generator` key of a Weaver `annotations`
map, next to any other namespaces (such as `code_generation`). The key has a
hyphen, so use bracket access in templates and policies
(`.annotations["traffic-generator"]`).

```yaml
groups:
  - id: registry.http
    type: attribute_group
    attributes:
      # Attribute definition: default for every signal using the attribute.
      - id: http.response.status_code
        type: int
        annotations:
          traffic-generator:
            values: [200, 404, 500]
            weights: [90, 7, 3]

  - id: span.http.server
    type: span
    # Signal group: signal-level keys (see below).
    annotations:
      traffic-generator:
        duration:
          range: { min: 1, max: 2000 }
          distribution: { type: lognormal, mean: 80, stddev: 150 }
        status: { values: [unset, ok, error], weights: [85, 12, 3] }
    attributes:
      # Reference: overrides the definition for this signal only.
      - ref: http.response.status_code
        annotations:
          traffic-generator:
            values: [500, 503]
```

An annotation on a `ref:` replaces the definition's whole `annotations` map
for that signal (Weaver does not merge them), so repeat any other namespaces
the reference still needs.

### Value spec

Every generated value is described by a value spec in one of two forms:

| Form | Keys | Meaning |
| --- | --- | --- |
| Discrete | `values` (required), `weights` (optional) | Pick one of `values`; uniform unless `weights` is given. Weights are relative, `>= 0`, and must sum to `> 0`. |
| Continuous | `range: { min, max }` (required), `distribution` (optional) | Sample a number in `[min, max]` (inclusive); uniform unless `distribution` is given. Int and double only. |

`values` and `range` are mutually exclusive, `weights` requires `values`, and
`distribution` requires `range`.

| `distribution.type` | Parameters | Notes |
| --- | --- | --- |
| `uniform` | none | Default |
| `normal` | `mean`, `stddev` | `stddev > 0` |
| `lognormal` | `mean`, `stddev` | Mean and standard deviation of the value itself (not its logarithm); `mean > 0`, `range.max > 0` |
| `exponential` | `mean` | `mean > 0`, `range.max >= 0` |
| `zipf` | `exponent` | Int only; rank 1 is `range.min`, so smaller values are the most frequent |

Parameters describe the distribution before truncation to `range`. Values are
drawn by inverse transform sampling over the truncated distribution, so every
sample lies in the range without clamping. For int values, normal, lognormal,
and exponential are sampled over `[min - 0.5, max + 0.5]` and rounded, so
both bounds are reachable. A range with no probability mass under its
distribution is rejected.

### Attribute types

| Attribute type | Annotation |
| --- | --- |
| `int`, `double` | Discrete or continuous value spec. YAML integers are accepted for doubles. |
| `string`, `boolean` | Discrete value spec. |
| enum | Discrete value spec; every value must be a member value. Members that are not listed are never generated. |
| `any` | Discrete value spec with mixed scalars. |
| `T[]` | `length` (int value spec, `>= 0`) and `items` (value spec for `T`); each item is sampled independently. |
| `template[T]` | `keys` (string value spec) and `value` (value spec for `T`, or `length` / `items` for array templates). The emitted name is `<prefix>.<key>`, for example `http.request.header.content-type`. |

```yaml
- id: process.command_args
  type: string[]
  annotations:
    traffic-generator:
      length: { range: { min: 1, max: 4 } }
      items: { values: [--config=a.yaml, -v, run], weights: [4, 2, 3] }

- id: http.request.header
  type: template[string[]]
  annotations:
    traffic-generator:
      keys: { values: [content-type, accept], weights: [3, 1] }
      value:
        length: { values: [1] }
        items: { values: [application/json, text/plain] }
```

### Signal keys

| Group | Key | Spec | Default when absent |
| --- | --- | --- | --- |
| metric | `value` | Int or double value spec; counters require values `>= 0` | `1` |
| metric | `data_points` | Int value spec, `>= 1`; attributes are resampled per point | `1` |
| histogram metric | `observations` | Int value spec, `>= 1`; observations aggregated into each point | `1` |
| histogram metric | `bounds` | Strictly increasing list of numbers | `[0, 5, 10, 25, 50, 75, 100, 250, 500, 750, 1000, 2500, 5000, 7500, 10000]` |
| span | `duration` | Value spec in milliseconds, `>= 0` | 10 microseconds |
| span | `status` | Discrete, `unset` / `ok` / `error` | Status not set |
| event | `severity` | Discrete, `SeverityNumber` short names (`TRACE`, `TRACE2`, ... `FATAL4`); sets number and text | `INFO`, no text |

```yaml
- id: metric.http.server.request.duration
  type: metric
  instrument: histogram
  unit: s
  annotations:
    code_generation:
      metric_value_type: double
    traffic-generator:
      value:
        range: { min: 0.0, max: 60.0 }
        distribution: { type: lognormal, mean: 0.1, stddev: 0.2 }
      bounds: [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10]
      observations: { range: { min: 10, max: 100 } }
      data_points: { values: [1, 2, 3], weights: [6, 3, 1] }

- id: event.http.server.request.exception
  type: event
  annotations:
    traffic-generator:
      severity: { values: [ERROR, WARN, FATAL], weights: [80, 15, 5] }
```

Metric output, with or without annotations:

- Number points are `as_int` when the group's
  `code_generation.metric_value_type` is `int`, otherwise `as_double`.
- Counters and updowncounters are emitted as sums with delta temporality, so
  every point is an independent increment and replayed `pre_generated`
  batches stay valid.
- Histograms use delta temporality; each point carries bucket counts, count,
  sum, min, and max. Bucket `i` counts observations `x` with
  `bounds[i-1] < x <= bounds[i]`.
- `max_signal_count` counts data points for metrics, so `data_points` above 1
  reaches the limit with fewer metrics.

### Value precedence

For each attribute the receiver uses the first source that applies:

1. The `traffic-generator` annotation on the signal's attribute reference, or
   else on the attribute definition.
2. Enum members, sampled uniformly.
3. `examples`, sampled uniformly.
4. A type default (`42`, `3.13`, `"value"`, `true`, ...).

### Validation

Annotations are parsed, validated, and compiled into samplers once, when the
receiver starts. Generation only draws from the compiled samplers. Any invalid
annotation fails receiver startup, and the error lists every invalid
annotation with its group, attribute, and key path:

```text
2 invalid traffic-generator annotation(s):
  span.http.client: traffic-generator.status.values[1]: unknown status "failed"
  registry.http (http.request.method): traffic-generator.values[2]: "PATCH" is not an enum member value
```

Validation rejects unknown keys at every level (for example `weight` instead
of `weights`), keys used in the wrong context (`observations` on a gauge,
`items` on a scalar), values that do not match the attribute type, invalid
distribution parameters, unknown status and severity names, and bounds that
are not strictly increasing.

With `generation_strategy: pre_generated`, values are sampled once at startup
and replayed, so variety exists only across the pre-generated batches.

## Purpose

Verify that a **single traffic-gen sender core can saturate a single
`df_engine` (SUT) core** over OTLP gRPC, so that performance tests
don't need multiple load-generator cores.

```text
  sender (1 core)           SUT (1 core)            backend (1 core)
  traffic-gen -> OTLP-export --> OTLP-recv -> OTLP-export --> OTLP-recv -> noop
       :8080                     :8081                      :8082
```

## Configs (`bench/`)

| File                           | Role        | Pipeline                           |
|--------------------------------|-------------|------------------------------------|
| `sender-synthetic-fresh.yaml`  | Load gen    | traffic-gen(fresh) -> OTLP export  |
| `sender-synthetic-pregen.yaml` | Load gen    | traffic-gen(pregen) -> OTLP export |
| `sut-otlp-forward.yaml`        | SUT         | OTLP recv -> OTLP export           |
| `backend-noop.yaml`            | Backend     | OTLP recv -> noop                  |

## Quick start

```bash
cd rust/otap-dataflow
cargo build --release
bash crates/dev-nodes/src/receivers/traffic_generator/bench/bench.sh
```

---

## Results

Apple M3 Pro (14 cores, 24 GB), 1 pipeline core per process, release
build, logs only. Averaged over 4 runs.

| Config               | Throughput       | Sender core | SUT core     |
|----------------------|------------------|-------------|--------------|
| synthetic/fresh 1KB  | ~480 K logs/sec  | ~82 %       | **~97 %**    |
| synthetic/pregen 1KB | ~500 K logs/sec  | ~66 %       | **~95 %**    |

---

## Conclusion

**Use `synthetic` + `pre_generated` + `log_body_size_bytes: 1024`.**

This configuration saturates the SUT to ~95 % while consuming only
~66 % of a single sender core. `pre_generated` uses ~16 % less sender
CPU than `fresh` (~66 % vs ~82 %) at similar throughput, leaving more
headroom. One load-gen core is sufficient!

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `receiver.traffic_generator`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `receiver.traffic_generator.smooth.runs.terminations` | `{run}` | `outcome` | Number of smooth-mode production runs terminated, dimensionalized by outcome (`on_time`, `late`, `cancelled`). |
| `receiver.traffic_generator.smooth.payload.sends.attempts` | `{attempt}` | `attempt`, `outcome` | Number of smooth-mode payload send attempts, dimensionalized by kind of attempt (`initial`, `retry`) and outcome (`success`, `refused`, `failure`). |
| `receiver.traffic_generator.smooth.runs_started` | `{run}` | | Number of smooth-mode production runs started. |
| `receiver.traffic_generator.smooth.schedule_deadline_misses` | `{miss}` | | Number of times smooth mode detects a missed scheduling boundary. |
| `receiver.traffic_generator.completion.pending` | `{batch}` | | Number of subscribed batches waiting for Ack/Nack completion. |
| `receiver.traffic_generator.completion.drain_deadline_forced` | `{drain}` | | Number of drains forced to finish with unresolved batches at the deadline. |
| `receiver.traffic_generator.smooth.late_remaining_batches` | `{batch}` | | Number of batches remaining when smooth mode detects that a run is late. |
| `receiver.traffic_generator.smooth.late_remaining_items` | `{item}` | | Number of signal items remaining when smooth mode detects that a run is late. |
| `receiver.traffic_generator.smooth.run_batches` | `{batch}` | | Smooth-mode configured batches per one-second run. |
| `receiver.traffic_generator.smooth.batch_interval` | `ns` | | Smooth-mode configured interval between batches. |
| `receiver.traffic_generator.smooth.batch_tick_lateness_duration` | `s` | | Lateness of smooth-mode batch ticks relative to their scheduled instant. |
| `receiver.traffic_generator.smooth.payload_generate_duration` | `s` | | Wall-clock time spent generating or cloning one smooth-mode payload. |
| `receiver.traffic_generator.smooth.payload_send_duration` | `s` | | Wall-clock time spent sending one smooth-mode payload into the downstream channel. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `traffic_generator.smooth_run_behind` | `warn` | Smooth-mode generation did not finish before the next run tick. |
| `traffic_generator.open_run_behind` | `debug` | Open-loop generation remained behind schedule. |
| `traffic_generator.drain_ingress` | `info` | Receiver ingress drain started. |
| `traffic_generator.completion_drain.deadline_reached` | `warn` | Ack/Nack drain deadline expired with unresolved batches. |
| `traffic_generator.shutdown` | `info` | Receiver shutdown completed. |
| `traffic_generator.smooth_fallback_open` | `warn` | Smooth mode fell back to open-loop behavior. |

## Limits

- This receiver is available only with the `dev-tools` feature.
- `semantic_conventions` data source requires access to the configured registry
  path at startup. The default (the vendored registry) is resolved from the
  source tree at build time; set `registry_path` when running elsewhere.
- Invalid `traffic-generator` annotations in the registry fail receiver
  startup.
- `pre_generated` mode repeats timestamps and IDs from pregenerated batches.
- `production_mode: open` can produce as fast as the runtime allows.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Core node catalog](../../../README.md)
