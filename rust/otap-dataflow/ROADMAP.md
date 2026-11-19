# Dataflow Engine roadmap

The [OpenTelemetry Arrow Dataflow Engine](../../README.md) is incubating.
The team is targeting **April&ndash;May 2027** for stabilization of the
**configuration model, main components, and core self-telemetry**.
API stability is outside that milestone.

The [capabilities reference][capabilities] describes what users can try today.
The [project phases][phases] document the history of the OTAP protocol and
the engine exploration.

## Operational maturity

| Priority | Work |
| --- | --- |
| Configuration and core components | Exercise the [configuration model][config] and main pipeline components with representative deployments. Improve validation, diagnostics, and examples as their contracts settle. See the [pipeline linting RFC][linting]. |
| Core self-telemetry | Stabilize essential engine and pipeline metrics and structured diagnostic events. Preserve the names, units, attributes, and meanings that dashboards, alerts, and queries depend on. See the [self-observability guide][self-observability] and [telemetry compatibility guide][telemetry-compatibility]. |
| Semantic conventions and instrumentation | Build out the [DFE-specific semantic convention registry][registry]. Extend existing [Weaver CI validation of emitted events][weaver-ci] to metrics and attributes as registry coverage grows. Generate an optimized, type-safe Rust instrumentation SDK from that registry using [Weaver][weaver]. |
| Delivery and recovery | Harden acknowledgements, retry, and [durable buffering][buffer] under overload, restart, and downstream failure. Continue work on [filelog discovery and checkpoints][filelog]. |
| Resource control | Refine [memory protection and admission control][memory] and explore [retained-work accounting][accounting] to attribute pressure to the work that causes it. |
| Live and fleet operations | Improve the experience of [rolling reconfiguration][reconfiguration], rollback, and diagnosis. Exercise the early [OpAMP agent][opamp] with remote configuration and health reporting. |

## Interoperability and programmability

- **Signal coverage.** Explore adding OTLP and OTAP support for profiles and
  track how OpenTelemetry entities should be represented.
- **Telemetry Policies.** Follow the [OpenTelemetry proposal][policies] and
  explore how portable policies could fit in DFE.
- **Processing languages.** Broaden and validate the [OPL, OTTL, and KQL
  capabilities][languages] with real transformation workloads.

## WASM plugins and codecs

- **WASM processor plugins.** The [experimental host][wasm] loads WASM
  components at pipeline startup behind the `wasm` Cargo feature. A plugin
  invokes host-side operations on Arrow batches through an opaque handle.
  Near-term work includes evolving the guest interface, resource controls,
  and processor error handling.
- **Pluggable codecs.** The [codec infrastructure][codecs] can carry encoded
  payloads through a pipeline, batch them in their original format, and
  decode them to Arrow when a processor needs records. The built-in codecs
  are `otlp-bytes` (decode and encode logs, metrics, and traces) and
  `syslog-batch` (decode framed Syslog/CEF logs). Codecs are registered at
  build time. Future work explores hosting codecs in WASM.

Follow the [project board][board] and [design proposals][rfcs] for work in
progress. Share workloads, issues, and feedback through the
[contribution guide][contributing].

[accounting]: rfcs/0000-observe-only-retained-work-accounting.md
[board]: https://github.com/orgs/open-telemetry/projects/139
[buffer]: crates/core-nodes/src/processors/durable_buffer_processor/README.md
[capabilities]: ../../docs/dfe-capabilities.md
[codecs]: docs/codecs.md
[config]: docs/configuration-model.md
[contributing]: ../../CONTRIBUTING.md
[filelog]: docs/filelog-receiver.md
[languages]: ../../docs/dfe-capabilities.md#processing-languages
[linting]: rfcs/0008-pipeline-linting.md
[memory]: docs/memory-resource-management.md
[opamp]: docs/opamp.md
[phases]: ../../docs/project-phases.md
[policies]:
    https://github.com/open-telemetry/opentelemetry-specification/blob/main/oteps/4738-telemetry-policy.md
[reconfiguration]: docs/admin/live-reconfiguration.md
[registry]: semconv/README.md
[rfcs]: rfcs/README.md
[self-observability]: docs/self-observability.md
[telemetry-compatibility]: docs/telemetry/stability-compatibility-guide.md
[wasm]: crates/wasm-host/README.md
[weaver]: https://github.com/open-telemetry/weaver
[weaver-ci]: ../../.github/workflows/df-engine-internal-observability.yml
