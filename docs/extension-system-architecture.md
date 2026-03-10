# Extension System Architecture

## Overview

This document describes the architecture of the extension
system in the OTAP dataflow engine -- what extensions are,
how they integrate into the pipeline lifecycle, and how to
implement new ones.

## What Are Extensions?

Extensions are standalone pipeline components that provide
**shared, cross-cutting capabilities** -- such as authentication,
service discovery, or health checking -- to data-path nodes
(receivers, exporters). They are configured as siblings to
`nodes`, not as nodes themselves, and they never touch pipeline
data directly.

### Why Do We Need Them?

Before extensions, cross-cutting concerns like authentication
were embedded directly inside individual exporters. This led to:

- **Duplication** -- every exporter needing auth carried its own
  credential management and token refresh loop.
- **Tight coupling** -- credential-specific dependencies (e.g.,
  `azure_identity`) leaked into exporter crates even when unused.
- **No sharing** -- multiple exporters targeting the same tenant
  each acquired and refreshed tokens independently.

Extensions solve this by extracting shared capabilities into
named, independently-running components. An extension can
optionally expose well-defined traits through a type-safe
registry for data-path nodes to look up by name, or it can
simply run as a pure background task (e.g., certificate
rotation, service discovery refresh) without publishing
any capabilities at all. Either
way: no direct dependencies between nodes, no duplicated logic,
no wasted resources.

## Architecture Overview

```text
+-----------------------------------------------+
|               Pipeline Engine                 |
|                                               |
|  +-------------+  +-------------+             |
|  | Extension A |  | Extension B |  ...        |
|  | (auth)      |  | (background)|             |
|  | Arc<State>  |  +-------------+             |
|  +------+------+                              |
|         | extension_capabilities!() macro     |
|         | clones self per trait, producing    |
|         | Vec<CapabilityRegistration>         |
|         v                                     |
|  +----------------------+                     |
|  |  CapabilityRegistry   |  (built once)      |
|  |  stores cloned trait |                     |
|  |  objects by name     |                     |
|  +--+---------------+---+                     |
|     | clone()       | clone()                 |
|     v               v                         |
|  +----------+  +----------+                   |
|  | Receiver |  | Exporter |                   |
|  | (own     |  | (own     |                   |
|  |  registry|  |  registry|                   |
|  |  clone)  |  |  clone)  |                   |
|  +----------+  +----------+                   |
|                                               |
|  get() returns a cloned Box<dyn Trait>;       |
|  all clones share state via Arc inside the    |
|  extension -- the registry itself holds       |
|  type-erased cloneable trait objects          |
+-----------------------------------------------+
```

### Key Design Decisions

1. **Extensions start first, shut down last.** The engine
   spawns extensions before any data-path nodes so their
   capabilities are available at initialization. At shutdown,
   extensions terminate only after all data-path nodes have
   drained -- ensuring capabilities like auth tokens remain
   available during final flushes. Extension instances are
   scoped to a single pipeline -- they are not shared across
   pipelines.

2. **PData-free.** Extensions are completely decoupled from
   the pipeline data type (`PData`). They receive their own
   `ExtensionControlMsg` messages (shutdown, timer ticks,
   config updates, telemetry collection) through a dedicated
   control channel and never process pipeline data directly.

3. **Separate control channel.** Extensions use
   `ExtensionControlSender` / `ExtensionControlMsg` instead
   of the pipeline's `PipelineCtrlMsgSender<PData>`. This
   prevents extensions from holding clones of the pipeline
   control channel sender, which would block the channel
   from closing and prevent graceful shutdown.

4. **Local/Shared split.** Like receivers and exporters,
   extensions have both local (`!Send` futures) and shared
   (`Send` futures) variants. Local extensions run on the
   single-threaded `LocalSet`; shared extensions can be
   spawned on multi-threaded runtimes. `ExtensionWrapper`
   abstracts over both variants.

5. **Registry-based lookup.** The `CapabilityRegistry` is
   passed to receiver, processor, and exporter factories
   at construction time -- not at `start()`. This means
   capabilities are resolved during pipeline build, catching
   missing extensions early. The API and naming are
   node-agnostic -- all node types receive the registry
   through the same factory parameter.

