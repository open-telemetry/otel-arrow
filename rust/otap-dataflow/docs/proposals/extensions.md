# Proposal: Extension System for the OTel Dataflow Engine

## Summary

This proposal introduces a **capability-based extension system** for the **OTel Dataflow Engine**.

Extensions allow receivers, processors, and exporters to access **non-pdata functionality**, such as:

* authentication
* policy lookup
* token sources
* enrichment services
* connection management

The design aims to provide:

* **high performance** compatible with the engine’s thread-per-core architecture
* **clear integration** into the existing configuration model
* **good user and developer experience**
* **flexible deployment scopes**
* **future extensibility**, including hierarchical placement and WASM-based extensions

The system is introduced incrementally in **three phases**.

---

# Goals

The extension system should:

* allow nodes to access **capabilities** such as authentication or policy lookup
* allow **multiple implementations** of the same capability
* allow **multiple configured instances** of providers in the same pipeline
* integrate cleanly into the **existing configuration model**
* provide a clear **user experience** for declaring and binding extensions
* provide a clear **developer experience** for implementing extensions
* preserve the engine’s performance model:

    * thread-per-core execution
    * minimal synchronization
    * local hot-path access
* support future hierarchical placement at:

    * pipeline
    * group
    * global
    * potentially distributed
* allow extensions to run **background tasks**

---

# Non-Goals

The system is not intended to:

* replace pdata processing nodes
* introduce dynamic runtime loading in phase 1
* allow arbitrary plugin execution initially

---

# Core Concepts

## Capability

A **capability** is a typed interface that a node can use.

Examples:

```text
AuthCheckV1
AuthCheckV2
BearerTokenSource
PolicyLookup
```

Capabilities are **known by the engine version** and validated during configuration loading.

This ensures:

* deterministic configuration validation
* compatibility guarantees
* clear error reporting

---

## Provider

A **provider** is an extension implementation exposing one or more capabilities.

Examples:

| Provider              | Capabilities                       |
| --------------------- | ---------------------------------- |
| OAuth2 auth provider  | `AuthCheckV1`, `BearerTokenSource` |
| Static token provider | `AuthCheckV1`                      |
| Policy service client | `PolicyLookup`                     |

---

## Provider Instance

A **provider instance** is a configured extension declared in an `extensions` section.

Example:

```yaml
extensions:
  auth_main:
    kind: auth
    impl: oauth2_cc
    scope: pipeline
    config:
      endpoint: https://auth.example.com
```

Multiple instances may exist using different configurations or different implementations.

---

## Capability Binding

Nodes bind capabilities to provider instances.

Example:

```yaml
nodes:
  otlp_recv1:
    type: receiver:otlp
    capabilities:
      AuthCheckV1: auth_main
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4327"
          wait_for_result: true
```

This creates an explicit mapping:

```text
node capability -> provider instance
```

This improves:

* configuration clarity
* validation
* flexibility

---

# Configuration Integration

The extension system integrates directly into the existing configuration hierarchy.

For phase 1, extensions are declared at the **pipeline level** and consumed by nodes within that pipeline.

Example:

```yaml
version: otel_dataflow/v1

groups:
  ingest_group:
    pipelines:
      ingest:
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: 0
                  end: 3

        extensions:
          auth_main:
            kind: auth
            impl: oauth2_cc
            scope: pipeline
            config:
              endpoint: https://auth.example.com
              client_id: ingest
              client_secret: ${AUTH_SECRET}

          auth_local:
            kind: auth
            impl: static_token
            scope: shard
            config:
              token: ${LOCAL_AUTH_TOKEN}

        nodes:
          otlp_recv1:
            type: receiver:otlp
            capabilities:
              AuthCheckV1: auth_main
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4327"
                  wait_for_result: true

          otlp_recv2:
            type: receiver:otlp
            capabilities:
              AuthCheckV1: auth_local
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4337"
                  wait_for_result: true

          logs_exporter:
            type: exporter:noop
            config: null

        connections:
          - from: otlp_recv1
            to: logs_exporter
          - from: otlp_recv2
            to: logs_exporter
```

This model keeps extension usage explicit and consistent with the existing `groups -> pipelines -> nodes` structure.

---

# User Experience

From the user perspective, the extension system should be:

* explicit
* predictable
* discoverable

A user workflow is expected to look like this:

### 1. Declare extension instances in the pipeline

```yaml
extensions:
  auth_main:
    kind: auth
    impl: oauth2_cc
    scope: pipeline
    config:
      endpoint: https://auth.example.com
```

### 2. Bind node capabilities to those instances

```yaml
nodes:
  otlp_recv1:
    type: receiver:otlp
    capabilities:
      AuthCheckV1: auth_main
    config:
      protocols:
        grpc:
          listening_addr: "127.0.0.1:4327"
```

This makes it easy to answer:

