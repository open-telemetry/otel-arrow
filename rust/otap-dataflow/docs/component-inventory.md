# Component Inventory Guide

The otap-dataflow engine tracks all security-relevant components using
link-time metadata registered via the #[component_inventory] attribute
macro (defined in otap-df-engine-macros).

This metadata is read by offline tooling (cargo xtask component-inventory)
and checked against a baseline to detect component additions, removals, or
security property drift during development and CI.

---

## How It Works

The Component Inventory System uses a hybrid compile-time registration and
static AST scanning pipeline to track components securely and reliably:

1. Link-Time Registration (The Proc-Macro):
   When components are compiled, the #[component_inventory] macro generates
   a static metadata block (ComponentMeta) and emits it into a unified link
   section via the linkme crate. This guarantees that compiled binaries
   carry a highly accurate inventory of exactly what feature/platform gates
   are active.

2. Source-Text Analysis (The AST Scanner):
   In local development and CI (cargo xtask check), an offline scanner parses
   the workspace .rs source code using the syn parser:
   *   Pass 1 (URN Resolution): It indexes all const NAME: &str = "..."
       declarations workspace-wide to resolve factory name constants back to
       their actual string values.
   *   Pass 2 (Verification): It maps all #[distributed_slice] factory
       registries and verifies that each one has a matching, well-formed
       #[component_inventory] annotation.

3. Drift Detection:
   The compiled inventory is evaluated against the checked-in baseline file
   components-baseline.json located at the root of the workspace. Any
   unauthorized additions, removals, or modifications to a component's security
   properties will trigger a validation failure.

---

## Annotating a Component

### Common Case: Factory Node (Receivers, Exporters, Processors, Extensions)

Every pipeline node factory static must be annotated with
#[component_inventory(...)]. The component's unique identity (id) is
derived automatically from the factory's name field (its URN), so you do not
need to provide an explicit id.

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

- category: Mandatory bare identifier. Must be one of Receiver, Exporter,
  Processor, Extension, Admin, Controller, Cli, Subsystem, or
  Safety. Misspellings are caught at compile time.
- description: Optional human-readable description string.
- attributes(...): Optional key-value pairs representing security-relevant properties.

### Fallback Case: Non-Factory Infrastructure

For non-factory components (e.g. admin HTTP server, controller, CLI tools,
memory limiter) that lack a factory static, apply the macro to the
struct/enum/fn and specify an explicit URN-shaped id:

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

Use well-known attribute key constants from otap_df_engine::inventory::attrs:

| Key Constant | Description | Example Values |
| :--- | :--- | :--- |
| port | Listening network port | "4317", "8080" |
| protocol | Wire protocol | "gRPC (HTTP/2)", "HTTP", "TCP" |
| auth | Authentication mechanism | "mTLS (opt-in)", "NONE", "OAuth2" |
| filesystem_access | Local filesystem access level | "READ_ONLY", "READ_WRITE", "NONE" |
| cloud_api | External cloud API endpoint | "Azure Monitor", "AWS Kinesis" |
| feature_flag | Cargo feature flag required | "etw", "kafka" |

---

## Baseline Verification

When adding or modifying a component annotation, verify your changes locally:

```bash
cd rust/otap-dataflow
cargo xtask check
```

---

## Verification Errors and Resolutions

If the cargo xtask check verification fails, the scanner will output specific
error classes. Here is how to resolve them:

### 1. NEW (annotated in code, not in baseline)
*   Cause: You added a new component macro annotation, but it has not been
    frozen into the baseline JSON file yet.
*   Fix: Update the baseline file locally by running:
    ```bash
    cargo xtask component-inventory --update-baseline
    ```
    Then stage and commit the updated components-baseline.json.

### 2. REMOVED (in baseline, no annotation found in code)
*   Cause: A component was deleted or unannotated, but still exists in the
    baseline.
*   Fix: Update the baseline file to record the removal:
    ```bash
    cargo xtask component-inventory --update-baseline
    ```

### 3. MODIFIED (properties differ from baseline)
*   Cause: You changed an existing component's category, description, or
    attributes.
*   Fix: Run the baseline update to freeze the new security properties:
    ```bash
    cargo xtask component-inventory --update-baseline
    ```

### 4. MISSING annotation
*   Cause: A component factory is registered in a static distributed slice
    (e.g., #[distributed_slice(OTAP_RECEIVER_FACTORIES)]) but is missing its
    companion #[component_inventory] attribute.
*   Fix: Apply the #[component_inventory(...)] attribute directly above
    the static definition.

### 5. UNRESOLVED URNs
*   Cause: The offline scanner could not resolve the name field identifier
    to a constant string value in your file.
*   Fix: Either ensure the constant is defined as a string literal in the
    same file/crate, or supply an explicit id directly on the macro:
    ```rust
    #[component_inventory(id = "urn:otel:receiver:custom", category = Receiver)]
    ```
