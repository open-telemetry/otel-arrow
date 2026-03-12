# Proposal: Extension System for the OTel Dataflow Engine

## Summary

This proposal introduces a **capability-based extension system** for the **OTel Dataflow Engine**.

Extensions allow receivers, processors, and exporters to access **non-pdata functionality**, such as:

* authentication
* token sources
* external data sources (file, database, etc.)
* enrichment services
* identity extraction

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
* allow arbitrary plugin execution initially

## Core Concepts

### Capability

A **capability** is a typed interface that nodes can use.

Examples:

```text
auth_check
dataset_lookup
```

Capabilities are **known by the engine version** and validated during configuration loading.

This ensures:

* deterministic configuration validation
* compatibility guarantees
* clear error reporting

### Extension Provider

An **extension provider** is an extension implementation exposing one or more capabilities.

Example extension providers:

| Extension Provider           | Capabilities   |
| ---------------------------- | -------------- |
| `basic_auth` provider        | auth_check     |
| `bearer_token_auth` provider | auth_check     |
| `oidc_auth` provider         | auth_check     |
| `file_storage` provider      | dataset_lookup |
| `db_storage` provider        | dataset_lookup |

### Extension Instance

An **extension instance** is a configured extension provider declared in the `extensions` section.

Example:

```yaml
extensions:
  auth_main:
    provider: oidc_auth
    scope: pipeline
    config:
      issuer: https://accounts.example.com
```

Multiple instances may exist using different configurations or implementations.

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
node capability → extension instance
```

This approach improves:

* configuration clarity
* validation
* flexibility

## Configuration Integration

The extension system integrates directly into the engine’s configuration hierarchy.

For phase 1, extensions are declared at the **pipeline level** and consumed by nodes within that pipeline.

Example:

```yaml
version: otel_dataflow/v1

groups:
  continuous_benchmark:
    pipelines:
      sut:

        extensions:

          oidc_auth_main:
            provider: oidc_auth
            scope: pipeline
            config:
              issuer: https://accounts.example.com

          local_auth:
            provider: basic_auth
            scope: shard
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

This model keeps extension usage explicit and consistent with the existing `groups -> pipelines -> nodes` structure.

## User Experience

From the user perspective, the extension system should be:

* explicit
* predictable
* discoverable

Users should be able to:

1. Declare extension instances in the configuration
2. Bind node capabilities to those instances
3. Select the appropriate sharing scope

Example workflow:

Define extensions:

```text
extensions:
  oidc_auth_main:
    provider: oidc_auth
```

Bind capabilities:

```text
nodes:
  otlp_recv:
    capabilities:
      auth_check: oidc_auth_main
```

## Developer Extension Experience

Developers implementing extensions should have a **simple integration model**.

An extension provider implementation should define:

* provider name
* supported capabilities
* supported scopes
* configuration schema
* runtime logic

Extensions may also run **background tasks**.

Example:

```text
oidc_auth provider
 ├─ capability: auth_check
 └─ background task: JWKS refresh
```

Extension developers should focus on **provider logic**, not runtime plumbing.

## Capability Discovery and Documentation

A key objective is to make extensions **self-descriptive and discoverable**.

Extension metadata should expose:

| Field             | Description                |
| ----------------- | -------------------------- |
| name              | extension identifier       |
| description       | human-readable description |
| capabilities      | supported capabilities     |
| supported_scopes  | shard/pipeline/etc         |
| documentation_url | usage documentation        |

This enables:

* extension catalog generation
* automatic documentation
* CLI inspection

### Extension Registration

Extensions register themselves using the same mechanism used for node discovery.

The engine already relies on the **`distributed_slice` crate** for node implementation discovery.

The same approach can be adopted for extensions.

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
capability catalog
```

## Runtime Architecture

The runtime system consists of:

* extension declarations (configuration)
* runtime units (instantiated providers)
* capability handles used by nodes

```text
Config
   │
   ▼
Extension declarations
   │
   ▼
Extension Runtime Manager
   │
   ▼
Extension runtime units
   │
   ▼
Capability handles
   │
   ▼
Nodes
```

Nodes resolve capability handles **during initialization only**.

There is **no registry lookup on the hot path**.

## Thread-Per-Core Execution Model

The engine runs **one pipeline shard per core**.

```text
Core 0   Core 1   Core 2   Core 3
  │        │        │        │
Pipeline  Pipeline  Pipeline  Pipeline
Shard     Shard     Shard     Shard
```

Extension providers may be instantiated:

* once per shard
* once per pipeline
* once per group
* once globally

## Extension Scopes

### Phase 1 Scopes

Two scopes are initially supported.

| Scope      | Description                       |
| ---------- | --------------------------------- |
| `shard`    | One instance per pipeline shard   |
| `pipeline` | One instance shared by all shards |

```text
Pipeline
├── shard 0 ─ extension instance
├── shard 1 ─ extension instance
├── shard 2 ─ extension instance
```

vs

```text
Pipeline
   │
   └── shared extension instance
