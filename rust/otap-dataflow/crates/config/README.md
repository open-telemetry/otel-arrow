# Pipeline Engine Config Model

This crate defines the configuration model for a multi-tenant, multi-pipeline observability engine
embeddable within the OpenTelemetry ecosystem.

## Overview

The configuration model is structured in 4 main components, each representing a distinct layer of
the configuration hierarchy:

- **EngineConfig**: The root configuration, containing global engine settings and all tenants.
- **TenantConfig**: Represents an individual tenant, including its own settings and pipelines.
- **PipelineConfig**: Describes a pipeline as a hyper-DAG of interconnected nodes, with
  pipeline-level settings.
- **NodeConfig**: Defines a node (receiver, processor, exporter, or connector) and its output ports,
  which represent hyper-edges to downstream nodes.

Each of these components is **directly addressable**, making it straightforward to manipulate and
retrieve configuration fragments.

## Design Philosophy

This configuration model is intentionally simple and self-contained:

- **No references, inheritance, or overwriting:**  
  The model does not support referencing other config objects, inheritance, or any kind of
  overwriting.
- **No templates or placeholders:**  
  There are no templates or placeholder mechanisms—each configuration is self-contained and
  explicit.
- **Easy to interpret:**  
  The configuration is designed to be unambiguous and easy for both humans and machines to parse and
  validate.

The goal is to make the configuration as **predictable and transparent** as possible, reducing
cognitive load and the risk of hidden or implicit behaviors.

> **Advanced Configuration Layer**  
> Support for advanced concepts such as references, inheritance, and templating is planned for a
> dedicated configuration layer aimed at human authors.  
> A translator/resolver will assemble these advanced, versionable configuration files into this more
> self-contained, straightforward model for engine consumption.

This configuration model is intended to be easily integrable with systems like **Kubernetes** as
well as other environments.

## Compatibility & Translation

This configuration model is intended to be a **superset of the current OTEL Go Collector
configuration**. It introduces advanced concepts—such as multi-tenancy and configurable dispatch
strategies—that are not present in the upstream Collector.

A translation mechanism will be developed to **automatically convert any OTEL Collector YAML
configuration file into this new config model**.  
Some aspects of the OTEL Collector, such as the extension mechanism, are still under consideration
and have not yet been fully mapped in the new model.

## Config Validation & Error Reporting

A **strict validation stage** will be developed to ensure the stability and robustness of the
engine. The validator will perform comprehensive checks on configuration files before they are
accepted by the engine.

Instead of stopping at the first error, the parser and validator will attempt to **collect all
configuration errors in a single run**, providing detailed and informative context for each issue.
This approach makes debugging and troubleshooting significantly easier, allowing users to resolve
multiple issues at once and increasing overall productivity.

## Roadmap

- An API will be introduced to allow for **dynamic management** of configuration:

  - Add, update, get, and delete tenants
  - Add, update, get, and delete pipelines within tenants
  - Add, update, get, and delete nodes within pipelines

- **Transactional updates:**  
  Updates can target multiple nodes as part of a single, consistent transaction.  
  A consistent transaction is an operation where, once applied, the pipeline remains in a valid and
  operational state. The **unit of operation is the pipeline**: transactional updates are atomic at
  the pipeline level.

- Every component of the configuration model will be addressable and manageable via this API.

- An **authorization framework** will be introduced to manage access and permissions at the level of
  tenants, pipelines, and nodes.
