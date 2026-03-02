# Extension System

This document describes the extension system architecture for the OTAP dataflow
engine. Extensions are standalone components (configured as a sibling to
`nodes`, not as a node) that provide shared capabilities -- such as
authentication, service discovery, or health checking -- to other pipeline
components.

## Motivation

Before extensions, cross-cutting concerns like Azure authentication were
embedded directly inside individual exporters. This had several drawbacks:

- **Duplication**: Every exporter needing authentication carried its own
  credential management, token refresh loop, and retry logic.
- **Tight coupling**: Credential types (e.g., `azure_identity`) leaked into
  exporter crate dependencies even when not needed.
- **No sharing**: Multiple exporters targeting the same Azure tenant each
  acquired and refreshed their own tokens independently.

The extension system solves these by extracting shared capabilities into
standalone, named components that run alongside the pipeline and expose
well-defined traits through a type-safe registry.

## Architecture Overview

```text
+-------------------------------------------------------------+
|                     Pipeline Engine                         |
|                                                             |
|  +--------------+  +--------------+  +--------------+      |
|  |  Extension A  |  |  Extension B  |  |     ...      |      |
|  |  (auth)       |  |  (discovery)  |  |              |      |
|  +------+-------+  +------+-------+  +--------------+      |
|         |                 |                                  |
|         v                 v                                  |
|  +-----------------------------------------------------+    |
|  |              ExtensionRegistry                       |    |
|  |  get::<dyn BearerTokenProvider>("auth") -> Box<dyn T> |    |
|  +---------------------+-------------------------------+    |
|                        |                                     |
|         +--------------+--------------+                      |
|         v                             v                      |
|  +----------+                  +----------+                |
|  | Receiver |                  | Exporter |                |
|  +----------+                  +----------+                |
+-------------------------------------------------------------+
```

### Key Design Decisions

1. **Extensions start first.** The engine spawns extensions before receivers,
   processors, and exporters so their capabilities are available when
   data-path components initialize.

2. **PData-free.** Extensions are completely decoupled from the pipeline data
   type (`PData`). They receive their own `ExtensionControlMsg` messages
   (shutdown, timer ticks, config updates, telemetry collection) through a
   dedicated control channel. They never process pipeline data directly.

3. **Local/Shared split.** Like receivers and exporters, extensions have both
   local (`!Send` futures) and shared (`Send` futures) variants. Local
   extensions run on the single-threaded `LocalSet`; shared extensions can be
   spawned on multi-threaded runtimes. The `ExtensionWrapper` enum abstracts
   over both variants.

4. **Registry-based lookup.** Receivers and exporters receive an
   `ExtensionRegistry` at `start()` and look up extensions by name and trait.
   Processors do not receive the registry (they don't need cross-cutting
   capabilities directly).

5. **Sealed extension traits.** New extension trait types can only be defined
   inside the engine crate (enforced via a sealed trait pattern). External
   crates can *implement* existing traits but cannot create new ones, ensuring
   the set of extension capabilities is well-defined and documented.

## Core Types

### `Extension` trait (local and shared variants)

The lifecycle trait every extension implements. Unlike receivers, processors,
and exporters, extensions are **not generic over `PData`** -- they are
completely decoupled from the pipeline data type. Two variants exist:

**Local variant** -- `engine/src/local/extension.rs`:

```rust
#[async_trait(?Send)]
pub trait Extension {
    async fn start(
        self: Box<Self>,
        control_channel: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, Error>;

    fn extension_traits(&self) -> Vec<TraitRegistration> {
        Vec::new()
    }
}
```

**Shared variant** -- `engine/src/shared/extension.rs`:

```rust
#[async_trait]
pub trait Extension: Send {
    async fn start(
        self: Box<Self>,
        control_channel: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, Error>;

    fn extension_traits(&self) -> Vec<TraitRegistration> {
        Vec::new()
    }
}
```

- The local variant uses `#[async_trait(?Send)]` (futures are `!Send`), while
  the shared variant uses `#[async_trait]` (futures must be `Send`).
- `start()` takes a `ControlChannel` (wrapping a receiver for
  `ExtensionControlMsg`) and a simplified `EffectHandler` (node ID and metrics
  reporter only -- no timer management through the pipeline control channel).
