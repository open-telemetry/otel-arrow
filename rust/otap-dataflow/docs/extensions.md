# Proposal: Extension System for the OTel Dataflow Engine

## Summary

This proposal introduces a **capability-based extension system** for the **OTel
Dataflow Engine**.

Extensions allow receivers, processors, and exporters to access **non-pdata
functionality**, such as:

* authentication
* identity extraction
* token sources
* external data sources (file, database, etc.)
* enrichment services

The system is introduced incrementally in **three phases**.

## Goals

The extension system should:

* allow nodes to access **capabilities** exposed by extensions
* allow **multiple implementations** of the same capability
* allow **multiple configured instances** of providers in the same pipeline
* integrate cleanly into the **existing configuration model**
* provide a **clear user experience** for configuring extensions
* provide a **clear developer experience** for implementing extensions
* preserve the engine's **performance model**:

  * thread-per-core execution
  * minimal synchronization
  * local hot-path access
* support **future hierarchical scopes** (global, group, pipeline)
* allow extensions to run **background tasks**

## Non-Goals

The system is **not intended to**:

* replace pdata processing nodes
* introduce dynamic runtime loading in phase 1
* allow external extensions to define new capability interfaces/traits

Capability interfaces (traits) are **defined and maintained in the OTel Dataflow
Engine core**. Extensions implement these predefined capabilities but **cannot
introduce new capability interfaces outside the core**.

## Core Concepts

### Capability

A **capability** is a typed interface/trait that nodes can use.

Examples:

```text
auth_check
dataset_lookup
```

Capabilities are **known by the engine version** and validated during
configuration loading.

This ensures:

* deterministic configuration validation
* compatibility guarantees
* clear error reporting

### Extension Provider

An **extension provider** is an extension implementation exposing one or more
capabilities.

Example extension providers:

| Extension Provider           | Capabilities   |
|------------------------------|----------------|
| `basic_auth` provider        | auth_check     |
| `bearer_token_auth` provider | auth_check     |
| `oidc_auth` provider         | auth_check     |
| `file_storage` provider      | dataset_lookup |
| `db_storage` provider        | dataset_lookup |

Different extension providers may implement the same capability with different
logic or external dependencies.

### Extension Instance

An **extension instance** is a configured extension provider declared in the
`extensions` section.

Example:

```yaml
extensions:
  auth_main:
    type: extension:oidc_auth
    config:
      issuer: https://accounts.example.com
```

Multiple instances may exist using different configurations or implementations.

The execution model of an extension (for example **local per core** or **shared
across cores**) is defined by the **extension provider implementation**, not by
the configuration. The placement of the extension declaration in the
configuration hierarchy remains an orthogonal concern handled by the runtime.

### Capability Binding

Nodes bind capabilities to extension instances.

Example:

```yaml
nodes:
  otlp_recv1:
    type: receiver:otlp
    capabilities:
      auth_check: auth_main
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4327"
```

This creates a mapping:

```text
node -> capability -> extension instance
```

This approach improves:

* configuration clarity
* validation
* flexibility

## Configuration Integration

The extension system integrates directly into the engine's configuration
hierarchy.

For phase 1, extensions are declared at the **pipeline level** and consumed by
nodes within that pipeline.

Example:

```yaml
version: otel_dataflow/v1

groups:
  continuous_benchmark:
    pipelines:
      sut:

        extensions:

          oidc_auth_main:
            type: extension:oidc_auth
            config:
              issuer: https://accounts.example.com

          local_auth:
            type: extension:basic_auth
            config:
              file: /etc/auth/tokens.yaml

        nodes:

          otlp_recv1:
            type: receiver:otlp
            capabilities:
              auth_check: oidc_auth_main
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4327"

          otlp_recv2:
            type: receiver:otlp
            capabilities:
              auth_check: local_auth
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4337"
```

This model keeps extension usage explicit and consistent with the existing
`groups -> pipelines -> nodes` structure.

## User Experience

From the user perspective, the extension system should be:

* explicit
* predictable
* discoverable

Users should be able to:

1. Declare extension instances in the configuration
2. Bind node capabilities to those instances

## Developer Extension Experience

Developers implementing extensions should have a **simple integration model**.

An extension provider implementation should define:

* provider name
* supported capabilities
* supported execution model (local or shared)
* configuration schema
* runtime logic

Extensions may also run **background tasks**.

Example:

```text
oidc_auth provider
 |- capability: auth_check
 `- background task: JWKS refresh
```

Extension developers should focus on **provider logic**, not runtime plumbing.

## Capability Discovery and Documentation

A key objective is to make extensions **self-descriptive and discoverable**.

Extension metadata should expose:

| Field                     | Description                |
|---------------------------|----------------------------|
| name                      | extension urn identifier   |
| description               | human-readable description |
| capabilities              | supported capabilities     |
| supported_execution_model | shared/local               |
| documentation_url         | usage documentation        |

This enables:

* extension catalog generation
* automatic documentation
* CLI inspection

### Extension Registration

Extensions register themselves using the same mechanism used for node discovery.

The engine already relies on the **`distributed_slice` crate** for node
implementation discovery.

The same approach can be adopted for extensions.

Conceptually:

```text
Extension implementation
        |
        v
distributed_slice registration
        |
        v
extension registry
        |
        v
