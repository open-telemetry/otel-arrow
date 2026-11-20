# otel-arrow-dfe-wasm-guest-sdk

Internal Rust SDK for authoring the experimental OTAP dataflow WASM processor
plugins in this repository. The SDK owns the guest bindings and embeds the WIT
contract. Plugin crates should use the SDK API rather than call `wit_bindgen`
or locate a WIT directory themselves.

## Features

The default `guest` feature provides the plugin API and generated bindings.
Hosts that only need `HOST_SERVICES_ABI_VERSION` can set
`default-features = false` to skip guest bindings and their dependencies.
Workspace dependencies disable defaults, so workspace guest consumers must
explicitly enable `features = ["guest"]`.

## Example

```rust
use wasm_guest_sdk::{
    AttrScope, InitError, Lifecycle, LogLevel, Pdata, Plugin, counter_add,
    export_plugin, filter_by_attribute_eq, log,
};

struct ExamplePlugin;

impl Plugin for ExamplePlugin {
    async fn process(data: Pdata) -> Option<Pdata> {
        Some(
            filter_by_attribute_eq(
                data,
                AttrScope::Record,
                String::from("severity_text"),
                String::from("ERROR"),
            )
            .await,
        )
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
`wasm32-wasip3` with the project Rust nightly toolchain. The SDK automatically
checks for an exact host-services ABI version match before calling
`Lifecycle::initialize`; a mismatch is a startup error.

## Configuration

Call `get_config::<T>()` once during `Lifecycle::initialize` and keep the typed
result for processing. Configuration is a read-only snapshot fixed for the
plugin instance's lifetime. Unset configuration is JSON `null`; use `Option<T>`
if configuration is optional.

Configuration reads share a host-service rate limit with `log` and `counter_add`.
When throttled, `get_config` returns `ConfigError::Throttled`, not a JSON parsing
error. Invalid JSON or a type mismatch returns `ConfigError::Deserialize`.
The separate copy-byte budget still traps on exhaustion.

This crate is internal and is not published to crates.io.