- `extension_traits()` returns trait registrations that the engine inserts into
  the `ExtensionRegistry`. Extensions that don't publish any traits (pure
  background tasks) can use the default empty implementation.
- Extensions manage their own timers directly (e.g., via `tokio::time`)
  rather than through the pipeline's timer infrastructure.

### `ExtensionWrapper`

Engine-internal adapter that wires an extension into the pipeline, defined in
`engine/src/extension.rs`. It is a **non-generic** enum with two variants:

```rust
pub enum ExtensionWrapper {
    Local { /* local::extension::Extension impl */ },
    Shared { /* shared::extension::Extension impl */ },
}
```

Constructors: `ExtensionWrapper::local(...)` and `ExtensionWrapper::shared(...)`.

It:

- Creates the `ExtensionControlMsg` control channel
- Provides an `extension_control_sender()` for shutdown orchestration
- Does **not** implement `Node<PData>` or `Controllable<PData>` (extensions
  are not data-path nodes)
- Provides telemetry integration
- Collects trait registrations during pipeline build

### `ExtensionRegistry`

A `Clone + Send` registry that stores type-erased extension values and produces
`Box<dyn Trait>` on lookup. Each `get()` returns a freshly-cloned trait object
that shares `Arc`-wrapped state with the original.

```rust
// Consumer (e.g., exporter) retrieves a provider by name:
let provider: Box<dyn BearerTokenProvider> = registry
    .get::<dyn BearerTokenProvider>("azure_auth")?;
let token = provider.get_token().await?;
```

Key properties:

- **Keyed by** `(extension_name: String, TypeId::of::<Box<dyn Trait>>())`
- **Clone** deep-copies each stored value (cheap when extensions use `Arc`)
- **Error discrimination**: `NotFound` vs. `TraitNotImplemented`

### `ExtensionTrait` (sealed)

Marker trait (`extension/registry.rs`) that restricts which trait types can be
registered. Each extension trait file self-registers using the pattern:

```rust
// In extension/registry.rs:
pub(crate) mod private {
    pub trait Sealed {}
}
pub trait ExtensionTrait: private::Sealed {}

// In extension/bearer_token_provider.rs (self-registering):
impl super::registry::private::Sealed for dyn BearerTokenProvider {}
impl super::registry::ExtensionTrait for dyn BearerTokenProvider {}
```

External crates cannot implement `Sealed`, so they cannot add new extension
trait types.

### `extension_traits!` macro

Convenience macro with two forms:

```rust
// Convenience form -- inside impl Extension:
otap_df_engine::extension_traits!(BearerTokenProvider);

// Explicit form -- returns Vec<TraitRegistration>:
fn extension_traits(&self) -> Vec<TraitRegistration> {
    extension_traits!(self => BearerTokenProvider)
}
```

The macro:

1. Verifies at compile time that each trait implements `ExtensionTrait` (sealed)
2. Verifies the concrete type implements each trait + `Clone + Send + 'static`
3. Creates monomorphised `coerce` function pointers for type-safe downcasting

## Extension Traits

### `BearerTokenProvider`

Defined in `engine/src/extension/bearer_token_provider.rs`. Provides
authentication tokens to consumers:

```rust
#[async_trait]
pub trait BearerTokenProvider: Send {
    async fn get_token(&self) -> Result<BearerToken, Error>;
    fn subscribe_token_refresh(&self) -> watch::Receiver<Option<BearerToken>>;
}
```

- `get_token()` returns a `BearerToken` (wrapping a `Secret` value and
  UNIX-timestamp expiry)
- `subscribe_token_refresh()` returns a `tokio::sync::watch::Receiver` for
  reactive notification when tokens are refreshed -- useful for updating HTTP
  headers without polling

## Extension Writer Contract

Extensions that publish traits must satisfy:

| Requirement            | Reason                                           |
|------------------------|--------------------------------------------------|
| `Clone`                | Registry stores clones; `get()` returns clones   |
| `Send`                 | Registry is `Send`; extensions may cross threads |
| `'static`              | Required for `Any`-based type erasure            |
| Shared state via `Arc` | Clones must observe the same state               |

