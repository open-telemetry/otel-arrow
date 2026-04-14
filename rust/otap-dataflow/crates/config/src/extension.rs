// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension configuration types.
//!
//! Extensions have a simpler configuration model than data-path nodes — they
//! have no output ports, no wiring contracts, and no header policies.

pub use crate::extension_urn::ExtensionUrn;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// User configuration for an extension in the pipeline.
///
/// Unlike [`NodeUserConfig`](crate::node::NodeUserConfig), extensions have no
/// output ports, wiring contracts, or transport header policies — they only
/// need a type URN and extension-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ExtensionUserConfig {
    /// The extension type URN identifying the plugin (factory) to use.
    pub r#type: ExtensionUrn,

    /// An optional description of this extension.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Extension-specific configuration (interpreted by the extension itself).
    #[serde(default)]
    #[schemars(extend("x-kubernetes-preserve-unknown-fields" = true))]
    pub config: Value,
}

impl ExtensionUserConfig {
    /// Creates a new `ExtensionUserConfig` with the specified type URN and config.
    #[must_use]
    pub fn new(r#type: ExtensionUrn, config: Value) -> Self {
        Self {
            r#type,
            description: None,
            config,
        }
    }

    /// Creates a new `ExtensionUserConfig` with the specified type URN and
    /// default (null) config.
    #[must_use]
    pub fn with_type<U: Into<ExtensionUrn>>(r#type: U) -> Self {
        Self {
            r#type: r#type.into(),
            description: None,
            config: Value::Null,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_user_config_deserialize() {
        let yaml = r#"
type: "urn:otap:sample_kv_store"
config:
  capacity: 100
"#;
        let config: ExtensionUserConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.r#type.id(), "sample_kv_store");
        assert_eq!(config.config["capacity"], 100);
    }

    #[test]
    fn test_extension_user_config_rejects_capabilities() {
        let yaml = r#"
type: "urn:otap:auth"
capabilities:
  some_cap: "ext"
"#;
        let result: Result<ExtensionUserConfig, _> = serde_yaml::from_str(yaml);
        assert!(result.is_err());
    }
}