6. **Optional capability publishing.** Extensions that expose
   capabilities override `extension_capabilities()` to register
   capability implementations in the registry. Extensions that are
   pure background tasks simply use the default (empty)
   implementation and never appear in the registry.

## Core Types

### Extension Lifecycle Trait

The lifecycle contract every extension implements. Two
variants exist -- local and shared -- mirroring the pattern
used by receivers and exporters.

**Local** (`engine/src/local/extension.rs`):

```rust
#[async_trait(?Send)]
pub trait Extension {
    async fn start(
        self: Box<Self>,
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, Error>;

    fn extension_capabilities(&self)
        -> Vec<CapabilityRegistration>
    {
        Vec::new()
    }
}
```

**Shared** (`engine/src/shared/extension.rs`):

```rust
#[async_trait]
pub trait Extension: Send {
    async fn start(
        self: Box<Self>,
        ctrl_chan: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, Error>;

    fn extension_capabilities(&self)
        -> Vec<CapabilityRegistration>
    {
        Vec::new()
    }
}
```

Key points:

- **Not generic over `PData`.** Unlike receivers,
  processors, and exporters, extensions never touch pipeline
  data. This is the fundamental difference.
- **`start()` takes ownership** via `Box<Self>`, moving the
  extension into its own task. After this, the engine can
  only reach it through the control channel.
- **`ControlChannel`** wraps a receiver for
  `ExtensionControlMsg` (shutdown, timer ticks, config
  updates, telemetry collection). No pipeline data ever
  flows through it.
- **`EffectHandler`** provides node identity and metrics
  reporting. Extensions manage their own timers directly
  (e.g., `tokio::time`) rather than through the engine's
  timer infrastructure.
- **`extension_capabilities()`** defaults to empty. Extensions
  that publish capabilities override it (typically via the
  `extension_capabilities!` macro) to return a
  `Vec<CapabilityRegistration>`. During pipeline build, the
  engine calls this method on each extension and inserts
  the returned registrations into the `CapabilityRegistry`
  under the extension's configured name. Pure background
  tasks leave the default and never appear in the
  registry.
- The only difference between local and shared is the
  `Send` bound: local uses `#[async_trait(?Send)]` (futures
  can be `!Send`), shared uses `#[async_trait]` (futures
  must be `Send`). This allows certain optimizations in
  code paths that don't cross into the extension traits
  the extension implements.

### ExtensionWrapper

Engine-internal adapter (`engine/src/extension.rs`) that
wraps a local or shared extension into a single type the
engine can manage uniformly. It is a **non-generic** enum:

```rust
pub enum ExtensionWrapper {
    Local { /* local::Extension impl + channels */ },
    Shared { /* shared::Extension impl + channels */ },
}
```

Each variant holds:

- The boxed extension instance
- A `ControlSender` / `ControlReceiver` pair for
  `ExtensionControlMsg`
- The extension's `NodeId`, user config, and runtime config
- An optional `NodeTelemetryGuard` for lifecycle cleanup

Responsibilities:

- **Construction** -- `ExtensionWrapper::local()` and
  `::shared()` create the control channel and box the
  extension.
- **Trait registration** -- `register_capabilities()` calls
  the extension's `extension_capabilities()` and inserts the
  results into the `CapabilityRegistry` under the
  extension's name.
- **Control sender** -- `extension_control_sender()`
  produces an `ExtensionControlSender` that the engine
  stores separately for shutdown orchestration.
- **Start** -- `start()` takes ownership, constructs the
  appropriate `ControlChannel` and `EffectHandler`, and
  calls the extension's `start()` method. No
  `PipelineCtrlMsgSender` is passed -- extensions are
  fully PData-free.
- **Telemetry** -- implements `TelemetryWrapped` for
  control-channel metrics and node telemetry guards.

`ExtensionWrapper` does **not** implement `Node<PData>` or
`Controllable<PData>` -- extensions are not data-path nodes.

### ExtensionControlMsg

Defined in `engine/src/control.rs`. A PData-free subset of
`NodeControlMsg` -- extensions never process pipeline data,
so they have no `Ack`, `Nack`, or `DelayedData` variants.