Pure background-task extensions (no published traits) have no special
requirements beyond implementing the `Extension` trait. Local extensions
can even use `!Send` futures.

## Pipeline Lifecycle

```text
1. Config parsing
   +- Extensions parsed from the `extensions` section (sibling to `nodes`)
   +- NodeKind::Extension recognized in node_urn.rs
   +- Extensions excluded from PData wiring (no connections)
   +- Placing an extension URN in `nodes` is rejected with an error

2. Pipeline build (PipelineFactory)
   +- Allocate node IDs for data-path nodes, then for extensions
   +- Create data-path runtime nodes (receivers, processors, exporters)
   +- create_extension() -- factory lookup by URN, config parsing
   +- ExtensionWrapper::local() / ::shared() -- control channel setup
   +- register_traits() -- collect TraitRegistration, insert into ExtensionRegistry
   +- build_node_wrapper() -- telemetry, channel metrics

3. Pipeline start (RuntimePipeline::run)
   +- Spawn extension tasks FIRST (FuturesUnordered)
   +- Spawn exporter tasks (with extension_registry.clone())
   +- Spawn processor tasks
   +- Spawn receiver tasks (with extension_registry.clone())

4. Steady state
   +- Extensions run their event loops (e.g., token refresh)
   +- Data-path components use registry lookups as needed
   +- Control messages flow normally (shutdown, timer, config)

5. Shutdown (extensions shut down LAST)
   +- Data-path nodes receive Shutdown and drain
   +- Pipeline control channel closes after all data-path nodes finish
   +- PipelineCtrlMsgManager::shutdown_extensions() sends
      ExtensionControlMsg::Shutdown to all extensions
   +- Extensions terminate after data-path is fully drained
```

**Why shutdown-last?** Extensions provide capabilities (e.g., authentication
tokens) that data-path nodes depend on during their graceful shutdown. If
extensions shut down first, exporters flushing final batches could lose access
to valid credentials.

**Why separate control channels?** Extensions use `ExtensionControlSender` /
`ExtensionControlMsg` instead of the pipeline's `PipelineCtrlMsgSender<PData>`.
This prevents extensions from holding clones of the pipeline control channel
sender, which would prevent the channel from closing and block graceful
shutdown (requiring the draining deadline to expire).

## Concrete Implementation: Azure Identity Auth Extension

Located in `contrib-nodes/src/extensions/azure_identity_auth_extension/`.

### Registration

```rust
#[distributed_slice(OTAP_EXTENSION_FACTORIES)]
pub static AZURE_IDENTITY_AUTH_EXTENSION: ExtensionFactory = ExtensionFactory {
    name: "urn:microsoft:extension:azure_identity_auth",
    create: |..| { /* deserialize Config, create AzureIdentityAuthExtension */ },
    validate_config: validate_typed_config::<Config>,
};
```

### Structure

`AzureIdentityAuthExtension` is a single `Clone` struct:

```rust
#[derive(Clone)]
pub struct AzureIdentityAuthExtension {
    credential: Arc<dyn TokenCredential>,    // Azure credential
    scope: String,                            // OAuth scope
    method: AuthMethod,                       // Managed Identity / Development
    token_sender: Arc<watch::Sender<Option<BearerToken>>>,  // broadcast channel
}
```

It implements both:

- **`Extension`** -- drives the proactive token refresh loop via
  `tokio::select!`, broadcasting new tokens to all subscribers
- **`BearerTokenProvider`** -- allows consumers to call `get_token()` or
  `subscribe_token_refresh()`

### Consumer Integration (Azure Monitor Exporter)

The exporter no longer manages its own authentication. Instead:

```rust
// In exporter start():
let auth = extension_registry
    .get::<dyn BearerTokenProvider>(&self.config.auth)?;

// Wait for initial token
let mut token_rx = auth.subscribe_token_refresh();
token_rx.wait_for(|t| t.is_some()).await?;

// React to refreshes in event loop
tokio::select! {
    _ = token_rx.changed() => {
        if let Some(token) = token_rx.borrow_and_update().as_ref() {
            client_pool.update_auth(bearer_header(token));
        }
    }
    // ... other branches
}
```

