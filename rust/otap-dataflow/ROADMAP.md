# Dataflow Engine roadmap

The [OpenTelemetry Arrow Dataflow Engine](../../README.md) is incubating.
The team is targeting **April&ndash;May 2027** for stabilization of the
**configuration model and main components**. The Rust and admin APIs
continue to evolve beyond the scope of this milestone.

The [capabilities reference][capabilities] describes what users can try today.
The [project phases][phases] document the history of the OTAP protocol and
the engine exploration.

## Operational maturity

| Priority | Work |
| --- | --- |
| Configuration and core components | Exercise the [configuration model][config] and main pipeline components with representative deployments. Improve validation, diagnostics, and examples as their contracts settle. See the [pipeline linting RFC][linting]. |
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
- **Extensibility.** Continue the experiments with [WASM processor
  plugins][wasm] and [extensible codecs][codecs], guided by integration
  experience.

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
[rfcs]: rfcs/README.md
[wasm]: crates/wasm-host/README.md