```rust
#[derive(Debug, Clone)]
pub enum ExtensionControlMsg {
    Config { config: serde_json::Value },
    TimerTick {},
    CollectTelemetry {
        metrics_reporter: MetricsReporter,
    },
    Shutdown { deadline: Instant, reason: String },
}
```

Each variant:

- **`Config`** -- notifies the extension of a configuration
  change (hot reload).
- **`TimerTick`** -- periodic tick from the engine.
- **`CollectTelemetry`** -- asks the extension to flush its
  local metrics into the provided `MetricsReporter`.
- **`Shutdown`** -- requests graceful shutdown with a
  deadline and human-readable reason.

These messages flow through a dedicated channel per
extension (created by `ExtensionWrapper`), kept separate
from the pipeline's `PipelineCtrlMsgSender<PData>` to avoid
blocking graceful shutdown (see Key Design Decision #3).

`ExtensionControlSender` wraps the sender side of this
channel and is stored by the engine's
`PipelineCtrlMsgManager` for shutdown orchestration.

### CapabilityRegistry and CapabilityRegistration

Defined in `engine/src/extension/registry.rs`.

**`CapabilityRegistration`** is a self-contained record produced
by the `extension_capabilities!` macro. Each registration carries:

- A cloned copy of the concrete extension value
  (type-erased via `Box<dyn CloneAnySend>`)
- A monomorphised `coerce` function pointer that knows how
  to clone the concrete value and wrap it as
  `Box<dyn Trait>`
- The `TypeId` of `Box<dyn Trait>` for lookup

**`CapabilityRegistry`** stores these registrations and
serves lookups. It is `Clone + Send + Default`.

```rust
// Keyed by (extension_name, TypeId::of::<Box<dyn Trait>>())
#[derive(Default, Clone)]
pub struct CapabilityRegistry {
    handles: HashMap<(String, TypeId), RegistryEntry>,
}
```

The lookup API:

```rust
// Consumer retrieves a trait object by name:
let provider: Box<dyn BearerTokenProvider> = registry
    .get::<dyn BearerTokenProvider>("azure_auth")?;
```

How it works end-to-end:

1. The `extension_capabilities!` macro clones the extension
   instance once per trait, pairs each clone with a
   monomorphised `coerce` fn, and returns
   `Vec<CapabilityRegistration>`.
2. During pipeline build, the engine calls
   `ExtensionWrapper::register_capabilities()`, which calls
   `extension_capabilities()` on the extension and inserts the
   registrations into the registry under the extension's
   configured name.
3. The registry is passed to each receiver/exporter/processor
   factory as the last parameter of `create()`. Components
   resolve their capabilities at construction time.
4. `get::<dyn Trait>(name)` looks up the entry by
   `(name, TypeId::of::<Box<dyn Trait>>())`, invokes the
   stored `coerce` fn to produce a fresh
   `Box<dyn Trait>`, and returns it.

A single extension can implement multiple capabilities,
exposing different interfaces through granular traits:

```rust
extension_capabilities!(BearerTokenProvider, HealthCheck);
```

This is useful for extensibility and version management --
an extension can implement both `TraitA` and `TraitAv2`
simultaneously, letting consumers migrate at their own
pace while the extension supports both versions.

Error discrimination:

- **`NotFound`** -- no extension registered under that name.
- **`TraitNotImplemented`** -- extension exists but doesn't
  expose the requested trait.

### Sealed Capabilities and the `extension_capabilities!` Macro

**Sealed capabilities** -- The `ExtensionCapability` marker trait
(`engine/src/extension/registry.rs`) restricts which capability
types can be stored in the registry. It uses a sealed
pattern:

```rust
pub(crate) mod private {
    pub trait Sealed {}
}
pub trait ExtensionCapability: private::Sealed {}
```

Each extension trait file self-registers:

```rust
// In bearer_token_provider.rs:
impl private::Sealed for dyn BearerTokenProvider {}
impl ExtensionCapability for dyn BearerTokenProvider {}
```

Because `Sealed` is `pub(crate)`, external crates can
*implement* existing extension capabilities but cannot define
new capability types -- keeping the set of extension capabilities
well-defined and documented within the engine crate.

**`extension_capabilities!` macro** -- A convenience macro that
extension writers use inside their `impl Extension` block
to wire up capability registration:

```rust
#[async_trait(?Send)]
impl Extension for MyExtension {
    extension_capabilities!(BearerTokenProvider);

    async fn start(...) { ... }
}
```

The macro handles the boilerplate that would otherwise
be error-prone:

- Verifies at **compile time** that each listed trait
  implements `ExtensionCapability` (sealed), catching attempts
  to register unsupported capabilities.
- Verifies the concrete type implements each listed trait
  plus `Clone + Send + 'static`.
- Creates monomorphised `coerce` function pointers for
  type-safe downcasting -- these are the `fn` pointers
  stored in `CapabilityRegistration` that the registry uses
  to produce `Box<dyn Trait>` on lookup.

Without the macro, extension writers would need to
manually construct `CapabilityRegistration` values with the
correct `TypeId` and coerce functions -- a process that
is both tedious and easy to get wrong.

The macro's `Clone` requirement is intentional -- it
signals to extension developers that their type will be
cloned during registration (and again on each registry
`get()` call). This encourages holding internal state
behind `Arc` so that clones are cheap (just a reference
count bump) and all clones observe the same underlying
state.

#### Design Alternative: `Arc` vs Boxed Clone

An alternative design would have the registry store
`Arc<dyn Trait>` directly, giving true single-instance
sharing via pointer incrementation. However, `Arc`
requires `Sync` on the inner type -- which conflicts with
the engine's architecture where neither local nor shared
components require `Sync`. By using boxed deep clones
with a `Send`-only requirement, the registry works
naturally with both local and shared components. Extension
authors get the same cheap-clone semantics in practice by
wrapping their internal state in `Arc`, but without
imposing `Sync` at the trait boundary.

#### Why Extension Capabilities Are `Send`-Only

Extension capabilities (e.g., `BearerTokenProvider`) require
`Send` but not `Sync`. There is no `!Send` variant of
extension traits -- unlike the `Extension` lifecycle trait
which has local/shared variants. This simplifies
extension implementation: a single trait implementation
works for both local and shared consumers.

Supporting additional boundary types is possible but
adds complexity at multiple levels:

- **`Send + Sync`** could be supported by adding an
  `Arc`-based storage bucket to the existing registry
  (no separate registry needed). But it introduces a
  decision point for every new trait: `Send`-only or
  `Send + Sync`?
- **`!Send`** cannot coexist in the same registry -- any
  `!Send` value (e.g., `Rc`) poisons the registry's
  `Send` bound, making it unusable by shared components.
  A separate local-only registry or a split view would
  be required.
- **Extension writers** would need to reason about which
  boundary their trait belongs to, likely requiring
  different macros or marker types to select the right
  storage path.

`Send`-only avoids all of this: one storage mechanism,
one macro, one mental model -- and it covers all current
use cases.

#### Returning `Sync` Values from `Send`-Only Traits

Some consumers need `Send + Sync` values -- for example,
tonic interceptors must be `Clone + Send + Sync`. The
current design handles this without requiring `Sync` on
the trait object itself: the extension trait stays
`Send`-only, but its methods can return `Send + Sync`
values:

```rust
#[async_trait]
pub trait InterceptorProvider: Send {
    fn interceptor(&self)
        -> Arc<dyn Interceptor + Send + Sync>;
}
```

The registry stores `Box<dyn InterceptorProvider>`
(`Send`, not `Sync`). The consumer calls
`.interceptor()` and gets back an
`Arc<dyn Interceptor + Send + Sync>` that it can share
across threads freely. The `Sync` requirement stays on
the returned value, not on the trait object or the
extension struct.

In practice the extension writer simply holds the
interceptor in an `Arc` field (which they already need
for cheap clones), so the implementation is trivial and
adds no friction.

### BearerTokenProvider

The first concrete extension trait, defined in
`engine/src/extension/bearer_token_provider.rs`. It
provides authentication tokens to consumers:

```rust
#[async_trait]
pub trait BearerTokenProvider: Send {
    async fn get_token(&self)
        -> Result<BearerToken, Error>;

    fn subscribe_token_refresh(&self)
        -> watch::Receiver<Option<BearerToken>>;
}
```

- **`get_token()`** returns a `BearerToken` containing
  a `Secret`-wrapped token value and a UNIX-timestamp
  expiry. `Secret` redacts the value in `Debug` output
  to prevent accidental credential leakage in logs.
- **`subscribe_token_refresh()`** returns a
  `tokio::sync::watch::Receiver` for reactive
  notification when tokens are refreshed -- consumers
  can update HTTP headers in a `tokio::select!` branch
  without polling.

This trait demonstrates the typical extension capability
pattern:

- `Send`-only (no `Sync` required)
- Self-registers as a sealed `ExtensionCapability` via the
  two-line `impl Sealed` / `impl ExtensionCapability` pattern
- Consumers look it up by name:
  `registry.get::<dyn BearerTokenProvider>("auth")`

### Adding a New Extension Capability

Using `BearerTokenProvider` as the real example.

**1. Define the trait** in a new file under
`engine/src/extension/`
(`bearer_token_provider.rs`):

```rust
use async_trait::async_trait;

#[async_trait]
pub trait BearerTokenProvider: Send {
    async fn get_token(&self)
        -> Result<BearerToken, Error>;

    fn subscribe_token_refresh(&self)
        -> watch::Receiver<Option<BearerToken>>;
}
```

**2. Seal it** in the same file -- these two lines
register the trait for use with the registry:

```rust
impl super::registry::private::Sealed
    for dyn BearerTokenProvider {}
impl super::registry::ExtensionCapability
    for dyn BearerTokenProvider {}
```

**3. Export the module** in `engine/src/extension.rs`:

```rust
pub mod bearer_token_provider;
```

That's it for the engine side. The capability is now usable
in extension implementations and registry lookups.

### Implementing an Extension

Using the Azure Identity Auth Extension
(`contrib-nodes/src/extensions/
azure_identity_auth_extension/`) as the real example.

**1. Define a `Clone` struct** with shared state behind
`Arc`:

```rust
#[derive(Clone)]
pub struct AzureIdentityAuthExtension {
    credential: Arc<dyn TokenCredential>,
    scope: String,
    method: AuthMethod,
    token_sender:
        Arc<watch::Sender<Option<BearerToken>>>,
}
```

All state is behind `Arc` -- cloning is cheap and all
clones observe the same token broadcast channel.

**2. Implement the extension trait** on the struct:

```rust
#[async_trait]
impl BearerTokenProvider
    for AzureIdentityAuthExtension
{
    async fn get_token(&self)
        -> Result<BearerToken, Error>
    {
        let access_token =
            self.get_token_with_retry().await?;
        Ok(BearerToken::new(
            access_token.token.secret().to_string(),
            access_token.expires_on.unix_timestamp(),
        ))
    }

    fn subscribe_token_refresh(&self)
        -> watch::Receiver<Option<BearerToken>>
    {
        self.token_sender.subscribe()
    }
}
```

**3. Implement the `Extension` lifecycle trait** with
`extension_capabilities!` to wire up registration:

```rust
#[async_trait(?Send)]
impl Extension for AzureIdentityAuthExtension {
    extension_capabilities!(BearerTokenProvider);

    async fn start(
        self: Box<Self>,
        mut ctrl_chan: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, Error> {
        // proactive token refresh loop via
        // tokio::select!, broadcasting new tokens
        // to all subscribers via watch::Sender
    }
}
```

**4. Register the factory** via `distributed_slice` so
the engine discovers it automatically:

```rust
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static AZURE_IDENTITY_AUTH_EXTENSION:
    ExtensionFactory = ExtensionFactory {
    name: AZURE_IDENTITY_AUTH_EXTENSION_URN,
    create: |_ctx, node, node_config, ext_config| {
        let cfg: Config = serde_json::from_value(
            node_config.config.clone(),
        )?;
        cfg.validate()?;
        let ext =
            AzureIdentityAuthExtension::new(cfg)?;
        Ok(ExtensionWrapper::local(
            ext, node, node_config, ext_config,
        ))
    },
    validate_config:
        validate_typed_config::<Config>,
};
```

### Using an Extension

**1. Configure it** in the pipeline YAML -- extensions
are siblings to `nodes`, not inside them:

```yaml
groups:
  default:
    pipelines:
      main:
        extensions:
          azure-auth:
            type: "urn:microsoft:extension:azure_identity_auth"
            config:
              method: "managedidentity"
              client_id: "your-client-id"
              scope: "https://monitor.azure.com/.default"

        nodes:
          azure-monitor-exporter:
            type: "urn:microsoft:exporter:azure_monitor"
            config:
              auth: "azure-auth"  # reference by name
```

Supports two auth methods:

- **`managed_identity`** -- system or user-assigned
  managed identity (production).
- **`development`** -- Azure CLI / Developer CLI
  credentials (local development).

**2. Look up in the factory** and subscribe to token
refreshes in `start()`:

```rust
// In the factory create() closure:
let auth = capability_registry
    .get::<dyn BearerTokenProvider>(
        &cfg.auth,
    )?;
// Pass auth to the exporter constructor:
let exporter = AzureMonitorExporter::new(
    pipeline_ctx, cfg, auth,
)?;

// In start(), subscribe to the stored auth field:
let mut token_rx =
    self.auth.subscribe_token_refresh();
token_rx.wait_for(|t| t.is_some()).await?;

// In the event loop:
tokio::select! {
    _ = token_rx.changed() => {
        if let Some(token) =
            token_rx.borrow_and_update().as_ref()
        {
            client_pool.update_auth(
                bearer_header(token),
            );
        }
    }
    // ... other branches
}
```

The exporter's config holds the extension name as a
string. The factory receives the `CapabilityRegistry`
and resolves the auth extension at construction time.
The exporter stores the resulting `Box<dyn BearerTokenProvider>`
as a field, using it directly in `start()` without
any registry lookup.

This pattern eliminated ~380 lines of duplicated auth
code from the Azure Monitor exporter, replacing it with
a ~10-line registry lookup and reactive subscription.

## Pipeline Lifecycle

How extensions integrate into the pipeline's build,
start, steady-state, and shutdown phases.

```text
1. Config parsing
   +- Extensions parsed from the `extensions`
   |  section (sibling to `nodes`)
   +- NodeKind::Extension recognized in node_urn
   +- Placing an extension URN in `nodes` is
      rejected with ExtensionInNodesSection error

2. Pipeline build (PipelineFactory)
   +- Create extensions FIRST from the
   |  `extensions` section
   +- register_capabilities() -- collect
   |  CapabilityRegistration from each extension,
   |  insert into CapabilityRegistry
   +- Create data-path nodes (receivers,
   |  processors, exporters) -- each factory
   |  receives &CapabilityRegistry as last param
   +- Telemetry setup (channel metrics, node
      telemetry guards)

3. Pipeline start (RuntimePipeline::run)
   +- Spawn extension tasks FIRST
   +- Spawn exporter tasks
   +- Spawn processor tasks
   +- Spawn receiver tasks

4. Steady state
   +- Extensions run their event loops (e.g.,
   |  token refresh)
   +- Data-path components use registry lookups
   |  as needed
   +- ExtensionControlMsg flows normally
      (config, timer, telemetry)

5. Shutdown
   +- Data-path nodes receive Shutdown and drain
   +- Pipeline control channel closes after all
   |  data-path nodes finish
   +- PipelineCtrlMsgManager::shutdown_extensions()
   |  sends ExtensionControlMsg::Shutdown to all
   |  extensions with a 5-second deadline
   +- Extensions terminate AFTER data-path is
      fully drained
```

**Why start-first?** Extensions provide capabilities
that data-path nodes depend on during initialization.
If an exporter needs a token at startup, the extension
must already be running and ready.

**Why shutdown-last?** Extensions provide capabilities
that data-path nodes depend on during graceful shutdown.
If exporters are flushing final batches, they may still
need valid credentials. Shutting down extensions first
would cause those final exports to fail.

**Why separate control senders?** Extension control
senders (`Vec<ExtensionControlSender>`) are stored
separately from data-path `ControlSenders<PData>`.
This is because extensions use `ExtensionControlMsg`
(PData-free) rather than `NodeControlMsg<PData>`, and
keeping them separate ensures the pipeline control
channel can close naturally when all data-path senders
are dropped -- without extensions holding it open.