### Configuration

```yaml
extensions:
  azure_auth:
    type: "urn:microsoft:extension:azure_identity_auth"
    config:
      method: managed_identity        # or "development"
      client_id: "optional-client-id" # for user-assigned managed identity
      scope: "https://monitor.azure.com/.default"

nodes:
  my_exporter:
    type: "urn:microsoft:exporter:azure_monitor"
    config:
      auth: "azure_auth"   # references the extension by name
      api: { ... }
```

## Adding a New Extension Trait

1. Create the trait file in `engine/src/extension/` (e.g., `health_check.rs`)
2. In that file, add self-registering sealed impls:

   ```rust
   impl super::registry::private::Sealed for dyn HealthCheck {}
   impl super::registry::ExtensionTrait for dyn HealthCheck {}
   ```

3. Add `pub mod health_check;` in `engine/src/extension.rs`
4. Extension implementors use `extension_traits!(HealthCheck)` in their
   `impl Extension` block

## Adding a New Extension Implementation

1. Create a module under `contrib-nodes/src/extensions/`
2. Implement `Extension` on a `Clone + Send + 'static` struct
3. Use `extension_traits!` to declare which traits are published
4. Register with `#[distributed_slice(OTAP_EXTENSION_FACTORIES)]`
5. Gate behind a Cargo feature in `contrib-nodes/Cargo.toml`

## Files Changed in This PR

| File | Change |
| ---- | ------ |
| `engine/src/extension.rs` | `ExtensionWrapper` enum (Local/Shared, non-generic), `pub mod` declarations, `ExtensionControlSender`, telemetry integration |
| `engine/src/extension/registry.rs` | `ExtensionRegistry`, `TraitRegistration`, `extension_traits!` macro, sealed `ExtensionTrait` + `Error` type |
| `engine/src/extension/bearer_token_provider.rs` | `BearerTokenProvider` trait, `BearerToken`, `Secret`, self-registering sealed impls |
| `engine/src/local/extension.rs` | Local (`!Send`) `Extension` trait + `ControlChannel` + `EffectHandler` (PData-free) |
| `engine/src/shared/extension.rs` | Shared (`Send`) `Extension` trait + `ControlChannel` + `EffectHandler` (PData-free) |
| `engine/src/config.rs` | New `ExtensionConfig` type |
| `engine/src/error.rs` | New `ExtensionAlreadyExists`, `UnknownExtension`, `ExtensionInNodesSection` variants |
| `engine/src/node.rs` | New `NodeType::Extension` variant |
| `engine/src/lib.rs` | `ExtensionFactory`, `PipelineFactory::with_extensions()`, `create_extension()`, registry build |
| `engine/src/runtime_pipeline.rs` | Extension spawning (before other nodes), separate `extension_control_senders`, registry passing |
| `engine/src/pipeline_ctrl.rs` | `extension_control_senders` field, `shutdown_extensions()` after data-plane draining |
| `engine/src/control.rs` | `ExtensionControlMsg` enum, `ExtensionControlSender` struct |
| `engine/src/exporter.rs` | `start()` signature gains `ExtensionRegistry` |
| `engine/src/receiver.rs` | `start()` signature gains `ExtensionRegistry` |
| `engine/src/local/{exporter,receiver}.rs` | Trait method gains `extension_registry` param |
| `engine/src/shared/{exporter,receiver}.rs` | Trait method gains `extension_registry` param |
| `engine-macros/src/lib.rs` | Macro generates `EXTENSION_FACTORIES` slice and factory map |
| `config/src/node.rs` | `NodeKind::Extension` variant |
| `config/src/node_urn.rs` | Parse/validate `extension` kind in URNs |
| `config/src/pipeline.rs` | Sibling `extensions` section in `PipelineConfig`, excluded from dead-node pruning |
| `contrib-nodes/src/extensions/` | New -- `azure_identity_auth_extension` module |
| `contrib-nodes/src/exporters/azure_monitor_exporter/` | Auth extracted to extension; `config.auth` is now a `String` name |
| All existing exporters/receivers | `extension_registry` param added to `start()` |
