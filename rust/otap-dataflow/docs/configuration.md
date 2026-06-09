# Configuration

This guide is the practical starting point for OTel Arrow Dataflow Engine
configuration. It walks from the file loaded by `df_engine` to the root
structure, runtime defaults, pipeline topology, node configuration, policies,
topics, observability, and validation workflow.

Read it sequentially if you are configuring the engine for the first time, or
use the section headings as a checklist when reviewing YAML. For exact field
semantics, defaults, precedence rules, and validation behavior, use the
[configuration model reference](configuration-model.md); for node-specific
config payloads, use the [core node catalog](../crates/core-nodes/README.md)
and [contrib node catalog](../crates/contrib-nodes/README.md).

> [!WARNING]
> This project is experimental. The configuration format is not yet stable and
> can change at any moment, including incompatible changes between releases.

## Configuration Location

The `df_engine` binary reads one root configuration through `--config`.
If you need to build or run `df_engine` locally first, start with the
[Development Setup](../README.md#development-setup) section in the main README.

If `--config` is omitted, the engine looks for `config.yaml` in the current
working directory.

Supported config sources:

<!-- markdownlint-disable MD013 -->

| Source                      | Description                                                                                |
| --------------------------- | ------------------------------------------------------------------------------------------ |
| `/path/to/config.yaml`      | Bare local path. Treated the same as `file:`.                                              |
| `file:/path/to/config.yaml` | Local file path. `.json` files parse as JSON; other files parse as YAML.                   |
| `env:MY_VAR`                | Environment variable containing the full configuration text.                               |
| `yaml:<content>`            | Inline YAML. `::` expands nested keys for small test fragments.                            |
| `http://host/path`          | Unauthenticated HTTP GET. JSON is detected from `Content-Type`; otherwise YAML is assumed. |

<!-- markdownlint-enable MD013 -->

The `http:` provider retries failed fetches with exponential backoff. `https:`,
authenticated config sources, and multi-file merge are not implemented.

Run with a config file:

```bash
cargo run -- --config configs/otlp-otlp.yaml
```

Validate a file before running it:

```bash
cargo run -- --config configs/otlp-otlp.yaml --validate-and-exit
```

Validation parses YAML or JSON, validates the root model, checks graph
references, checks that every node type is registered in the binary, and runs
node-specific config validation when the component provides it.

After loading, the CLI can override selected engine-level settings:

- `--num-cores`
- `--core-id-range`
- `--http-admin-bind`

Raw configuration text supports environment substitution before parsing:

- `${env:VAR}`: replace with `$VAR`; error if the variable is unset.
- `${env:VAR:-default}`: replace with `$VAR`, or `default` when unset.
- `${env:VAR:-}`: replace with `$VAR`, or the empty string when unset.
- `$$`: literal `$`.

## Configuration Structure

Every runtime file is a single root document that describes the engine process:

```yaml
version: otel_dataflow/v1
policies: {}
topics: {}
engine: {}
groups:
    default:
        topics: {}
        policies: {}
        pipelines:
            main:
                type: otap
                policies: {}
                extensions: {}
                nodes: {}
                connections: []
```

Required root fields:

- `version`: must be `otel_dataflow/v1`.
- `groups`: pipeline groups keyed by group id. A single engine can run many
  groups, and each group can run many pipelines.

Optional root fields:

- `policies`: top-level policy defaults.
- `topics`: global topic declarations.
- `engine`: engine-wide settings.

Most simple configurations only need `version` and `groups`.

Minimal OTLP receive, batch, and export pipeline:

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

                    batch:
                        type: processor:batch
                        config: {}

                    otlp/export:
                        type: exporter:otlp_grpc
                        config:
                            grpc_endpoint: "http://192.0.2.10:4317"

                connections:
                    - from: otlp/ingest
                      to: batch
                    - from: batch
                      to: otlp/export
```

## Receivers, Processors, and Exporters

Receivers, processors, and exporters are configured as pipeline `nodes`. Each
node has a pipeline-local id and a `type`:

```yaml
nodes:
    otlp/ingest:
        type: receiver:otlp
        config: {}
    batch:
        type: processor:batch
        config: {}
    otlp/export:
        type: exporter:otlp_grpc
        config: {}
```

The `type` can use either the OTel shortcut form or the full URN:

```yaml
type: receiver:otlp
```

```yaml
type: urn:otel:receiver:otlp
```

The node kind is inferred from the `type`. The engine does not use separate
Collector-style `receivers`, `processors`, and `exporters` maps.

Common node fields:

- `type`: required node implementation URN or shortcut.
- `description`: optional human-readable description.
- `config`: node-specific configuration owned by the selected component.
- `outputs`: optional declared output ports for multi-output receivers or
  processors.
- `default_output`: optional default output port used by nodes that emit without
  selecting a port.
- `capabilities`: optional bindings from capability name to pipeline extension.
- `entity`: optional node entity enrichment metadata.
- `header_capture`: receiver-only transport header capture override.
- `header_propagation`: exporter-only transport header propagation override.

Core node types are listed in the
[core-node catalog](../crates/core-nodes/README.md). Each node links to a
README beside its implementation with node-specific configuration examples,
limits, stability, and telemetry.

Optional contrib nodes are listed in the
[contrib-node catalog](../crates/contrib-nodes/README.md). They are registered
only when the corresponding crate features are enabled in the binary you run.

## Pipeline Groups and Pipelines

The engine is designed to run and manage many pipelines in parallel inside one
process. Groups are logical containers for related pipelines and can be mapped
to operational boundaries such as a team, project, tenant, environment, or
deployment slice.

```yaml
groups:
    ingest:
        policies: {}
        topics: {}
        pipelines:
            traces:
                nodes: {}
                connections: []
```

A pipeline is an executable graph:

- `type`: pipeline data type. Defaults to `otap`.
- `nodes`: receivers, processors, and exporters in the data path.
- `extensions`: long-lived components available to nodes through capabilities.
- `connections`: explicit graph wiring.
- `policies`: optional pipeline-level policy overrides.

An `otap` pipeline is multi-signal by default. Logs, metrics, and traces can
move through the same pipeline graph, unlike the Collector model where
pipelines are usually split by signal type. You do not need a connector just to
move data from a traces path into a metrics path; model the routing or
conversion you need with nodes and explicit connections.

The engine does not infer pipeline order from node roles. If data should flow
between two nodes, add a `connections` entry.

Policies are scoped by hierarchy. For regular pipelines, precedence is:

1. Pipeline-level `policies`
2. Group-level `policies`
3. Top-level `policies`

Policy overrides apply by policy family rather than by deep-merging every
nested field. The process-wide memory limiter is only supported at top-level
`policies.resources.memory_limiter`.

For detailed policy guides, see:

- [Transport header policies](transport-headers.md)
- [Memory limiter policy](memory-limiter-phase1.md)

## Connections and Output Ports

Connections define the pipeline graph:

```yaml
connections:
    - from: otlp/ingest
      to: batch
    - from: batch
      to: otlp/export
```

Connection sources must be receivers or processors. Exporters are sinks and
cannot be used as `from` endpoints.

Because connections are explicit, the configuration can describe topologies
beyond a simple receiver-processor-exporter chain, including fan-in, fan-out,
named output ports, competing consumers, topic bridges, and observability
pipelines.

Fan-in and fan-out are explicit:

```yaml
connections:
    - from: [ingest/a, ingest/b]
      to: batch
    - from: batch
      to: [worker/a, worker/b]
      policies:
          dispatch: one_of
```

`dispatch: one_of` sends each item to one destination. With multiple
destinations, the destinations act as competing consumers. The `broadcast`
dispatch policy is parsed but is not currently supported for multi-destination
connections.

Most nodes use the default output. Multi-output processors can expose named
ports:

```yaml
nodes:
    router:
        type: processor:type_router
        outputs: ["logs", "metrics", "traces"]
        config: {}

connections:
    - from: router["logs"]
      to: logs/export
    - from: router["metrics"]
      to: metrics/export
    - from: router["traces"]
      to: traces/export
```

If `from` omits a port selector, the engine selects the `default` output. When
a node declares `outputs`, any selected source port must be listed there.

For details, see [Output Ports](configuration-model.md#output-ports).

## Topics

Topics are named in-process communication points. Use them when one pipeline
should publish data that another pipeline consumes without direct
pipeline-to-pipeline wiring.

Declare a global topic:

```yaml
topics:
    raw_signals:
        description: "raw ingest stream"
        backend: in_memory
```

Declare a group-local topic:

```yaml
groups:
    ingest:
        topics:
            raw_signals:
                description: "ingest-local raw stream"
```

For a pipeline in a group, group-local topics override global topics with the
same local name.

Publish to a topic with `exporter:topic`:

```yaml
type: exporter:topic
config:
    topic: raw_signals
```

Consume from a topic with `receiver:topic`:

```yaml
type: receiver:topic
config:
    topic: raw_signals
```

Use `backend: in_memory` for current runtime configurations. The `quiver`
backend is reserved in the schema and rejected by the current runtime.

Topic policies control balanced queue capacity, broadcast lag behavior, and
Ack/Nack propagation across topic hops. For exact topic policy fields and
limits, see [Topic Declarations](configuration-model.md#topic-declarations).

## Engine Section

The optional `engine` section controls engine-wide settings:

- `http_admin`: HTTP admin server bind address.
- `telemetry`: telemetry backend configuration shared across pipelines.
- `observed_state`: observed-state store settings.
- `topics`: engine-wide topic runtime defaults.
- `observability`: dedicated internal observability pipeline.
- `custom`: ignored by the engine and reserved for embedding applications.

HTTP admin bind example:

```yaml
engine:
    http_admin:
        bind_address: "127.0.0.1:8080"
```

An observability pipeline reads internal telemetry and exports it like any other
pipeline:

```yaml
engine:
    observability:
        pipeline:
            nodes:
                internal:
                    type: receiver:internal_telemetry
                    config: {}
                console:
                    type: exporter:console
                    config: {}
            connections:
                - from: internal
                  to: console
```

Observability pipelines use the same node and connection model as regular
pipelines. They support `channel_capacity`, `health`, and `telemetry` policies,
but resource policies are intentionally not supported there.

For exact engine-level fields, see
[Engine Section](configuration-model.md#engine-section).

## Other Information

Useful adjacent docs:

- [Core-node catalog](../crates/core-nodes/README.md): node list and links to
  per-node configuration docs beside the implementation.
- [Contrib-node catalog](../crates/contrib-nodes/README.md): optional
  feature-gated node implementations.
- [Configuration model reference](configuration-model.md): exact field
  semantics, defaults, precedence, and validation behavior.
- [URN reference](urns.md): node type and extension type syntax.
- [Processor behavior taxonomy](processors.md): processor behavior categories.
- [Transport header policies](transport-headers.md): inbound header capture and
  outbound header propagation.
- [TLS examples](../configs/README.md): `test-tls-only.yaml` and
  `test-mtls.yaml`.
- [Proxy support](proxy-support.md): outbound proxy behavior.

For agent consumption, start from this page, then follow the core-node and
contrib-node catalogs. Per-node READMEs use predictable headings such as
`Metadata`, `Overview`, `Configuration`, `Examples`, `Telemetry`, `Limits`, and
`Related Docs`.

## Validate and Troubleshoot

Configuration parsing is strict. Unknown fields are rejected so typos fail
early.

Common validation checks include:

- `version` must be `otel_dataflow/v1`.
- Connections must reference existing nodes.
- Connection sources must be receivers or processors.
- Graphs must not contain cycles.
- Referenced output ports must exist when `outputs` is declared.
- Channel and topic capacities must be non-zero.
- `groups.system` is reserved for engine-managed pipelines.
- `backend: quiver` topics are rejected by the current runtime.
- Node `config` fields must match the selected node type.
- Node types must be registered in the `df_engine` binary.
- Node-level `header_capture` is receiver-only.
- Node-level `header_propagation` is exporter-only.

Use `--validate-and-exit` while editing:

```bash
cargo run -- --config path/to/config.yaml --validate-and-exit
```

If validation fails inside a node config, open that node's README from the
[core-node catalog](../crates/core-nodes/README.md) or
[contrib-node catalog](../crates/contrib-nodes/README.md). If validation fails
in the root model, use the
[configuration model reference](configuration-model.md).

## Reference

- [Runnable examples](../configs/README.md)
- [Core-node catalog](../crates/core-nodes/README.md)
- [Contrib-node catalog](../crates/contrib-nodes/README.md)
- [Configuration model reference](configuration-model.md)
- [URN reference](urns.md)
- [Processor behavior taxonomy](processors.md)
- [Transport header policies](transport-headers.md)
- [Memory limiter policy](memory-limiter-phase1.md)
