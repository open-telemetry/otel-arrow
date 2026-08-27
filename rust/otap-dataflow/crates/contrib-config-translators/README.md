# Contrib Config Translators

Vendor-specific configuration translators that produce OTAP dataflow pipeline
specifications.

A *translator* converts a vendor's own configuration document into an
`OtelDataflowSpec` the engine can run. This sits one stage after
`ConfigProvider`, which resolves a URI to raw content:

```text
  vendor config (JSON)  --ConfigTranslator-->  OtelDataflowSpec  -->  pipeline YAML
```

## Available translators

- `amcs` (`src/amcs/mod.rs`) -- Azure Monitor Configuration Service, third-party
  (3P) Data Collection Rules.

A first-party (1P) Geneva/GigLA translator can be added as a sibling module
implementing the same `ConfigTranslator` trait.

## Library usage

```rust,no_run
use otap_df_contrib_config_translators::ConfigTranslator;
use otap_df_contrib_config_translators::amcs::AmcsTranslator;

let payload = std::fs::read_to_string("amcs-config.json")?;
let yaml = AmcsTranslator::new().translate_to_yaml(&payload)?;
# Ok::<(), Box<dyn std::error::Error>>(())
```

## CLI usage

A development binary is provided for inspecting translator output:

```bash
cargo run -p otap-df-contrib-config-translators --bin config-translator -- \
  --input tests/fixtures/AMCSConfig.json \
  --validate
```

- `--input <path>` -- vendor configuration document to translate.
- `--output <path>` -- where to write the YAML; defaults to stdout.
- `--dialect <name>` -- configuration dialect; defaults to `amcs`.
- `--validate` -- check the YAML parses back through the engine loader.

The binary source lives at `src/cli.rs` rather than the conventional `src/bin/`,
because the repository's top-level `.gitignore` matches `bin/` and would make the
file invisible to git.

The generated file can be handed straight to the engine:

```bash
cargo run --bin df_engine -- --config generated.yaml --num-cores 1
```

## The AMCS translator

AMCS delivers the Azure Monitor Agent every customer-authored Data Collection
Rule (DCR) that applies to a host, as a single JSON document. This crate is a
port of `AMCSParser.ExtractConfiguration` from the .NET `AMCSConfiguration`
project: the input is byte-for-byte the same payload the .NET agent consumes,
and only the output differs -- a pipeline specification instead of an in-memory
list of endpoint bindings.

### Stages

- `schema` -- serde model of the AMCS JSON payload.
- `listener` -- OTLP listener discovery from environment variables.
- `extract` -- payload plus listeners to routable endpoint bindings.
- `emit` -- endpoint bindings to a pipeline specification.

### Translation rules

- A configuration whose `content.kind` is `AgentSettings` carries listener
  settings, not telemetry routing. It is consumed for its ports and never
  becomes a pipeline branch. Unknown kinds are skipped rather than rejected, so
  a newer control plane can add rule kinds without breaking existing agents.
- `dataSource.kind` is matched case-insensitively: `otelLogs` becomes logs and
  `otelTraces` becomes traces. Every other kind (`perfCounter`, `extension`,
  and so on) is skipped.
- Only channels with `protocol: gig` carry OTLP endpoint templates. `ods`
  channels are legacy and are ignored.
- The literal `<STREAM>` token in an endpoint template is replaced with each of
  the data source's stream names.
- Each export path is identified by `{configurationId}.{channelId}`.
- `resourceAttributeRouting` is optional and tracked **per signal**, because a
  rule may filter its logs while broadcasting its traces. A missing filter
  means broadcast, and is logged.
- `otelLogsEndpointUriTemplate` and `otelTracesEndpointUriTemplate` are
  independently optional.

### Listener configuration

Listeners are global to the host, not per rule, which is why the generated
pipeline has a single OTLP receiver. Ports come from the environment or from an
Agent Settings DCR; the host is environment-only.

Port values resolve in this order, highest priority first, per
`Telemetry-Collection-Spec/AMACoreAgent/otel-port-configuration.md`:

1. **Environment variable** -- `OTLP_GRPC_LOGS_TRACES_PORT` and
   `OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT`. If set, the Agent Settings value is
   ignored, so AKS deployments driven by environment variables are never
   disrupted by a DCR.
2. **Agent Settings DCR** -- the `OtlpGrpcLogsTracesPort` and
   `OtlpHttpProtobufLogsTracesPort` settings.
3. **Built-in default** -- `4319` for gRPC and `4320` for HTTP/protobuf.

Value handling:

- `-1` disables that listener outright, overriding any Agent Settings value.
- Any other invalid value -- unparseable, or outside `[1, 65535]` -- is treated
  as **unset**, so resolution falls through to the next source.
- `OTLP_GRPC_LOGS_TRACES_HOST` and `OTLP_HTTP_PROTOBUF_LOGS_TRACES_HOST`
  override the default host `localhost`. There is no Agent Settings equivalent.

A listener being configured does not by itself open a port: an OTel data-source
rule must also be present. An Agent Settings DCR on its own produces no
pipeline, which surfaces as an `EmptyPipeline` error.

### Agent Settings DCR shape

