# otel-arrow-dfe-wasm-guest-sdk

Internal Rust SDK for authoring the experimental OTAP dataflow WASM processor
plugins in this repository. The SDK owns the guest bindings and embeds the WIT
contract. Plugin crates should use the SDK API rather than call `wit_bindgen`
or locate a WIT directory themselves.

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

This crate is internal and is not published to crates.io.
