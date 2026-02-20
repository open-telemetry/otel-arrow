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
event is part of a dataflow trace, use a regular event with a trace ID, not a
span event record, as span events are
becoming [deprecated](https://github.com/open-telemetry/opentelemetry-specification/blob/main/oteps/4430-span-event-api-deprecation-plan.md).

Exception rule (traces):

- If you are recording an actual exception on a span, the regular event name
  MUST be `exception` and the standard exception attributes MUST be used.

## How to emit events in code

All events MUST be emitted using the `otel_*` macros from the
`otap_df_telemetry` crate. **Do not** use `tracing::info!`, `log::info!`, or
`println!` directly. This rule is enforced by
`scripts/check-direct-telemetry-macros.sh` (run in CI).

**Why wrappers instead of raw `tracing` macros?**

- **Mandatory event name.** The first argument to every `otel_*!` macro is the
  event name. Raw `tracing` macros do not require one, and their default name
  includes the file path and line number -- which is not durable and breaks
  filtering, alerting, and dashboards whenever code is moved or reformatted.
- **Automatic `target`.** The wrappers set the tracing `target` field to the
  crate name (`env!("CARGO_PKG_NAME")`) automatically. With raw `tracing`
  macros the default target is the module path, which is an internal
  implementation detail and can change without notice.

### Available macros

| Macro | Severity |
|---|---|
| `otel_debug!` | DEBUG |
| `otel_info!`  | INFO  |
| `otel_warn!`  | WARN  |
| `otel_error!` | ERROR |

### Basic usage

The first argument is always the **event name** (a string literal). Optional
key-value pairs follow as structured attributes.

```rust
use otap_df_telemetry::otel_info;

// Event name only (no attributes):
otel_info!("pipeline.run.start");

// Event name with attributes:
otel_info!("receiver.grpc.start",
    endpoint = %addr,
);
```

### The `message` attribute

Use an attribute named **`message`** when the event name alone is not sufficient
to convey what happened. This value is mapped to the OTel LogRecord **body**,
making it the primary text shown in log viewers, consoles, and observability UIs.

Not every event needs a `message` -- if the event name is self-explanatory,
omit it. Avoid messages that just restate the event name; they add no value.

```rust
// Bad -- message just restates the event name:
otel_info!("pipeline.run.start",
    message = "Pipeline run started",
);

// Good -- event name says it all, no message needed:
otel_info!("pipeline.run.start");

// Good -- message explains consequences beyond what the event name conveys:
otel_warn!("core_affinity.set_failed",
    message = "Failed to set core affinity for pipeline thread. Performance may be less predictable.",
);
```

### Attribute formatting

The macros support `tracing`-style formatting hints:

- `%value` -- Display formatting (`fmt::Display`)
- `?value` -- Debug formatting (`fmt::Debug`)
- `value` -- passed directly (integers, booleans, etc.)

Prefer `%` (Display) for info/warn/error severity events. Reserve `?` (Debug)
for debug-level events, as it can expose internal struct layouts.

```rust
otel_info!("node.connect",
    endpoint = %addr,
    count    = 42,
);

// Debug formatting is acceptable at debug level:
otel_debug!("node.connect",
    config = ?node_config,
);
```

## Consolidating events

Every `otel_*!` callsite adds to the binary's static metadata. Avoid
proliferating near-identical events that differ only by one attribute -- use a
single event with a distinguishing **attribute** instead.

### Use attributes for variation, not separate event names

When several code paths represent the same *kind* of occurrence and differ only
in a categorical dimension (status code, credential type, error class, etc.),
emit **one event** with that dimension as an attribute rather than creating a
separate event for each value.

```rust
// BAD -- four callsites for the same conceptual event:
otel_warn!("receiver.grpc.unauthenticated", status_code = 16, message = %msg);
otel_warn!("receiver.grpc.permission_denied", status_code = 7, message = %msg);
otel_warn!("receiver.grpc.unavailable", status_code = 14, message = %msg);
otel_warn!("receiver.grpc.resource_exhausted", status_code = 8, message = %msg);

// GOOD -- one callsite, status_code as an attribute:
otel_warn!("receiver.grpc.error",
    status_code = code,
    message = %msg,
);
```

### Consolidate one-time startup information

Informational events emitted once during initialization (e.g. credential type,
listening address, feature flags) SHOULD be folded into a single startup event
rather than emitted as dedicated events per field.

```rust
// BAD -- separate events for each piece of startup info:
otel_info!("exporter.start");
otel_info!("exporter.endpoint", endpoint = %endpoint);
otel_info!("exporter.auth_type", auth_type = %auth_type);

// GOOD -- single startup event with all relevant attributes:
otel_info!("exporter.start",
    endpoint = %endpoint,
    auth_type = %auth_type,
);
```

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
- Use **distinct event names** for different outcomes of the same operation
  (e.g. `otlp.exporter.start.complete` and `otlp.exporter.start.fail`). Do not rely
  solely on severity to distinguish success from failure.

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

Note: OpenTelemetry Events are represented as LogRecords with an
[event name](https://github.com/open-telemetry/opentelemetry-specification/blob/v1.50.0/specification/logs/data-model.md#field-eventname).
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

When events are exported as logs, set an appropriate severity.

Regarding severity, choose the log level that best reflects the significance of
the event. For example, `node.shutdown.complete` at INFO for a graceful
shutdown and `node.shutdown.fail` at ERROR for a critical failure -- these are
distinct events, not the same event at different severity levels.

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
- `cancel`: An operation was intentionally stopped by an external decision
  before it finished. Triggered by a caller, operator, controller, or policy.
  Usually expected and often benign. Not an error in itself.
- `abort`: An operation was forced to stop due to an internal safety condition
  or unrecoverable state. Triggered inside the system. Indicates something went
  wrong or became unsafe. Usually unexpected.
- `timeout`: A timeout occurrence.

This list is not exhaustive. Choose verbs that best describe the action while
maintaining clarity and consistency. Avoid synonyms that fragment cardinality
across names. Don't introduce alternatives such as `finish` or `error`. Use
one success verb `complete`, one failure verb `fail`, one external
termination verb `cancel`, and one internal safety verb `abort`.

## Checklist for new events

- The event name follows the semantic conventions guide and the
  `otelcol.<entity>[.<thing>].<verb>` pattern.
- The event name is stable, low-cardinality, and contains no IDs or dynamic
  values.
- The event represents a discrete occurrence; use metrics instead for
  high-volume signals.
- Relevant entity attributes are included (pipeline/node/channel/etc).
- Dynamic attributes are bounded and avoid sensitive or high-cardinality data.
- Error events use standard exception attributes; stacktraces only at debug or
  lower.
- Severity is appropriate and consistent with the event meaning.
- No `format!` calls in attribute values; use `%`/`?` formatting or raw values.
- Near-identical events have been consolidated into a single event with a
  distinguishing attribute (see [Consolidating events](#consolidating-events)).
- The number of new callsites is minimized; each callsite adds static memory
  overhead.