The Agent Settings DCR is a separate configuration entry alongside the
data-source rules, and a host has at most one:

```json
{
  "configurationId": "dcr-00000000000000000000000000000003",
  "content": {
    "kind": "AgentSettings",
    "settings": [
      { "name": "MaxDiskQuotaInMB", "value": "10240" },
      { "name": "OtlpGrpcLogsTracesPort", "value": "4319" },
      { "name": "OtlpHttpProtobufLogsTracesPort", "value": "4320" }
    ]
  }
}
```

Settings not related to listeners, such as `MaxDiskQuotaInMB`, are parsed and
ignored. `agentSettings` is accepted as an alias for `settings` because the
specification writes it that way.

### Generated topologies

With a single export path the result is one pipeline:

```text
  receiver:otlp -> [filter] -> batch -> exporter:otlp_http
```

With several export paths the receiver's single output port cannot fan out --
the engine rejects broadcast to more than one target -- so a `processor:fanout`
clones each message to one named output port per rule. Every path therefore
sees every message and decides for itself, matching the .NET publisher.

```text
                    +-- port_a --> [filter] -> batch -> exporter
  receiver -> fanout+
                    +-- port_b --> [filter] -> batch -> exporter
```

Each path carries its own batch node. A batch shared across rules would merge
their telemetry with no way to separate it again before the exporters, because
the batch processor has no partition key or per-key queueing.

The `filter` node is omitted when a path neither routes on a resource attribute
nor needs to drop a signal, so a rule that accepts everything connects straight
to its batch node.

### Where each generated value comes from

- Receiver `listening_addr` -- environment variables, with defaults.
- Filter `resource_attributes` -- AMCS `resourceAttributeRouting`.
- Exporter `logs_endpoint` and `traces_endpoint` -- AMCS channel templates, with
  `<STREAM>` substituted.
- Exporter `user_agent` and the `azure-monitor-source-resourceId` /
  `x-ms-AzureRegion` headers -- the embedding host, not the AMCS payload. The
  resource id is percent-encoded, matching the .NET agent.
- Receiver `max_decoding_message_size` / `max_request_body_size` -- fixed at
  64 MiB. Without this the engine's own 4 MiB default silently rejects larger
  payloads before they reach the pipeline.
- `batch`, `policies`, `version` and `engine` -- fixed defaults. Batches
  flush at 1043333 bytes or after 20 seconds, in `otlp` format so payloads are
  not converted to OTAP and back on a path that is OTLP end to end.

### Dropping a signal that has no endpoint

When a path has an endpoint for only one signal, the other must be discarded.
Leaving the filter's section empty would not work: the filter processor treats
an empty match list as "match everything". Instead the absent signal is given a
match list containing a sentinel value that cannot occur in real telemetry,
producing an all-false mask.

Without this the exporter would fall back to `endpoint + "/v1/<signal>"` and
misroute the signal.

### Intentional divergence from the .NET reference

`AMCSParser.cs:235-242` selects the endpoint template with:

```csharp
if (eventName == Log && otelLogsEndpointUriTemplate != null) { ...logs... }
else if (otelTracesEndpointUriTemplate != null)              { ...traces... }
```

When the signal is `Log` but `otelLogsEndpointUriTemplate` is null, control
falls into the `else if` and a **traces** URL is emitted for a **logs** data
source. Since both templates are independently optional, that path is reachable
with real customer configuration and would silently misroute logs.

This port selects each signal's own template only, so a data source whose
template is absent produces no endpoint at all. See `endpoint_template` in
`src/amcs/extract.rs`.

## Testing

```bash
cargo test -p otap-df-contrib-config-translators
```

The fixtures in `tests/fixtures/` are verbatim copies of
`PipelineAgentTests/TestData/ConfigPackage/AMCSConfig*` from the
`PipelineAgent` repository.

- `AMCSConfig` -- one rule; logs, traces and `perfCounter`; `gig` and `ods`
  channels.
- `AMCSConfig2` -- as above, but the traces source has no routing attribute.
- `AMCSConfig3` -- two rules sharing one channel id, with different routing
  values.
- `AMCSConfig4` -- no data sources at all.
- `AMCSConfig5` -- logs only; the `gig` channel has no traces template.
- `AMCSConfigAgentSettings` -- an Agent Settings rule alongside a data-source
  rule.
- `AMCSConfigAgentSettingsOnly` -- an Agent Settings rule with no data-source
  rule, which must open no ports.

`tests/parity.rs` asserts values captured by invoking the real
`AMCSParser.ExtractConfigurationByIdentifier` from the prebuilt
`AMCSConfiguration.dll`, which is what makes this a port rather than a
reimplementation. Note that the .NET parser does not read Agent Settings --
`Content.kind` and `Content.settings` are commented out in `Configurations.cs` --
so that behaviour is specified by
`Telemetry-Collection-Spec/AMACoreAgent/otel-port-configuration.md` rather than
by parity.

## References

- `Telemetry-Collection-Spec/AMACoreAgent/otel-port-configuration.md` -- the
  approved specification for OTel port configuration via environment variables
  and the Agent Settings DCR.
- `PipelineAgent/AMCSConfiguration/AMCSParser.cs` -- the .NET reference parser.
