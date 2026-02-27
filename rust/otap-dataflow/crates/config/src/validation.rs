// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation helpers for node configuration.
//!
//! These helpers are intended for use with the `validate_config` field on
//! factory structs ([`ReceiverFactory`], [`ProcessorFactory`], [`ExporterFactory`]).
//!
//! **Scope:** `validate_config` performs *static* validation only — it checks
//! that the config value can be deserialized into the expected type. It does
//! **not** detect runtime issues such as port conflicts, unreachable endpoints,
//! missing files, or other conditions that only manifest when the engine starts.
//! Those errors will still surface at startup time.

use crate::error::Error;

/// Validates that a JSON config value can be deserialized into the expected
/// configuration type `T`.
///
/// This is the most common validator: pass it as a monomorphised function
/// pointer and the compiler ensures every component with a typed `Config`
/// struct gets deserialization-level validation for free.
///
/// # Example
/// ```ignore
/// validate_config: validate_typed_config::<MyComponentConfig>,
/// ```
pub fn validate_typed_config<T: serde::de::DeserializeOwned>(
    config: &serde_json::Value,
) -> Result<(), Error> {
    let _: T = serde_json::from_value(config.clone()).map_err(|e| Error::InvalidUserConfig {
        error: e.to_string(),
    })?;
    Ok(())
}

/// Validator for components that accept **no** user configuration.
///
/// Accepts `Value::Null` (config key omitted / set to `null`) and empty
/// objects `{}`. Rejects anything else so that typos or misplaced config
/// blocks are caught early.
///
/// # Example
/// ```ignore
/// validate_config: no_config,
/// ```
pub fn no_config(config: &serde_json::Value) -> Result<(), Error> {
    match config {
        serde_json::Value::Null => Ok(()),
        serde_json::Value::Object(map) if map.is_empty() => Ok(()),
        _ => Err(Error::InvalidUserConfig {
            error: format!(
                "This component does not accept configuration, but received: {}",
                config
            ),
        }),
    }
}
