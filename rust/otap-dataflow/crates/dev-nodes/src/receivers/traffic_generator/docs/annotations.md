# Traffic Generator -- Annotation Reference

<!-- markdownlint-disable MD013 -->

With `data_source: semantic_conventions` and `annotations: true`, the
`traffic_generator` receiver fills attribute values, metric values, histogram
buckets, span durations and status, and log severities from
`traffic-generator` annotations in the semantic conventions registry. See the
[receiver README](../README.md#annotations) for how to enable annotations and
choose a registry.

## Placement

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

## Value spec

Every generated value is described by a value spec in one of two forms:

| Form | Keys | Meaning |
| --- | --- | --- |
| Discrete | `values` (required), `weights` (optional) | Pick one of `values` (at most 128); uniform unless `weights` is given. Weights are relative, `>= 0`, and must have a finite sum `> 0`. |
| Continuous | `range: { min, max }` (required), `distribution` (optional) | Sample a number in `[min, max]` (inclusive); uniform unless `distribution` is given. Int and double only. |

`values` and `range` are mutually exclusive, `weights` requires `values`, and
`distribution` requires `range`.

On scalar and enum attributes, `weights` may be given without `values`: the
attribute's registry `examples` are then used as `values`, in order, so they do
not have to be repeated in the annotation. The examples are validated like
`values` (type, enum membership, at most 128), and `weights` must match their
count. An attribute without `examples` is rejected. Array `items`, template
`keys` and `value`, and signal keys still require `values`.

```yaml
- id: jvm.gc.action
  type: string
  examples: ["end of minor GC", "end of major GC"]
  annotations:
    traffic-generator:
      weights: [9, 1]
```

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

## Attribute types

| Attribute type | Annotation |
| --- | --- |
| `int`, `double` | Discrete or continuous value spec. YAML integers are accepted for doubles. |
| `string`, `boolean` | Discrete value spec. |
| enum | Discrete value spec; every value must be a member value. Members that are not listed are never generated. |
| `any` | Discrete value spec with mixed scalars. |
| `T[]` | `length` (int value spec, `0` to `32`) and `items` (value spec for `T`); each item is sampled independently. |
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

## Signal keys

| Group | Key | Spec | Default when absent |
| --- | --- | --- | --- |
| metric | `value` | Int or double value spec; counters and histograms require values `>= 0` | `1` |
| counter, updowncounter, histogram metric | `temporality` | `delta` (the only supported value); also sets each point's start time | `Unspecified`, no start time |
| histogram metric | `observations` | Int value spec, `1` to `50`; observations aggregated into each point | `1` |
| histogram metric | `bounds` | Strictly increasing list of numbers (at most 32) | `[0, 5, 10, 25, 50, 75, 100, 250, 500, 750, 1000, 2500, 5000, 7500, 10000]` |
| span | `duration` | Value spec in milliseconds, `>= 0` | 10 microseconds |
| span | `status` | Discrete, `unset` / `ok` / `error` | Status not set |
| event | `severity` | Discrete, `SeverityNumber` short names (`TRACE`, `TRACE2`, ... `FATAL4`); sets number and text | `INFO`, no text |

```yaml
- id: metric.http.server.request.duration
  type: metric
  instrument: histogram
  unit: s
  annotations:
    traffic-generator:
      value:
        range: { min: 0.0, max: 60.0 }
        distribution: { type: lognormal, mean: 0.1, stddev: 0.2 }
      bounds: [0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1, 2.5, 5, 10]
      observations: { range: { min: 10, max: 50 } }
      temporality: delta

- id: event.http.server.request.exception
  type: event
  annotations:
    traffic-generator:
      severity: { values: [ERROR, WARN, FATAL], weights: [80, 15, 5] }
```

Metric output with `annotations: true`, with or without annotations on the
group:

- Number points are `as_double`.
- Counters and updowncounters are emitted as sums. Their temporality, and
  that of histograms, is `Unspecified` unless the group sets
  `temporality: delta`; delta points also get a start time just before the
  point time. Only `delta` is supported: every point is sampled
  independently, so cumulative totals would not grow between points and
  would repeat on `pre_generated` replay.
- Each histogram point carries bucket counts, count, sum, min, and max.
  Bucket `i` counts observations `x` with `bounds[i-1] < x <= bounds[i]`.
  Without annotations, a point holds one observation of `1` in the default
  bounds.
- Each metric carries a single data point, so batch sizes, rates, and
  `max_signal_count` count metrics and data points alike.
- Metric groups without an `instrument` are skipped.

## Value precedence

For each attribute the receiver uses the first source that applies (step 1
only with `annotations: true`):

1. The `traffic-generator` annotation on the signal's attribute reference, or
   else on the attribute definition.
2. The first enum member.
3. `examples`, sampled uniformly.
4. A type default (`42`, `3.13`, `"value"`, `true`, ...).

## Validation

With `annotations: true`, annotations are parsed, validated, and compiled
into samplers once, when the receiver starts. Generation only draws from the
compiled samplers. Any invalid annotation fails receiver startup, and the
error lists every invalid annotation with its group, attribute, and key path:

```text
2 invalid traffic-generator annotation(s):
  span.http.client: traffic-generator.status.values[1]: unknown status "failed"
  registry.http (http.request.method): traffic-generator.values[2]: "PATCH" is not an enum member value
```

Validation rejects unknown keys at every level (for example `weight` instead
of `weights`), keys used in the wrong context (`observations` or
`temporality` on a gauge, `items` on a scalar), values that do not match the
attribute type, invalid distribution parameters, unknown status, severity,
and temporality names, bounds that are not strictly increasing, lists above
their size caps (`values` 128, `bounds` 32), counts above their limits
(`observations` 50, array `length` 32), int ranges beyond +/-(2^52 - 1),
weights whose sum is not finite, and `weights` without `values` on an
attribute that has no `examples`. Errors on values taken from `examples` say
so in the message.

These caps bound both the memory a compiled sampler retains and the per-batch
sampling work. The `observations` limit bounds the sampling work per histogram
point; the work per batch still scales with `max_batch_size` (up to
`max_batch_size * 50` histogram samples, about 50k per batch at the default
`max_batch_size` of 1000). The receiver yields between batches, so this bounds
only the uninterrupted work inside one batch.

If `annotations: true` but the registry has no `traffic-generator` annotations
(for example the default upstream registry), the receiver logs a startup
warning and every value falls back to enum members, examples, and type
defaults.

With `generation_strategy: pre_generated`, values are sampled once at startup
and replayed, so variety exists only across the pre-generated batches.
