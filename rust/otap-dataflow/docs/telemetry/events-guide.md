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
- **Automatic `target`.** Registered component modules bind the wrappers to a
  stable component target. Other code uses the Cargo package name. When
  exported via OTLP, this becomes the `InstrumentationScope.name`. With raw
  `tracing` macros the default target is the module path, which is an internal
  implementation detail and can change without notice.

### Available macros

| Macro | Severity |
| --- | --- |
| `otel_debug!` | DEBUG |
| `otel_info!` | INFO |
| `otel_warn!` | WARN |
| `otel_error!` | ERROR |

### Component-aware targets

#### Motivation

The tracing target serves two externally observable purposes:

- `EnvFilter` uses it to select which callsites are enabled.
- OTLP log export records it as `InstrumentationScope.name`.

A package-only target is stable, but it is too broad for packages containing
many components. For example, setting `otap-df-core-nodes=debug` enables debug
events from every receiver, processor, and exporter in that package. During an
incident, an operator should be able to temporarily increase verbosity for the
transform processor without also increasing log volume from unrelated nodes.

Events semantically owned by a registered component MUST therefore use a stable
component-aware target:

```text
<cargo-package>::<component-kind>::<component-name>
```

The component kind and name MUST be taken from the registered component URN:

```text
urn:<namespace>:<component-kind>:<component-name>
```

For example:

| Component URN | Component-aware target |
| --- | --- |
| `urn:otel:processor:transform` | `otap-df-core-nodes::processor::transform` |
| `urn:otel:processor:durable_buffer` | `otap-df-core-nodes::processor::durable_buffer` |
| `urn:otel:receiver:otlp` | `otap-df-core-nodes::receiver::otlp` |
| `urn:otel:receiver:host_metrics` | `otap-df-core-nodes::receiver::host_metrics` |
| `urn:otel:exporter:otlp_grpc` | `otap-df-core-nodes::exporter::otlp_grpc` |
| `urn:otel:receiver:kafka` | `otap-df-contrib-nodes::receiver::kafka` |
| `urn:otel:exporter:kafka` | `otap-df-contrib-nodes::exporter::kafka` |
| `urn:microsoft:exporter:geneva` | `otap-df-contrib-nodes::exporter::geneva` |

The URN namespace is omitted because the target is an operational hierarchy,
not the canonical component identifier. The complete identity remains
available in `otelcol.node.urn`. A package MUST NOT register two components
with the same `<component-kind>::<component-name>` pair, even when their URN
namespaces differ.

Use singular component kinds such as `receiver`, `processor`, `exporter`, and
`extension`, matching the URN. Use the exact component name from the URN rather
than a Rust module name. For example, use `processor::transform`, not
`processors::transform_processor`. Module names are implementation details and
may change during refactoring.

Code that is not owned by one registered component MUST retain its package-level
target. Shared code MUST NOT guess a component from the module in which it is
implemented. When an event semantically belongs to the calling component, the
component target should be propagated explicitly; otherwise, the shared
subsystem should use its package-level target.

#### Binding a component target

Declare the component telemetry scope once in the component's root module,
before its child module declarations:

```rust
pub const TRANSFORM_PROCESSOR_URN: &str = "urn:otel:processor:transform";

otap_df_telemetry::otel_component_scope!(
    urn = TRANSFORM_PROCESSOR_URN,
    kind = "processor",
    name = "transform",
);

mod config;
mod metrics;
mod routing;
```

The declaration creates lexically scoped `otel_debug!`, `otel_info!`,
`otel_warn!`, `otel_error!`, and `otel_event!` wrappers for that module subtree.
Callsites therefore remain concise and do not repeat the target:

```rust
otel_debug!("transform.expression.evaluate", expression = %expression);
```

The macro validates at compile time that `kind` and `name` match the component
URN, then constructs the static target from the consuming Cargo package. Child
modules MUST use the lexically scoped macros rather than importing or invoking
the package-level macros by a fully qualified path.

