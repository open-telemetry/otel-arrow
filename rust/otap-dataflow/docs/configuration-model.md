# OTAP Dataflow Configuration Model (v1)

This document describes the configuration model used by the OTAP Dataflow
Engine.

If you want background on why this model is structured this way, see the
`Design Rationale` section below.

If you implement receivers/processors/exporters and need crate-level model/API
details plus custom node config guidance, see:

- [crates/config/README.md](../crates/config/README.md)

## Pipeline Structure

At pipeline level:

- `nodes`: map of node id -> node declaration
- `connections`: explicit graph wiring
- `settings`, `quota`, `service`

Note: `settings`, `quota`, and `service` are not yet fully stabilized in v1.

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

## Output Ports

Terminology:

- In YAML, we use `outputs` and `default_output` for succinctness.
- Conceptually, these fields define output ports.

Node behavior:

- Receivers and processors emit data through output ports.
- They always have a default output port named `default`.
- They can explicitly declare additional named output ports with `outputs`.
- Exporters are sinks and do not expose output ports.

Note: We currently don't have a use case for named input ports, but this could
be added in the future if needed.

## Quick Start

Minimal pipeline:

```yaml
nodes:
  otlp/ingest:
    type: otlp:receiver
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"

  batcher:
    type: batch:processor
    config: { }

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

## Multi-Output Example

```yaml
nodes:
  otlp/ingest:
    type: otlp:receiver
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4317"

  router:
    type: type_router:processor
    outputs: [ "logs", "metrics", "traces" ]
    config: { }

  logs_exporter:
    type: otlp:exporter
    config:
      grpc_endpoint: "http://127.0.0.1:4318"

  metrics_exporter:
    type: noop:exporter
    config: { }

  traces_exporter:
    type: noop:exporter
    config: { }

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
- Additional node-specific constraints (for example, fanout destination rules).

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

- https://github.com/open-telemetry/otel-arrow/issues/1970

## Future Extensions (Not Implemented in v1)

The following ideas are discussed and intentionally left for later steps:

- Additional reserved metadata maps:
    - node-level `attributes`
    - connection-level `attributes`
    - policy-level `attributes`
- Global defaults section (for example edge policy defaults).
- Node-level lifecycle/tenancy/telemetry policies.
- Topic-based inter-pipeline wiring.

## URN Reference

See `docs/urns.md` for the canonical URN format and examples.