* which extensions exist in this pipeline?
* which node uses which capability?
* which implementation is behind a given provider instance?
* how broadly is that provider shared?

---

# Developer Extension Experience

Developers implementing extensions should have a simple and structured model.

An extension implementation should define:

* provider kind
* provider implementation name
* supported capabilities
* supported scopes
* configuration schema
* runtime behavior

Extensions may also run background tasks.

Example:

```text
OAuth2 Auth Provider
├─ capability: AuthCheckV1
├─ capability: BearerTokenSource
└─ background task: token refresh
```

Extension developers should focus on the provider logic rather than on runtime plumbing or custom discovery mechanisms.

---

# Capability Discovery and Documentation

A key objective is to make extensions **self-descriptive and discoverable**.

The engine should be able to generate a catalog of:

* available extensions
* supported capabilities
* supported scopes
* configuration schema summary

This improves:

* documentation
* discoverability
* tooling
* validation UX

## Registration and Discovery

For node implementation discovery, the engine already relies on the **`distributed_slice` crate**.

The same mechanism can be adopted for extensions.

That would give us:

* compile-time registration
* automatic extension discovery
* a uniform developer experience across nodes and extensions
* the ability to build an extension/capability catalog automatically

Conceptually:

```text
Extension implementation
        │
        ▼
distributed_slice registration
        │
        ▼
extension registry
        │
        ▼
capability catalog / documentation
```

This is a desirable objective of the extension system, even if the exact generated documentation format can be refined later.

---

# Runtime Architecture

The runtime system consists of:

* provider declarations from configuration
* runtime units instantiated from those declarations
* capability handles used by nodes

```text
Config
   │
   ▼
Provider declarations
   │
   ▼
Runtime planner
   │
   ▼
Provider runtime units
   │
   ▼
Capability handles
   │
   ▼
Nodes
```

Nodes resolve capability handles **during initialization only**.

There is **no registry lookup on the hot path**.

---

# Thread-Per-Core Execution Model

The engine runs one pipeline shard per core.

```text
Core 0   Core 1   Core 2   Core 3
  │        │        │        │
Pipeline  Pipeline  Pipeline  Pipeline
Shard     Shard     Shard     Shard
```

Providers may be instantiated:

* once per shard
* once per pipeline
* in later phases, once per group
* in later phases, once globally
* potentially in the future as distributed providers shared across processes

---

# Extension Scopes

## Phase 1 Scopes

Phase 1 supports two scopes at the **pipeline declaration level**:

| Scope      | Description                                                    |
| ---------- | -------------------------------------------------------------- |
| `shard`    | one provider runtime unit per pipeline shard                   |
| `pipeline` | one provider runtime unit shared by all shards of the pipeline |

```text
Pipeline
├── shard 0 -> provider runtime unit
├── shard 1 -> provider runtime unit
└── shard 2 -> provider runtime unit
```

vs

```text
Pipeline
   │
   └── shared provider runtime unit
```

## Why both scopes are useful

`shard` is a good fit for:

* hot-path helpers
* local caches
* per-core runtime locality
* avoiding cross-core synchronization

`pipeline` is a good fit for:

* shared refresh coordination
* shared policy distribution
* shared datasets reused across shards

---

# Phase 2 Hierarchical Placement

Phase 2 extends the system so that `extensions` may appear at different levels of the configuration hierarchy.

Candidate placements:

* top-level
* group-level
* pipeline-level

A simple model is to derive the sharing domain from declaration placement:

| Declaration placement                            | Sharing domain    |
| ------------------------------------------------ | ----------------- |
| top-level `extensions`                           | global            |
| `groups.<group>.extensions`                      | group             |
| `groups.<group>.pipelines.<pipeline>.extensions` | pipeline or shard |

Under this model:

* top-level declarations are implicitly global
* group-level declarations are implicitly group-scoped
* pipeline-level declarations may choose between `pipeline` and `shard`

This keeps the configuration simpler than allowing every declaration to choose every possible scope.

---

# Future Scope: Distributed

In phase 2 or later, we may consider a **distributed** scope, or a similar concept, to represent providers shared across **multiple engine processes**.

Example use cases:

* distributed policy store
* distributed quota manager
* shared control-plane state
* process-independent configuration service

This scope is conceptually different from `global` because `global` is process-local, whereas `distributed` implies sharing beyond a single engine process.

---

# Scope Examples

## `shard`

Good fit for:

* bearer token snapshot cache used directly on the hot path
* local rate limiter
* local metadata cache
* runtime-local helper task

Benefits:

* maximum locality
* no cross-core synchronization
* predictable latency

## `pipeline`

Good fit for:

* token refresh coordinator
* pipeline-wide policy provider
* shared enrichment dictionary
* shared connection/session manager

Benefits:

* avoids duplicated background work
* still bounded to one pipeline

## `group`

Good fit for:

* shared auth configuration for related pipelines
* shared service discovery
* shared reference data for one group

