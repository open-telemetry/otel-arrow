# System Events Guide

This guide defines how to add system events for the OTAP engine. It complements
the [semantic conventions guide](semantic-conventions-guide.md) and the
[entity model](entity-model.md).

## What Events Are For

Events are discrete occurrences that benefit from context and correlation but do
not need to be aggregated as metrics. In OTLP, events are represented as logs
with a required name.

Use events to record:
- Controller/Pipelibe actions (config reload, shutdown, ack, timer ticks).
- State transitions (batch flush, backpressure, queue full).
- Exceptional outcomes (errors, retries, drops).

If the signal is high-volume or needs aggregation, prefer metrics. If the
event is part of a dataflow trace, record it as a span event.

In this project, events are preferred to unstructured logs. Event names are
codified (see below), and their attributes consist of the attributes of the
relevant entity or entities (stable context), combined with event-specific
attributes (dynamic context).

## Event Naming

Event names MUST be low-cardinality and stable. Follow the semantic conventions
guide for naming:
- Lowercase and dot-separated. It identifies a class of event, not an instance.
- Keep the name stable and "type-like". Treat it as a schema identifier.
- Use verbs for actions (e.g. `pipeline.config.reload`).
- Avoid embedding IDs or dynamic values in the name. Encode variability as
  attributes.
- Avoid synonyms that fragment cardinality across names (`finish` vs `complete`,
  `error` vs `fail`). Pick one verb set and stick to it. 

More precisely, in this project, event names SHOULD follow this pattern:
`otelcol.<entity>[.<thing>].<verb>`

Where:
- `otelcol.` is the prefix/namespace used for OpenTelemetry Collector-related
  events.
- `<entity>` is the primary entity involved (e.g. `pipeline`, `node`,
  `channel`). See the [entity model](entity-model.md) for the list of entities.
- `<thing>` is an optional sub-entity, subject, or stage (e.g. `build`, `run`,
  `receiver`, `exporter`).
- `<verb>` is the action or occurrence (e.g. `start`, `complete`, `fail`,
  `reload`, `shutdown`).

Note: The `event_name` field in OTLP logs corresponds to the event name defined
here.

## Attributes and Context

Always attach the relevant entity attributes (stable context):

- Pipeline attributes for pipeline-level events.
- Node attributes for node-level events.
- Channel attributes for channel-related events.

Optionally, add occurrence-specific attributes (dynamic context):

- Prefer enums or stable categorical values whenever possible.
- Use standard exception attributes for errors (`exception.type`,
  `exception.message`). Report `exception.stacktrace` only when the log level is
  `debug` or lower.
- Avoid including sensitive data (passwords, emails, non-sanitized URLs).

## Severity and Placement

When events are exported as logs, set an appropriate severity. When they are
attached to traces, use span events with the same name and attributes.

Regarding severity, some events may be logged at different levels depending on
their severity or impact. For example, a `node.shutdown` event may be logged at
INFO level during a graceful shutdown, but at ERROR level if the shutdown is due
to a critical failure. When exporting events as logs, choose the log level that
best reflects the significance of the event.

## Stages

The following stages are recommended for event names:

- `pipeline`:
  - `build`: Pipeline construction phase.
  - `run`: Pipeline execution phase.
  - `report`: Pipeline metrics reporting phase.
- `node`:
  - `build`: Node construction phase.
  - `run`: Node execution phase.
- `channel`:
  - `send`: Channel send phase.
  - `recv`: Channel receive phase.

This list is not exhaustive. Choose stages that best describe the context while
maintaining clarity and consistency.

## Verbs

The following verbs are recommended for event names:
- `create`: The creation of an entity or resource.
- `init`: The initialization of an entity or resource.
- `start`: The beginning of an operation or process.
- `complete`: The successful end of an operation or process.
- `fail`: An operation or process that ended with an error.
- `stop`: The beginning of a stop or shutdown process.
- `pause`: The pausing of an operation or process.
- `resume`: The resumption of an operation or process.
- `apply`: An application of configuration or state.
- `flush`: A batch or buffer flush.
- `drop`: A drop occurrence.
- `backpressure`: A backpressure occurrence.
- `retry`: A retry attempt.
- `ack`: An acknowledgment occurrence.
- `nack`: An negative acknowledgment occurrence.
- `tick`: A timer tick occurrence.
- `sleep`: A sleep occurrence.
- `cancel`: An operation was intentionally stopped by an external decision before it finished. Triggered by a caller, operator, controller, or policy. Usually expected and often benign. Not an error in itself.
- `abort`: An operation was forced to stop due to an internal safety condition or unrecoverable state. Triggered inside the system. Indicates something went wrong or became unsafe. Usually unexpected
- `timeout`: A timeout occurrence.

This list is not exhaustive. Choose verbs that best describe the action while
maintaining clarity and consistency. Avoid synonyms that fragment cardinality
across names. Don't introduce alternatives such as `finish` or `error`, we'd
like one success verb `complete`, one failure verb `fail`, one external
termination verb `cancel`, and one internal safety verb `abort`.

## Checklist for New Events

<create a checklist for new events similar to the one for metrics>
