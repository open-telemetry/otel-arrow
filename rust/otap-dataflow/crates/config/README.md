# `otap-df-config`

Configuration model crate for the OTAP Dataflow Engine.

If you are authoring runtime YAML, start with:

- [`docs/configuration-model.md`](../../docs/configuration-model.md)

Design rationale and prior-art discussion:

- Issue [#1970](https://github.com/open-telemetry/otel-arrow/issues/1970)

This README focuses on crate-level model and API details.

## What This Crate Defines

Main public model types:

- `engine::OtelDataflowSpec`: runtime root spec (`version`, `policies`, `engine`
  , `groups`)
- `engine::EngineConfig`: engine-wide section (`engine: ...`)
- `pipeline_group::PipelineGroupConfig`
- `pipeline::PipelineConfig`: nodes, connections, optional policies
- `policy::Policies`: channel-capacity/health/telemetry/resources
  policy families
- `topic::TopicSpec`: named inter-pipeline topic specification
- `node::NodeUserConfig`: per-node configuration envelope
- `node_urn::NodeUrn`: parsed/canonicalized node type URN
- `engine::ResolvedOtelDataflowSpec`: deterministic resolved runtime snapshot

The model is strict (`serde(deny_unknown_fields)` on key types) and validated on
load.

## Runtime Config Format

The runtime root config is:

- `engine::OtelDataflowSpec`

Required root field:

- `version: otel_dataflow/v1`

The engine binary loads this root spec via `--config`.

`pipeline::PipelineConfig` parsing APIs remain available for in-memory parsing
and tests, but are not a runtime root format for the engine process.

## Parsing and Validation Entry Points

Runtime/root APIs:

- `OtelDataflowSpec::from_file`
- `OtelDataflowSpec::from_yaml`
- `OtelDataflowSpec::from_json`

Loading performs:

1. Deserialization (YAML/JSON)
2. Node URN canonicalization
3. Structural and policy validation

## Resolution Phase

For runtime consumption, resolve hierarchy once:

- `OtelDataflowSpec::resolve()` -> `engine::ResolvedOtelDataflowSpec`

Resolved model highlights:

- deterministic pipeline ordering for regular pipelines
  (`group_id`, `pipeline_id`)
- role-tagged resolved pipelines:
  - `ResolvedPipelineRole::Regular`
  - `ResolvedPipelineRole::ObservabilityInternal`
- helper split API:
  - `ResolvedOtelDataflowSpec::into_parts()`

## Policy Hierarchy

Policy families:

- `policies.channel_capacity.control.node`
- `policies.channel_capacity.control.pipeline`
- `policies.channel_capacity.control.completion`
- `policies.channel_capacity.pdata`
- `policies.health`
- `policies.telemetry.pipeline_metrics`
- `policies.telemetry.tokio_metrics`
- `policies.telemetry.runtime_metrics`
- `policies.resources.core_allocation`

Defaults:

- `channel_capacity.control.node = 256`
- `channel_capacity.control.pipeline = 256`
- `channel_capacity.control.completion = 512`
- `channel_capacity.pdata = 128`
- `telemetry.pipeline_metrics = true`
- `telemetry.tokio_metrics = true`
- `telemetry.runtime_metrics = basic`
- `resources.core_allocation = all_cores`

Control channel keys:

- `node`: capacity of per-node control inboxes
- `pipeline`: capacity of the shared pipeline-runtime orchestration channel
- `completion`: capacity of the shared Ack/Nack completion channel

Telemetry policy note:

- `policies.telemetry.runtime_metrics` accepts `none`, `basic`, `normal`, or
  `detailed`
- it gates channel endpoint transport metrics, per-node produced/consumed
  outcome metrics, and the shared control-plane metric families exported on the
  pipeline entity (`pipeline.runtime_control` and `pipeline.completion`)

Resolution precedence:

- regular pipelines:
  `pipeline.policies` -> `group.policies` -> top-level `policies`
- observability pipeline:
  `engine.observability.pipeline.policies` -> top-level `policies`

Observability note:

- `engine.observability.pipeline.policies.resources` is intentionally
  unsupported and rejected.

Resolution semantics:

- precedence applies per policy family (`channel_capacity`, `health`,
  `telemetry`, `resources`)
- no cross-scope deep merge of nested fields
- policy objects are default-filled: if a lower-scope `policies` block exists,
  omitted families are populated with defaults at that scope (they do not
  inherit from upper scopes)

## Topic Declarations

Topics can be declared in two scopes:

- top-level: `topics.<name>`
- group-level: `groups.<group>.topics.<name>` (visible only in that group)

General topic capabilities:

- decouple pipelines through named in-memory communication points
- support balanced worker-pool delivery via subscription groups
- support broadcast fan-out / tap pipelines
- support mixed balanced + broadcast consumers on one topic

Topic wiring must remain acyclic across topic hops. During startup, the
controller rejects feedback loops that involve declared topics, including:

- same-pipeline loops such as `receiver:topic -> ... -> exporter:topic`
- cross-pipeline loops where one pipeline eventually routes back into an
  earlier topic

Current topic declaration shape:

```yaml
topics:
  raw_signals:
    description: "raw ingest stream"
    backend: in_memory
    impl_selection: auto
    policies:
      balanced:
        queue_capacity: 1000
        on_full: drop_newest
      broadcast:
        queue_capacity: 4096
        on_lag: drop_oldest
      ack_propagation:
        mode: auto
        max_in_flight: 1024
        timeout: 30s
```

- `backend`:
  - `in_memory` (default, currently implemented)
  - `quiver` (accepted by config, not implemented by the runtime yet)
- `impl_selection`:
  - `auto`
  - `force_mixed`

Unsupported backend or policy combinations are rejected during startup topic
declaration with explicit errors.

- `policies.balanced.queue_capacity` (default: `128`, must be > 0)
- `policies.balanced.on_full`:
  - `block` (default)
  - `drop_newest`
- `policies.broadcast.queue_capacity` (default: `128`, must be > 0)
- `policies.broadcast.on_lag`:
  - `drop_oldest` (default)
  - `disconnect`
- `policies.ack_propagation.mode`:
  - `disabled` (default)
  - `auto`
- `policies.ack_propagation.max_in_flight` (default: `1024`, must be > 0)
- `policies.ack_propagation.timeout` (default: `30s`)

`policies.balanced.on_full` applies to balanced delivery paths.
`policies.broadcast.on_lag` applies to broadcast delivery paths.
`policies.ack_propagation.mode` applies to the topic hop as a whole.
`policies.ack_propagation.max_in_flight` and
`policies.ack_propagation.timeout` apply to tracked publish outcomes when
Ack/Nack propagation is enabled.

Current limitation: in broadcast mode, `ack_propagation.mode: auto` does not
aggregate acknowledgements across all subscribers. The first broadcast
subscriber Ack/Nack resolves the upstream message, so upstream completion does
not mean all broadcast subscribers processed the message. This matters
especially with `broadcast.on_lag: drop_oldest`, where one subscriber may miss
a message that another subscriber still Acks upstream. Future enhancements are
tracked in [GH-2252](https://github.com/open-telemetry/otel-arrow/issues/2252).

Topic declaration precedence (for a pipeline in a given group):

- `groups.<group>.topics.<name>` -> `topics.<name>`

`exporter:topic` node config can optionally override `queue_on_full` locally:

- `config.queue_on_full`: `block` | `drop_newest`
- effective precedence:
  `topic:exporter.config.queue_on_full` ->
  `topic.policies.balanced.on_full` -> `block`
- queue capacities remain topic-scope only
- broadcast lag handling remains topic-scope only via
  `policies.broadcast.on_lag`
- Ack/Nack tracking limits remain topic-scope only via
  `policies.ack_propagation`

## Engine Observability Pipeline

The dedicated engine internal telemetry pipeline is configured at:

- `engine.observability.pipeline.nodes`
- `engine.observability.pipeline.connections`

It is represented in resolved output as a role-tagged internal pipeline.

## Node Type (`NodeUrn`)

Accepted forms:

- Full: `urn:<namespace>:<kind>:<id>`
- OTel shortcut: `<kind>:<id>` (expanded to `urn:otel:<kind>:<id>`)

See also:

- [`docs/urns.md`](../../docs/urns.md)

## `NodeUserConfig` and Custom Node Config Payloads

`NodeUserConfig` includes:

- `type: NodeUrn`
- `outputs: Vec<PortName>` (optional declaration for named output ports)
- `default_output: Option<PortName>` (optional default output port)
- `config: serde_json::Value` (node-specific payload)
- `entity: Option<NodeEntity>` (optional node entity extension, e.g.,
  identifying attributes).

`config` is intentionally untyped in this crate so node implementations can own
their own schema and compatibility policy.

Terminology:

- In configuration, `outputs` / `default_output` are short names.
- Conceptually, these are always output ports.
- Receivers and processors have a default output port and can optionally define
  additional named output ports.
- Exporters are sinks and do not emit output ports.

## Guidance for Node Implementers

When implementing a receiver/processor/exporter:

1. Define a strongly-typed config struct in your node crate.
2. Deserialize `NodeUserConfig.config` into that struct.
3. Validate semantic constraints in your factory/constructor.
4. Return clear `InvalidUserConfig` errors with actionable messages.

Example pattern:

```rust
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MyProcessorConfig {
    threshold: usize,
    mode: String,
}

fn parse_node_config(
    raw: &serde_json::Value,
) -> Result<MyProcessorConfig, String> {
    serde_json::from_value(raw.clone())
        .map_err(|e| format!("invalid my_processor config: {e}"))
}
```

Recommendations:

- Use `#[serde(deny_unknown_fields)]` on node-specific config types.
- Keep defaults explicit and documented.
- Validate cross-field constraints early (factory time), not deep in hot path.

## Connections-Oriented Graph Model

Pipeline wiring is defined at `PipelineConfig.connections` (not inside node
declarations).

Connection defaults:

- source output defaults to `"default"` when `from` has no selector
- `policies.dispatch` is optional and defaults to `one_of` (will most likely be
  changed to `broadcast` in the future once its implementation is complete)
- with multiple destinations, `one_of` means each message is consumed by exactly
  one destination (competing consumers)

`outputs`/`default_output` are optional in many single-output pipelines and
mainly useful for explicit multi-output-port declaration and validation.
