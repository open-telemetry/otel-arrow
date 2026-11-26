# otel-arrow-dfe-wasm-guest-sdk

Internal Rust SDK for authoring the experimental OTAP dataflow WASM processor
plugins in this repository. The SDK owns the guest bindings and embeds the WIT
contract. Plugin crates should use the SDK API rather than call `wit_bindgen`
or locate a WIT directory themselves.

> Before WIT package version `1.0.0`, backward compatibility is not guaranteed.
> Host upgrades may require updating plugin source and rebuilding binaries
> against the matching SDK.

## Features

The default `guest` feature provides the plugin API and generated bindings.
Hosts that only need `HOST_SERVICES_ABI_VERSION` can set
`default-features = false` to skip guest bindings and their dependencies.
Workspace dependencies disable defaults, so workspace guest consumers must
explicitly enable `features = ["guest"]`.

## Example

```rust
use wasm_guest_sdk::{
    AttrScope, InitError, Lifecycle, LogLevel, Output, Pdata, Plugin, counter_add,
    export_plugin, filter_by_attribute_eq, log,
};

struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    async fn process(data: Pdata) -> Vec<Output> {
        vec![Output {
            port: None,
            data: filter_by_attribute_eq(
                data,
                AttrScope::Record,
                String::from("severity_text"),
                String::from("ERROR"),
            )
            .await,
        }]
    }
}

impl Lifecycle for ExamplePlugin {
    async fn initialize() -> Result<(), InitError> {
        log(LogLevel::Info, String::from("plugin initialized")).await;
        counter_add(String::from("plugin.initialize"), 1).await;
        Ok(())
    }
}

export_plugin!(ExamplePlugin);
```

Guest crates depend on this SDK by local Cargo path and build for
`wasm32-wasip3` with Rust stable (1.100 or newer). The SDK automatically
checks for an exact host-services ABI version match before calling
`Lifecycle::initialize`; a mismatch is a startup error.

## Outputs and routing

This SDK targets WIT package `otel:otap-dataflow-plugin@0.2.0` and host-services
behavior version 2.

Return zero or more owned `Output` values. An empty list intentionally drops
the input and acknowledges it. `port: None` selects the engine's default
output (including the single-connected-port fallback); `Some(name)` selects
a connected named port. Several outputs may use the same port.

Call `get_output_info().await` from `process` for sorted `connected_ports`, the
resolved `default_port`, and `max_outputs_per_call`. The snapshot reflects actual
wiring, not declared outputs or plugin configuration, and is fixed before the
first `process` call. Cache a successful result in guest-local state.
During instantiation and `initialize`, discovery returns
`OutputInfoError::Unavailable`.

Discovery returns `OutputInfoError::Throttled` when the shared host-service
call-rate budget is exhausted; an empty port list is a valid result, not an error.
Copies charge strings and list metadata against the shared byte budgets before
allocation; exhaustion traps.

Use `clone_pdata(&data).await` to obtain another independently owned handle.
The host shares Arrow buffers rather than copying bulk telemetry into WASM.
Consuming a clone with a filter does not change the original. For example:

```rust,ignore
let copy = clone_pdata(&data).await;
vec![
    Output { port: Some("primary".into()), data },
    Output { port: Some("audit".into()), data: copy },
]
```

The host validates the whole list before sending anything. Output-count limits,
invalid handles, retained resources, and invalid destinations are terminal
contract violations. Sends follow list order and apply downstream backpressure,
but delivery is not atomic across outputs or ports.

The host owns delivery accounting: all outputs must succeed to Ack the input.
If any fail, the input is Nacked after all outputs settle. Only an input whose
outputs all failed transiently gets a retryable Nack; mixed success/failure
is permanent to avoid automatically replaying already-delivered data.

When upstream requests `RETURN_DATA`, completion returns the original input
records or encoded bytes, not a guest-transformed output, even for one output.

Guest-local state may survive calls, but every host pdata handle must be returned
or dropped before `process` ends. Completion handling is host-owned; there are no
guest completion or shutdown callbacks.

Closing upstream input can stop completion handling before outstanding outputs
settle, including single-output calls (#2477). Cancellation and terminal failure
can also prevent completion; resource cleanup does not send Ack/Nack.

## Configuration

Call `get_config::<T>()` once during `Lifecycle::initialize` and keep the typed
result for processing. Configuration is a read-only snapshot fixed for the
plugin instance's lifetime. Unset configuration is JSON `null`; use `Option<T>`
if configuration is optional.

Configuration reads share a host-service rate limit with `get_output_info`,
`log`, and `counter_add`.
When throttled, `get_config` returns `ConfigError::Throttled`, not a JSON parsing
error. Invalid JSON or a type mismatch returns `ConfigError::Deserialize`.
The separate copy-byte budget still traps on exhaustion.

This crate is internal and is not published to crates.io.
