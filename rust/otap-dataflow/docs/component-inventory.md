# Component Inventory Guide

The `otap-dataflow` engine tracks all security-relevant components using link-time metadata registered via the `#[component_inventory]` attribute macro (defined in `otap-df-engine-macros`).

This metadata is read by offline tooling (`cargo xtask component-inventory`) and checked against a baseline to detect component additions, removals, or security property drift during development and CI.

---

## Annotating a Component

### Common Case: Factory Node (Receivers, Exporters, Processors, Extensions)

Every pipeline node factory static must be annotated with `#[component_inventory(...)]`. The component's unique identity (`id`) is derived automatically from the factory's `name` field (its URN), so you do not need to provide an explicit `id`.

```rust
use otap_df_engine_macros::component_inventory;

#[component_inventory(
    category = Receiver,
    description = "OTLP gRPC and HTTP receiver for traces, metrics, and logs",
    attributes(port = "4317", protocol = "gRPC (HTTP/2)", auth = "mTLS (opt-in)"),
)]
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTLP_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTLP_RECEIVER_URN,
    // ...
};
```

#### Macro Parameters

- `category`: Mandatory bare identifier. Must be one of `Receiver`, `Exporter`, `Processor`, `Extension`, `Admin`, `Controller`, `Cli`, `Subsystem`, or `Safety`. Misspellings are caught at compile time.
- `description`: Optional human-readable description string.
- `attributes(...)`: Optional key-value pairs representing security-relevant properties.

### Fallback Case: Non-Factory Infrastructure

For non-factory components (e.g. admin HTTP server, controller, CLI tools, memory limiter) that lack a factory static, apply the macro to the struct/enum/fn and specify an explicit URN-shaped `id`:

```rust
#[component_inventory(
    id = "urn:otel:admin:http_server",
    category = Admin,
    description = "Built-in HTTP admin server for health, metrics, and pipeline control",
    attributes(port = "8080", protocol = "HTTP", auth = "NONE"),
)]
pub struct AdminServer { /* ... */ }
```

---

## Standard Attribute Keys

Use well-known attribute key constants from `otap_df_engine::inventory::attrs`:

| Key Constant | Description | Example Values |
| :--- | :--- | :--- |
| `port` | Listening network port | `"4317"`, `"8080"` |
| `protocol` | Wire protocol | `"gRPC (HTTP/2)"`, `"HTTP"`, `"TCP"` |
| `auth` | Authentication mechanism | `"mTLS (opt-in)"`, `"NONE"`, `"OAuth2"` |
| `filesystem_access` | Local filesystem access level | `"READ_ONLY"`, `"READ_WRITE"`, `"NONE"` |
| `cloud_api` | External cloud API endpoint | `"Azure Monitor"`, `"AWS Kinesis"` |
| `feature_flag` | Cargo feature flag required | `"etw"`, `"kafka"` |

---

## Baseline Verification

When adding or modifying a component annotation, verify your changes locally:

```bash
cd rust/otap-dataflow
cargo xtask check
```
