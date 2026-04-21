# Extension System Architecture

## Overview

This document describes the proposed Phase 1 architecture
for the extension system in the OTAP dataflow engine,
building on the [extension system proposal](
extension-requirements.md) which establishes the vision, goals, and
phased rollout plan.

A working proof of concept is available on the
[PoC branch](https://github.com/gouslu/otel-arrow/tree/gouslu/extension-system-p1-local-shared).

### How this document relates to the proposal

The proposal defines *what* the extension system should do
and *why*. This document describes *how* each requirement
is addressed in the Phase 1 implementation:

| Proposal Requirement | Phase 1 Approach |
| --- | --- |
| Capability-based access | `#[capability]` proc macro generates typed traits; consumers resolve via `require_local()` / `require_shared()` |
| Multiple implementations of same capability | `CapabilityRegistry` keyed by `(extension_name, TypeId)` -- different extensions can provide the same capability |
| Multiple configured instances | `extensions:` section in YAML, each with a unique name; nodes bind by name in `capabilities:` |
| Existing config model integration | Extensions are siblings to `nodes` in the pipeline config hierarchy |
| Preserve performance model (thread-per-core) | Local extensions use `Rc` (no locks); shared extensions use `Clone + Send` with `Arc`-wrapped state |
| Background tasks | Active extensions get their own event loop via `Extension::start()` |
| Explicit capability binding | Nodes declare `capabilities: { name: extension_instance }` -- no implicit discovery |
| No hot-path registry lookup | Capabilities resolved once at factory time; nodes hold typed handles for their lifetime |
| Future hierarchical scopes | `CapabilityRegistry` and `resolve_bindings()` are scope-agnostic by design |

In addition to implementing the proposal's requirements,
this design introduces three refinements beyond what the
proposal envisioned:

- **Pipeline-scoped shared is per-core.** The proposal
  defines shared extensions as "one runtime instance
  shared across cores." In Phase 1, both local and shared
  pipeline-scoped extensions are instantiated **per
  pipeline instance** (i.e., per core). The local/shared
  distinction at pipeline scope is about type constraints
  (`!Send` vs `Send + Clone`), not about cross-core
  sharing. This follows a consistent principle: **an
  extension's sharing boundary is determined by the scope
  it is declared in**, not by its execution model.
  Pipeline-scoped extensions are per pipeline instance,
  group-scoped extensions are shared across the group,
  and engine-scoped extensions are shared globally.
  The execution model (local vs shared) determines only
  the type constraints imposed on the implementation.
- **Active/Passive lifecycle distinction.** The proposal
  describes extensions with background tasks, but does not
  distinguish extensions that only provide capabilities
  without running a task. The Active/Passive model (via
  `Active<E>` / `Passive<E>` wrappers) makes this
  explicit at the type level -- passive extensions skip
  task spawning, control channels, and shutdown messaging
  entirely, with zero runtime overhead.
- **Shared-to-local transparent fallback.** The proposal
  treats local and shared as separate execution models.
  This design adds automatic fallback: a shared-only
  extension can transparently serve local consumers via
  a `SharedAsLocal` adapter, without the extension author
  writing any additional code. This means most extensions
  only need a shared implementation -- local consumers
  are served automatically. Extension authors who need
  the lock-free performance of a local-only implementation
  (e.g., `Rc<RefCell>` instead of `Arc<RwLock>`) can
  still opt in by providing a dedicated local variant.

### Ownership and cloning model

Local capabilities return `Rc<dyn local::Trait>` -- all
local consumers share the same instance via reference
counting. No cloning, no locks.

Shared capabilities return `Box<dyn shared::Trait>` --
each consumer gets an independent clone. For shared
extensions to share mutable state across clones (e.g.,
token senders, connection pools), fields should be wrapped
in `Arc`, similar to how `tokio`, `axum`, and `reqwest`
handle shared state:

```rust
#[derive(Clone)]
struct MyExtension {
    // Shared mutable state -- Arc ensures clones
    // see the same data
    token_sender: Arc<watch::Sender<Option<BearerToken>>>,
    credential: Arc<dyn TokenCredential>,
    // Plain data -- cloned independently per consumer
    scope: String,
}
```

## What Are Extensions?

Extensions are standalone pipeline components that provide
**shared, cross-cutting capabilities** -- such as
authentication, storage etc. -- to
data-path nodes (receivers, processors, exporters). They
are configured as siblings to `nodes`, not as nodes
themselves, and they never touch pipeline data directly.

## Architecture Overview

```text
+----------------------------------------------------------+
|                     Pipeline Engine                      |
|                                                          |
|  +-------------------+  +-------------------+            |
|  | Extension A       |  | Extension B       |  ...       |
|  | Active(auth)      |  | Passive(kv store) |            |
|  | local + shared    |  | shared only       |            |
|  | lifecycle         |  | no task spawned   |            |
|  +---------+---------+  +---------+---------+            |
|            | #[capability] proc macro                    |
|            | + extension_capabilities!() macro           |
|            v                                             |
|  +----------------------------+                          |
|  |    CapabilityRegistry      |  (built once per         |
|  |  local_handles HashMap     |   pipeline)              |
|  |  shared_handles HashMap    |                          |
|  +----+-----------------+-----+                          |
|       | resolve_bindings|                                |
|       v                 v                                |
|  +-----------+  +-----------+                            |
|  | Receiver  |  | Exporter  |                            |
|  | require   |  | require   |                            |
|  | _local()  |  | _shared() |                            |
|  | -> Rc<T>  |  | -> Box<T> |                            |
|  +-----------+  +-----------+                            |
|                                                          |
|  Local consumers get Rc<dyn local::Trait>                |
|  Shared consumers get Box<dyn shared::Trait> (Send)      |
+----------------------------------------------------------+
```

## Key Design Decisions

1. **Extensions start first, shut down last.** Active
   extensions are spawned before data-path nodes. At
   shutdown, extensions terminate only after all data-path
   nodes have drained. Passive extensions (no lifecycle)
   skip spawning entirely.

2. **PData-free.** Extensions are completely decoupled from
   the pipeline data type. They use `ExtensionControlMsg`
   through a dedicated control channel.

3. **Active vs Passive.** Extensions signal their lifecycle
   intent at build time via `Active(ext)` or `Passive(ext)`
   newtype wrappers. Active extensions get a task and control
   channel. Passive extensions only provide capabilities --
   no task is spawned, no control channel is allocated, no
   messages are sent. This is enforced at the type level:
   `Active<E>` requires `E: Extension`, `Passive<E>` does
   not.

4. **Local/Shared split.** Extensions support both local
   (`!Send`, `Rc`-based) and shared (`Send`, `Clone`-based)
   variants. A single extension can provide one or both:

   - **Shared-only (with local fallback):**
     `with_shared(Active(ext))`
     -- the shared type serves both local and shared consumers
     via registry fallback. This is the most common pattern.
   - **Local-only:** `with_local(Active(Rc::new(ext)))` --
     only local consumers can use this extension. Shared
     consumers (`require_shared()`) will get a config error.
     Use this when the extension is inherently `!Send`.
   - **Dual-type:** `with_local(Active(Rc::new(l)))` +
     `with_shared(Active(s))` -- separate types with
     independent lifecycles. The builder enforces different
     `TypeId`s via a runtime assertion.
   - **Passive:** `with_shared(Passive(ext))` -- no lifecycle,
     capabilities only.

5. **Type-safe capability resolution.** Consumers call
   `capabilities.require_local::<BearerTokenProvider>()`
   (returns `Rc<dyn local::BearerTokenProvider>`) or
   `capabilities.require_shared::<KeyValueStore>()`
   (returns `Box<dyn shared::KeyValueStore>`, which is `Send`).
   The zero-sized registration struct carries associated
   types (`Local` and `Shared`) that map to the correct
   trait object variants. Sealing via `ExtensionCapability`
   ensures only engine-defined capabilities are accepted
   at compile time. Local fallback from shared extensions is
   pre-populated at build time via `SharedAsLocal` adapters.

6. **`#[capability]` proc macro.** Each capability is
   defined via a single `#[capability]` attribute on a
   trait definition. The macro generates: `local::` and
   `shared::` trait variants, a `SharedAsLocal` adapter
   for transparent fallback, sealed trait impls, a
   zero-sized registration struct, a `KNOWN_CAPABILITIES`
   link-time entry, and type-erased coercion functions.
   Consumers use trait objects directly. Shared
   data types (e.g., `BearerToken`, `Secret`) are
   hand-written alongside the macro invocation.

## Module Layout

```text
engine/src/
  lib.rs                    -> ExtensionFactory, engine build logic
  extension.rs              -> ExtensionWrapper, builder, Active, Passive,
                              ControlChannel, EffectHandler, provider traits
  capability/
    mod.rs                  -> module root
    registry.rs             -> CapabilityRegistry, Capabilities,
                              extension_capabilities! macro, Error type
    bearer_token_provider.rs -> BearerToken, Secret,
                              #[capability] macro invocation
    key_value_store.rs      -> #[capability] macro invocation

  local/
    extension.rs            -> Extension trait (!Send, Rc<Self>)
    capability.rs           -> re-exports + Sealed trait
    exporter.rs, receiver.rs, processor.rs  (unchanged)

  shared/
    extension.rs            -> Extension trait (Send, Box<Self>)
    capability.rs           -> re-exports + Sealed trait
    exporter.rs, receiver.rs, processor.rs  (unchanged)
```

### Import Paths

Extension authors (local):

```rust
use otap_df_engine::local::capability::BearerTokenProvider;
use otap_df_engine::local::extension::Extension;
```

Extension authors (shared):

```rust
use otap_df_engine::shared::capability::BearerTokenProvider;
use otap_df_engine::shared::extension::Extension;
```

Consumers (in factories):

```rust
// Local consumer -- returns Rc<dyn local::BearerTokenProvider>
use otap_df_engine::capability::bearer_token_provider::BearerTokenProvider;
let auth = capabilities
    .require_local::<BearerTokenProvider>()?;

// Shared consumer -- returns Box<dyn shared::KeyValueStore>
use otap_df_engine::capability::key_value_store::KeyValueStore;
let kv = capabilities
    .require_shared::<KeyValueStore>()?;
```

### Dependency Flow

```text
capability/registry.rs        -> CapabilityRegistry, Capabilities, Error type
capability/bearer_token_provider.rs -> types + #[capability] macro
    ^                                    (generates local/shared traits, adapter,
                                          sealed impls, registration, coercion)
local::capability   -> re-exports + Sealed trait
shared::capability  -> re-exports + Sealed trait
```

All arrows point one way. No circular dependencies.

## Core Types

### Active and Passive Wrappers

Extensions signal their lifecycle intent at the builder call
site using newtype wrappers:

```rust
/// Active -- has an event loop, gets a task + control channel.
pub struct Active<E>(pub E);

/// Passive -- capabilities only, no task, no control channel.
pub struct Passive<E>(pub E);
```

These implement sealed `SharedProvider` / `LocalProvider`
traits that decompose the wrapped value into type-erased
components:

- `Active<E>` where `E: shared::Extension + Clone + Send` ->
  stores both `shared_any` (capabilities) and
  `shared_extension` (lifecycle)
- `Passive<E>` where `E: Clone + Send` -> stores only
  `shared_any` (capabilities), no `Extension` bound needed

This means:

- A passive extension **cannot** have a `start()` method
  silently ignored -- it doesn't implement `Extension` at all.
- An active extension **must** implement `Extension` -- the
  compiler enforces this.
- The engine skips task spawning for passive extensions --
  no control channel, no messages, zero overhead.

### ExtensionWrapper

Engine-internal struct that manages an extension's
lifecycle(s) and capability registrations:

```rust
pub struct ExtensionWrapper {
    node_id: NodeId,
    user_config: Arc<NodeUserConfig>,
    runtime_config: ExtensionConfig,

    // Lifecycle -- None for passive
    shared_extension: Option<Box<dyn shared::Extension>>,
    local_extension: Option<Rc<dyn local::Extension>>,

    // Capabilities -- always present
    shared_any: Option<Box<dyn CloneAnySend>>,
    local_any: Option<Rc<dyn Any>>,
    capabilities: ExtensionCapabilities,

    // Control channels -- None for passive
    control_sender: Option<SharedSender<ExtensionControlMsg>>,
    control_receiver: Option<SharedReceiver<ExtensionControlMsg>>,
    shared_control_sender: Option<SharedSender<ExtensionControlMsg>>,
    shared_control_receiver: Option<SharedReceiver<ExtensionControlMsg>>,

    telemetry: Option<NodeTelemetryGuard>,
}
```

#### Builder Pattern

```rust
// Active shared-only (with local fallback)
ExtensionWrapper::builder(node, config, ext_config)
    .with_shared(Active(ext))
    .build()

// Passive shared-only
ExtensionWrapper::builder(node, config, ext_config)
    .with_shared(Passive(ext))
    .build()

// Dual-type active (independent lifecycles)
ExtensionWrapper::builder(node, config, ext_config)
    .with_local(Active(Rc::new(local_ext)))
    .with_shared(Active(shared_ext))
    .build()
```

#### TypeId Guard

When both `with_local` and `with_shared` are called, the
builder asserts at `build()` that the inner types have
different `TypeId`s. This catches a pointless pattern:
registering the same type as both local (`Rc<T>`) and
shared (`T`) creates two lifecycles for the same object --
one just unnecessarily wrapped in `Rc`. Since the registry
already wraps any shared impl in `SharedAsLocal` for local
consumers automatically, a same-type dual registration is
always redundant. Use `with_shared()` alone (shared-only
with local fallback).

When both types are registered, they must be genuinely
different -- e.g., a local type using `Rc<RefCell<HashMap>>`
(lock-free) and a shared type using `Arc<RwLock<HashMap>>`
(thread-safe). Different `TypeId`s guarantee the two
variants are intentionally distinct implementations, not
accidental duplicates.

#### Dual Control Channels

When both local and shared lifecycles are present (always
different types per the TypeId guard), the builder creates
two control channels. At `start()`, the shared lifecycle is
spawned on `tokio::spawn` (Send) and the local lifecycle
runs on the current `LocalSet` thread. Both receive
independent shutdown messages.

### Capability System

#### `#[capability]` Proc Macro

Each capability is defined via a single `#[capability]`
attribute on a trait definition in `capability/<name>.rs`.
The proc macro (in `engine-macros`) generates all
infrastructure from that one annotation:

```rust
#[capability(
    name = "bearer_token_provider",
    description = "Provides bearer tokens for HTTP",
)]
pub trait BearerTokenProvider {
    async fn get_token(&self) -> Result<BearerToken, Error>;
    fn subscribe_token_refresh(&self)
        -> tokio::sync::watch::Receiver<Option<BearerToken>>;
}
```

The macro generates:

- `local::BearerTokenProvider` trait (`#[async_trait(?Send)]`)
- `shared::BearerTokenProvider` trait (`#[async_trait]` + `Send`)
- `SharedAsLocal` adapter for transparent shared->local fallback
- A zero-sized `BearerTokenProvider` registration struct
- `Sealed` / `ExtensionCapability` impls (sealing)
- A `KNOWN_CAPABILITIES` static entry (via `distributed_slice`)
  for config validation
- `shared_capabilities()` / `local_capabilities()` methods
  for type-erased coercion
- `_adapt_shared_entry_to_local` function for shared->local
  fallback at `resolve_bindings()` time

#### Consuming Capabilities

Consumers use the zero-sized registration struct as the
generic parameter. The `Local` and `Shared` associated
types on `ExtensionCapability` determine the return type:

```rust
// Local consumer -- returns Rc<dyn local::BearerTokenProvider>
let auth = capabilities
    .require_local::<BearerTokenProvider>()?;
auth.get_token().await?;

// Shared consumer -- returns Box<dyn shared::KeyValueStore> (Send)
let kv = capabilities
    .require_shared::<KeyValueStore>()?;
kv.get("key").await?;
```

Sealing via `ExtensionCapability` (which requires
`private::Sealed`) ensures at compile time that only
engine-defined capabilities can be passed.
Local fallback from shared extensions is pre-populated
at `resolve_bindings()` time -- `require_local()` does
a flat HashMap lookup with no runtime adapter logic.

#### require_local/shared and optional_local/shared

The `Capabilities` struct (produced by `resolve_bindings()`)
is passed to every node factory. It provides four methods
for resolving capabilities:

**`require_local()`** -- Returns `Rc<dyn local::Trait>`.
Fallback from shared is pre-populated at build time.
If not bound, returns an error:

```rust
let auth = capabilities
    .require_local::<BearerTokenProvider>()?;
```

**`require_shared()`** -- Returns `Box<dyn shared::Trait>`
(which is `Send`). Only the shared variant is considered:

```rust
let kv = capabilities
    .require_shared::<KeyValueStore>()?;
```

**`optional_local()`** / **`optional_shared()`** -- Same
semantics but return `Option` instead of `Result`:

```rust
if let Some(store) = capabilities
    .optional_local::<KeyValueStore>()
{
    store.set("offset", offset_bytes).await?;
}
```

All methods track which variants were consumed
(`consumed_local()` / `consumed_shared()`). After all
nodes are built, the engine uses this to drop unused
extension variants -- if no consumer asked for the local
variant, `drop_local()` is called, freeing the `Rc`
and preventing an orphaned lifecycle from starting.

When a local entry is a `SharedAsLocal` adapter
(shared-only with local fallback), consuming it via
`require_local()` also marks `consumed_shared() = true`. This ensures the
shared lifecycle is never dropped when local consumers
depend on it through the adapter.

### Extension Traits

Two lifecycle traits -- local and shared:

**Local** (`local/extension.rs`):

```rust
#[async_trait(?Send)]
pub trait Extension {
    async fn start(
        self: Rc<Self>,
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, Error>;
}
```

**Shared** (`shared/extension.rs`):

```rust
#[async_trait]
pub trait Extension: Send {
    async fn start(
        self: Box<Self>,
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, Error>;
}
```

Key difference: local takes `Rc<Self>` (true single-instance
sharing with capability trait objects), shared takes
`Box<Self>` (ownership transfer).

Only active extensions implement these traits. Passive
extensions do not implement `Extension` at all.

### ExtensionFactory

```rust
pub struct ExtensionFactory {
    pub name: &'static str,
    pub description: &'static str,
    pub documentation_url: &'static str,
    pub capabilities: ExtensionCapabilities,
    pub create: fn(
        PipelineContext, NodeId, Arc<NodeUserConfig>,
        &ExtensionConfig,
    ) -> Result<ExtensionWrapper, Error>,
    pub validate_config: fn(&Value) -> Result<(), Error>,
}
```

The `capabilities` field carries the registration functions
produced by `extension_capabilities!`. The engine calls
these during build to populate the `CapabilityRegistry`.

## Implementing Extensions

### Active Extension (Azure Identity Auth)

```rust
// Factory -- shared-only (with local fallback)
ExtensionWrapper::builder(node, node_config, ext_config)
    .with_shared(Active(ext))
    .build()
```

The shared variant implements `Extension` with its own event
loop. Local consumers are served automatically via the
`SharedAsLocal` adapter -- no separate local type needed.

### Dual-Type Active Extension

For extensions that need genuinely different local and shared
implementations (e.g., lock-free `Rc<RefCell>` locally vs
thread-safe `Arc<RwLock>` shared):

```rust
// Factory -- separate types with independent lifecycles
ExtensionWrapper::builder(node, node_config, ext_config)
    .with_local(Active(Rc::new(local_ext)))
    .with_shared(Active(shared_ext))
    .build()
```

Both variants implement `Extension` with their own event
loop. The builder detects different `TypeId`s, creates
dual control channels, and `start()` spawns both.

### Passive Extension (Sample Shared KV Store)

```rust
// Factory -- no Extension trait impl needed
ExtensionWrapper::builder(node, node_config, ext_config)
    .with_shared(Passive(ext))
    .build()
```

No task spawned, no control channel. The extension only
registers capabilities.

### Dual-Type Passive Extension (Sample Shared/Local KV Store)

```rust
// Factory -- local uses Rc<RefCell<HashMap>> (no locks),
//           shared uses Arc<RwLock<HashMap>> (thread-safe)
ExtensionWrapper::builder(node, node_config, ext_config)
    .with_local(Passive(Rc::new(local_ext)))
    .with_shared(Passive(shared_ext))
    .build()
```

Different types -> different `TypeId`s -> accepted by
builder. Local consumers get the lock-free variant,
shared consumers get the thread-safe variant.

### Adding a New Capability

**1.** Create `capability/<name>.rs` with the `#[capability]`
macro and any shared data types:

```rust
use otap_df_engine_macros::capability;

type Error = super::registry::Error;

#[capability(
    name = "my_capability",
    description = "Does something useful",
)]
pub trait MyCapability {
    async fn do_thing(&self, input: &str) -> Result<String, Error>;
}
```

The macro generates: `local::MyCapability` trait
(`#[async_trait(?Send)]`), `shared::MyCapability` trait
(`#[async_trait]` + `Send`), `SharedAsLocal` adapter,
sealed trait impls, a zero-sized registration struct,
`KNOWN_CAPABILITIES` entry, and coercion functions.

**2.** Add re-exports in `local/capability.rs` and
`shared/capability.rs`:

```rust
pub use crate::capability::my_capability::local::MyCapability;
```

**3.** Register the module in `capability/mod.rs`.

## Configuration

### Pipeline YAML

Extensions are configured as siblings to `nodes` in the
pipeline config. Each extension has a `type` (URN) and
optional `config`. Consumers reference extensions by name
in their `capabilities` section:

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          azure-auth:
            type: "urn:microsoft:extension:azure_identity_auth"
            config:
              method: "managed_identity"
              client_id: "your-client-id"
              scope: "https://monitor.azure.com/.default"

          kv-store:
            type: "urn:otap:extension:sample_shared_key_value_store"
            # no config needed -- uses no_config validator

        nodes:
          azure-monitor-exporter:
            type: "urn:microsoft:exporter:azure_monitor"
            config:
              # ... exporter-specific config
            capabilities:
              bearer_token_provider: azure-auth
```

The `capabilities` section maps capability names to
extension instance names. This is how consumers declare
their dependencies -- the engine resolves them at build
time via `resolve_bindings()`.

### Config Validation

Each `ExtensionFactory` carries a `validate_config`
function pointer that performs static validation during
config parsing -- before any extension is created:

```rust
pub struct ExtensionFactory {
    // ...
    pub validate_config: fn(
        config: &serde_json::Value,
    ) -> Result<(), Error>,
}
```

Two built-in validators:

- **`validate_typed_config::<T>`** -- Deserializes the JSON
  config into type `T`. If deserialization fails, the error
  surfaces immediately at config parse time with a clear
  message. This is the most common validator:

  ```rust
  validate_config: validate_typed_config::<Config>,
  ```

- **`no_config`** -- Accepts `null` or `{}` only. Rejects
  any other value, catching typos or misplaced config
  blocks early:

  ```rust
  validate_config: no_config,
  ```

### Capability Binding Validation

During `resolve_bindings()`, the engine validates each
capability binding with four checks:

1. **Extension exists** -- The named extension instance must
   be registered. Error: "no extension with that name exists."

2. **Known capability type** -- The capability name must be
   in `KNOWN_CAPABILITIES` (registered at link time via
   the `#[capability]` proc macro). Error: "Unknown capability"
   with a list of known types.

3. **Capability provided** -- Some loaded extension must
   actually provide the requested capability. Error:
   "no loaded extension provides it."

4. **Specific extension provides it** -- The specific named
   extension must expose the requested capability. Error:
   "does not provide capability" with a list of what it
   does provide.

After all nodes are built, the engine also detects
**unused bindings** -- capabilities that were configured
but never consumed by any `require_local()`,
`require_shared()`, `optional_local()`, or
`optional_shared()` call. These are reported as warnings
for configuration hygiene.

## Pipeline Lifecycle

```text
1. Config parsing
   |- Extensions parsed from `extensions` section
   \- ExtensionInNodesSection error if misplaced

2. Pipeline build
   |- Create extensions (factories return ExtensionWrapper)
   |- register_traits() -> populate CapabilityRegistry
   |- resolve_bindings() -> per-node Capabilities
   |- Create data-path nodes (receive &Capabilities)
   |- Track consumption (consumed_local/consumed_shared)
   \- Drop unused variants (drop_local/drop_shared)

3. Pipeline start (RuntimePipeline::run)
   |- Passive extensions: skip (is_passive() == true)
   |- Active extensions: spawn tasks, track control senders
   |- Spawn exporters, processors, receivers
   \- Extension control senders stored separately

4. Steady state
   |- Active extensions run event loops
   |- Passive extensions exist only as registered capabilities
   \- ExtensionControlMsg flows to active extensions only

5. Shutdown
   |- Data-path nodes drain
   |- shutdown_extensions() sends Shutdown to active only
   \- Extensions terminate after data-path is fully drained
```

## Future: Hierarchical Extension Scopes

The Phase 1 design supports **pipeline-scoped**
extensions only. The extension system is designed to
evolve toward hierarchical scoping in future phases.

### Pipeline Scope (Phase 1)

Extensions are declared at the pipeline level and consumed
by nodes within that pipeline. Following the principle that
**sharing boundary is determined by the scope**, both local
and shared pipeline-scoped extensions are instantiated per
pipeline instance (one per core in thread-per-core mode).
The local/shared distinction at this scope controls type
constraints only: local uses `Rc`-based `!Send` types,
shared uses `Clone + Send` types. See the
[refinements section](#how-this-document-relates-to-the-proposal)
for the rationale behind this design choice.

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:        # pipeline-scoped
          azure-auth: ...
        nodes:
          exporter:
            capabilities:
              bearer_token_provider: azure-auth
```

### Planned: Group and Engine Scopes (Phase 2)

Extension declarations will be allowed at higher levels
in the configuration hierarchy:

```yaml
engine:
  extensions:              # engine-scoped (global)
    shared-auth: ...

groups:
  default:
    extensions:            # group-scoped
      group-kv-store: ...
    pipelines:
      main:
        extensions:        # pipeline-scoped (Phase 1)
          local-cache: ...
```

**Scope resolution order:** when a node binds a capability,
the engine resolves extensions from the innermost scope
outward: pipeline -> group -> engine. The first matching
extension wins.

**Execution model implications:**

| Scope    | Execution Model | Sharing                          |
|----------|-----------------|----------------------------------|
| Pipeline | Local           | Per pipeline instance (per core) |
| Pipeline | Shared          | Per pipeline instance (per core) |
| Group    | Shared only     | Across pipelines in the group    |
| Engine   | Shared only     | Across all pipelines             |

At the pipeline level, both local and shared extensions
are instantiated **per pipeline instance** (i.e., per
core in thread-per-core mode). The distinction between
local and shared at this scope is about **type
constraints**, not about cross-core sharing:

- **Local** extensions use `Rc`-based, `!Send` types.
  They cannot leave the core they were created on.
  This enables lock-free designs using `Rc`, `RefCell`,
  and `Cell`.
- **Shared** extensions use `Clone + Send` types. They
  are still instantiated per pipeline instance, but
  their `Send` bound means they *could* be shared across
  cores. At the pipeline scope this is not needed --
  it simply means the implementation uses thread-safe
  primitives (e.g., `Arc`, `RwLock`).

Cross-core sharing becomes meaningful when extensions
are declared at **higher scopes** (group or engine) in
Phase 2. At those scopes, a single extension instance
serves multiple pipeline instances running on different
cores, so only the shared (`Send + Clone`) execution
model is permitted -- local extensions are not allowed
at group or engine scope.

This design keeps Phase 1 simple: pipeline-scoped
extensions are always per-core, with no cross-core
coordination. Extension authors who only need pipeline
scope can choose local for maximum performance or shared
for convenience, knowing both are core-local. When
Phase 2 introduces higher scopes, shared extensions
naturally promote to cross-core usage without API
changes.

**No architectural changes required for Phase 2:** the
Phase 1 `CapabilityRegistry`, `resolve_bindings()`, and
`require_local()` / `require_shared()` mechanisms are
scope-agnostic. Adding higher scopes requires:

1. Config parsing for `extensions` at group and engine
   levels
2. A scope-aware resolution pass in `resolve_bindings()`
   that merges registries from inner to outer scope
3. Validation that group/engine-scoped extensions only
   use the shared execution model

The `SharedAsLocal` adapter ensures that even when a
group or engine-scoped shared extension is consumed by a
local node (via `require_local()`), the local consumer
gets an `Rc`-wrapped adapter with no API change.

### Phase 3 (future): WASM Extensions

WASM-based sandboxed extensions could allow third-party
extension authors to provide implementations of
engine-defined capabilities without native code
compilation.

A possible integration path: a native `WasmExtensionHost`
extension embeds a WASM runtime (e.g., `wasmtime`), loads
user-provided `.wasm` modules, and bridges WASM function
exports to engine capability traits. The host would
register capabilities into the `CapabilityRegistry` at
runtime via the existing `register_all_shared()` API.
WASM runtimes produce `Send + Sync` instances, so WASM
extensions would always be shared, with local consumers
served via the `SharedAsLocal` adapter.

The consumer-side API (`require_local` / `require_shared`)
would not change. Capability types would still be defined
in the engine core.

No design work has started. Details will depend on
actual use cases and requirements when this phase is
explored.

## Migration Plan (Phase 1)

The extension system is developed on a feature branch and
will be merged to `main` incrementally. Each PR is
self-contained and leaves `main` in a working state.

### PR 1 -- Config: `extensions` section + `capabilities` bindings

Add config parsing for the `extensions:` section and
`capabilities:` bindings on nodes. Includes:

- `ExtensionConfig` struct in `otap-df-config`
- Pipeline config parsing: `extensions` as siblings to `nodes`
- `ExtensionInNodesSection` error if extensions are
  misplaced inside `nodes`
- `capabilities` field on `NodeUserConfig`
- Config validation: `validate_typed_config`, `no_config`
- No runtime behavior -- config is parsed and validated
  but extensions are not created or started

### PR 2 -- Engine: `ExtensionFactory` + `ExtensionWrapper` + builder

Add the core engine types for extension lifecycle:

- `ExtensionFactory` struct + `OTAP_EXTENSION_FACTORIES`
  distributed slice
- `ExtensionWrapper` with builder pattern
- `Active<E>` / `Passive<E>` newtype wrappers
- `ExtensionControlMsg`, control channel types
- Extension `start()` traits (local + shared)
- TypeId guard, dual control channels
- No capability system yet -- extensions can start/stop
  but don't provide capabilities

### PR 3 -- Capability system: `#[capability]` proc macro + registry

Add type-safe capability resolution:

- `#[capability]` proc macro in `engine-macros`
- `CapabilityRegistry`, `Capabilities` struct
- `resolve_bindings()` with 4-step validation
- `require_local()` / `require_shared()` /
  `optional_local()` / `optional_shared()`
- `extension_capabilities!` macro (3 arms:
  `shared:`, `local:`, dual)
- `SharedAsLocal` adapter + `shared_as_local` tracking
- `consumed_local()` / `consumed_shared()` +
  `drop_local()` / `drop_shared()`
- `KNOWN_CAPABILITIES` link-time registration
- Sealed traits + `ExtensionCapability` with
  `Local`/`Shared` associated types
- Compile-time assertions in `extension_capabilities!`

### PR 4 -- First capabilities: `BearerTokenProvider` + `KeyValueStore`

Define the initial capability traits:

- `capability/bearer_token_provider.rs` -- `BearerToken`,
  `Secret`, `#[capability]` invocation
- `capability/key_value_store.rs` -- `#[capability]`
  invocation
- Re-exports in `local/capability.rs` and
  `shared/capability.rs`

### PR 5 -- Runtime pipeline: extension spawning + shutdown

Wire extensions into the pipeline lifecycle:

- `RuntimePipeline::run` -- spawn extensions before
  data-path nodes, skip passive ones
- `shutdown_extensions()` -- send `Shutdown` to active
  extensions after data-path nodes drain
- Extension control sender tracking in `pipeline_ctrl.rs`
- `is_passive()` check

### PR 6 -- Node factories: `&Capabilities` parameter

Update `ExporterFactory`, `ProcessorFactory`,
`ReceiverFactory` to accept `&Capabilities`:

- Add `capabilities` parameter to factory `create` fn
- Update all existing node factories to accept (and
  ignore) the parameter
- No consumer changes yet -- factories receive
  `Capabilities` but don't call `require_*`

### PR 7 -- Azure Identity Auth Extension

First real extension implementation:

- `azure_identity_auth_extension` -- shared-only (with local fallback)
  active extension providing `BearerTokenProvider`
- Token acquisition with retry + exponential backoff
- Token refresh event loop with `watch::Sender` broadcast
- Config: `method` (managed_identity / development),
  `scope`, `client_id`

### PR 8 -- Azure Monitor Exporter: consume `BearerTokenProvider`

Migrate the exporter from built-in auth to extension-based:

- Remove `auth` module from azure_monitor_exporter
- Use `require_local::<BearerTokenProvider>()` in factory
- Subscribe to `token_rx.changed()` in event loop
- Config: remove `auth` section, add `capabilities`
  binding in YAML
