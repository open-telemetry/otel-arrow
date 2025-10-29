# OpenTelemetry Data Model for OTAP Dataflow engine

The OpenTelemetry project has a number of related specifications that
we wish to adhere to as we build the OTAP Dataflow engine.
OpenTelemetry has acted as a computational engine for telemetry in
limited ways. The SDK for example can perform tasks such as:

- Reduce metric dimensionality
- Sample traces and exemplars
- Aggregate synchronous metric events
- Change metric temporality.

OpenTelemetry specifications explain a lot about these processes, with
a focus on how SDKs perform the necessary tasks. However, we know
these tasks are relatively simple when performed inside an SDK because
SDKs have contact with the original API events. This relative simplicity
is compared against performing the same tasks in a downstream query engine:

- **SDKs**: An SDK remembers its start time and assumes a functional
  system clock. The SDK assumes its resource value is unique. SDKs
  coordinate collection (typically through hardware synchronization)
  to ensure outputs do not overlap in time. The SDK chooses a single
  metric "temporality" and manages the necessary state internally.
- **Query engines**: Query engines receive pre-aggregated data. A
  query engine receives data aggregated by multiple system
  clocks. They receive data with varying intervals, with uncoordinated
  reporting intervals. Query engines sometimes face duplicate data,
  overlapping data, and so on. Query engines receive a mix of
  cumulative and delta metric temporality.

There are two primary mechanisms used in OpenTelemetry that connect
its various specifications, for example to say how we start with
instrumentation events like `Counter.Add(10)` and arrive at a correct
and meaningful rate calculation. These are discussed below.

## Meaning is derived from events

The first mechanism appeals to the event-oriented structure of the
OpenTelemetry instrumentation APIs. OpenTelemetry specifies API
methods, for example that the `Counter` instrument has an
`Add(NonNegative)` method with an optional set of attributes. When a
Counter instrument is used, it captures one event at the current
instant that logically increments a monotonic sum. Meaning is tied to
API events. When we produce OpenTelemetry data, the data is
representative of the original API events.

To aggregate OpenTelemetry data, as in a query engine, where we accept
OpenTelemetry data as input, combine it, then output it again, the
most important question is whether we preserve representivity (i.e.,
whether the data accurately represents the original events). We use this
definition because it applies across signal types, and it gives a
substantial amount of freedom to aggregate the data without distorting
an observability signal. This means respecting:

- Event timestamps: Preferably, do not presume the event time is
  close to the arrival time for aggregation purposes, as that will
  distort representivity.
- Start timestamps: In OpenTelemetry metrics, the start timestamp
  associated with Counter, UpDownCounter, and Histogram data points
  conveys an unbroken sequence of measurements. Query engines that
  interpolate metric values for temporal alignment should use values
  from the same sequence, otherwise they distort representivity.
- Missing start timestamps: OpenTelemetry prescribes how to fill in
  timestamp information to indicate a new unbroken sequence, starting
  with a point where the start timestamp equals the event timestamp,
  stating the initial measurement value.
- Sample thresholds: Sampling and exemplar selection algorithms should
  aim to preserve representivity. This is usually achieved by
  respecting the sampling thresholds in the input data and yielding
  correct representative sampling thresholds in the output data.

All of these features in the OpenTelemetry data model are meant to
ensure meaningful aggregation can be applied without distorting
representivity.

## Single-writer rule

The [single-writer rule][SINGLEWRITER] is an OpenTelemetry metric data
model principle that ensures aggregated OpenTelemetry data is complete
and unique. This rule ensures that two entities do not produce
telemetry data with the same identity, and it ensures that no further
aggregation is required to take the intended meaning from the data.

This rule is used to motivate certain aspects of Metric SDK design. As
long as the prerequisites are met (e.g., the SDK resource is unique),
and the SDK is implemented correctly, the single-writer rule will
never be broken, because the SDK ensures there are distinct scopes,
distinct metric instruments, and distinct attribute sets in every
export.

This rule has implications for aggregate producers of OpenTelemetry
data as well as importers of non-OpenTelemetry data. When an aggregate
function removes a metric attribute, scope name, scope attribute, or
resource attribute, there is a chance that the output will conflict
with another writer, making the aggregate non-unique, breaking the
single-writer principle. Often, the way to restore the single-writer
property is by adding a new unique attribute to identify the writer so
that consumers will see unique aggregates, allowing them to correctly
compute the final aggregation from multiple distinct inputs.