## `global`

Good fit for:

* license manager
* feature flag provider
* trust anchor manager
* engine-wide policy catalog

## `distributed`

Good fit for:

* distributed policy provider
* distributed quota management
* control-plane coordinated state

---

# Provider Background Tasks

Providers may run background tasks on the runtime.

Example:

```text
Receiver
   │
   ▼
AuthCheck capability
   │
   ▼
local token snapshot
   │
   ▼
background refresh task
```

The intended performance model is:

* local fast-path reads when possible
* background refresh / coordination off the hot path
* minimal synchronization

---

# Validation

Configuration validation should ensure:

* the capability trait exists
* the provider instance exists
* the provider exposes the requested capability
* the implementation supports the requested scope

Example error:

```text
extension `auth_main` uses implementation `oauth2_cc`,
which only supports `shard` scope,
but `pipeline` scope was requested
```

For phase 2, validation should also ensure the declaration placement and requested scope are compatible.

---

# Example in the Current Configuration Style

Below is a more realistic example aligned with the current OTel Dataflow Engine configuration structure.

```yaml
version: otel_dataflow/v1

policies:
  channel_capacity:
    control:
      node: 256
      pipeline: 256
    pdata: 128

engine:
  http_admin:
    bind_address: 127.0.0.1:8085

groups:
  continuous_benchmark:
    pipelines:
      sut:
        policies:
          resources:
            core_allocation:
              type: core_set
              set:
                - start: 0
                  end: 1

        extensions:
          auth_shared:
            kind: auth
            impl: oauth2_cc
            scope: pipeline
            config:
              endpoint: https://auth.example.com
              client_id: sut
              client_secret: ${AUTH_SECRET}

          auth_per_shard:
            kind: auth
            impl: static_token
            scope: shard
            config:
              token: ${STATIC_TOKEN}

        nodes:
          otlp_recv1:
            type: receiver:otlp
            capabilities:
              AuthCheckV1: auth_shared
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4327"
                  wait_for_result: true

          otlp_recv2:
            type: receiver:otlp
            capabilities:
              AuthCheckV1: auth_per_shard
            config:
              protocols:
                grpc:
                  listening_addr: "127.0.0.1:4337"
                  wait_for_result: true

          router:
            type: processor:type_router
            outputs: ["logs", "metrics", "traces"]
            config: {}

          logs_exporter:
            type: exporter:noop
            config: null

          metrics_exporter:
            type: exporter:noop
            config: null

          spans_exporter:
            type: exporter:noop
            config: null

        connections:
          - from: otlp_recv1
            to: router
          - from: otlp_recv2
            to: router
          - from: router["logs"]
            to: logs_exporter
          - from: router["metrics"]
            to: metrics_exporter
          - from: router["traces"]
            to: spans_exporter
```

---

# Performance Characteristics

The design preserves the engine’s performance model.

Key properties:

* capability resolution happens **only during initialization**
* nodes use **typed capability handles**
* no global locks on the hot path
* `shard` scope provides full runtime locality
* shared scopes can still expose local read views where useful

The intent is to preserve:

* fast execution
* low memory overhead
* minimal synchronization
* predictable latency

---

# Evolution Plan

## Phase 1 — Basic Extension Support

Features:

* pipeline-level `extensions`
* scopes: `shard`, `pipeline`
* node-level `capabilities` binding
* support for background tasks
* clear validation of capability and scope compatibility

## Phase 2 — Hierarchical Extensions

Adds:

* `extensions` at top-level, group-level, and pipeline-level
* derived sharing semantics from declaration placement
* optional future `distributed` scope or equivalent concept

## Phase 3 — WASM Extensions

Adds:

* WASM runtime for providers
* sandboxed execution
* host capability ABI

This phase builds on the provider/capability model introduced in phases 1 and 2 rather than redefining it.

---

# Benefits

## Performance

Compatible with the engine’s thread-per-core design.

## Flexibility

Supports multiple implementations and multiple configured instances in the same pipeline.

## Evolvability

Provides a phased roadmap from native extensions to hierarchical placement and later WASM support.

## Discoverability

Enables extension and capability documentation to be generated from self-descriptive registrations.

## Configuration Clarity

Fits naturally into the existing `groups / pipelines / nodes` model and keeps extension usage explicit.

---

# Open Questions

Potential discussion topics:

* final terminology for `shard` vs `instance`
* exact semantics and naming of the future distributed scope
* generated documentation format for extension and capability catalogs

---

# Conclusion

This proposal introduces a capability-based extension architecture for the **OTel Dataflow Engine** that aligns with the engine’s high-performance design and current configuration model.

The phased rollout keeps the initial implementation focused while leaving room for hierarchical placement, stronger discoverability, and future WASM-based providers.

The result is a flexible extension mechanism that preserves **performance, clarity, and long-term extensibility**.