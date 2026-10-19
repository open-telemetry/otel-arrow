// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Guest-side SDK for the experimental OTAP dataflow WASM plugin contract.
//!
//! Bindings for the embedded WIT contract are generated here. Plugins should
//! implement [`Plugin`] and [`Lifecycle`], then invoke [`export_plugin!`].

#![allow(unexpected_cfgs)]

/// Generated bindings for the SDK's embedded plugin WIT contract.
#[doc(hidden)]
pub mod bindings {
    wit_bindgen::generate!({
        world: "kernel-processor",
        path: "wit",
        async: true,
        std_feature,
        pub_export_macro: true,
        export_macro_name: "__export_kernel_processor",
    });
}

pub use bindings::exports::otel::otap_dataflow_plugin::lifecycle::InitError;
pub use bindings::otel::otap_dataflow_plugin::host_services::LogLevel;
pub use bindings::otel::otap_dataflow_plugin::otel_kernels::{
    AttrScope, Pdata, filter_by_attribute_eq, pdata_num_rows,
};

/// Exact host-services ABI version supported by this SDK.
pub const HOST_SERVICES_ABI_VERSION: u32 = 2;

/// Error returned when plugin configuration cannot be deserialized.
pub type ConfigError = serde_json::Error;

/// Processor behavior implemented by a guest plugin.
#[allow(async_fn_in_trait)]
pub trait Plugin {
    /// Process one host-managed pdata resource, or return `None` to drop it.
    async fn process(data: Pdata) -> Option<Pdata>;
}

/// One-time initialization behavior implemented by a guest plugin.
#[allow(async_fn_in_trait)]
pub trait Lifecycle {
    /// Initialize the plugin after the SDK confirms the exact host ABI version.
    async fn initialize() -> Result<(), InitError>;
}

/// Emit a log message through the host telemetry pipeline.
pub async fn log(level: LogLevel, message: String) {
    bindings::otel::otap_dataflow_plugin::host_services::log(level, message).await;
}

/// Add to a bounded, plugin-instance-scoped host counter.
pub async fn counter_add(name: String, value: u64) {
    bindings::otel::otap_dataflow_plugin::host_services::counter_add(name, value).await;
}

/// Fetch and deserialize the plugin-specific JSON configuration.
pub async fn get_config<T: serde::de::DeserializeOwned>() -> Result<T, ConfigError> {
    let raw = bindings::otel::otap_dataflow_plugin::host_services::get_config().await;
    decode_config(&raw)
}

fn decode_config<T: serde::de::DeserializeOwned>(raw: &str) -> Result<T, ConfigError> {
    serde_json::from_str(raw)
}

#[doc(hidden)]
pub mod __private {
    use crate::{HOST_SERVICES_ABI_VERSION, InitError, Lifecycle};

    /// Check the current exact ABI version before invoking plugin initialization.
    pub async fn initialize_plugin<T: Lifecycle>(host_abi_version: u32) -> Result<(), InitError> {
        validate_host_abi_version(host_abi_version)?;
        T::initialize().await
    }

    pub(crate) fn validate_host_abi_version(host_abi_version: u32) -> Result<(), InitError> {
        if host_abi_version != HOST_SERVICES_ABI_VERSION {
            return Err(InitError {
                message: format!(
                    "unsupported host-services ABI version {host_abi_version} (expected {HOST_SERVICES_ABI_VERSION})"
                ),
            });
        }
        Ok(())
    }
}

/// Implement the WIT exports for a type implementing [`Plugin`] and [`Lifecycle`].
///
/// The generated lifecycle export checks that the host ABI is exactly
/// [`HOST_SERVICES_ABI_VERSION`] before calling the plugin's initializer.
#[macro_export]
macro_rules! export_plugin {
    ($plugin:ident) => {
        impl $crate::bindings::exports::otel::otap_dataflow_plugin::processor::Guest
            for $plugin
        {
            async fn process(
                data: $crate::Pdata,
            ) -> Option<$crate::Pdata> {
                <$plugin as $crate::Plugin>::process(data).await
            }
        }

        impl $crate::bindings::exports::otel::otap_dataflow_plugin::lifecycle::Guest
            for $plugin
        {
            async fn initialize() -> Result<(), $crate::InitError> {
                let host_abi_version =
                    $crate::bindings::otel::otap_dataflow_plugin::host_services::host_abi_version()
                        .await;
                $crate::__private::initialize_plugin::<$plugin>(host_abi_version).await
            }
        }

        $crate::bindings::__export_kernel_processor!(
            $plugin with_types_in $crate::bindings
        );
    };
}

#[cfg(test)]
#[allow(unsafe_code)]
mod tests {
    use super::*;
    use serde::Deserialize;

    struct ExportFixture;

    impl Plugin for ExportFixture {
        /// Scenario: An exported plugin counts rows using only the SDK's public API.
        /// Guarantees: Borrowing pdata for an async row count compiles without generated binding paths.
        async fn process(data: Pdata) -> Option<Pdata> {
            let rows: u32 = pdata_num_rows(&data).await;
            (rows > 0).then_some(data)
        }
    }

    impl Lifecycle for ExportFixture {
        async fn initialize() -> Result<(), InitError> {
            Ok(())
        }
    }

    export_plugin!(ExportFixture);

    #[derive(Debug, Deserialize, PartialEq)]
    struct TestConfig {
        enabled: bool,
    }

    /// Scenario: Valid JSON is decoded into the requested plugin config type.
    /// Guarantees: `get_config`'s deserialization path produces typed values without changing fields.
    #[test]
    fn config_json_deserializes_to_requested_type() {
        let config: TestConfig = decode_config(r#"{"enabled":true}"#).expect("valid plugin config");
        assert_eq!(config, TestConfig { enabled: true });
    }

    /// Scenario: The host represents an unset plugin configuration as JSON `null`.
    /// Guarantees: Plugins can deserialize absent optional configuration without treating it as malformed JSON.
    #[test]
    fn null_config_deserializes_as_none() {
        let config: Option<TestConfig> = decode_config("null").expect("null optional config");
        assert_eq!(config, None);
    }

    /// Scenario: Plugin configuration contains JSON that does not match its declared Rust type.
    /// Guarantees: The SDK returns the serde JSON error instead of silently constructing partial configuration.
    #[test]
    fn config_json_returns_error_for_mismatched_type() {
        assert!(decode_config::<TestConfig>(r#"{"enabled":"yes"}"#).is_err());
    }

    /// Scenario: The host reports a version adjacent to the SDK's required ABI version.
    /// Guarantees: Initialization accepts only exact ABI version 2 and rejects both older and newer versions.
    #[test]
    fn host_abi_version_requires_exact_match() {
        for version in [HOST_SERVICES_ABI_VERSION - 1, HOST_SERVICES_ABI_VERSION + 1] {
            let error = __private::validate_host_abi_version(version)
                .expect_err("non-matching ABI must be rejected");
            assert!(
                error
                    .message
                    .contains("unsupported host-services ABI version")
            );
        }
        assert!(__private::validate_host_abi_version(HOST_SERVICES_ABI_VERSION).is_ok());
    }
}