```

### Shard Scope Implementation Advantages

Shard scope allows extension implementations to remain **thread-local**.

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

This avoids cross-core synchronization and preserves the engine’s performance characteristics.

## Go Collector vs OTel Dataflow Engine Extensions

### Go Collector Extension Model

Characteristics:

* extensions are global components
* typically singletons
* accessed indirectly by receivers/processors/exporters

Pros:

* simple architecture
* mature ecosystem

Cons:

* limited scoping flexibility
* less explicit capability contracts
* not optimized for thread-per-core execution

### OTel Dataflow Engine Extension Model

Characteristics:

* **capability-based**
* explicit capability binding in nodes
* multiple extension instances per pipeline
* shard-aware architecture

Pros:

* explicit capability contracts
* optimized for thread-per-core runtime
* flexible extension scoping
* clearer configuration semantics

Cons:

* slightly more configuration
* ecosystem still developing

## Evolution Plan

### Phase 1 — Basic Extension Support

Features:

* pipeline-level extensions
* scopes: `shard`, `pipeline`
* capability binding in nodes
* background tasks supported

### Phase 2 — Hierarchical Extensions

Adds:

* extension declarations at

  * top-level
  * group-level
  * pipeline-level
* derived sharing scopes

Possible future distributed scope.

### Phase 3 — WASM Extensions

Adds:

* WASM runtime for providers
* sandboxed execution
* host capability ABI

## Conclusion

This proposal introduces a **capability-based extension architecture** aligned with the **OTel Dataflow Engine’s high-performance design** and configuration model.

The phased rollout allows the system to evolve gradually while keeping the initial implementation small and focused.

The result is a flexible extension mechanism that preserves **performance, clarity, and long-term extensibility**.

---

# Appendix: Implementation Recommendations

This appendix provides implementation guidance intended to help extension authors build something performant, aligned with the supported scopes, and compatible with the engine architecture.

## 1. Capability handles should be lightweight

Capability handles are expected to be used by nodes on hot paths. They should therefore be:

* cheap to clone
* cheap to pass around during initialization
* inexpensive to call
* free of runtime registry lookups

In practice, the handle should behave like a thin reference to already-resolved extension state.

## 2. Resolve once during initialization

Capability binding should happen during node initialization only.

Nodes should receive typed capability handles once and keep them for their lifetime. They should not perform dynamic capability lookups during runtime.

## 3. Prefer shard-local implementations when the capability is hot-path

If a capability is frequently used during request or batch processing, `shard` scope should generally be preferred.

This gives:

* better cache locality
* more predictable latency
* no cross-core synchronization on the hot path

## 4. `shard` scope should enable lock-free local designs

For `shard` scope, implementations should be designed so they can often rely on thread-local ownership and avoid `Arc<Mutex<...>>`.

Typical building blocks for shard-local implementations include:

* `Rc`
* `RefCell`
* `Cell`

This is one of the main performance advantages of shard-scoped extensions in a thread-per-core engine.

## 5. Use background tasks for slow-path work

Extension providers may need to:

* refresh external state
* reload files
* fetch JWKS data
* query a remote service
* update caches

These activities should happen in background tasks rather than directly on the node hot path.

A typical pattern is:

* node uses a lightweight capability handle
* handle reads local cached state
* background task refreshes or reloads that state asynchronously

## 6. Shared scopes should avoid making every call cross-core

For `pipeline` scope and future broader scopes, implementations should avoid designs where every capability call requires cross-core communication.

A better pattern is usually:

* shared ownership for coordination or refresh
* local read views or local cached snapshots for hot-path usage

## 7. Scope support should be explicit per extension provider

Not all extension providers should support all scopes.

An implementation should declare which scopes it supports. During initialization, the runtime should validate that the requested scope is compatible and emit a clear error otherwise.

## 8. Keep capability surfaces small and focused

Capabilities should expose only what nodes need.

Small capability interfaces are easier to:

* validate
* document
* optimize
* evolve over time

## 9. Extension metadata should stay accurate

Because extension metadata may be used to generate documentation and catalogs, it should remain up to date and include at least:

* description
* supported capabilities
* supported scopes
* documentation URL

## 10. Prefer compatibility with the existing engine model

Extensions should fit naturally into the existing architecture:

* declared in configuration
* discovered through `distributed_slice`
* instantiated by the Extension Runtime Manager
* consumed by receivers/processors/exporters via typed capability handles

This keeps the extension system aligned with the rest of the engine and avoids introducing a separate plugin model.
