# DFE capabilities

The [OpenTelemetry Arrow Dataflow Engine](../README.md) is incubating.
**Experimental** and **Alpha** components can run with the listed features
and platform requirements. **In progress** identifies unfinished work. Cargo
features listed for contrib and opt-in components are required when building
the engine. Core features are included in the default build.

Use the [core node catalog][core], [contrib node catalog][contrib], and
[contrib extension catalog][extension-catalog] for exact type names, settings,
and component-specific limits. The [configuration guide][config] explains
how to wire the components into a pipeline.

## Receivers

| Receiver | Build | Status | Coverage |
| --- | --- | --- | --- |
| [OTLP][core-receivers] | Core, `otlp` | Experimental | gRPC and HTTP for logs, metrics, and traces. |
| [OTAP][core-receivers] | Core, `otap` | Experimental | Arrow streams over gRPC for logs, metrics, and traces. |
| [Prometheus][prometheus] | Core, `prometheus` | Experimental | Scrapes Prometheus and OpenMetrics into Arrow metrics. One-core source pipeline. |
| [Host metrics][core-receivers] | Core, `host-metrics` | Experimental | Linux host metrics from procfs and sysfs. |
| [Journald][core-receivers] | Core, `journald` | Experimental | Linux systemd journal logs. |
| [Syslog/CEF][core-receivers] | Core, `syslog-cef` | Experimental | Syslog and CEF messages. |
| [Internal telemetry][core-receivers] | Core | Experimental | Engine events and metrics for observability pipelines. |
| [Topic][topics] | Core, `topic` | Experimental | Reads from an in-process topic. |
| [Kafka][contrib-receivers] | Contrib, `kafka` | Experimental | Logs, metrics, and traces. |
| [ETW and Windows performance counters][contrib-receivers] | Contrib, `etw` or `windows-perf-counters` | Experimental | Windows event logs or local performance metrics, respectively. |
| [Linux user events][contrib-receivers] | Contrib, `user-events` | Experimental | Linux tracepoints as logs. See the component docs for kernel requirements. |
| [Oracle, PostgreSQL, and SQL Server][contrib-receivers] | Individual opt-in, `oracle`, `postgresql`, or `sql-server` | Experimental | Poll query results into logs. Oracle requires separately installed client libraries. |
| [Microsoft Common Schema JSON][contrib-receivers] | Contrib, `microsoft-json` | In progress | HTTP log ingestion implementation is not operational. |
| [MySQL][contrib-receivers] | Individual opt-in, `mysql` | In progress | Module placeholder with no runtime receiver. |
| [Filelog][filelog] | Source development | In progress | Local file discovery, decoding, and checkpoints. Source primitives are not yet registered as a receiver. |

## Processors

| Processor | Build | Status | Coverage |
| --- | --- | --- | --- |
| [Batch, fan-out, and retry][core-processors] | Core | Experimental | Combine batches, send to multiple outputs, or retry downstream NACKs. |
| [Attribute, content router, and signal router][core-processors] | Core | Experimental | Edit attributes or route by content or signal. |
| [Filter and log sampling][core-processors] | Core | Experimental | Filter logs and traces or reduce log volume by sampling. |
| [Temporal reaggregation][core-processors] | Core | Experimental | Reaggregate metrics into a lower-frequency stream. |
| [Debug][core-processors] | Core | Experimental | Emit diagnostics about passing data. |
| [Log parser][log-parser] | Core | Experimental | Parse framed string logs with regex, JSON Pointers, or headerless CSV. |
| [Durable buffer][core-processors] | Core, `durable-buffer` | Experimental | Persist telemetry batches in a local durable queue. |
| [Partition][core-processors] | Core, `partition` | Experimental | Partition by expression and attach a transport header. |
| [Transform][transform] | Core, `transform` | Experimental | Apply OPL, OTTL, or KQL with language-specific limits. |
| [Resource validator][contrib-processors] | Contrib, `resource-validator` | Experimental | NACK batches missing required resource attributes. |
| [Recordset KQL][contrib-processors] | Contrib, `recordset-kql` | Experimental | Evaluate KQL over a prescribed OTAP recordset shape. |
| [Condense attributes][contrib-processors] | Contrib, `condense-attributes` | In progress | Condense multiple log attributes into one string attribute. |
| [WASM processor][wasm] | Opt-in, `wasm` | Experimental | In-tree processor plugin prototype with an evolving guest contract. |

## Exporters

| Exporter | Build | Status | Coverage |
| --- | --- | --- | --- |
| [OTLP gRPC and HTTP][core-exporters] | Core, `otlp` | Experimental | Logs, metrics, and traces. |
| [OTAP][core-exporters] | Core, `otap` | Experimental | Arrow streams over gRPC for logs, metrics, and traces. |
| [Parquet][core-exporters] | Core, `parquet` | Experimental | Write OTAP batches as Parquet files. |
| [File][core-exporters] | Core, `file` | Experimental | Write OTLP JSON Lines, one signal per exporter instance. |
| [Console and no-op][core-exporters] | Core | Experimental | Inspect supported output formats or acknowledge and discard data. |
| [Topic][topics] | Core, `topic` | Experimental | Publishes to an in-process topic. |
| [Kafka][contrib-exporters] | Contrib, `kafka` | Experimental | Logs, metrics, and traces. |
| [ClickHouse][contrib-exporters] | Contrib, `clickhouse` | Experimental | Logs and traces. |
| [Azure Monitor][contrib-exporters] | Contrib, `azure-monitor` | Alpha | Logs. |
| [Geneva][contrib-exporters] | Contrib, `geneva` | Alpha | Logs and traces. |
| [Azure Data Explorer][contrib-exporters] | Contrib, `azure-data-explorer` | In progress | Exporter implementation is not operational. |