This mechanism is intentionally lexical. A tracing target is static callsite
metadata, so it cannot be selected from a runtime node instance. Deriving the
target from `module_path!()` is also prohibited because Rust module paths are
implementation details rather than registered component identities.

#### Identity boundaries

The target identifies the stable software component type, not a configured
runtime instance:

| Concern | Representation |
| --- | --- |
| Emitting package and component type | Target / `InstrumentationScope.name` |
| Canonical component identity | `otelcol.node.urn` |
| Configured node instance | `otelcol.node.id` |
| Pipeline instance | `otelcol.pipeline_group.id` and `otelcol.pipeline.id` |
| Event class and schema | LogRecord `event_name` |
| Occurrence-specific details | Event attributes and body |

Targets MUST be compile-time-stable and MUST NOT contain configured node IDs,
pipeline IDs, endpoints, tenant names, or other runtime values. Two configured
instances of the transform processor use the same component target and are
distinguished by their entity attributes.

#### Filtering

The hierarchy preserves package-wide filtering while adding narrower scopes:

```text
# All core nodes at debug.
warn,otap-df-core-nodes=debug

# All core processors at debug.
warn,otap-df-core-nodes::processor=debug

# Only the transform processor at debug.
warn,otap-df-core-nodes::processor::transform=debug
```

`EnvFilter` target directives use prefix matching. Retaining the Cargo package
as the first segment means existing package-level directives continue to select
the new component-aware targets. Because `engine.telemetry.logs.level` can be
reconciled at runtime, an OpAMP or admin control plane can apply the specific
component directive during an active incident and restore the normal filter
afterward without restarting the engine.

Filtering selects the component type. Filtering a single configured instance
remains a separate concern and MUST NOT be implemented by putting the instance
ID in the target.

#### Relationship to EventName

EventName values are unchanged by this target migration. EventName naming and
compatibility are outside the scope of this change.

#### Compatibility and migration

Changing a target changes the exported `InstrumentationScope.name`. Existing
package-prefix `EnvFilter` directives remain effective, but consumers using an
exact scope-name comparison, scope grouping, or an allowlist must be updated.

Package-prefix `EnvFilter` directives require no change. Exact-match queries and
allowlists must replace the old package scope with one or more component scopes.
For example, the old exact scope:

```text
otap-df-core-nodes
```

becomes, depending on the components of interest:

```text
otap-df-core-nodes::receiver::otlp
otap-df-core-nodes::processor::transform
otap-df-core-nodes::exporter::otlp_grpc
```

See the
[Stability and Compatibility Guide](stability-compatibility-guide.md#instrumentation-scopes)
for the compatibility rules governing scope-name changes.

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

Avoid Debug-formatting (`?`) large or deeply nested structs at info/warn/error
severity -- break them into individual meaningful fields instead. For **error
values**, prefer `%` (Display) when the type has a well-crafted `Display` impl
(especially first-party `thiserror` types); `?` (Debug) is acceptable when
`Display` is too terse or unavailable. For **simple types** (enums, paths,
durations), either sigil is fine at any level.

```rust
otel_info!("node.connect",
    endpoint = %addr,
    count    = 42,
);

// BAD -- Debug-dumping a large nested struct at info level:
otel_info!("state.observed_event", observed_event = ?observed_event);

// GOOD -- break the struct into individual fields:
otel_info!("state.observed_event",
    pipeline_group_id = %observed_event.key.pipeline_group_id,
    pipeline_id = %observed_event.key.pipeline_id,
    core_id = observed_event.key.core_id,
    event_type = ?req,
    message = observed_event.message.as_deref().unwrap_or(""),
);

// Debug on simple enums or types without Display is fine at any level:
otel_info!("durable_buffer.shutdown.start", deadline = ?deadline);

// Full Debug formatting for complex types is best at debug level:
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

Event names MUST be low-cardinality and stable. Follow the
[semantic conventions guide](semantic-conventions-guide.md#event-naming) for
naming:

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
