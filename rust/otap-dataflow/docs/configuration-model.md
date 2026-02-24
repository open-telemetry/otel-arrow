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
            type: otlp:receiver
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4317"
          batcher:
            type: batch:processor
            config: {}
          otlp/export:
            type: otlp:exporter
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
- It is the intermediate level between root defaults and pipeline-specific overrides.

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

- `type`: `NodeUrn` (`urn:<namespace>:<id>:<kind>` or `<id>:<kind>` for `otel`)
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

- `http_admin`
- `telemetry`
- `observed_state`
- `observability`

### Observability Pipeline

Internal telemetry pipeline wiring is declared at:
`engine.observability.pipeline`.

```yaml
engine:
  observability:
    pipeline:
      nodes:
        itr:
          type: "urn:otel:internal_telemetry:receiver"
          config: {}
        sink:
          type: "urn:otel:console:exporter"
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

Policies include channel capacity, health, runtime telemetry, and resources controls:

```yaml
policies:
  channel_capacity:
      control:
        node: 256
        pipeline: 256
      pdata: 128
  health:
    # optional overrides; defaults are applied when omitted
  telemetry:
    pipeline_metrics: true
    tokio_metrics: true
    channel_metrics: true
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
- `channel_capacity.pdata = 128`
- `telemetry.pipeline_metrics = true`
- `telemetry.tokio_metrics = true`
- `telemetry.channel_metrics = true`
- `resources.core_allocation = all_cores`

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

Current topic policy support:

```yaml
topics:
  raw_signals:
    description: "raw ingest stream"
    policies:
      queue_capacity: 1000
      queue_on_full: drop_newest
```

Supported `queue_on_full` values:

- `block`
- `drop_newest`

Topic defaults:

- `policies.queue_capacity = 128`
- `policies.queue_on_full = block`

`topic:exporter` may locally override full-queue behavior:

```yaml
nodes:
  publish/raw:
    type: topic:exporter
    config:
      topic: raw_signals
      queue_on_full: drop_newest
```

Exporter-local `queue_on_full` behavior:

- optional (`block` or `drop_newest`)
- precedence: exporter `config.queue_on_full` -> topic `policies.queue_on_full`
  -> default `block`
- `queue_capacity` remains topic-declaration-only (no exporter-local override)

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
            type: otlp:receiver
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4317"

          router:
            type: type_router:processor
            outputs: ["logs", "metrics", "traces"]
            config: {}

          logs_exporter:
            type: otlp:exporter
            config:
              grpc_endpoint: "http://127.0.0.1:4318"

          metrics_exporter:
            type: noop:exporter
            config: {}

          traces_exporter:
            type: noop:exporter
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
- Non-zero channel capacities (`control.node`, `control.pipeline`, `pdata`).
- Non-zero topic queue capacity (`topics.*.policies.queue_capacity`).
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
- Node type is a URN with kind suffix (`otlp:receiver`, `batch:processor`,
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