The OpenTelemetry metric SDK specification explains how to translate
from original metric API events into aggregate metric data that
respects the single-writer principle. Essential to the approach, every
metric instrument has a natural aggregation function determined by its
instrument kind. For example, to combine multiple aggregate Counter
measurements into a single aggregate (e.g., in order to remove a
metric attribute), we compute the aggregate Sum, because Sum preserves
representivity for all forms of Counter. For the Gauge instrument, we
use the latest measurement value to preserve representivity.

For a query engine to meet the single-writer rule and preserve
representivity, it should begin by defining for each distinct
aggregate it produces, a new start time for the series it will
produce. It must ensure that its outputs are fully aggregated,
distinct from the outputs of all other producers, and not overlapping
in time.

[SINGLEWRITER]: https://opentelemetry.io/docs/specs/otel/metrics/data-model/#single-writer

## Limitations

The OpenTelemetry specification goes only as far as required to define
basic SDK behavior. Defining and implementing a query engine could
require introducing new specifications and semantic conventions that
OpenTelemetry has not required for its work on SDKs.

These are areas where a working query engine prototype may be needed
first in order to develop OpenTelemetry specifications. The following
are limitations a query-engine designer might face.

### Two-dimensional time axis

In a stream of telemetry data, we have two notions of time. There is a
timestamp in the data, and there is the time it arrives at the query
engine. As we stream and batch OpenTelemetry data, we have no
sanctioned way to emit partial temporally-aggregated data.

Therefore, we can imagine solutions. In a hypothetical scenario, we
expect 90% of telemetry data is received within 1 minute of the event
timestamp, 99% within 2 minutes, 100% within 10 minutes. Could
OpenTelemetry producers and consumers recognize intentional repetition
as an extension of the single-writer rule, allowing producers to
replace or update earlier outputs as time passes? This would require
new specifications to handle progressive refinement of late-arriving
data.

### How to set start-time

An essential question, also faced by SDKs, is how to set the start
time for a series. Consider this scenario: an SDK starts, it begins
reporting metrics, an hour passes, and then the first use of a Counter
occurs with a new attribute set. The choices are:

1. New series get the SDK's start time. This is a safe and simple
   choice, and it is true that the Counter was zero when the SDK
   started. However, this gives the interpretation that the rate has
   been approximately 0.00027 for an hour. Worse, this makes it
   impossible for the aggregator to forget groups it has seen and
   recycle memory; a "restarted" series requires a newer start time
   than the last time it was recycled.
2. New series start time is the instant that it was first recognized
   in the aggregation logic. However, the rate contribution from the
   first point in a timeseries is always arbitrary, unless it has
   zero width.
3. New series start time is equal to event time. This explicitly
   states that the first value in a series is arbitrary and
   contributes zero in a rate calculation. This is the ideal way
   to avoid an arbitrary rate contribution from the choice of start time.

It is somewhat disappointing, however, to lose the value of the first
point in every Counter timeseries because it contributes arbitrarily
to the rate. OpenTelemetry could conceivably add a data point flag to
indicate that a series has a true-zero value at its start time.

### Fractional histogram

Frequently, metric engines will apply temporal alignment to aggregate
metric data onto pre-defined intervals. For Counter values, this can
be done this using interpolation, accumulating a fraction of the
Counter's value into new points on either side. This makes sense
because it preserves representivity, but this will not work for
histogram values because there is not a histogram data type with
fractional bucket counts.

As we know, it is common to compute metrics from span and log
events. In a typical use-case, for example, we compute a histogram
from span latencies. When spans arrive pre-sampled, however, we cannot
compute a representative metric. This breaks representivity because
sampled spans have fractional adjusted counts, and there is not a
histogram data type with fractional bucket counts.

### Gauge histogram

OpenTelemetry does not define the Gauge histogram concept that we
obtain naturally from aggregating Gauge values. This is an omission in
the OpenTelemetry specification, in part because this type of
aggregation is not typically performed in an SDK.

In an OpenTelemetry-based query engine, we will be capable of defining
Gauge histogram data, but there is not a histogram data type that
represents spatial or temporal aggregation of Gauge values.
