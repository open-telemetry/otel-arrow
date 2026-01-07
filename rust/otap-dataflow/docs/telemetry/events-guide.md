# System events guide

This guide defines how to add **system events** for the OTAP engine. It
complements the [semantic conventions guide](semantic-conventions-guide.md) and
the [entity model](entity-model.md).

## Related guides

- Attribute policy (including attributes vs event body guidance):
  [Attributes Guide](attributes-guide.md)
- Stability model and compatibility rules for event schemas:
  [Stability and Compatibility Guide](stability-compatibility-guide.md)
- Sensitive data and stacktrace gating:
  [Security and Privacy Guide](security-privacy-guide.md)

## What events are for

Events are discrete occurrences that benefit from context and correlation but do
not need to be aggregated as metrics. In OTLP, the event name MUST be carried in
the LogRecord `event_name` field. Do not introduce new telemetry that sets
`event.name` as an attribute.

Use events to record:

- Controller/Pipeline actions (config reload, shutdown, ack, timer ticks).
- State transitions (batch flush, backpressure, queue full).
- Exceptional outcomes (errors, retries, drops).

If the signal is high-volume or needs aggregation, prefer metrics. If the
event is part of a dataflow trace, record it as a span event.

Exception rule (traces):

- If you are recording an actual exception on a span, the span event name MUST
  be `exception` and the standard exception attributes MUST be used.

In this project, events are preferred to unstructured logs. Event names are
codified (see below), and their attributes consist of the attributes of the
relevant entity or entities (stable context), combined with event-specific
attributes (dynamic context).

Treat event names as schema identifiers. Evolution rules are defined in
[Stability and Compatibility Guide](stability-compatibility-guide.md).

## Event naming

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

- `otelcol.` is the project prefix/namespace used for events and other custom
  telemetry.
- `<entity>` is the primary entity involved (e.g. `pipeline`, `node`,
  `channel`). See the [entity model](entity-model.md) for the list of entities.
- `<thing>` is an optional sub-entity, subject, or stage (e.g. `build`, `run`,
  `receiver`, `exporter`).
- `<verb>` is the action or occurrence (e.g. `start`, `complete`, `fail`,
  `reload`, `shutdown`).

Note: OpenTelemetry Events are represented as LogRecords with an event name.
In OTLP, this is carried in the LogRecord `event_name` field (not in the body).

## Attributes and context

Always attach the relevant entity attributes (stable context):

- Pipeline attributes for pipeline-level events.
- Node attributes for node-level events.
- Channel attributes for channel-related events.

Optionally, add occurrence-specific attributes (dynamic context):

- Prefer enums or stable categorical values whenever possible.
- Use standard exception attributes for errors (`exception.type`,
  `exception.message`). Stacktrace gating rules are in
  [Security and Privacy Guide](security-privacy-guide.md).
- Follow [Security and Privacy Guide](security-privacy-guide.md) to avoid
  sensitive data.

## Severity and placement

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
- `nack`: A negative acknowledgment occurrence.
- `tick`: A timer tick occurrence.
- `sleep`: A sleep occurrence.
- `cancel`: An operation was intentionally stopped by an external decision before it finished. Triggered by a caller, operator, controller, or policy. Usually expected and often benign. Not an error in itself.
- `abort`: An operation was forced to stop due to an internal safety condition or unrecoverable state. Triggered inside the system. Indicates something went wrong or became unsafe. Usually unexpected.
- `timeout`: A timeout occurrence.

This list is not exhaustive. Choose verbs that best describe the action while
maintaining clarity and consistency. Avoid synonyms that fragment cardinality
across names. Don't introduce alternatives such as `finish` or `error`. Use
one success verb `complete`, one failure verb `fail`, one external
termination verb `cancel`, and one internal safety verb `abort`.

## Checklist for new events

- The event name follows the semantic conventions guide and the `otelcol.<entity>[.<thing>].<verb>` pattern.
- The event name is stable, low-cardinality, and contains no IDs or dynamic values.
- The event represents a discrete occurrence; use metrics instead for high-volume signals.
- Relevant entity attributes are included (pipeline/node/channel/etc).
- Dynamic attributes are bounded and avoid sensitive or high-cardinality data.
- Error events use standard exception attributes; stacktraces only at debug or lower.
- Severity is appropriate and consistent with the event meaning.