## Extensions

| Extension | Build | Status | Coverage |
| --- | --- | --- | --- |
| [Azure Identity and Google Cloud auth][extension-catalog] | Contrib, `azure-identity-auth` or `gcp-auth` | Available, evolving | Bearer token providers for supported integrations. |
| [Azure Key Vault auth][extension-catalog] | Contrib, `azure-key-vault-sasl-auth` or `azure-key-vault-user-pass-auth` | Available, evolving | SASL or basic auth credentials. |
| [Flat-file auth][extension-catalog] | Contrib, `flat-file-api-key-auth`, `flat-file-sasl-auth`, or `flat-file-user-pass-auth` | Available, evolving | API key, SASL, or basic auth credentials. |
| [Kubernetes service account token auth][extension-catalog] | Contrib, `k8s-service-account-token-auth` | Available, evolving | Authorizes incoming bearer tokens. |
| [OAuth 2.0 client auth][extension-catalog] | Contrib, `oauth2-client-auth` | Available, evolving | Client credentials bearer tokens. |
| [Controller monitor][controller-monitor] | Built-in controller extension | Available, evolving | Emits controller metrics and optional internal snapshot logs. |
| [OpAMP agent][opamp] | Built-in controller extension | Early | Remote configuration and health reporting. |

The [development-node catalog][dev-nodes] covers synthetic traffic,
performance measurement, and fault injection.

## Processing languages

| Language | Philosophy and strengths | Tradeoffs in DFE |
| --- | --- | --- |
| [OpenTelemetry Processing Language (OPL)][opl] | Signal-aware pipelines compose filtering, transformation, and routing in pipe order. The language is designed around OpenTelemetry's data model and Arrow execution. | The language, tooling, and ecosystem are evolving. Check operators and signal behavior before depending on them. |
| [OpenTelemetry Transformation Language (OTTL)][ottl] | Collector users can express edits with familiar functions and per-statement conditions. Its Collector ecosystem has many functions and examples. | DFE currently implements literal assignments to log fields. Existing Collector statements may use unsupported functions, conditions, or signal contexts. |
| [Kusto Query Language (KQL)][kql] | Tabular, pipe-oriented queries are familiar for filtering and shaping data. | DFE implements a subset against specific telemetry shapes. The Transform and Recordset KQL processors have different input contracts. |

The [Transform processor][transform] documents the supported syntax and
limits. See the [OPL and OTTL comparison][opl-ottl] for examples of how
the two languages express the same operation.

[config]: ../rust/otap-dataflow/docs/configuration.md
[contrib]: ../rust/otap-dataflow/crates/contrib-nodes/README.md
[contrib-exporters]: ../rust/otap-dataflow/crates/contrib-nodes/README.md#exporters
[contrib-processors]: ../rust/otap-dataflow/crates/contrib-nodes/README.md#processors
[contrib-receivers]: ../rust/otap-dataflow/crates/contrib-nodes/README.md#receivers
[controller-monitor]:
    ../rust/otap-dataflow/crates/controller/src/controller_monitor.rs
[core]: ../rust/otap-dataflow/crates/core-nodes/README.md
[core-exporters]: ../rust/otap-dataflow/crates/core-nodes/README.md#exporters
[core-processors]: ../rust/otap-dataflow/crates/core-nodes/README.md#processors
[core-receivers]: ../rust/otap-dataflow/crates/core-nodes/README.md#receivers
[dev-nodes]: ../rust/otap-dataflow/crates/dev-nodes/README.md
[extension-catalog]: ../rust/otap-dataflow/crates/contrib-extensions/README.md
[filelog]: ../rust/otap-dataflow/docs/filelog-receiver.md
[kql]: https://learn.microsoft.com/en-us/kusto/query/?view=microsoft-fabric
[log-parser]:
    ../rust/otap-dataflow/crates/core-nodes/src/processors/log_parser_processor/README.md
[opamp]: ../rust/otap-dataflow/docs/opamp.md
[opl]:
    ../rust/otap-dataflow/crates/query-engine-languages/docs/opl-user-guide/src/introduction.md
[opl-ottl]:
    ../rust/otap-dataflow/crates/query-engine-languages/docs/opl-user-guide/src/ottl_comparison.md
[ottl]: https://github.com/open-telemetry/opentelemetry-collector-contrib/tree/main/pkg/ottl
[prometheus]: ../rust/otap-dataflow/configs/prometheus-otlp.yaml
[topics]: ../rust/otap-dataflow/docs/topic-architecture.md
[transform]:
    ../rust/otap-dataflow/crates/core-nodes/src/processors/transform_processor/README.md
[wasm]: ../rust/otap-dataflow/crates/wasm-host/README.md
