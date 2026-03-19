# OTAP Dataflow Configuration Model (v1)

This document describes the runtime configuration model used by the OTAP
Dataflow Engine.

If you want background on why this model is structured this way, see the
`Design Rationale` section below.

If you implement receivers/processors/exporters and need crate-level API
details plus custom node config guidance, see:

- [crates/config/README.md](../crates/config/README.md)

## Configuration File Spec

The engine runtime accepts a single configuration file format (v1).

- `version`: required schema version (`otel_dataflow/v1`)
- `policies`: optional top-level defaults
- `topics`: optional top-level topic declarations
- `engine`: optional engine-wide settings
- `groups`: pipeline groups map

The engine binary loads this configuration file via `--config`.
Standalone pipeline files are not a runtime root format.

Contributor note: this top-level model is represented in code as
`OtelDataflowSpec`.

Minimal configuration example:

```yaml
version: otel_dataflow/v1
groups:
  default:
    pipelines:
      main:
        nodes:
          otlp/ingest:
            type: receiver:otlp
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4317"
          batcher:
            type: processor:batch
            config: {}
          otlp/export:
            type: exporter:otlp_grpc
            config:
              grpc_endpoint: "http://127.0.0.1:4318"
        connections:
          - from: otlp/ingest
            to: batcher
          - from: batcher
            to: otlp/export
```

## Pipeline Groups and Pipelines

The runtime model is hierarchical:

- top-level configuration (root)
- `groups` (map of pipeline groups)
- `pipelines` (map of pipelines inside each group)

A **pipeline group** is a logical container for related pipelines.

- It scopes a set of pipelines under one group id.
- It can define group-level `policies` applied to pipelines in that group.
- It is the intermediate level between root defaults and
  pipeline-specific overrides.

A **pipeline** is an executable dataflow graph.

- It contains `nodes`, `connections`, and optional pipeline-level `policies`.
- It is identified by `(group_id, pipeline_id)`.
- Pipeline ids must be unique within their group.

Example:

```yaml
version: otel_dataflow/v1
groups:
  ingest:
    policies:
      channel_capacity:
          control:
            node: 256
            pipeline: 256
          pdata: 128
    pipelines:
      traces:
        nodes: { ... }
        connections: [ ... ]
      metrics:
        nodes: { ... }
        connections: [ ... ]

  egress:
    pipelines:
      otlp_out:
        nodes: { ... }
        connections: [ ... ]
```

Policy precedence for regular pipelines follows the hierarchy:

- pipeline-level `policies`
- group-level `policies`
- top-level `policies`

Topic declaration precedence for a pipeline in a given group:

- `groups.<group>.topics.<topic>`
- `topics.<topic>`

## Pipeline Structure

At pipeline level:

- `nodes`: map of node id -> node declaration
- `connections`: explicit graph wiring
- `policies` (optional)

At node level:

- `type`: `NodeUrn` (`urn:<namespace>:<kind>:<id>` or `<kind>:<id>` for `otel`)
- `config`: node-specific payload
- `outputs` (optional): named output ports for multi-output nodes
- `default_output` (optional): explicit default output port for implicit sends

At connection level:

- `from`: source node id, source selector, or source array (fan-in)
- `to`: destination node id or destination array (fan-out)
- `policies.dispatch` (optional)

Important behavior:

- Graph topology is explicit via `connections`.
- `kind` is inferred from node `type`.
- For single-output nodes, you do not need `outputs` or `default_output`.
- Parsing is strict: unknown fields are rejected (might be relaxed in the future
  if we find good use cases for extensibility, but for now this is intentional
  to catch typos and mistakes early).

## Engine Section

`engine` is the home for engine-wide settings:

- `topics`
- `http_admin`
- `telemetry`
- `observed_state`
- `observability`

### Engine Topic Settings

Engine-wide topic runtime defaults are declared at `engine.topics`.

- `engine.topics.impl_selection`:
  - `auto` (default): infer the most efficient runtime variant from topology
  - `force_mixed`: disable topology-based optimization and always use the
    mixed implementation

Per-topic `topics.*.impl_selection` overrides this engine-wide default when set.

### Observability Pipeline

Internal telemetry pipeline wiring is declared at:
`engine.observability.pipeline`.

```yaml
engine:
  observability:
    pipeline:
      nodes:
        itr:
          type: "urn:otel:receiver:internal_telemetry"
          config: {}
        sink:
          type: "urn:otel:exporter:console"
          config: {}
      connections:
        - from: itr
          to: sink
```

Optional observability policies are supported at:
`engine.observability.pipeline.policies` for:

- `channel_capacity`
- `health`
- `telemetry`

`resources` is intentionally not supported for observability and is rejected.

## Policy Hierarchy

Policies include channel capacity, health, runtime telemetry, and
resources controls:

```yaml
policies:
  channel_capacity:
      control:
        node: 256
        pipeline: 256
        completion: 512
      pdata: 128
  health:
    # optional overrides; defaults are applied when omitted
  telemetry:
    pipeline_metrics: true
    tokio_metrics: true
    runtime_metrics: basic
  resources:
    core_allocation:
      type: all_cores
```

Resolution order:

- regular pipelines:
  `groups.<group>.pipelines.<pipeline>.policies` -> `groups.<group>.policies` ->
  top-level `policies`
- observability pipeline:
  `engine.observability.pipeline.policies` -> top-level `policies`

Defaults at top-level:

- `channel_capacity.control.node = 256`
- `channel_capacity.control.pipeline = 256`
- `channel_capacity.control.completion = 512`
- `channel_capacity.pdata = 128`
- `telemetry.pipeline_metrics = true`
- `telemetry.tokio_metrics = true`
- `telemetry.runtime_metrics = basic`
- `resources.core_allocation = all_cores`

Control channel keys:

- `node`: per-node control inboxes
- `pipeline`: shared pipeline-runtime orchestration channel
- `completion`: shared Ack/Nack completion channel

Telemetry policy notes:

- `telemetry.runtime_metrics` accepts `none`, `basic`, `normal`, or `detailed`
- this level now gates:
  - channel endpoint transport metrics
  - per-node produced/consumed outcome metrics
  - shared pipeline control-plane metrics such as `pipeline.runtime_control`
    and `pipeline.completion`
- `basic` exports gauges and transport counters
- `normal` adds message and phase counters
- `detailed` adds latency/duration summaries and completion unwind-depth
  distribution

Resolution semantics:

- precedence is applied at policy-family level (`channel_capacity`, `health`,
  `telemetry`, `resources`)
- selected lower scope replaces upper scope for that family
- no cross-scope deep merge of nested fields
- policy objects are default-filled: if a lower-scope `policies` block exists,
  omitted families are populated with defaults at that scope (they do not
  inherit from upper scopes)

## Topic Declarations

Topics are named in-process communication points used to decouple pipelines.
Producers publish to a topic name, and consumers subscribe to that topic name
without direct pipeline-to-pipeline wiring.

Common use cases:

- Fan-out distribution where multiple downstream pipelines consume the same
  stream.
- Worker-pool style processing where multiple consumers share one input stream.
- Isolation between ingest, transform, and egress stages while keeping dataflow
  composition flexible.

Topics are declared in two places:

- global scope: `topics.<name>`
- group scope: `groups.<group>.topics.<name>`

General topic capabilities:

- Inter-pipeline decoupling between ingest, transform, and egress stages.
- Balanced worker-pool processing with one logical stream per subscription
  group.
- Broadcast fan-out / tap pipelines where multiple downstream consumers
  observe the same stream.
- Mixed topologies where one topic serves both balanced and broadcast consumers.

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
        queue_capacity: 1000
        on_lag: drop_oldest
      ack_propagation:
        mode: auto
        max_in_flight: 1024
        timeout: 30s
```

Supported `backend` values:

- `in_memory` (default, currently implemented)
- `quiver` (accepted by config, not implemented by the runtime yet)

Unsupported backend or policy combinations are rejected during startup topic
declaration with explicit errors.

Supported `impl_selection` values:

- `auto`
- `force_mixed`

Supported `balanced.on_full` values:

- `block`
- `drop_newest`

Supported `broadcast.on_lag` values:

- `drop_oldest`
- `disconnect`

Supported `ack_propagation` fields:

- `mode`:
  - `disabled`
  - `auto`
- `max_in_flight` (default: `1024`, must be > 0)
- `timeout` (default: `30s`)

`balanced.on_full` applies to balanced delivery paths. `broadcast.on_lag`
applies to broadcast delivery paths. `ack_propagation.mode` applies to the
topic hop as a whole and controls whether Ack/Nack can be bridged across
pipelines. `ack_propagation.max_in_flight` and `ack_propagation.timeout`
govern tracked publish outcomes per publisher handle when Ack/Nack propagation
is enabled.

Current limitation: in broadcast mode, `ack_propagation.mode: auto` does not
aggregate acknowledgements across all subscribers. The first broadcast
subscriber Ack/Nack resolves the upstream message, so upstream completion does
not mean all broadcast subscribers processed the message. This matters
especially with `broadcast.on_lag: drop_oldest`, where one subscriber may miss
a message that another subscriber still Acks upstream. Future enhancements are
tracked in [GH-2252](https://github.com/open-telemetry/otel-arrow/issues/2252).

Topic defaults:

- `backend = in_memory`
- `impl_selection = engine.topics.impl_selection` (whose default is `auto`)
- `policies.balanced.queue_capacity = 128`
- `policies.balanced.on_full = block`
- `policies.broadcast.queue_capacity = 128`
- `policies.broadcast.on_lag = drop_oldest`
- `policies.ack_propagation.mode = disabled`
- `policies.ack_propagation.max_in_flight = 1024`
- `policies.ack_propagation.timeout = 30s`

`exporter:topic` may locally override full-queue behavior:

```yaml
nodes:
  publish/raw:
    type: exporter:topic
    config:
      topic: raw_signals
      queue_on_full: drop_newest