capability catalog
```

## Runtime Architecture

The runtime system consists of:

* extension declarations (configuration)
* runtime units (instantiated extension providers)
* capability handles used by nodes

```text
Config
   |
   v
Extension declarations
   |
   v
Extension Runtime Manager
   |
   v
Extension runtime units
   |
   v
Capability handles
   |
   v
Nodes
```

Nodes resolve capability handles **during initialization only**.

There is **no registry lookup on the hot path**.

## Thread-Per-Core Engine Execution Model

The engine runs **one pipeline instance per core**.

```text
Core 0   Core 1   Core 2   Core 3
  |        |        |        |
Pipeline  Pipeline  Pipeline  Pipeline     <- All configured from the same config
Instance  Instance  Instance  Instance
```

The execution model of an extension (local per core or shared across cores) is
determined by the **extension provider implementation**, not by user
configuration.

## Extension Scopes

### Phase 1 - Pipeline Scope

In phase 1, extensions are declared at the **pipeline level** and consumed by
nodes within that pipeline.

Two execution models are supported by extension providers.

| Execution Model | Description                              |
|-----------------|------------------------------------------|
| `local`         | One runtime instance per core            |
| `shared`        | One runtime instance shared across cores |

The supported model is declared by the extension provider implementation.

### Local Execution Model Advantages

Local execution allows extension implementations to remain **thread-local**.

In many cases this allows implementations without:

```text
Arc
Mutex
```

and instead rely on:

```text
Rc
RefCell
Cell
```

This avoids cross-core synchronization and preserves the engine's performance
characteristics.

## Evolution Plan

### Phase 1 - Basic Extension Support

Features:

* pipeline-level extensions
* capability binding in nodes
* background tasks supported

### Phase 2 - Hierarchical Extensions

Adds:

* extension declarations at

  * top/engine-level
  * group-level

Possible future distributed scope.

### Phase 3 - WASM Extensions

Adds:

* WASM runtime for providers
* sandboxed execution
* host capability ABI

## Conclusion

This proposal introduces a **capability-based extension architecture** aligned
with the **OTel Dataflow Engine's high-performance design** and configuration
model.

The phased rollout allows the system to evolve gradually while keeping the
initial implementation small and focused.

The result is a flexible extension mechanism that preserves **performance,
clarity, and long-term extensibility**.

---

## Appendix 1: Go Collector vs OTel Dataflow Engine Extensions

### Go Collector Extension Model

Characteristics:

* extensions are global components
* accessed indirectly by receivers/processors/exporters

Pros:

* simple architecture
* mature ecosystem

Cons:

* limited scoping flexibility
* not optimized for thread-per-core execution

### OTel Dataflow Engine Extension Model

Characteristics:

* **capability-based**
* explicit capability binding in nodes
* thread-per-core and NUMA-aware architecture

Pros:

* explicit capability contracts
* optimized for thread-per-core runtime
* flexible extension scoping
* clearer configuration semantics

Cons:

* slightly more configuration
* ecosystem still developing

## Appendix 2: Implementation Recommendations

This appendix provides implementation guidance intended to help extension
authors build something performant, aligned with the supported scopes, and
compatible with the engine architecture.

### 1. Capability handles should be lightweight

Capability handles are expected to be used by nodes on hot paths. They should
therefore be:

* cheap to clone
* cheap to pass around during initialization
* inexpensive to call
* free of runtime registry lookups

In practice, the handle should behave like a thin reference to already-resolved
extension state.

They can also include local cached state for hot-path usage, with background
tasks responsible for refreshing that state as needed.

### 2. Resolve once during initialization

Capability binding should happen during node initialization only.

Nodes should receive typed capability handles once and keep them for their
lifetime. They should not perform dynamic capability lookups during runtime.

### 3. Prefer local implementations when the capability is hot-path

If a capability is frequently used during request or batch processing, `local`
execution model should generally be preferred.

This gives:

* better cache locality
* more predictable latency
* no cross-core synchronization on the hot path

### 4. `local` execution model should enable lock-free local designs

For `local` execution model, implementations should be designed so they can
often rely on thread-local ownership and avoid `Arc<Mutex<...>>`.

Typical building blocks for local implementations include:

* `Rc`
* `RefCell`
* `Cell`

This is one of the main performance advantages of local extensions in a
thread-per-core engine.

### 5. Use background tasks for slow-path work

Extension providers may need to:

* refresh external state
* reload files
* fetch JWKS data
* query a remote service
* update caches

These activities should happen in background tasks rather than directly on the
node hot path.

A typical pattern is:

* node uses a lightweight capability handle
* handle reads local cached state
* background task refreshes or reloads that state asynchronously

### 6. Shared scopes should avoid making every call cross-core

For non-local `pipeline` scope and future broader scopes, implementations should
avoid designs where every capability call requires cross-core communication.

A better pattern is usually:

* shared ownership for coordination or refresh
* local read views or local cached snapshots for hot-path usage

### 7. Keep capability surfaces small and focused

Capabilities should expose only what nodes need.

Small capability interfaces are easier to:

* validate
* document
* optimize
* evolve over time

### 8. Extension metadata should stay accurate

Because extension metadata may be used to generate documentation and catalogs,
it should remain up to date and include at least:

* description
* supported capabilities
* supported scopes
* documentation URL

Note: We probably want to enforce this with a macro to register extensions,
which would require metadata fields to be provided at compile time.
