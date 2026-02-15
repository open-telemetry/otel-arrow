# `otap-df-config`

Configuration model crate for the OTAP Dataflow Engine.

If you are authoring pipeline YAML for runtime use, start with:

- [`docs/configuration-model.md`](../../docs/configuration-model.md)

Design rationale and prior-art discussion are also captured in:

- Issue [#1970](https://github.com/open-telemetry/otel-arrow/issues/1970)

This README focuses on crate-level model details and implementer-oriented
guidance.

## What This Crate Defines

Main public model types:

- `engine::OtelDataflowSpec`: root multi-pipeline-group config
- `pipeline_group::PipelineGroupConfig`
- `pipeline::PipelineConfig`: nodes, connections, settings
- `node::NodeUserConfig`: per-node configuration envelope
- `node_urn::NodeUrn`: parsed/canonicalized node type URN

The model is strict (`serde(deny_unknown_fields)` in key types) and validated on
load.
This means fields discussed as future extensions (for example node/connection
attributes or advanced edge policies) are currently rejected unless and until
the schema is extended.

## Parsing and Validation Entry Points

Typical loading APIs:

- `OtelDataflowSpec::from_file`, `from_yaml`, `from_json`
- `PipelineConfig::from_yaml`, `from_json` (in-memory parsing only)

These entry points perform:

1. Deserialization (YAML/JSON)
2. Node URN canonicalization
3. Structural validation (connections, graph constraints, etc.)

## Node Type (`NodeUrn`)

Accepted forms:

- Full: `urn:<namespace>:<id>:<kind>`
- OTel shortcut: `<id>:<kind>` (expanded to `urn:otel:<id>:<kind>`)

See also:

- [`docs/urns.md`](../../docs/urns.md)

## `NodeUserConfig` and Custom Node Config Payloads

`NodeUserConfig` includes:

- `type: NodeUrn`
- `outputs: Vec<PortName>` (optional declaration for named output ports)
- `default_output: Option<PortName>` (optional default output port)
- `config: serde_json::Value` (node-specific payload)

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

fn parse_node_config(raw: &serde_json::Value) -> Result<MyProcessorConfig, String> {
    serde_json::from_value(raw.clone()).map_err(|e| format!("invalid my_processor config: {e}"))
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