```

Exporter-local `queue_on_full` behavior:

- optional (`block` or `drop_newest`)
- precedence: exporter `config.queue_on_full` ->
  topic `policies.balanced.on_full` -> default `block`
- queue capacities remain topic-declaration-only (no exporter-local override)
- broadcast lag handling remains topic-declaration-only via
  `policies.broadcast.on_lag`
- Ack/Nack tracking limits remain topic-declaration-only via
  `policies.ack_propagation`

## Output Ports

Terminology:

- In YAML, we use `outputs` and `default_output` for succinctness.
- Conceptually, these fields define output ports.

Node behavior:

- Receivers and processors emit data through output ports.
- They always have a default output port named `default`.
- They can explicitly declare additional named output ports with `outputs`.
- Exporters are sinks and do not expose output ports.

## Multi-Output Example

```yaml
version: otel_dataflow/v1
groups:
  default:
    pipelines:
      main:
        nodes:
          otlp/ingest:
            type: receiver:otlp
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4317"

          router:
            type: processor:type_router
            outputs: ["logs", "metrics", "traces"]
            config: {}

          logs_exporter:
            type: exporter:otlp_grpc
            config:
              grpc_endpoint: "http://127.0.0.1:4318"

          metrics_exporter:
            type: exporter:noop
            config: {}

          traces_exporter:
            type: exporter:noop
            config: {}

        connections:
          - from: otlp/ingest
            to: router
          - from: router["logs"]
            to: logs_exporter
          - from: router["metrics"]
            to: metrics_exporter
          - from: router["traces"]
            to: traces_exporter
```

Port selection rules:

- If a source uses `router["logs"]`, that named output port is used.
- If `from` omits selector, output port `"default"` is used.
- If `outputs` is declared on a node, referenced source ports must belong to
  that list.

Dispatch policy rules:

- If `policies.dispatch` is omitted, behavior defaults to `one_of`.
- `dispatch: one_of` sends each item to exactly one destination.
- With multiple destinations, `one_of` uses competing consumers on the same
  channel.
- Destination selection is runtime-driven and best-effort balanced, not strict
  deterministic round-robin.
- `dispatch: broadcast` is parsed but not yet supported when `to` has multiple
  destinations.
- For single-destination connections, `policies.dispatch` is accepted but has no
  runtime effect.

## Validation Behavior

Config loading validates:

- Missing source/destination nodes in connections.
- Graph cycles.
- Source output selector validity when node `outputs` is declared.
- Non-zero channel capacities (`control.node`, `control.pipeline`,
  `control.completion`, `pdata`).
- Non-zero topic queue capacities
  (`topics.*.policies.balanced.queue_capacity`,
  `topics.*.policies.broadcast.queue_capacity`).
- Root schema version compatibility (`version: otel_dataflow/v1`).
- Observability constraints (`engine.observability.pipeline.policies.resources`
  is rejected).

## Go Collector Users: Mapping and Differences

### Conceptual Mapping

- Collector `receivers/processors/exporters` -> `nodes`
- Collector pipeline lists (`receivers -> processors -> exporters`) -> explicit
  `connections`
- Collector component instance id -> node id key under `nodes`
- Collector branch routing patterns -> node `outputs` +
  `from: node["output_port_name"]`

### Key Differences vs Go Collector Config

- Wiring is explicit in a DAG, not implicit from ordered arrays.
- A single pipeline can express richer fan-in/fan-out relationships directly.
- Node type is a URN with kind suffix (`receiver:otlp`, `processor:batch`,
  etc.).
- Complex topologies (branching, routing, fallback patterns) are modeled through
  graph shape and node semantics.

## Design Rationale

The design goals behind this model are:

- Make topology explicit.
- Avoid implicit behaviors that hide fan-in/fan-out semantics.
- Support hyper-edge instead of just point-to-point connections to enable richer
  topologies.
- Separate node declaration from graph wiring.
- Keep node config payloads isolated and owned by node implementations.
- Allow named output ports for advanced topologies.

Issue reference:

- Issue [#1970](https://github.com/open-telemetry/otel-arrow/issues/1970)

## Future Extensions (Not Implemented in v1)

The following ideas are discussed and intentionally left for later steps:

- Additional reserved metadata maps:
  - node-level `attributes`
  - connection-level `attributes`
  - policy-level `attributes`
- Global defaults section (for example edge policy defaults).
- Node-level lifecycle/tenancy/telemetry policies.
- Topic receiver/exporter runtime wiring and additional topic policy families
  (slow consumer handling, persistence, delivery guarantees).

## URN Reference

See `docs/urns.md` for the canonical URN format and examples.
