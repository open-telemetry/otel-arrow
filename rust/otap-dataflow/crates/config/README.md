# Pipeline Engine Config Model

This crate defines the configuration model for a multi-pipeline-group,
multi-pipeline observability engine embeddable within the OpenTelemetry
ecosystem.

## Overview

The configuration model is structured in several main components, each
representing a distinct layer of the configuration hierarchy:

- **EngineConfig**: The root configuration, containing global engine settings
  and all pipeline groups.
- **PipelineGroupConfig**: Represents an individual pipeline group, including
  its own settings and pipelines.
- **PipelineConfig**: Describes a pipeline as a directed-acyclic-hypergraph of
  interconnected nodes, with pipeline-level settings and service configuration.
- **NodeConfig**: Defines a node (receiver, processor, exporter, or connector)
  and its output ports, which represent hyper-edges to downstream nodes.
- **ServiceConfig**: Pipeline-level service configuration, including telemetry
  settings for observing the pipeline itself.

Each of these components is **directly addressable**, making it straightforward
to manipulate and retrieve configuration fragments.

The concept of a directed-acyclic-hypergraph (or hyperDAG) is used to extend the
expressiveness beyond what is currently possible with the existing collector
configuration. For example, we would like to be able to express that a processor
can broadcast the same message to multiple destinations, or that a processor can
load balance a message to one of the destinations connected to the hyper-edge.

## Design Philosophy

This configuration model is intentionally simple and self-contained:

- **No references, inheritance, or overwriting:** The model does not support
  referencing other config objects, inheritance, or any kind of overwriting.
- **No templates or placeholders:** There are no templates or placeholder
  mechanisms-each configuration is self-contained and explicit.
- **Easy to interpret:** The configuration is designed to be unambiguous and
  easy for both humans and machines to parse and validate.

The goal is to make the configuration as **predictable and transparent** as
possible, reducing cognitive load and the risk of hidden or implicit behaviors.

> **Advanced Configuration Layer** Support for advanced concepts such as
> references, inheritance, and templating is planned for a dedicated
> configuration layer aimed at human authors. A translator/resolver will
> assemble these advanced, versionable configuration files into this more
> self-contained, straightforward model for engine consumption.

This configuration model is intended to be easily integrable with systems like
**Kubernetes** as well as other environments.

## Service-Level Telemetry

Each pipeline can be configured with its own **telemetry settings** to observe
the pipeline's internal behavior and performance. This allows for fine-grained
monitoring and debugging of individual pipelines.

### Telemetry Configuration

The telemetry configuration includes:

- **Metrics**: OpenTelemetry metrics for pipeline observability
  - **Readers**: Periodic or pull-based metric readers
  - **Exporters**: Console, OTLP (gRPC/HTTP), or custom exporters
  - **Views**: Metric aggregation and transformation rules
  - **Temporality**: Delta or cumulative aggregation
- **Logs**: Internal logging configuration
  - **Level**: Off, Debug, Info, Warn, or Error
  - **Processors**: Batch log processors with configurable exporters
  - **Exporters**: Console, OTLP (gRPC/HTTP) for logs
  - Integrates with Rust's `tracing` ecosystem
  - Supports `RUST_LOG` environment variable for fine-grained control
- **Resource Attributes**: Key-value pairs describing the service
  - Supports string, boolean, integer (i64), float (f64), and array types
  - Common attributes: `service.name`, `service.version`, `process.pid`, etc.

### Example Configuration

```yaml
service:
  telemetry:
    resource:
      service.name: "my-pipeline"
      service.version: "1.0.0"
      process.pid: 12345
      deployment.environment: "production"
    metrics:
      readers:
        - periodic:
            exporter:
              otlp:
                endpoint: "http://localhost:4318"
                protocol: "grpc/protobuf"
      views:
        - selector:
            instrument_name: "logs.produced"
          stream:
            name: "otlp.logs.produced.count"
            description: "Count of logs produced"
    logs:
      level: "info"
      processors:
        - batch:
            exporter:
              otlp:
                endpoint: "http://localhost:4318"
                protocol: "grpc/protobuf"
```

### Supported Exporters

#### Metric Exporters

- **Console**: Prints metrics to stdout (useful for debugging)
- **OTLP**: OpenTelemetry Protocol exporters
  - **grpc/protobuf**: Binary protocol over gRPC
  - **http/protobuf**: Binary protobuf over HTTP
  - **http/json**: JSON over HTTP

#### Log Exporters

- **Console**: Prints logs to stdout with structured formatting
- **OTLP**: OpenTelemetry Protocol exporters for logs
  - **grpc/protobuf**: Binary protocol over gRPC
  - **http/protobuf**: Binary protobuf over HTTP
  - **http/json**: JSON over HTTP

### Log Configuration

The logging system integrates with Rust's `tracing` ecosystem:

- **Log Levels**: Control verbosity (`off`, `debug`, `info`, `warn`, `error`)
- **Environment Override**: `RUST_LOG` environment variable takes precedence
  - Example: `RUST_LOG=info,h2=warn,hyper=warn` - info level with silenced HTTP logs
- **Processors**: Batch log processors buffer and export logs efficiently
- **Thread-aware**: Includes thread names and IDs in log output for debugging
- **OpenTelemetry Bridge**: Logs are automatically converted to OpenTelemetry format

### Metric Views

Views allow you to customize how metrics are aggregated and reported:

- **Selector**: Match instruments by name
- **Stream**: Configure the output metric stream
  - Rename instruments
  - Add or modify descriptions

## Compatibility & Translation

This configuration model is intended to be a **superset of the current OTEL Go
Collector configuration**. It introduces advanced concepts, such as
multi-tenancy (based on pipeline group) and configurable dispatch strategies,
that are not present in the upstream Collector.

A translation mechanism will be developed to **automatically convert any OTEL
Collector YAML configuration file into this new config model**. Some aspects of
the OTEL Collector, such as the extension mechanism, are still under
consideration and have not yet been fully mapped in the new model.

## Config Validation & Error Reporting

A **strict validation stage** will be developed to ensure the stability and
robustness of the engine. The validator will perform comprehensive checks on
configuration files before they are accepted by the engine.

Instead of stopping at the first error, the parser and validator will attempt to
**collect all configuration errors in a single run**, providing detailed and
informative context for each issue. This approach makes debugging and
troubleshooting significantly easier, allowing users to resolve multiple issues
at once and increasing overall productivity.

## Roadmap

- An API will be introduced to allow for **dynamic management** of
  configuration:

  - Add, update, get, and delete pipeline groups
  - Add, update, get, and delete pipelines within pipeline groups
  - Add, update, get, and delete nodes within pipelines

- **Transactional updates:** Updates can target multiple nodes as part of a
  single, consistent transaction. A consistent transaction is an operation
  where, once applied, the pipeline remains in a valid and operational state.
  The **unit of operation is the pipeline**: transactional updates are atomic at
  the pipeline level.

- Every component of the configuration model will be addressable and manageable
  via this API.

- An **authorization framework** will be introduced to manage access and
  permissions at the level of pipeline groups, pipelines, and potentially nodes.
